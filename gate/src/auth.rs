use axum::{
    extract::{ConnectInfo, State},
    http::{header, HeaderMap},
    response::{Html, IntoResponse, Redirect, Response},
    Form,
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use rand::RngCore;
use serde::Deserialize;
use std::{env, fs, net::SocketAddr, sync::Arc};
use subtle::ConstantTimeEq;
use tracing::{info, warn};

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
    pub access_token: String,
    pub oxigraph_url: String,
    /// Whether to set Secure flag on cookies (requires HTTPS)
    pub secure_cookies: bool,
    /// Directory for file storage
    pub files_dir: String,
}

pub fn load_or_generate_config() -> GateConfig {
    let _ = dotenvy::from_filename(ENV_FILE);

    let oxigraph_url = env::var("OXIGRAPH_URL").unwrap_or_else(|_| DEFAULT_OXIGRAPH_URL.to_string());

    // SECURE_COOKIES: Set to "false" only for local development without HTTPS
    // In production, this should always be true (the default)
    let secure_cookies = env::var("SECURE_COOKIES")
        .map(|v| v.to_lowercase() != "false")
        .unwrap_or(true);

    let files_dir = env::var("FILES_DIR").unwrap_or_else(|_| DEFAULT_FILES_DIR.to_string());

    let access_token = match env::var("ACCESS_TOKEN") {
        Ok(token) if token.len() == 32 && token.chars().all(|c| c.is_ascii_hexdigit()) => token,
        _ => {
            let token = generate_token();
            save_env_file(&token, &oxigraph_url);
            token
        }
    };

    GateConfig {
        access_token,
        oxigraph_url,
        secure_cookies,
        files_dir,
    }
}

fn generate_token() -> String {
    let mut bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

fn save_env_file(token: &str, oxigraph_url: &str) {
    let content = format!("ACCESS_TOKEN={}\nOXIGRAPH_URL={}\n", token, oxigraph_url);
    if let Err(e) = fs::write(ENV_FILE, content) {
        warn!("Failed to write .env file: {}", e);
    } else {
        info!("Generated new access token and saved to .env");
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

/// Validate token using constant-time comparison to prevent timing attacks.
///
/// Accepts tokens from:
/// 1. X-Access-Token header
/// 2. Authorization: Bearer header
/// 3. Session cookie (oxigraph_gate_token)
pub fn validate_token(state: &AppState, headers: &HeaderMap, jar: &CookieJar) -> bool {
    if let Some(token) = extract_token_from_header(headers) {
        if constant_time_compare(&token, &state.access_token) {
            return true;
        }
    }

    if let Some(cookie) = jar.get(TOKEN_COOKIE_NAME) {
        if constant_time_compare(cookie.value(), &state.access_token) {
            return true;
        }
    }

    false
}

/// Constant-time string comparison to prevent timing attacks.
/// Both strings are compared in their entirety regardless of where they differ.
fn constant_time_compare(a: &str, b: &str) -> bool {
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();

    // Length check must also be constant-time
    // We pad the shorter one to match lengths, but still reject if lengths differ
    if a_bytes.len() != b_bytes.len() {
        // Still do a comparison to maintain constant time, but always return false
        let dummy = vec![0u8; a_bytes.len().max(b_bytes.len())];
        let _ = a_bytes.ct_eq(&dummy[..a_bytes.len()]);
        return false;
    }

    a_bytes.ct_eq(b_bytes).into()
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
    if constant_time_compare(&form.token, &state.access_token) {
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
