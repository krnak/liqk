use axum::{
    extract::{ConnectInfo, State},
    http::{header, HeaderMap},
    response::{Html, IntoResponse, Redirect, Response},
    Form,
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use serde::Deserialize;
use std::{env, net::SocketAddr, sync::Arc};
use tracing::{info, warn};

use crate::files::{hash_token, verify_token_exists};
use crate::templates::{LOGIN_ERROR_HTML, LOGIN_HTML};
use crate::AppState;

pub const DEFAULT_OXIGRAPH_URL: &str = "http://localhost:7878";
pub const ENV_FILE: &str = ".env";
pub const TOKEN_COOKIE_NAME: &str = "oxigraph_gate_token";
/// Session cookie max age in seconds (3 months / ~90 days)
pub const SESSION_MAX_AGE_SECS: i64 = 7_776_000;

pub const DEFAULT_FILES_DIR: &str = "../files";

/// Configuration loaded from environment
pub struct GateConfig {
    pub oxigraph_url: String,
    /// Whether to set Secure flag on cookies (requires HTTPS)
    pub secure_cookies: bool,
    /// Directory for file storage
    pub files_dir: String,
}

pub fn load_config() -> GateConfig {
    let _ = dotenvy::from_filename(ENV_FILE);

    let oxigraph_url = env::var("OXIGRAPH_URL").unwrap_or_else(|_| DEFAULT_OXIGRAPH_URL.to_string());

    // SECURE_COOKIES: Set to "false" only for local development without HTTPS
    // In production, this should always be true (the default)
    let secure_cookies = env::var("SECURE_COOKIES")
        .map(|v| v.to_lowercase() != "false")
        .unwrap_or(true);

    let files_dir = env::var("FILES_DIR").unwrap_or_else(|_| DEFAULT_FILES_DIR.to_string());

    GateConfig {
        oxigraph_url,
        secure_cookies,
        files_dir,
    }
}

pub fn extract_token_from_header(headers: &HeaderMap) -> Option<String> {
    headers
        .get("X-Access-Token")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .or_else(|| {
            headers
                .get(header::AUTHORIZATION)
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.strip_prefix("Bearer "))
                .map(|s| s.to_string())
        })
}

pub async fn login_page() -> Html<&'static str> {
    Html(LOGIN_HTML)
}

#[derive(Deserialize)]
pub struct LoginForm {
    token: String,
}

pub async fn login_submit(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    jar: CookieJar,
    Form(form): Form<LoginForm>,
) -> Response {
    // Hash the submitted token and verify it exists in the RDF access graph
    let token_hash = hash_token(&form.token);
    let token_exists = verify_token_exists(&state.client, &state.oxigraph_url, &token_hash).await;

    if token_exists {
        info!(client = %addr, "Login successful");

        // Build secure cookie with all security flags
        let mut cookie_builder = Cookie::build((TOKEN_COOKIE_NAME, form.token))
            .path("/")
            .http_only(true)  // Prevent JavaScript access (XSS protection)
            .same_site(SameSite::Strict)  // CSRF protection
            .max_age(time::Duration::seconds(SESSION_MAX_AGE_SECS));  // Session expiration

        // Only set Secure flag if configured (requires HTTPS)
        // In development without HTTPS, set SECURE_COOKIES=false
        if state.secure_cookies {
            cookie_builder = cookie_builder.secure(true);
        }

        let cookie = cookie_builder.build();
        let jar = jar.add(cookie);

        (jar, Redirect::to("/")).into_response()
    } else {
        warn!(client = %addr, "Login failed - invalid token");
        Html(LOGIN_ERROR_HTML).into_response()
    }
}
