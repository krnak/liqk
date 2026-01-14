use axum::{
    extract::{ConnectInfo, Multipart, Path, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use axum_extra::extract::cookie::CookieJar;
use chrono::Utc;
use sha2::{Sha256, Digest};
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use tokio::io::AsyncWriteExt;
use tracing::{info, warn};
use uuid::Uuid;

use crate::auth::{extract_token_from_header, TOKEN_COOKIE_NAME};
use crate::AppState;

pub const MAX_UPLOAD_SIZE: usize = 4 * 1024 * 1024 * 1024; // 4 GB
const FILESYSTEM_GRAPH: &str = "http://liqk.org/graph/filesystem";
const ACCESS_GRAPH: &str = "http://liqk.org/graph/access";

/// Escape a string for use in SPARQL string literals.
fn escape_sparql_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => result.push_str("\\\\"),
            '"' => result.push_str("\\\""),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            _ => result.push(c),
        }
    }
    result
}

/// Validate that a string is a valid UUID format.
fn validate_uuid(s: &str) -> Option<Uuid> {
    Uuid::parse_str(s).ok()
}

/// Hash a token using SHA-256
pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

/// Extract access token from request (header or cookie)
fn get_access_token(headers: &HeaderMap, jar: &CookieJar) -> Option<String> {
    extract_token_from_header(headers).or_else(|| {
        jar.get(TOKEN_COOKIE_NAME).map(|c| c.value().to_string())
    })
}

/// Query public access rank for a resource (UUID-based)
async fn query_public_access(
    client: &reqwest::Client,
    oxigraph_url: &str,
    resource_uuid: &Uuid,
) -> Result<i32, String> {
    let query = format!(
        r#"PREFIX liqk: <http://liqk.org/schema#>
PREFIX posix: <http://www.w3.org/ns/posix/stat#>

SELECT (COALESCE(MAX(?rank), 0) AS ?accessRank)
FROM <{access_graph}>
FROM <{fs_graph}>
WHERE {{
  ?policy a liqk:AccessPolicy ;
          liqk:policy-type liqk:policy-type-public ;
          liqk:policy-target ?target ;
          liqk:access-level ?level .

  ?level liqk:rank ?rank .
  ?target posix:includes* <urn:uuid:{resource}> .
}}"#,
        access_graph = ACCESS_GRAPH,
        fs_graph = FILESYSTEM_GRAPH,
        resource = resource_uuid,
    );

    execute_access_query(client, oxigraph_url, &query).await
}

/// Query public access rank for an IRI resource
async fn query_public_access_iri(
    client: &reqwest::Client,
    oxigraph_url: &str,
    resource_iri: &str,
) -> Result<i32, String> {
    let query = format!(
        r#"PREFIX liqk: <http://liqk.org/schema#>

SELECT (COALESCE(MAX(?rank), 0) AS ?accessRank)
FROM <{access_graph}>
WHERE {{
  ?policy a liqk:AccessPolicy ;
          liqk:policy-type liqk:policy-type-public ;
          liqk:policy-target <{resource}> ;
          liqk:access-level ?level .

  ?level liqk:rank ?rank .
}}"#,
        access_graph = ACCESS_GRAPH,
        resource = resource_iri,
    );

    execute_access_query(client, oxigraph_url, &query).await
}

/// Query token-based access rank for a resource (UUID-based)
async fn query_token_access(
    client: &reqwest::Client,
    oxigraph_url: &str,
    resource_uuid: &Uuid,
    token_hash: &str,
) -> Result<i32, String> {
    let query = format!(
        r#"PREFIX liqk: <http://liqk.org/schema#>
PREFIX posix: <http://www.w3.org/ns/posix/stat#>

SELECT (COALESCE(MAX(?rank), 0) AS ?accessRank)
FROM <{access_graph}>
FROM <{fs_graph}>
WHERE {{
  ?policy a liqk:AccessPolicy ;
          liqk:policy-type liqk:policy-type-token ;
          liqk:policy-target ?target ;
          liqk:access-level ?level ;
          liqk:policy-grantee ?token .

  ?token a liqk:AccessToken ;
         liqk:token-hash "{token_hash}" .

  ?level liqk:rank ?rank .
  ?target posix:includes* <urn:uuid:{resource}> .
}}"#,
        access_graph = ACCESS_GRAPH,
        fs_graph = FILESYSTEM_GRAPH,
        resource = resource_uuid,
        token_hash = escape_sparql_string(token_hash),
    );

    execute_access_query(client, oxigraph_url, &query).await
}

