#[cfg(feature = "web")]
use axum::Router;
#[cfg(feature = "web")]
use tower_http::services::ServeDir;

#[cfg(feature = "web")]
pub fn serve_static() -> Router {
    let serve_dir = ServeDir::new("ui_dist");
    Router::new().nest_service("/", serve_dir)
}
