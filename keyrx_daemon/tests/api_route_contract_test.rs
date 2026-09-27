//! Every REST path the UI calls must exist on the server.
//!
//! Scans `keyrx_ui/src` (tests excluded) for `/api/...` string literals and
//! sends a GET for each to the real router. An unmatched route is axum's bare
//! 404 with an empty body. A handler's own 404 ("profile not found") has a
//! JSON body, and a wrong method is 405, so both count as "route exists".
//!
//! Found on 2026-09-27: the UI called `/api/state`, `/api/settings/global-layout`,
//! `/api/profiles/validate` and `/api/config/{profile}/...`, none of which were
//! served.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::body::Body;
use axum::http::Request;
use tower::ServiceExt;

use keyrx_daemon::web::{create_router, AppState};

fn ui_src() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../keyrx_ui/src")
}

fn is_test_or_generated(path: &Path) -> bool {
    let p = path.to_string_lossy();
    p.contains(".test.") || p.contains(".spec.") || p.contains("/test/") || p.contains("/wasm/pkg/")
}

fn source_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            source_files(&path, out);
        } else if matches!(
            path.extension().and_then(|e| e.to_str()),
            Some("ts" | "tsx")
        ) && !is_test_or_generated(&path)
        {
            out.push(path);
        }
    }
}

/// `/api/...` literals in quotes or template strings; `${...}` becomes `x`.
fn api_paths(source: &str) -> Vec<String> {
    let mut paths = Vec::new();
    for quote in ['\'', '"', '`'] {
        for (i, _) in source.match_indices(&format!("{quote}/api/")) {
            let rest = &source[i + 1..];
            let end = rest.find([quote, '?']).unwrap_or(rest.len());
            let mut path = rest[..end].to_string();
            while let Some(start) = path.find("${") {
                let close = path[start..]
                    .find('}')
                    .map_or(path.len(), |c| start + c + 1);
                path.replace_range(start..close, "x");
            }
            if path.len() > "/api/".len() && !path.contains(char::is_whitespace) {
                paths.push(path);
            }
        }
    }
    paths
}

#[tokio::test]
async fn every_ui_api_path_is_served() {
    let dir = tempfile::tempdir().unwrap();
    let app = Arc::new(AppState::new_for_testing(dir.path().to_path_buf()));

    let mut files = Vec::new();
    source_files(&ui_src(), &mut files);
    assert!(
        !files.is_empty(),
        "no UI sources found under {}",
        ui_src().display()
    );

    let mut missing = Vec::new();
    let mut checked = std::collections::BTreeSet::new();
    for file in &files {
        let source = std::fs::read_to_string(file).unwrap();
        for path in api_paths(&source) {
            if !checked.insert(path.clone()) {
                continue;
            }
            let request = Request::get(&path).body(Body::empty()).unwrap();
            let response = create_router(Arc::clone(&app))
                .oneshot(request)
                .await
                .unwrap();
            let status = response.status().as_u16();
            let body = axum::body::to_bytes(response.into_body(), 1 << 20)
                .await
                .unwrap();
            if status == 404 && body.is_empty() {
                let rel = file.strip_prefix(ui_src()).unwrap_or(file);
                missing.push(format!("{path}  (keyrx_ui/src/{})", rel.display()));
            }
        }
    }
    assert!(
        checked.len() > 10,
        "suspiciously few UI API paths: {checked:?}"
    );
    assert!(
        missing.is_empty(),
        "UI calls REST paths the server does not route:\n  {}",
        missing.join("\n  ")
    );
}