/// Query token-based access rank for an IRI resource
async fn query_token_access_iri(
    client: &reqwest::Client,
    oxigraph_url: &str,
    resource_iri: &str,
    token_hash: &str,
) -> Result<i32, String> {
    let query = format!(
        r#"PREFIX liqk: <http://liqk.org/schema#>

SELECT (COALESCE(MAX(?rank), 0) AS ?accessRank)
FROM <{access_graph}>
WHERE {{
  ?policy a liqk:AccessPolicy ;
          liqk:policy-type liqk:policy-type-token ;
          liqk:policy-target <{resource}> ;
          liqk:access-level ?level ;
          liqk:policy-grantee ?token .

  ?token a liqk:AccessToken ;
         liqk:token-hash "{token_hash}" .

  ?level liqk:rank ?rank .
}}"#,
        access_graph = ACCESS_GRAPH,
        resource = resource_iri,
        token_hash = escape_sparql_string(token_hash),
    );

    execute_access_query(client, oxigraph_url, &query).await
}

/// Execute an access query and extract the rank
async fn execute_access_query(
    client: &reqwest::Client,
    oxigraph_url: &str,
    query: &str,
) -> Result<i32, String> {
    let query_url = format!("{}/query", oxigraph_url);

    let response = client
        .post(&query_url)
        .header("Content-Type", "application/sparql-query")
        .header("Accept", "application/sparql-results+json")
        .body(query.to_string())
        .send()
        .await
        .map_err(|e| format!("Failed to send access query: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Access query failed with status {}: {}", status, body));
    }

    let body = response.text().await.map_err(|e| format!("Failed to read response: {}", e))?;
    let json: serde_json::Value = serde_json::from_str(&body)
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    let rank = json
        .get("results")
        .and_then(|r| r.get("bindings"))
        .and_then(|b| b.get(0))
        .and_then(|b| b.get("accessRank"))
        .and_then(|r| r.get("value"))
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(0);

    Ok(rank)
}

/// Verify if a token hash exists in the RDF access graph
pub async fn verify_token_exists(
    client: &reqwest::Client,
    oxigraph_url: &str,
    token_hash: &str,
) -> bool {
    let query = format!(
        r#"PREFIX liqk: <http://liqk.org/schema#>

ASK
FROM <{access_graph}>
WHERE {{
  ?token a liqk:AccessToken ;
         liqk:token-hash "{token_hash}" .
}}"#,
        access_graph = ACCESS_GRAPH,
        token_hash = escape_sparql_string(token_hash),
    );

    let query_url = format!("{}/query", oxigraph_url);

    let response = match client
        .post(&query_url)
        .header("Content-Type", "application/sparql-query")
        .header("Accept", "application/sparql-results+json")
        .body(query)
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(_) => return false,
    };

    if !response.status().is_success() {
        return false;
    }

    let body = match response.text().await {
        Ok(b) => b,
        Err(_) => return false,
    };

    // ASK queries return {"boolean": true/false}
    serde_json::from_str::<serde_json::Value>(&body)
        .ok()
        .and_then(|json| json.get("boolean")?.as_bool())
        .unwrap_or(false)
}

/// Get maximum access rank for a resource (combining public and token access)
pub async fn get_access_rank(
    client: &reqwest::Client,
    oxigraph_url: &str,
    resource_uuid: &Uuid,
    headers: &HeaderMap,
    jar: &CookieJar,
) -> i32 {
    let public_rank = query_public_access(client, oxigraph_url, resource_uuid)
        .await
        .unwrap_or(0);

    let token_rank = if let Some(token) = get_access_token(headers, jar) {
        let token_hash = hash_token(&token);
        query_token_access(client, oxigraph_url, resource_uuid, &token_hash)
            .await
            .unwrap_or(0)
    } else {
        0
    };

    public_rank.max(token_rank)
}

/// Get maximum access rank for an IRI resource (combining public and token access)
pub async fn get_access_rank_iri(
    client: &reqwest::Client,
    oxigraph_url: &str,
    resource_iri: &str,
    headers: &HeaderMap,
    jar: &CookieJar,
) -> i32 {
    let public_rank = query_public_access_iri(client, oxigraph_url, resource_iri)
        .await
        .unwrap_or(0);

    let token_rank = if let Some(token) = get_access_token(headers, jar) {
        let token_hash = hash_token(&token);
        query_token_access_iri(client, oxigraph_url, resource_iri, &token_hash)
            .await
            .unwrap_or(0)
    } else {
        0
    };

    public_rank.max(token_rank)
}

