use axum::{
    extract::{ConnectInfo, Multipart, Path, State},
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
};
use axum_extra::extract::cookie::CookieJar;
use chrono::Utc;
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use tokio::io::AsyncWriteExt;
use tracing::{info, warn};
use uuid::Uuid;

use crate::auth::validate_token;
use crate::templates::{upload_success_html, UPLOAD_HTML};
use crate::AppState;

pub const FILES_DIR: &str = "../files";
pub const MAX_UPLOAD_SIZE: usize = 4 * 1024 * 1024 * 1024; // 4 GB
const FILESYSTEM_GRAPH: &str = "http://liqk.org/graph/filesystem";

/// Extract full extension from filename (e.g., "archive.tar.gz" -> "tar.gz")
fn extract_extension(filename: &str) -> Option<String> {
    let parts: Vec<&str> = filename.split('.').collect();
    if parts.len() > 1 {
        Some(parts[1..].join("."))
    } else {
        None
    }
}

/// Create SPARQL INSERT query for the uploaded file
fn build_sparql_insert(
    file_uuid: &Uuid,
    original_filename: &str,
    extension: &str,
    file_size: usize,
    mime_type: &str,
    upload_timestamp: &str,
    upload_dir_uuid: &str,
) -> String {
    let uuid_urn = format!("urn:uuid:{}", file_uuid);
    let stored_filename = format!("{}.{}", file_uuid, extension);

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
        <{upload_dir}> posix:includes <{uuid_urn}> .
    }}
}}"#,
        graph = FILESYSTEM_GRAPH,
        uuid_urn = uuid_urn,
        filename = original_filename.replace('"', "\\\""),
        size = file_size,
        mime = mime_type,
        timestamp = upload_timestamp,
        stored_filename = stored_filename,
        upload_dir = upload_dir_uuid,
    )
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

/// Build SPARQL query to resolve file path by traversing rdfs:label and get stored filename
fn build_file_lookup_query(dir_labels: &[&str], filename: &str) -> String {
    let mut query = format!(
        r#"PREFIX posix: <http://www.w3.org/ns/posix/stat#>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
PREFIX liqk: <http://liqk.org/schema#>

SELECT ?storedAs FROM <{}> WHERE {{
    ?root a posix:Directory ;
          rdfs:label "/" .
"#,
        FILESYSTEM_GRAPH
    );

    // Start from root
    let mut current_var = "?root".to_string();

    // Traverse through directories by label
    for (i, label) in dir_labels.iter().enumerate() {
        let next_var = format!("?dir{}", i);
        query.push_str(&format!(
            "    {} posix:includes {} .\n    {} rdfs:label \"{}\" .\n",
            current_var,
            next_var,
            next_var,
            label.replace('"', "\\\"")
        ));
        current_var = next_var;
    }

    // Final: directory includes file with matching label
    query.push_str(&format!(
        r#"    {} posix:includes ?file .
    ?file rdfs:label "{}" .
    ?file liqk:storedAs ?storedAs .
}}"#,
        current_var,
        filename.replace('"', "\\\"")
    ));

    query
}

