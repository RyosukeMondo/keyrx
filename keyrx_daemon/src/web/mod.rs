#[cfg(feature = "web")]
pub mod api;
#[cfg(feature = "web")]
pub mod ws;
#[cfg(feature = "web")]
pub mod static_files;

#[cfg(feature = "web")]
use axum::{Router, routing::get};
#[cfg(feature = "web")]
use std::net::SocketAddr;

#[cfg(feature = "web")]
pub async fn create_app() -> Router {
    Router::new()
        .route("/", get(|| async { "KeyRx Daemon Web Server" }))
        .nest("/api", api::create_router())
        .nest("/ws", ws::create_router())
        .fallback_service(static_files::serve_static())
}

#[cfg(feature = "web")]
pub async fn serve(addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    let app = create_app().await;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