/// Extract full extension from filename (e.g., "archive.tar.gz" -> "tar.gz")
fn extract_extension(filename: &str) -> Option<String> {
    let parts: Vec<&str> = filename.split('.').collect();
    if parts.len() > 1 {
        Some(parts[1..].join("."))
    } else {
        None
    }
}

/// Send SPARQL update to Oxigraph
async fn send_sparql_update(client: &reqwest::Client, oxigraph_url: &str, query: &str) -> Result<(), String> {
    let update_url = format!("{}/update", oxigraph_url);

    let response = client
        .post(&update_url)
        .header("Content-Type", "application/sparql-update")
        .body(query.to_string())
        .send()
        .await
        .map_err(|e| format!("Failed to send SPARQL update: {}", e))?;

    if response.status().is_success() {
        Ok(())
    } else {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        Err(format!("SPARQL update failed with status {}: {}", status, body))
    }
}

/// Lookup file by UUID and return stored filename
async fn lookup_file_by_uuid(
    client: &reqwest::Client,
    oxigraph_url: &str,
    uuid: &str,
) -> Result<Option<String>, String> {
    let query = format!(
        r#"PREFIX liqk: <http://liqk.org/schema#>

SELECT ?storedAs FROM <{}> WHERE {{
    <urn:uuid:{}> liqk:storedAs ?storedAs .
}}"#,
        FILESYSTEM_GRAPH, uuid
    );

    let query_url = format!("{}/query", oxigraph_url);

    let response = client
        .post(&query_url)
        .header("Content-Type", "application/sparql-query")
        .header("Accept", "application/sparql-results+json")
        .body(query)
        .send()
        .await
        .map_err(|e| format!("Failed to send SPARQL query: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("SPARQL query failed with status {}: {}", status, body));
    }

    let body = response.text().await.map_err(|e| format!("Failed to read response: {}", e))?;
    let json: serde_json::Value = serde_json::from_str(&body)
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    if let Some(stored_as) = json
        .get("results")
        .and_then(|r| r.get("bindings"))
        .and_then(|b| b.get(0))
        .and_then(|b| b.get("storedAs"))
        .and_then(|s| s.get("value"))
        .and_then(|v| v.as_str())
    {
        Ok(Some(stored_as.to_string()))
    } else {
        Ok(None)
    }
}

/// Update file size in RDF
async fn update_file_size(
    client: &reqwest::Client,
    oxigraph_url: &str,
    uuid: &Uuid,
    new_size: usize,
) -> Result<(), String> {
    let uuid_urn = format!("urn:uuid:{}", uuid);

    let query = format!(
        r#"PREFIX posix: <http://www.w3.org/ns/posix/stat#>

DELETE {{ GRAPH <{graph}> {{ <{uuid_urn}> posix:size ?oldSize }} }}
INSERT {{ GRAPH <{graph}> {{ <{uuid_urn}> posix:size {new_size} }} }}
WHERE {{ GRAPH <{graph}> {{ <{uuid_urn}> posix:size ?oldSize }} }}"#,
        graph = FILESYSTEM_GRAPH,
        uuid_urn = uuid_urn,
        new_size = new_size,
    );

    send_sparql_update(client, oxigraph_url, &query).await
}

/// Create SPARQL INSERT query for a new file (no directory linking)
fn build_file_insert(
    file_uuid: &Uuid,
    original_filename: &str,
    stored_filename: &str,
    file_size: usize,
    mime_type: &str,
    timestamp: &str,
) -> String {
    let uuid_urn = format!("urn:uuid:{}", file_uuid);

    format!(
        r#"PREFIX posix: <http://www.w3.org/ns/posix/stat#>
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>
PREFIX dc: <http://purl.org/dc/terms/>
PREFIX liqk: <http://liqk.org/schema#>

INSERT DATA {{
    GRAPH <{graph}> {{
        <{uuid_urn}> rdf:type posix:File ;
            rdfs:label "{filename}" ;
            posix:size {size} ;
            dc:format "{mime}" ;
            dc:created "{timestamp}"^^xsd:dateTime ;
            liqk:storedAs "{stored_filename}" .
    }}
}}"#,
        graph = FILESYSTEM_GRAPH,
        uuid_urn = uuid_urn,
        filename = escape_sparql_string(original_filename),
        size = file_size,
        mime = escape_sparql_string(mime_type),
        timestamp = timestamp,
        stored_filename = stored_filename,
    )
}

// =============================================================================
// Handlers
// =============================================================================

/// GET /res/:uuid - Download file by UUID
pub async fn res_handler(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    jar: CookieJar,
    headers: HeaderMap,
    Path(uuid_str): Path<String>,
) -> Response {
    let uuid = match validate_uuid(&uuid_str) {
        Some(u) => u,
        None => {
            warn!(client = %addr, uuid = %uuid_str, "Invalid UUID format");
            return (StatusCode::BAD_REQUEST, "Invalid UUID format").into_response();
        }
    };

    // Check access rank (requires >= 1 for view)
    let rank = get_access_rank(&state.client, &state.oxigraph_url, &uuid, &headers, &jar).await;
    if rank < 1 {
        warn!(client = %addr, uuid = %uuid, rank = rank, "Access denied - insufficient rank for view");
        return (StatusCode::FORBIDDEN, "Access denied").into_response();
    }

    let stored_filename = match lookup_file_by_uuid(&state.client, &state.oxigraph_url, &uuid.to_string()).await {
        Ok(Some(name)) => name,
        Ok(None) => {
            warn!(client = %addr, uuid = %uuid, "File not found");
            return (StatusCode::NOT_FOUND, "File not found").into_response();
        }
        Err(e) => {
            warn!(client = %addr, uuid = %uuid, error = %e, "SPARQL lookup failed");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to lookup file").into_response();
        }
    };

    let file_path = PathBuf::from(&state.files_dir).join(&stored_filename);

    match tokio::fs::read(&file_path).await {
        Ok(contents) => {
            let mime = mime_guess::from_path(&stored_filename)
                .first_or_octet_stream()
                .to_string();

            info!(client = %addr, uuid = %uuid, rank = rank, stored_as = %stored_filename, bytes = contents.len(), "File served");

            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, mime)],
                contents,
            )
                .into_response()
        }
        Err(e) => {
            warn!(client = %addr, uuid = %uuid, stored_as = %stored_filename, error = %e, "Failed to read file from disk");
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read file").into_response()
        }
    }
}

