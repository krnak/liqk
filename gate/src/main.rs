mod auth;
mod files;
mod proxy;
mod templates;

use axum::{routing::{get, post}, Router};
use reqwest::Client;
use std::{net::SocketAddr, sync::Arc};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use auth::{load_or_generate_config, login_page, login_submit};
use files::{file_handler, file_root_handler, res_handler, upload_handler, upload_page};
use proxy::proxy_handler;

const BIND_ADDR: &str = "0.0.0.0:8080";

pub struct AppState {
    pub access_token: String,
    pub oxigraph_url: String,
    pub client: Client,
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

    let (access_token, oxigraph_url) = load_or_generate_config();
    let client = Client::new();

    let files_path = std::fs::canonicalize(files::FILES_DIR)
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| files::FILES_DIR.to_string());

    info!("┌──────────────────────────────────────────┐");
    info!("│         Oxigraph Gate Starting           │");
    info!("├──────────────────────────────────────────┤");
    info!("│ {:<40} │", format!("Listen:   http://{}", BIND_ADDR));
    info!("│ {:<40} │", format!("Upstream: {}", oxigraph_url));
    info!("│ {:<40} │", format!("Files:    {}", files_path));
    info!("│ {:<40} │", format!("Token:  {}", access_token));
    info!("└──────────────────────────────────────────┘");

    let state = Arc::new(AppState {
        access_token,
        oxigraph_url,
        client,
    });

    let app = Router::new()
        .route("/gate/login", get(login_page))
        .route("/gate/login", post(login_submit))
        .route("/file", get(file_root_handler))
        .route("/file/", get(file_root_handler))
        .route("/file/*path", get(file_handler))
        .route("/res/{uuid}", get(res_handler))
        .route("/upload", get(upload_page))
        .route("/upload", post(upload_handler))
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
