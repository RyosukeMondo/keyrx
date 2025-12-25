pub mod api;
pub mod static_files;
pub mod ws;

use axum::{routing::get, Router};
use std::net::SocketAddr;

#[allow(dead_code)]
pub async fn create_app() -> Router {
    Router::new()
        .route("/", get(|| async { "KeyRx Daemon Web Server" }))
        .nest("/api", api::create_router())
        .nest("/ws", ws::create_router())
        .fallback_service(static_files::serve_static())
}

#[allow(dead_code)]
pub async fn serve(addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    let app = create_app().await;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