/// PUT /res/:uuid - Update existing file
pub async fn res_put_handler(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    jar: CookieJar,
    headers: HeaderMap,
    Path(uuid_str): Path<String>,
    body: axum::body::Bytes,
) -> Response {
    let uuid = match validate_uuid(&uuid_str) {
        Some(u) => u,
        None => {
            warn!(client = %addr, uuid = %uuid_str, "Invalid UUID format");
            return (StatusCode::BAD_REQUEST, "Invalid UUID format").into_response();
        }
    };

    // Check access rank (requires >= 3 for edit)
    let rank = get_access_rank(&state.client, &state.oxigraph_url, &uuid, &headers, &jar).await;
    if rank < 3 {
        warn!(client = %addr, uuid = %uuid, rank = rank, "Access denied - insufficient rank for edit");
        return (StatusCode::FORBIDDEN, "Access denied - edit requires higher access level").into_response();
    }

    if body.len() > MAX_UPLOAD_SIZE {
        warn!(client = %addr, uuid = %uuid, size = body.len(), "File too large");
        return (StatusCode::PAYLOAD_TOO_LARGE, "File too large (max 4 GB)").into_response();
    }

    let stored_filename = match lookup_file_by_uuid(&state.client, &state.oxigraph_url, &uuid.to_string()).await {
        Ok(Some(name)) => name,
        Ok(None) => {
            warn!(client = %addr, uuid = %uuid, "File not found");
            return (StatusCode::NOT_FOUND, "File not found").into_response();
        }
        Err(e) => {
            warn!(client = %addr, uuid = %uuid, error = %e, "SPARQL lookup failed");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to lookup file").into_response();
        }
    };

    let file_path = PathBuf::from(&state.files_dir).join(&stored_filename);
    let file_size = body.len();

    if let Err(e) = tokio::fs::write(&file_path, &body).await {
        warn!(client = %addr, uuid = %uuid, error = %e, "Failed to write file");
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to write file").into_response();
    }

    if let Err(e) = update_file_size(&state.client, &state.oxigraph_url, &uuid, file_size).await {
        warn!(client = %addr, uuid = %uuid, error = %e, "Failed to update file size in RDF");
    }

    info!(client = %addr, uuid = %uuid, rank = rank, stored_as = %stored_filename, bytes = file_size, "File updated");

    (StatusCode::OK, format!("File updated ({} bytes)", file_size)).into_response()
}

const UPLOAD_ACTION_IRI: &str = "http://liqk.org/schema#action-upload-file";

