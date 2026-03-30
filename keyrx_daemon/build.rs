use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

fn main() {
    let workspace_root = get_workspace_root();

    validate_version_consistency(&workspace_root);

    #[cfg(target_os = "windows")]
    embed_windows_resources();

    enforce_frontend_freshness(&workspace_root);
    set_build_metadata();
    emit_rerun_triggers(&workspace_root);
}

fn get_workspace_root() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    PathBuf::from(&manifest_dir)
        .parent()
        .expect("Failed to get workspace root")
        .to_path_buf()
}

// ── Version consistency ──────────────────────────────────────────────

/// Fail compilation if Cargo.toml (SSOT) and package.json versions differ.
fn validate_version_consistency(workspace_root: &Path) {
    let cargo_toml = workspace_root.join("Cargo.toml");
    let package_json = workspace_root.join("keyrx_ui/package.json");

    let cargo_version = extract_cargo_version(&cargo_toml);
    let pkg_version = extract_package_json_version(&package_json);

    if cargo_version != pkg_version {
        panic!(
            "\n\n\
            VERSION MISMATCH: Cargo.toml={cargo_version} vs package.json={pkg_version}\n\
            Fix: ./scripts/sync-version.sh\n"
        );
    }
}

fn extract_cargo_version(path: &Path) -> String {
    let content = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", path.display()));

    let mut in_section = false;
    for line in content.lines() {
        let t = line.trim();
        if t == "[workspace.package]" {
            in_section = true;
            continue;
        }
        if in_section && t.starts_with('[') {
            break;
        }
        if in_section && t.starts_with("version") {
            if let Some(v) = t
                .split('=')
                .nth(1)
                .and_then(|v| v.trim().trim_matches('"').split_whitespace().next())
            {
                return v.to_string();
            }
        }
    }
    panic!("No version in Cargo.toml [workspace.package]");
}

fn extract_package_json_version(path: &Path) -> String {
    let content = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", path.display()));

    for line in content.lines() {
        let t = line.trim();
        if t.starts_with("\"version\"") {
            if let Some(v) = t.split(':').nth(1).and_then(|v| {
                v.trim()
                    .trim_matches(|c| c == '"' || c == ',' || c == ' ')
                    .split_whitespace()
                    .next()
            }) {
                return v.to_string();
            }
        }
    }
    panic!("No version in {}", path.display());
}

// ── Windows resources ────────────────────────────────────────────────

#[cfg(target_os = "windows")]
fn embed_windows_resources() {
    let profile = std::env::var("PROFILE").unwrap_or_default();
    let mut res = winres::WindowsResource::new();
    if profile == "release" {
        res.set_manifest_file("keyrx_daemon.exe.manifest");
    }
    let icon_path = PathBuf::from("assets/icon.ico");
    if icon_path.exists() {
        res.set_icon("assets/icon.ico");
    }
    if let Err(e) = res.compile() {
        println!("cargo:warning=Failed to compile Windows resources: {e}");
    }
}

// ── Frontend freshness enforcement ───────────────────────────────────

/// Fail the build if WASM or UI dist is stale.
///
/// This prevents embedding outdated frontend artifacts into the daemon.
/// Bypass with: KEYRX_SKIP_FRONTEND_CHECK=1 cargo build
fn enforce_frontend_freshness(workspace_root: &Path) {
    if std::env::var("KEYRX_SKIP_FRONTEND_CHECK").is_ok() {
        println!(
            "cargo:warning=KEYRX_SKIP_FRONTEND_CHECK set \
             — skipping UI/WASM freshness checks"
        );
        return;
    }

    check_wasm_freshness(workspace_root);
    check_ui_freshness(workspace_root);
}

