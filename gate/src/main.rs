mod auth;
mod files;
mod proxy;
mod templates;

use axum::{routing::{get, post}, Router};
use http::Method;
use reqwest::Client;
use std::{net::SocketAddr, sync::Arc};
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use auth::{load_or_generate_config, login_page, login_submit};
use files::{res_handler, res_post_handler, res_put_handler};
use proxy::proxy_handler;

const BIND_ADDR: &str = "0.0.0.0:8080";

pub struct AppState {
    pub access_token: String,
    pub oxigraph_url: String,
    pub client: Client,
    /// Whether to set Secure flag on cookies (requires HTTPS)
    pub secure_cookies: bool,
    /// Directory for file storage
    pub files_dir: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let config = load_or_generate_config();
    let client = Client::new();

    let files_path = std::fs::canonicalize(&config.files_dir)
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| config.files_dir.clone());

    let secure_mode = if config.secure_cookies { "HTTPS (Secure)" } else { "HTTP (Development)" };

    info!("┌──────────────────────────────────────────┐");
    info!("│         Oxigraph Gate Starting           │");
    info!("├──────────────────────────────────────────┤");
    info!("│ {:<40} │", format!("Listen:   http://{}", BIND_ADDR));
    info!("│ {:<40} │", format!("Upstream: {}", config.oxigraph_url));
    info!("│ {:<40} │", format!("Files:    {}", files_path));
    info!("│ {:<40} │", format!("Mode:     {}", secure_mode));
    info!("│ {:<40} │", format!("Token:  {}", config.access_token));
    info!("└──────────────────────────────────────────┘");

    if !config.secure_cookies {
        warn!("⚠️  Running in development mode (SECURE_COOKIES=false)");
        warn!("⚠️  Cookies will be sent over HTTP - NOT SAFE FOR PRODUCTION");
    }

    let state = Arc::new(AppState {
        access_token: config.access_token,
        oxigraph_url: config.oxigraph_url,
        client,
        secure_cookies: config.secure_cookies,
        files_dir: config.files_dir,
    });

    // CORS Configuration for SPARQL endpoint access
    // - allow_origin(Any): Required for SPARQL clients from any domain
    // - NOT setting allow_credentials: Cookies won't be sent cross-origin
    //   (Cross-origin clients must use X-Access-Token or Authorization header)
    // - This is safe because:
    //   1. Browser won't send cookies with cross-origin requests by default
    //   2. Cross-origin clients authenticate via headers, not cookies
    //   3. Same-origin requests (from the gate's own UI) use cookies normally
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_origin(Any)
        .allow_headers(Any)
        // Note: We explicitly do NOT call .allow_credentials(true)
        // This prevents cross-origin requests from sending cookies
        // Cross-origin SPARQL clients must use X-Access-Token header instead
        ;

    let app = Router::new()
        .route("/gate/login", get(login_page))
        .route("/gate/login", post(login_submit))
        .route("/res", post(res_post_handler))
        .route("/res/:uuid", get(res_handler).put(res_put_handler))
        .fallback(proxy_handler)
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(BIND_ADDR).await.unwrap();

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