/// POST /res - Upload new file
pub async fn res_post_handler(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    jar: CookieJar,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Response {
    // Check access to upload action (requires >= 3 for upload)
    let rank = get_access_rank_iri(&state.client, &state.oxigraph_url, UPLOAD_ACTION_IRI, &headers, &jar).await;
    if rank < 3 {
        warn!(client = %addr, rank = rank, "Access denied - insufficient rank for upload");
        return (StatusCode::FORBIDDEN, "Access denied - upload requires edit access").into_response();
    }

    let files_dir = PathBuf::from(&state.files_dir);
    if let Err(e) = tokio::fs::create_dir_all(&files_dir).await {
        warn!(client = %addr, error = %e, "Failed to create files directory");
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create files directory").into_response();
    }

    let mut uploaded_files: Vec<(String, Uuid)> = Vec::new();
    let mut total_size: usize = 0;

    while let Ok(Some(field)) = multipart.next_field().await {
        let original_filename = match field.file_name() {
            Some(name) => name.to_string(),
            None => continue,
        };

        let safe_filename = PathBuf::from(&original_filename)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unnamed")
            .to_string();

        if safe_filename.is_empty() || safe_filename.starts_with('.') {
            warn!(client = %addr, filename = %original_filename, "Invalid filename");
            continue;
        }

        let file_uuid = Uuid::new_v4();
        let extension = extract_extension(&safe_filename).unwrap_or_else(|| "bin".to_string());
        let stored_filename = format!("{}.{}", file_uuid, extension);
        let file_path = files_dir.join(&stored_filename);

        let mut file = match tokio::fs::File::create(&file_path).await {
            Ok(f) => f,
            Err(e) => {
                warn!(client = %addr, filename = %safe_filename, error = %e, "Failed to create file");
                return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create file: {}", safe_filename)).into_response();
            }
        };

        let mut file_size: usize = 0;
        let mut stream = field;

        loop {
            match stream.chunk().await {
                Ok(Some(chunk)) => {
                    file_size += chunk.len();
                    total_size += chunk.len();

                    if total_size > MAX_UPLOAD_SIZE {
                        warn!(client = %addr, "Upload size limit exceeded");
                        let _ = tokio::fs::remove_file(&file_path).await;
                        return (StatusCode::PAYLOAD_TOO_LARGE, "Upload size limit exceeded (max 4 GB)").into_response();
                    }

                    if let Err(e) = file.write_all(&chunk).await {
                        warn!(client = %addr, filename = %safe_filename, error = %e, "Failed to write file");
                        let _ = tokio::fs::remove_file(&file_path).await;
                        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to write file").into_response();
                    }
                }
                Ok(None) => break,
                Err(e) => {
                    warn!(client = %addr, filename = %safe_filename, error = %e, "Failed to read upload stream");
                    let _ = tokio::fs::remove_file(&file_path).await;
                    return (StatusCode::BAD_REQUEST, "Failed to read upload").into_response();
                }
            }
        }

        if let Err(e) = file.flush().await {
            warn!(client = %addr, filename = %safe_filename, error = %e, "Failed to flush file");
        }

        let mime_type = mime_guess::from_path(&safe_filename)
            .first_or_octet_stream()
            .to_string();
        let timestamp = Utc::now().to_rfc3339();

        let sparql_query = build_file_insert(
            &file_uuid,
            &safe_filename,
            &stored_filename,
            file_size,
            &mime_type,
            &timestamp,
        );

        match send_sparql_update(&state.client, &state.oxigraph_url, &sparql_query).await {
            Ok(()) => {
                info!(
                    client = %addr,
                    filename = %safe_filename,
                    stored_as = %stored_filename,
                    uuid = %file_uuid,
                    bytes = file_size,
                    "File uploaded"
                );
            }
            Err(e) => {
                warn!(
                    client = %addr,
                    filename = %safe_filename,
                    error = %e,
                    "File uploaded but indexing failed"
                );
            }
        }

        uploaded_files.push((safe_filename, file_uuid));
    }

    if uploaded_files.is_empty() {
        return (StatusCode::BAD_REQUEST, "No files uploaded").into_response();
    }

    // Return JSON response
    let json_response = serde_json::json!({
        "success": true,
        "files": uploaded_files.iter().map(|(name, uuid)| {
            serde_json::json!({
                "filename": name,
                "uuid": uuid.to_string()
            })
        }).collect::<Vec<_>>()
    });

    (
        StatusCode::CREATED,
        [(header::CONTENT_TYPE, "application/json")],
        json_response.to_string(),
    ).into_response()
}
