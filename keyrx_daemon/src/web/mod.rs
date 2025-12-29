pub mod api;
pub mod events;
pub mod static_files;
pub mod ws;

use axum::Router;
use std::net::SocketAddr;
use tokio::sync::broadcast;

pub use events::DaemonEvent;

#[allow(dead_code)]
pub async fn create_app(event_tx: broadcast::Sender<DaemonEvent>) -> Router {
    Router::new()
        .nest("/api", api::create_router())
        .nest("/ws", ws::create_router(event_tx))
        .fallback_service(static_files::serve_static())
}

#[allow(dead_code)]
pub async fn serve(
    addr: SocketAddr,
    event_tx: broadcast::Sender<DaemonEvent>,
) -> Result<(), Box<dyn std::error::Error>> {
    let app = create_app(event_tx).await;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
