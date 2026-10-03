use axum::{
    body::Body,
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
    Router,
};
use include_dir::{include_dir, Dir};

// Embed the UI files at compile time
static UI_DIR: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/../keyrx_ui/dist");

/// Serve embedded static files
pub fn serve_static() -> Router {
    Router::new().fallback(static_handler)
}

/// `Cache-Control` for an embedded file. Hashed build output under `assets/`
/// never changes; everything else (index.html above all) must be revalidated
/// so an open browser tab finds the new asset names after an upgrade.
fn cache_control(path: &str) -> &'static str {
    if path.starts_with("assets/") {
        "public, max-age=31536000, immutable"
    } else {
        "no-cache"
    }
}

/// Whether a request path names a file (`/assets/x.js`, `/favicon.ico`) rather
/// than a client-side route (`/devices`, `/profiles/work/config`).
fn names_a_file(path: &str) -> bool {
    path.starts_with("assets/") || path.rsplit('/').next().is_some_and(|s| s.contains('.'))
}

fn respond(status: StatusCode, content_type: &str, cache: &str, body: Body) -> Response {
    let mut response = Response::new(body);
    *response.status_mut() = status;
    let headers = response.headers_mut();
    if let Ok(value) = content_type.parse() {
        headers.insert(header::CONTENT_TYPE, value);
    }
    if let Ok(value) = cache.parse() {
        headers.insert(header::CACHE_CONTROL, value);
    }
    response
}

fn not_found() -> Response {
    respond(
        StatusCode::NOT_FOUND,
        "text/plain; charset=utf-8",
        "no-cache",
        Body::from("404 Not Found"),
    )
}

/// Handler for serving embedded static files
async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    if let Some(file) = UI_DIR.get_file(path) {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        return respond(
            StatusCode::OK,
            mime.as_ref(),
            cache_control(path),
            Body::from(file.contents()),
        );
    }

    // A missing FILE is a 404. Answering it with index.html (200, text/html)
    // is what made a browser tab opened before an upgrade fail with "Failed
    // to fetch dynamically imported module" when it navigated to a page whose
    // hashed chunk no longer exists: the script request "succeeded" with HTML.
    if names_a_file(path) {
        return not_found();
    }

    // A client-side route (/devices, /profiles/x/config): the SPA shell.
    match UI_DIR.get_file("index.html") {
        Some(index) => respond(
            StatusCode::OK,
            "text/html",
            "no-cache",
            Body::from(index.contents()),
        ),
        None => not_found(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use axum::http::Request;
    use tower::ServiceExt;

    async fn get(path: &str) -> Response {
        serve_static()
            .oneshot(Request::get(path).body(Body::empty()).unwrap())
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn a_client_route_gets_the_app_shell_uncached() {
        for route in ["/", "/devices", "/profiles/work/config"] {
            let response = get(route).await;
            assert_eq!(response.status(), StatusCode::OK, "{route}");
            assert_eq!(response.headers()[header::CONTENT_TYPE], "text/html");
            assert_eq!(response.headers()[header::CACHE_CONTROL], "no-cache");
        }
    }

    /// The bug class: a stale hashed chunk must be a real 404, not HTML.
    #[tokio::test]
    async fn a_missing_file_is_404_not_the_app_shell() {
        for missing in [
            "/assets/DevicesPage-OLDHASH.js",
            "/favicon-missing.ico",
            "/x/y.wasm",
        ] {
            let response = get(missing).await;
            assert_eq!(response.status(), StatusCode::NOT_FOUND, "{missing}");
            let body = to_bytes(response.into_body(), 1024).await.unwrap();
            assert!(
                !String::from_utf8_lossy(&body).contains("<html"),
                "{missing}"
            );
        }
    }

    #[tokio::test]
    async fn embedded_assets_are_cached_forever_and_index_is_not() {
        let asset = UI_DIR
            .get_dir("assets")
            .and_then(|d| d.files().next())
            .expect("an embedded asset")
            .path()
            .to_string_lossy()
            .into_owned();
        let response = get(&format!("/{asset}")).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert!(response.headers()[header::CACHE_CONTROL]
            .to_str()
            .unwrap()
            .contains("immutable"));
    }

    #[test]
    fn test_ui_dir_has_index() {
        assert!(
            UI_DIR.get_file("index.html").is_some(),
            "index.html should be embedded in the binary"
        );
    }

    #[test]
    fn test_ui_dir_has_assets() {
        assert!(
            UI_DIR.get_dir("assets").is_some(),
            "assets directory should be embedded in the binary"
        );
    }
}
