use axum::{
    extract::{ConnectInfo, Request, State},
    http::{header, HeaderMap, HeaderValue, Method, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
    Form, Router,
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use rand::RngCore;
use reqwest::Client;
use serde::Deserialize;
use std::{env, fs, net::SocketAddr, sync::Arc};
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const DEFAULT_OXIGRAPH_URL: &str = "http://localhost:7878";
const ENV_FILE: &str = ".env";
const TOKEN_COOKIE_NAME: &str = "oxigraph_gate_token";

struct AppState {
    access_token: String,
    oxigraph_url: String,
    client: Client,
}

const BIND_ADDR: &str = "0.0.0.0:8080";

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let (access_token, oxigraph_url) = load_or_generate_config();

    info!("========================================");
    info!("        Oxigraph Gate Starting");
    info!("========================================");
    info!("Listen URL:    http://{}", BIND_ADDR);
    info!("Oxigraph URL:  {}", oxigraph_url);
    info!("Access Token:  {}", access_token);
    info!("========================================");

    let state = Arc::new(AppState {
        access_token,
        oxigraph_url,
        client: Client::new(),
    });

    let app = Router::new()
        .route("/gate/login", get(login_page))
        .route("/gate/login", post(login_submit))
        .fallback(proxy_handler)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(BIND_ADDR).await.unwrap();

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

fn load_or_generate_config() -> (String, String) {
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

fn extract_token_from_header(headers: &HeaderMap) -> Option<String> {
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

fn validate_token(state: &AppState, headers: &HeaderMap, jar: &CookieJar) -> bool {
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

async fn login_page() -> Html<&'static str> {
    Html(LOGIN_HTML)
}

#[derive(Deserialize)]
struct LoginForm {
    token: String,
}

async fn login_submit(
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

async fn proxy_handler(
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

const LOGIN_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Oxigraph Gate - Login</title>
    <style>
        * {
            box-sizing: border-box;
        }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: #1a1a2e;
            color: #eee;
            min-height: 100vh;
            margin: 0;
            display: flex;
            align-items: center;
            justify-content: center;
        }
        .container {
            text-align: center;
            background: #16213e;
            padding: 3rem;
            border-radius: 12px;
            box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
            max-width: 400px;
            width: 90%;
        }
        h1 {
            margin: 0 0 0.5rem 0;
            color: #e94560;
            font-size: 1.8rem;
        }
        p {
            margin: 0 0 2rem 0;
            color: #aaa;
        }
        input[type="text"] {
            width: 100%;
            padding: 0.875rem;
            font-size: 1rem;
            font-family: monospace;
            border: 2px solid #0f3460;
            border-radius: 6px;
            background: #1a1a2e;
            color: #eee;
            text-align: center;
            margin-bottom: 1rem;
        }
        input[type="text"]:focus {
            outline: none;
            border-color: #e94560;
        }
        button {
            width: 100%;
            padding: 0.875rem;
            font-size: 1rem;
            font-weight: 600;
            background: #e94560;
            color: white;
            border: none;
            border-radius: 6px;
            cursor: pointer;
            transition: background 0.2s;
        }
        button:hover {
            background: #ff6b6b;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>Oxigraph Gate</h1>
        <p>Enter your access token to continue</p>
        <form method="POST" action="/gate/login">
            <input type="text" name="token" placeholder="Access Token" autocomplete="off" required>
            <button type="submit">Authenticate</button>
        </form>
    </div>
</body>
</html>
"#;

const LOGIN_ERROR_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Oxigraph Gate - Login Failed</title>
    <style>
        * {
            box-sizing: border-box;
        }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: #1a1a2e;
            color: #eee;
            min-height: 100vh;
            margin: 0;
            display: flex;
            align-items: center;
            justify-content: center;
        }
        .container {
            text-align: center;
            background: #16213e;
            padding: 3rem;
            border-radius: 12px;
            box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
            max-width: 400px;
            width: 90%;
        }
        h1 {
            margin: 0 0 0.5rem 0;
            color: #e94560;
            font-size: 1.8rem;
        }
        p {
            margin: 0 0 2rem 0;
            color: #ff6b6b;
        }
        input[type="text"] {
            width: 100%;
            padding: 0.875rem;
            font-size: 1rem;
            font-family: monospace;
            border: 2px solid #e94560;
            border-radius: 6px;
            background: #1a1a2e;
            color: #eee;
            text-align: center;
            margin-bottom: 1rem;
        }
        input[type="text"]:focus {
            outline: none;
            border-color: #e94560;
        }
        button {
            width: 100%;
            padding: 0.875rem;
            font-size: 1rem;
            font-weight: 600;
            background: #e94560;
            color: white;
            border: none;
            border-radius: 6px;
            cursor: pointer;
            transition: background 0.2s;
        }
        button:hover {
            background: #ff6b6b;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>Oxigraph Gate</h1>
        <p>Invalid token. Please try again.</p>
        <form method="POST" action="/gate/login">
            <input type="text" name="token" placeholder="Access Token" autocomplete="off" required>
            <button type="submit">Authenticate</button>
        </form>
    </div>
</body>
</html>
"#;
