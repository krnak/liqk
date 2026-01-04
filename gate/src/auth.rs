use axum::{
    extract::{ConnectInfo, State},
    http::{header, HeaderMap},
    response::{Html, IntoResponse, Redirect, Response},
    Form,
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use rand::RngCore;
use serde::Deserialize;
use std::{env, fs, net::SocketAddr, sync::Arc};
use tracing::{info, warn};

use crate::templates::{LOGIN_ERROR_HTML, LOGIN_HTML};
use crate::AppState;

pub const DEFAULT_OXIGRAPH_URL: &str = "http://localhost:7878";
pub const ENV_FILE: &str = ".env";
pub const TOKEN_COOKIE_NAME: &str = "oxigraph_gate_token";

pub fn load_or_generate_config() -> (String, String) {
    let _ = dotenvy::from_filename(ENV_FILE);

    let oxigraph_url = env::var("OXIGRAPH_URL").unwrap_or_else(|_| DEFAULT_OXIGRAPH_URL.to_string());

    let access_token = match env::var("ACCESS_TOKEN") {
        Ok(token) if token.len() == 32 && token.chars().all(|c| c.is_ascii_hexdigit()) => token,
        _ => {
            let token = generate_token();
            save_env_file(&token, &oxigraph_url);
            token
        }
    };

    (access_token, oxigraph_url)
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

pub fn validate_token(state: &AppState, headers: &HeaderMap, jar: &CookieJar) -> bool {
    if let Some(token) = extract_token_from_header(headers) {
        if token == state.access_token {
            return true;
        }
    }

    if let Some(cookie) = jar.get(TOKEN_COOKIE_NAME) {
        if cookie.value() == state.access_token {
            return true;
        }
    }

    false
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
    if form.token == state.access_token {
        info!(client = %addr, "Login successful");

        let cookie = Cookie::build((TOKEN_COOKIE_NAME, form.token))
            .path("/")
            .http_only(true)
            .build();

        let jar = jar.add(cookie);

        (jar, Redirect::to("/")).into_response()
    } else {
        warn!(client = %addr, "Login failed - invalid token");
        Html(LOGIN_ERROR_HTML).into_response()
    }
}