/// Execute SPARQL query and extract stored filename
async fn lookup_file(
    client: &reqwest::Client,
    oxigraph_url: &str,
    dir_labels: &[&str],
    filename: &str,
) -> Result<Option<String>, String> {
    let query = build_file_lookup_query(dir_labels, filename);
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

    // Parse JSON response
    let json: serde_json::Value = serde_json::from_str(&body)
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    // Extract storedAs value from first binding
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

/// Directory entry with type and label
#[derive(Debug)]
struct DirEntry {
    label: String,
    is_dir: bool,
}

/// Check if path is a directory
async fn is_directory(
    client: &reqwest::Client,
    oxigraph_url: &str,
    path_parts: &[&str],
) -> Result<bool, String> {
    let mut query = format!(
        r#"PREFIX posix: <http://www.w3.org/ns/posix/stat#>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

ASK FROM <{}> WHERE {{
    ?root a posix:Directory ;
          rdfs:label "/" .
"#,
        FILESYSTEM_GRAPH
    );

    let mut current_var = "?root".to_string();

    for (i, label) in path_parts.iter().enumerate() {
        let next_var = format!("?dir{}", i);
        query.push_str(&format!(
            "    {} posix:includes {} .\n    {} rdfs:label \"{}\" .\n    {} a posix:Directory .\n",
            current_var,
            next_var,
            next_var,
            label.replace('"', "\\\""),
            next_var
        ));
        current_var = next_var;
    }

    query.push_str("}");

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

    Ok(json.get("boolean").and_then(|b| b.as_bool()).unwrap_or(false))
}

/// Lookup directory contents (assumes directory exists)
async fn lookup_directory_contents(
    client: &reqwest::Client,
    oxigraph_url: &str,
    path_parts: &[&str],
) -> Result<Vec<DirEntry>, String> {
    let mut query = format!(
        r#"PREFIX posix: <http://www.w3.org/ns/posix/stat#>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

SELECT ?label ?isDir FROM <{}> WHERE {{
    ?root a posix:Directory ;
          rdfs:label "/" .
"#,
        FILESYSTEM_GRAPH
    );

    let mut current_var = "?root".to_string();

    for (i, label) in path_parts.iter().enumerate() {
        let next_var = format!("?dir{}", i);
        query.push_str(&format!(
            "    {} posix:includes {} .\n    {} rdfs:label \"{}\" .\n    {} a posix:Directory .\n",
            current_var,
            next_var,
            next_var,
            label.replace('"', "\\\""),
            next_var
        ));
        current_var = next_var;
    }

    query.push_str(&format!(
        r#"    {} posix:includes ?child .
    ?child rdfs:label ?label .
    BIND(EXISTS {{ ?child a posix:Directory }} AS ?isDir)
}}"#,
        current_var
    ));

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

    let bindings = json
        .get("results")
        .and_then(|r| r.get("bindings"))
        .and_then(|b| b.as_array())
        .map(|b| b.as_slice())
        .unwrap_or(&[]);

    let mut entries: Vec<DirEntry> = bindings
        .iter()
        .filter_map(|b| {
            let label = b.get("label")?.get("value")?.as_str()?.to_string();
            let is_dir = b.get("isDir")
                .and_then(|d| d.get("value"))
                .and_then(|v| v.as_str())
                .map(|s| s == "true")
                .unwrap_or(false);
            Some(DirEntry { label, is_dir })
        })
        .collect();

    // Sort: directories first, then alphabetically
    entries.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.label.to_lowercase().cmp(&b.label.to_lowercase()),
        }
    });

    Ok(entries)
}