/// Fail if keyrx_core source has changed since the WASM binary was built.
///
/// Uses content-based hashing (SHA256 of source files) stored in
/// `wasm-manifest.json` instead of unreliable file modification times.
fn check_wasm_freshness(workspace_root: &Path) {
    let manifest = workspace_root.join("keyrx_ui/src/wasm/pkg/wasm-manifest.json");
    let core_src = workspace_root.join("keyrx_core/src");

    if !core_src.exists() {
        return;
    }
    if !manifest.exists() {
        println!(
            "cargo:warning=WASM manifest not found. \
             Run 'make build' for full build with WASM."
        );
        return;
    }

    let manifest_hash = read_manifest_source_hash(&manifest);
    let Some(expected) = manifest_hash else {
        println!(
            "cargo:warning=WASM manifest missing source_hash field. \
             Rebuild WASM with 'make build' to enable staleness detection."
        );
        return;
    };

    let current = compute_source_hash(workspace_root);
    if current != expected {
        panic!(
            "\n\n\
            STALE WASM: keyrx_core source hash changed.\n\
            \n\
            Expected (from manifest): {expected}\n\
            Current (from source):    {current}\n\
            \n\
            The WASM binary was built from different source code.\n\n\
            Fix:  make build\n\
            Skip: KEYRX_SKIP_FRONTEND_CHECK=1 cargo build\n"
        );
    }
}

/// Fail if UI source files are newer than the built dist.
fn check_ui_freshness(workspace_root: &Path) {
    let dist_index = workspace_root.join("keyrx_ui/dist/index.html");

    if !dist_index.exists() {
        println!(
            "cargo:warning=UI dist not found. \
             Run 'make build' for full build with embedded UI."
        );
        return;
    }

    println!("cargo:warning=UI dist found and will be embedded");

    let dist_mtime = file_mtime(&dist_index);

    // Check keyrx_ui/src/ (includes wasm/pkg/ — catches rebuilt WASM too)
    let mut newest_src = newest_mtime_in_dir(&workspace_root.join("keyrx_ui/src"));

    // Also check config files that affect the UI bundle
    let config_files = [
        workspace_root.join("keyrx_ui/package.json"),
        workspace_root.join("keyrx_ui/vite.config.ts"),
        workspace_root.join("keyrx_ui/index.html"),
    ];
    for f in &config_files {
        if let Some(t) = file_mtime(f) {
            newest_src = Some(newest_src.map_or(t, |n| n.max(t)));
        }
    }

    if let (Some(dist_t), Some(src_t)) = (dist_mtime, newest_src) {
        if src_t > dist_t {
            panic!(
                "\n\n\
                STALE UI: source files are newer than dist/index.html.\n\
                The daemon would embed outdated frontend chunks.\n\n\
                Fix:  make build\n\
                Skip: KEYRX_SKIP_FRONTEND_CHECK=1 cargo build\n"
            );
        }
    }
}

// ── Build metadata ───────────────────────────────────────────────────

fn set_build_metadata() {
    use chrono::offset::FixedOffset;
    let jst = FixedOffset::east_opt(9 * 3600).expect("JST offset");
    let now_jst = chrono::Utc::now().with_timezone(&jst);

    println!(
        "cargo:rustc-env=BUILD_DATE={}",
        now_jst.format("%Y-%m-%d %H:%M JST")
    );
    println!(
        "cargo:rustc-env=BUILD_TIMESTAMP={}",
        chrono::Utc::now().to_rfc3339()
    );

    let git_hash = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    println!("cargo:rustc-env=GIT_HASH={git_hash}");
}

// ── Rerun triggers ───────────────────────────────────────────────────

