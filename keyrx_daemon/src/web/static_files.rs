#[cfg(feature = "web")]
use axum::Router;
#[cfg(feature = "web")]
use tower_http::services::ServeDir;

#[cfg(feature = "web")]
#[allow(dead_code)]
pub fn serve_static() -> Router {
    let serve_dir = ServeDir::new("ui_dist");
    Router::new().nest_service("/", serve_dir)
}

#[cfg(all(test, feature = "web"))]
mod tests {
    use super::*;

    #[test]
    fn test_serve_static() {
        let router = serve_static();
        assert!(std::mem::size_of_val(&router) > 0);
    }
}
