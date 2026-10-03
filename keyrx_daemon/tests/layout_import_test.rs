//! Importing a `.krx` / `.rhai` layout file as a profile, over REST and the
//! WS-RPC handler (both call `ProfileService::import_layout`).
//!
//! Every `.krx` the repo ships must import, and the stored profile must
//! compile back to exactly the mappings the file contained.

mod common;

use base64::Engine as _;
use common::test_app::TestApp;
use keyrx_daemon::config_loader::load_config;
use serde_json::{json, Value};
use serial_test::serial;
use std::path::{Path, PathBuf};

fn repo_file(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join(rel)
}

fn shipped_krx() -> Vec<PathBuf> {
    let mut files = vec![repo_file("examples/user_layout.krx")];
    for n in [
        "test1_simple",
        "test2_modifiers",
        "test3_locks",
        "test4_chords",
        "test5_vim",
        "test9_multidevice",
    ] {
        files.push(repo_file(&format!("uat_tests/{n}.krx")));
    }
    files
}

/// A shipped `.krx` must stay loadable AND in step with its `.rhai`: they
/// went stale once (an older rkyv layout), so the importer's own examples
/// could not be opened.
#[test]
fn shipped_krx_files_are_valid_and_match_their_source() {
    for path in shipped_krx() {
        let loaded = load_config(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        let source = path.with_extension("rhai");
        let (compiled, _) = keyrx_daemon::config::ProfileCompiler::new()
            .parse(&source)
            .unwrap_or_else(|e| panic!("{}: {e}", source.display()));
        assert_eq!(
            loaded.devices,
            compiled.devices,
            "{} is stale: recompile it from {}",
            path.display(),
            source.display()
        );
    }
}

fn b64(bytes: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

fn request(name: &str, format: &str, bytes: &[u8]) -> Value {
    json!({ "name": name, "format": format, "contentBase64": b64(bytes) })
}

fn profile_files(app: &TestApp, name: &str) -> (PathBuf, PathBuf) {
    let dir = app.config_path().join("profiles");
    (
        dir.join(format!("{name}.rhai")),
        dir.join(format!("{name}.krx")),
    )
}

fn assert_nothing_stored(app: &TestApp, name: &str) {
    let (rhai, krx) = profile_files(app, name);
    assert!(!rhai.exists() && !krx.exists(), "{name} left files behind");
    let dir = app.config_path().join("profiles");
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let file = entry.file_name().to_string_lossy().into_owned();
        assert!(
            !file.ends_with(".tmp") && !file.ends_with(".part"),
            "temp file left behind: {file}"
        );
    }
}

#[tokio::test]
#[serial]
async fn every_shipped_krx_imports_and_keeps_its_mappings() {
    let app = TestApp::new().await;
    for (i, path) in shipped_krx().iter().enumerate() {
        let bytes = std::fs::read(path).unwrap();
        let original = load_config(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        let name = format!("imported-{i}");

        let response = app
            .post("/api/profiles/import", &request(&name, "krx", &bytes))
            .await;
        let status = response.status();
        let body: Value = response.json().await.unwrap();
        assert_eq!(status, 200, "{}: {body}", path.display());
        assert_eq!(body["converted"], true);
        assert_eq!(body["format"], "krx");
        assert_eq!(body["profile"]["name"], name);

        // The compiled profile holds exactly the original mappings.
        let (rhai, krx) = profile_files(&app, &name);
        let stored = load_config(&krx).unwrap();
        assert_eq!(stored.devices, original.devices, "{}", path.display());

        // The editor gets Rhai source that shows them.
        let config = app
            .get(&format!("/api/profiles/{name}/config"))
            .await
            .json::<Value>()
            .await
            .unwrap();
        let source = config["source"].as_str().unwrap();
        assert_eq!(source, std::fs::read_to_string(rhai).unwrap());
        assert!(source.contains("device_start("), "{source}");
        if original.devices.iter().any(|d| !d.mappings.is_empty()) {
            assert!(
                source.contains("map(") || source.contains("tap_hold("),
                "{source}"
            );
        }
    }
}

#[tokio::test]
#[serial]
async fn rhai_is_stored_verbatim_and_not_flagged_as_converted() {
    let app = TestApp::new().await;
    let source = "// mine\ndevice_start(\"*\");\n  map(\"A\", \"VK_B\");\ndevice_end();\n";
    let response = app
        .post(
            "/api/profiles/import",
            &request("mine", "rhai", source.as_bytes()),
        )
        .await;
    assert_eq!(response.status(), 200);
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["converted"], false);
    let (rhai, krx) = profile_files(&app, "mine");
    assert_eq!(std::fs::read_to_string(rhai).unwrap(), source);
    assert!(krx.exists());
}

#[tokio::test]
#[serial]
async fn a_taken_name_is_a_conflict_and_changes_nothing() {
    let app = TestApp::new().await;
    let bytes = std::fs::read(repo_file("uat_tests/test1_simple.krx")).unwrap();
    let first = app
        .post("/api/profiles/import", &request("dup", "krx", &bytes))
        .await;
    assert_eq!(first.status(), 200);
    let (rhai, _) = profile_files(&app, "dup");
    let before = std::fs::read_to_string(&rhai).unwrap();

    let second = app
        .post(
            "/api/profiles/import",
            &request("dup", "rhai", b"device_start(\"*\");device_end();"),
        )
        .await;
    assert_eq!(second.status(), 409);
    assert_eq!(std::fs::read_to_string(&rhai).unwrap(), before);

    // A different name works for the same file.
    let renamed = app
        .post("/api/profiles/import", &request("dup-2", "krx", &bytes))
        .await;
    assert_eq!(renamed.status(), 200);
}

#[tokio::test]
#[serial]
async fn invalid_files_are_rejected_with_400_and_store_nothing() {
    let app = TestApp::new().await;
    let good = std::fs::read(repo_file("uat_tests/test5_vim.krx")).unwrap();

    let mut flipped = good.clone();
    let last = flipped.len() - 1;
    flipped[last] ^= 0xFF;
    let mut bad_magic = good.clone();
    bad_magic[0] ^= 0xFF;
    let oversized = vec![0u8; keyrx_daemon::services::MAX_LAYOUT_BYTES + 1];

    let cases: Vec<(&str, &str, Vec<u8>)> = vec![
        ("empty-krx", "krx", vec![]),
        ("trunc-krx", "krx", good[..good.len() / 2].to_vec()),
        ("tiny-krx", "krx", vec![1, 2, 3]),
        ("flip-krx", "krx", flipped),
        ("magic-krx", "krx", bad_magic),
        ("text-as-krx", "krx", b"device_start(\"*\");".to_vec()),
        ("big-krx", "krx", oversized.clone()),
        ("big-rhai", "rhai", oversized),
        ("empty-rhai", "rhai", vec![]),
        ("binary-rhai", "rhai", good.clone()),
        ("nul-rhai", "rhai", b"device_start(\"*\");\0".to_vec()),
        ("syntax-rhai", "rhai", b"device_start(".to_vec()),
        (
            "unknown-key-rhai",
            "rhai",
            b"device_start(\"*\"); map(\"Nope\", \"VK_A\"); device_end();".to_vec(),
        ),
    ];
    for (name, format, bytes) in cases {
        let response = app
            .post("/api/profiles/import", &request(name, format, &bytes))
            .await;
        assert_eq!(
            response.status(),
            400,
            "{name}: {}",
            response.text().await.unwrap()
        );
        assert_nothing_stored(&app, name);
    }

    // Not base64 at all.
    let response = app
        .post(
            "/api/profiles/import",
            &json!({ "name": "b64", "format": "krx", "contentBase64": "***not base64***" }),
        )
        .await;
    assert_eq!(response.status(), 400);
    // Path traversal in the name.
    let response = app
        .post("/api/profiles/import", &request("../evil", "rhai", b"x"))
        .await;
    assert_eq!(response.status(), 400);
}

#[tokio::test]
#[serial]
async fn rpc_and_rest_import_the_same_way() {
    use keyrx_daemon::config::ProfileManager;
    use keyrx_daemon::services::ProfileService;
    use keyrx_daemon::web::handlers::profile::import_profile;
    use std::sync::Arc;

    let dir = tempfile::tempdir().unwrap();
    let service = ProfileService::new(Arc::new(ProfileManager::new(dir.path().into()).unwrap()));
    let bytes = std::fs::read(repo_file("uat_tests/test2_modifiers.krx")).unwrap();
    let params = request("via-rpc", "krx", &bytes);

    let wire = import_profile(&service, params.clone()).await.unwrap();
    assert_eq!(wire["success"], true);
    assert_eq!(wire["converted"], true);
    assert_eq!(wire["profile"]["name"], "via-rpc");

    // Name collision and corrupt file are invalid-params errors, not crashes.
    assert!(import_profile(&service, params).await.is_err());
    let corrupt = request("corrupt", "krx", &bytes[..10]);
    assert!(import_profile(&service, corrupt).await.is_err());
    assert!(!Path::new(&dir.path().join("profiles/corrupt.rhai")).exists());
}