/// Tell Cargo when to re-run this build script.
fn emit_rerun_triggers(workspace_root: &Path) {
    // Version files (re-validate on change)
    println!(
        "cargo:rerun-if-changed={}",
        workspace_root.join("Cargo.toml").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        workspace_root.join("keyrx_ui/package.json").display()
    );

    // UI dist (re-embed when rebuilt)
    println!("cargo:rerun-if-changed=../keyrx_ui/dist/index.html");

    // WASM manifest (re-check staleness after WASM rebuild)
    println!(
        "cargo:rerun-if-changed={}",
        workspace_root
            .join("keyrx_ui/src/wasm/pkg/wasm-manifest.json")
            .display()
    );

    // keyrx_core source (trigger staleness check on changes)
    println!(
        "cargo:rerun-if-changed={}",
        workspace_root.join("keyrx_core/src").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        workspace_root.join("keyrx_core/Cargo.toml").display()
    );

    // Key source files (trigger staleness check on common changes)
    for entry in &[
        "../keyrx_ui/src/App.tsx",
        "../keyrx_ui/src/main.tsx",
        "../keyrx_ui/src/version.ts",
        "../keyrx_ui/index.html",
        "../keyrx_ui/vite.config.ts",
    ] {
        println!("cargo:rerun-if-changed={entry}");
    }

    // Re-run when bypass env var changes
    println!("cargo:rerun-if-env-changed=KEYRX_SKIP_FRONTEND_CHECK");
}

// ── Source hash helpers ───────────────────────────────────────────────

/// Read `source_hash` from wasm-manifest.json (simple string extraction).
fn read_manifest_source_hash(manifest: &Path) -> Option<String> {
    let content = std::fs::read_to_string(manifest).ok()?;
    // Simple JSON value extraction — avoids adding serde_json build-dep
    let key = "\"source_hash\"";
    let idx = content.find(key)?;
    let rest = &content[idx + key.len()..];
    let colon = rest.find(':')?;
    let after_colon = &rest[colon + 1..];
    let open_quote = after_colon.find('"')?;
    let value_start = open_quote + 1;
    let close_quote = after_colon[value_start..].find('"')?;
    Some(after_colon[value_start..value_start + close_quote].to_string())
}

/// Compute SHA256 of all keyrx_core source files + Cargo.toml.
///
/// Algorithm (must match scripts/lib/build-wasm.sh):
/// 1. Collect all *.rs files in keyrx_core/src/ + keyrx_core/Cargo.toml
/// 2. Sort by forward-slash relative path
/// 3. For each file: feed path + "\n" + content (with \r stripped) to SHA256
fn compute_source_hash(workspace_root: &Path) -> String {
    let core_src = workspace_root.join("keyrx_core").join("src");
    let mut files = Vec::new();
    collect_rs_files(&core_src, &mut files);
    files.push(workspace_root.join("keyrx_core").join("Cargo.toml"));

    let mut relative: Vec<String> = files
        .iter()
        .filter_map(|f| f.strip_prefix(workspace_root).ok())
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .collect();
    relative.sort();

    let mut hasher = Sha256::new();
    for rel_path in &relative {
        hasher.update(rel_path.as_bytes());
        hasher.update(b"\n");
        if let Ok(content) = std::fs::read(workspace_root.join(rel_path)) {
            let normalized: Vec<u8> = content.into_iter().filter(|&b| b != b'\r').collect();
            hasher.update(&normalized);
        }
    }
    format!("{:x}", hasher.finalize())
}

/// Recursively collect all *.rs files in a directory.
fn collect_rs_files(dir: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, files);
        } else if path.extension().is_some_and(|e| e == "rs") {
            files.push(path);
        }
    }
}

// ── Helpers ──────────────────────────────────────────────────────────

fn file_mtime(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path).ok()?.modified().ok()
}

/// Newest file modification time in a directory (recursive).
fn newest_mtime_in_dir(dir: &Path) -> Option<SystemTime> {
    let mut newest: Option<SystemTime> = None;
    for entry in std::fs::read_dir(dir).ok()?.flatten() {
        let path = entry.path();
        let mtime = if path.is_dir() {
            newest_mtime_in_dir(&path)
        } else {
            file_mtime(&path)
        };
        if let Some(t) = mtime {
            newest = Some(newest.map_or(t, |n| n.max(t)));
        }
    }
    newest
}