/// Generate HTML for directory listing
fn render_directory_html(path: &str, entries: &[DirEntry]) -> String {
    let display_path = if path.is_empty() { "/" } else { path };
    let parent_path = if path.is_empty() || path == "/" {
        "/file/".to_string()
    } else {
        let parts: Vec<&str> = path.trim_matches('/').split('/').collect();
        if parts.len() <= 1 {
            "/file/".to_string()
        } else {
            format!("/file/{}", parts[..parts.len() - 1].join("/"))
        }
    };

    let current_path = if path.is_empty() {
        "/file/".to_string()
    } else {
        format!("/file/{}", path.trim_matches('/'))
    };

    let mut items_html = String::new();

    // Add . and ..
    items_html.push_str(&format!(
        r#"<li><a href="{}">.</a></li>"#,
        current_path
    ));
    items_html.push_str(&format!(
        r#"<li><a href="{}">..</a></li>"#,
        parent_path
    ));

    // Add entries
    for entry in entries {
        let entry_path = if path.is_empty() {
            format!("/file/{}", entry.label)
        } else {
            format!("/file/{}/{}", path.trim_matches('/'), entry.label)
        };
        let icon = if entry.is_dir { "📁" } else { "📄" };
        let suffix = if entry.is_dir { "/" } else { "" };
        items_html.push_str(&format!(
            r#"<li>{} <a href="{}">{}{}</a></li>"#,
            icon, entry_path, entry.label, suffix
        ));
    }

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Index of {}</title>
    <style>
        body {{ font-family: monospace; margin: 2em; background: #1a1a2e; color: #eee; }}
        h1 {{ color: #00d4ff; }}
        ul {{ list-style: none; padding: 0; }}
        li {{ padding: 0.3em 0; }}
        a {{ color: #00d4ff; text-decoration: none; }}
        a:hover {{ text-decoration: underline; }}
    </style>
</head>
<body>
    <h1>Index of {}</h1>
    <ul>
        {}
    </ul>
</body>
</html>"#,
        display_path, display_path, items_html
    )
}

/// Lookup a child directory of root by label
async fn lookup_child_dir_of_root(
    client: &reqwest::Client,
    oxigraph_url: &str,
    label: &str,
) -> Result<Option<String>, String> {
    let query = format!(
        r#"PREFIX posix: <http://www.w3.org/ns/posix/stat#>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

SELECT ?dir FROM <{}> WHERE {{
    ?root a posix:Directory ;
          rdfs:label "/" .
    ?root posix:includes ?dir .
    ?dir a posix:Directory .
    ?dir rdfs:label "{}" .
}}"#,
        FILESYSTEM_GRAPH,
        label.replace('"', "\\\"")
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

    if let Some(dir_uuid) = json
        .get("results")
        .and_then(|r| r.get("bindings"))
        .and_then(|b| b.get(0))
        .and_then(|b| b.get("dir"))
        .and_then(|d| d.get("value"))
        .and_then(|v| v.as_str())
    {
        Ok(Some(dir_uuid.to_string()))
    } else {
        Ok(None)
    }
}

pub async fn file_root_handler(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    jar: CookieJar,
    headers: HeaderMap,
) -> Response {
    if !validate_token(&state, &headers, &jar) {
        warn!(client = %addr, "Unauthorized file request");
        return (StatusCode::SEE_OTHER, [(header::LOCATION, "/gate/login")]).into_response();
    }

    // List root directory
    match lookup_directory_contents(&state.client, &state.oxigraph_url, &[]).await {
        Ok(entries) => {
            info!(client = %addr, entries = entries.len(), "Root directory listed");
            let html = render_directory_html("", &entries);
            Html(html).into_response()
        }
        Err(e) => {
            warn!(client = %addr, error = %e, "Failed to list root directory");
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to list directory").into_response()
        }
    }
}

pub async fn file_handler(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    jar: CookieJar,
    headers: HeaderMap,
    Path(path): Path<String>,
) -> Response {
    if !validate_token(&state, &headers, &jar) {
        warn!(client = %addr, path = %path, "Unauthorized file request");
        return (StatusCode::SEE_OTHER, [(header::LOCATION, "/gate/login")]).into_response();
    }

    // Split path into parts
    let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    // Check if path is a directory
    if let Ok(true) = is_directory(&state.client, &state.oxigraph_url, &parts).await {
        match lookup_directory_contents(&state.client, &state.oxigraph_url, &parts).await {
            Ok(entries) => {
                info!(client = %addr, path = %path, entries = entries.len(), "Directory listed");
                let html = render_directory_html(&path, &entries);
                return Html(html).into_response();
            }
            Err(e) => {
                warn!(client = %addr, path = %path, error = %e, "Failed to list directory");
                return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to list directory").into_response();
            }
        }
    }

    // If no parts, redirect to root handler
    if parts.is_empty() {
        let html = render_directory_html("", &[]);
        return Html(html).into_response();
    }

    // Try as file: last part is filename, rest are directories
    let filename = parts[parts.len() - 1];
    let dir_labels = &parts[..parts.len() - 1];

    // Lookup file in SPARQL graph
    let stored_filename = match lookup_file(
        &state.client,
        &state.oxigraph_url,
        dir_labels,
        filename,
    ).await {
        Ok(Some(name)) => name,
        Ok(None) => {
            warn!(client = %addr, path = %path, "Not found in graph");
            return (StatusCode::NOT_FOUND, "Not found").into_response();
        }
        Err(e) => {
            warn!(client = %addr, path = %path, error = %e, "SPARQL lookup failed");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to lookup").into_response();
        }
    };

    // Read file from disk
    let file_path = PathBuf::from(FILES_DIR).join(&stored_filename);

    match tokio::fs::read(&file_path).await {
        Ok(contents) => {
            let mime = mime_guess::from_path(&stored_filename)
                .first_or_octet_stream()
                .to_string();

            info!(client = %addr, path = %path, stored_as = %stored_filename, bytes = contents.len(), "File served");

            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, mime)],
                contents,
            )
                .into_response()
        }
        Err(e) => {
            warn!(client = %addr, path = %path, stored_as = %stored_filename, error = %e, "Failed to read file from disk");
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read file").into_response()
        }
    }
}

pub async fn res_handler(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    jar: CookieJar,
    headers: HeaderMap,
    Path(uuid): Path<String>,
) -> Response {
    if !validate_token(&state, &headers, &jar) {
        warn!(client = %addr, uuid = %uuid, "Unauthorized res request");
        return (StatusCode::SEE_OTHER, [(header::LOCATION, "/gate/login")]).into_response();
    }

    // Lookup file by UUID
    let stored_filename = match lookup_file_by_uuid(&state.client, &state.oxigraph_url, &uuid).await {
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

    // Read file from disk
    let file_path = PathBuf::from(FILES_DIR).join(&stored_filename);

    match tokio::fs::read(&file_path).await {
        Ok(contents) => {
            let mime = mime_guess::from_path(&stored_filename)
                .first_or_octet_stream()
                .to_string();

            info!(client = %addr, uuid = %uuid, stored_as = %stored_filename, bytes = contents.len(), "File served");

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

pub async fn upload_page(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    jar: CookieJar,
    headers: HeaderMap,
) -> Response {
    if !validate_token(&state, &headers, &jar) {
        warn!(client = %addr, "Unauthorized upload page request");
        return (StatusCode::SEE_OTHER, [(header::LOCATION, "/gate/login")]).into_response();
    }

    Html(UPLOAD_HTML).into_response()
}

pub async fn upload_handler(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    jar: CookieJar,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Response {
    if !validate_token(&state, &headers, &jar) {
        warn!(client = %addr, "Unauthorized upload request");
        return (StatusCode::SEE_OTHER, [(header::LOCATION, "/gate/login")]).into_response();
    }

    // Lookup "upload" directory UUID
    let upload_dir_uuid = match lookup_child_dir_of_root(
        &state.client,
        &state.oxigraph_url,
        "upload",
    ).await {
        Ok(Some(uuid)) => uuid,
        Ok(None) => {
            warn!(client = %addr, "Upload directory not found in graph");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Upload directory not configured").into_response();
        }
        Err(e) => {
            warn!(client = %addr, error = %e, "Failed to lookup upload directory");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to lookup upload directory").into_response();
        }
    };

    let files_dir = PathBuf::from(FILES_DIR);
    if let Err(e) = tokio::fs::create_dir_all(&files_dir).await {
        warn!(client = %addr, error = %e, "Failed to create files directory");
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create files directory").into_response();
    }

    let mut uploaded_files = Vec::new();
    let mut total_size: usize = 0;

    while let Ok(Some(field)) = multipart.next_field().await {
        let original_filename = match field.file_name() {
            Some(name) => name.to_string(),
            None => continue,
        };

        // Sanitize filename - only keep the basename
        let safe_filename = PathBuf::from(&original_filename)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unnamed")
            .to_string();

        if safe_filename.is_empty() || safe_filename.starts_with('.') {
            warn!(client = %addr, filename = %original_filename, "Invalid filename");
            continue;
        }

        // Generate UUID and extract extension
        let file_uuid = Uuid::new_v4();
        let extension = extract_extension(&safe_filename).unwrap_or_else(|| "bin".to_string());
        let stored_filename = format!("{}.{}", file_uuid, extension);
        let file_path = files_dir.join(&stored_filename);

        // Stream the file to disk to handle large files
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

        // Get MIME type and timestamp
        let mime_type = mime_guess::from_path(&safe_filename)
            .first_or_octet_stream()
            .to_string();
        let upload_timestamp = Utc::now().to_rfc3339();

        // Create SPARQL INSERT and send to Oxigraph
        let sparql_query = build_sparql_insert(
            &file_uuid,
            &safe_filename,
            &extension,
            file_size,
            &mime_type,
            &upload_timestamp,
            &upload_dir_uuid,
        );

        match send_sparql_update(&state.client, &state.oxigraph_url, &sparql_query).await {
            Ok(()) => {
                info!(
                    client = %addr,
                    filename = %safe_filename,
                    stored_as = %stored_filename,
                    uuid = %file_uuid,
                    bytes = file_size,
                    "File uploaded and indexed"
                );
            }
            Err(e) => {
                warn!(
                    client = %addr,
                    filename = %safe_filename,
                    error = %e,
                    "File uploaded but SPARQL indexing failed"
                );
            }
        }

        uploaded_files.push(safe_filename);
    }

    if uploaded_files.is_empty() {
        return (StatusCode::BAD_REQUEST, "No files uploaded").into_response();
    }

    let message = format!("Uploaded {} file(s): {}", uploaded_files.len(), uploaded_files.join(", "));
    Html(upload_success_html(&message)).into_response()
}
