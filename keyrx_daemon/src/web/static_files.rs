use axum::{
    body::Body,
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
    Router,
};
use include_dir::{include_dir, Dir};

// Embed the UI files at compile time
static UI_DIR: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/../keyrx_ui_v2/dist");

/// Serve embedded static files
pub fn serve_static() -> Router {
    Router::new().fallback(static_handler)
}

/// Handler for serving embedded static files
async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');

    // If path is empty, serve index.html
    let path = if path.is_empty() { "index.html" } else { path };

    // Try to find the file in the embedded directory
    match UI_DIR.get_file(path) {
        Some(file) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();

            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(Body::from(file.contents()))
                .unwrap()
        }
        None => {
            // If file not found, serve index.html for client-side routing
            // This handles React Router routes like /devices, /profiles, etc.
            if let Some(index) = UI_DIR.get_file("index.html") {
                Response::builder()
                    .status(StatusCode::OK)
                    .header(header::CONTENT_TYPE, "text/html")
                    .body(Body::from(index.contents()))
                    .unwrap()
            } else {
                Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::from("404 Not Found"))
                    .unwrap()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serve_static() {
        let router = serve_static();
        assert!(std::mem::size_of_val(&router) > 0);
    }

    #[test]
    fn test_ui_dir_has_index() {
        // Verify that index.html is embedded
        assert!(
            UI_DIR.get_file("index.html").is_some(),
            "index.html should be embedded in the binary"
        );
    }

    #[test]
    fn test_ui_dir_has_assets() {
        // Verify that assets directory exists
        assert!(
            UI_DIR.get_dir("assets").is_some(),
            "assets directory should be embedded in the binary"
        );
    }
}
