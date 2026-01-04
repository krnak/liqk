use axum::{
    extract::{ConnectInfo, Request, State},
    http::{header, HeaderMap, HeaderValue, Method, StatusCode},
    response::{IntoResponse, Response},
};
use axum_extra::extract::cookie::CookieJar;
use std::{net::SocketAddr, sync::Arc};
use tracing::{info, warn};

use crate::auth::validate_token;
use crate::AppState;

pub async fn proxy_handler(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    jar: CookieJar,
    req: Request,
) -> Response {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let path_and_query = uri
        .path_and_query()
        .map(|pq| pq.as_str())
        .unwrap_or("/");

    if !validate_token(&state, req.headers(), &jar) {
        warn!(
            client = %addr,
            method = %method,
            path = %path_and_query,
            "Unauthorized request - redirecting to login"
        );
        return (StatusCode::SEE_OTHER, [(header::LOCATION, "/gate/login")]).into_response();
    }

    let headers = req.headers().clone();
    let target_url = format!("{}{}", state.oxigraph_url, path_and_query);

    let body_bytes = match axum::body::to_bytes(req.into_body(), usize::MAX).await {
        Ok(bytes) => bytes,
        Err(e) => {
            warn!(
                client = %addr,
                method = %method,
                path = %path_and_query,
                error = %e,
                "Failed to read request body"
            );
            return (
                StatusCode::BAD_REQUEST,
                format!("Failed to read request body: {}", e),
            )
                .into_response();
        }
    };

    let mut proxy_req = state.client.request(method_to_reqwest(&method), &target_url);

    for (name, value) in headers.iter() {
        if should_forward_header(name.as_str()) {
            if let Ok(v) = value.to_str() {
                proxy_req = proxy_req.header(name.as_str(), v);
            }
        }
    }

    if !body_bytes.is_empty() {
        proxy_req = proxy_req.body(body_bytes);
    }

    match proxy_req.send().await {
        Ok(resp) => {
            let status = StatusCode::from_u16(resp.status().as_u16()).unwrap_or(StatusCode::OK);
            let mut response_headers = HeaderMap::new();

            for (name, value) in resp.headers().iter() {
                if let Ok(v) = HeaderValue::from_str(value.to_str().unwrap_or("")) {
                    if should_forward_header(name.as_str()) {
                        response_headers.insert(name.clone(), v);
                    }
                }
            }

            match resp.bytes().await {
                Ok(body) => {
                    info!(
                        client = %addr,
                        method = %method,
                        path = %path_and_query,
                        status = %status,
                        bytes = body.len(),
                        "Request proxied"
                    );
                    (status, response_headers, body).into_response()
                }
                Err(e) => {
                    warn!(
                        client = %addr,
                        method = %method,
                        path = %path_and_query,
                        error = %e,
                        "Failed to read response body"
                    );
                    (
                        StatusCode::BAD_GATEWAY,
                        format!("Failed to read response body: {}", e),
                    )
                        .into_response()
                }
            }
        }
        Err(e) => {
            warn!(
                client = %addr,
                method = %method,
                path = %path_and_query,
                error = %e,
                "Proxy request failed"
            );
            (
                StatusCode::BAD_GATEWAY,
                format!("Proxy request failed: {}", e),
            )
                .into_response()
        }
    }
}

fn method_to_reqwest(method: &Method) -> reqwest::Method {
    match *method {
        Method::GET => reqwest::Method::GET,
        Method::POST => reqwest::Method::POST,
        Method::PUT => reqwest::Method::PUT,
        Method::DELETE => reqwest::Method::DELETE,
        Method::HEAD => reqwest::Method::HEAD,
        Method::OPTIONS => reqwest::Method::OPTIONS,
        Method::PATCH => reqwest::Method::PATCH,
        _ => reqwest::Method::GET,
    }
}

fn should_forward_header(name: &str) -> bool {
    let name_lower = name.to_lowercase();
    !matches!(
        name_lower.as_str(),
        "host"
            | "connection"
            | "keep-alive"
            | "proxy-authenticate"
            | "proxy-authorization"
            | "te"
            | "trailers"
            | "transfer-encoding"
            | "upgrade"
            | "x-access-token"
            | "authorization"
            | "cookie"
    )
}
