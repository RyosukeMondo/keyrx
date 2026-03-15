use std::path::{Path, PathBuf};
use std::time::SystemTime;

fn main() {
    // Validate version consistency between Cargo.toml and package.json
    validate_version_consistency();

    // Windows: Embed manifest and icon for admin elevation (release only)
    #[cfg(target_os = "windows")]
    {
        // Only embed admin manifest for release builds
        let profile = std::env::var("PROFILE").unwrap_or_default();
        let mut res = winres::WindowsResource::new();
        if profile == "release" {
            res.set_manifest_file("keyrx_daemon.exe.manifest");
        }
        // Embed icon if available
        let icon_path = PathBuf::from("assets/icon.ico");
        if icon_path.exists() {
            res.set_icon("assets/icon.ico");
        }
        if let Err(e) = res.compile() {
            eprintln!("cargo:warning=Failed to compile Windows resources: {}", e);
        }
    }

    // Verify that the UI dist directory exists
    check_ui_dist();

    // Set build timestamp and git hash
    set_build_metadata();
}

/// Validate version consistency at compile time
///
/// This function enforces Cargo.toml as the single source of truth (SSOT)
/// for version. It reads both Cargo.toml and package.json versions and
/// fails compilation if they don't match.
///
/// # Panics
/// Panics with a clear error message if:
/// - Version extraction fails from either file
/// - Versions don't match between Cargo.toml and package.json
fn validate_version_consistency() {
    // Get workspace root
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let manifest_path = PathBuf::from(&manifest_dir);
    let workspace_root = manifest_path
        .parent()
        .expect("Failed to get workspace root");

    // Paths to version files
    let cargo_toml = workspace_root.join("Cargo.toml");
    let package_json = workspace_root.join("keyrx_ui").join("package.json");

    // Read Cargo.toml version (SSOT)
    let cargo_version = extract_cargo_version(&cargo_toml);

    // Read package.json version
    let package_json_version = extract_package_json_version(&package_json);

    // Compare versions
    if cargo_version != package_json_version {
        panic!(
            "\n\n\
            ╔═══════════════════════════════════════════════════════════════════════════════╗\n\
            ║ ❌ VERSION MISMATCH DETECTED                                                   ║\n\
            ╠═══════════════════════════════════════════════════════════════════════════════╣\n\
            ║                                                                               ║\n\
            ║ Cargo.toml version (SSOT):  {}                                             ║\n\
            ║ package.json version:       {}                                             ║\n\
            ║                                                                               ║\n\
            ║ To fix this issue, run:                                                       ║\n\
            ║   ./scripts/sync-version.sh                                                   ║\n\
            ║                                                                               ║\n\
            ║ This will synchronize all version files to match Cargo.toml (SSOT).          ║\n\
            ╚═══════════════════════════════════════════════════════════════════════════════╝\n\
            ",
            cargo_version, package_json_version
        );
    }

    // Rerun if version files change
    println!("cargo:rerun-if-changed={}", cargo_toml.display());
    println!("cargo:rerun-if-changed={}", package_json.display());
    println!("cargo:rerun-if-changed=../scripts/sync-version.sh");
}

/// Extract version from Cargo.toml [workspace.package]
fn extract_cargo_version(path: &PathBuf) -> String {
    let content = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("Failed to read Cargo.toml: {}", e));

    let mut in_workspace_package = false;
    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed == "[workspace.package]" {
            in_workspace_package = true;
            continue;
        }

        // Stop if we hit another section
        if in_workspace_package && trimmed.starts_with('[') {
            break;
        }

        if in_workspace_package && trimmed.starts_with("version") {
            if let Some(version) = trimmed
                .split('=')
                .nth(1)
                .and_then(|v| v.trim().trim_matches('"').split_whitespace().next())
            {
                return version.to_string();
            }
        }
    }

    panic!("Failed to extract version from Cargo.toml [workspace.package]");
}

/// Extract version from package.json
fn extract_package_json_version(path: &PathBuf) -> String {
    let content = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("Failed to read package.json: {}", e));

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("\"version\"") {
            if let Some(version) = trimmed.split(':').nth(1).and_then(|v| {
                v.trim()
                    .trim_matches(|c| c == '"' || c == ',' || c == ' ')
                    .split_whitespace()
                    .next()
            }) {
                return version.to_string();
            }
        }
    }

    panic!("Failed to extract version from package.json");
}

/// Check UI dist directory exists
fn check_ui_dist() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let workspace_root_binding = PathBuf::from(&manifest_dir);
    let workspace_root = workspace_root_binding
        .parent()
        .expect("Failed to get workspace root");

    let ui_dist_path = workspace_root.join("keyrx_ui/dist");

    if !ui_dist_path.exists() {
        println!(
            "cargo:warning=UI dist directory not found at {:?}",
            ui_dist_path
        );
        println!("cargo:warning=Run 'cd keyrx_ui && npm run build' to build the UI");
        println!(
            "cargo:warning=The daemon will still compile but will not be able to serve the UI"
        );
    } else {
        // Verify index.html exists
        let index_html = ui_dist_path.join("index.html");
        if !index_html.exists() {
            println!("cargo:warning=index.html not found in UI dist directory");
            println!("cargo:warning=The UI build may be incomplete");
        } else {
            println!("cargo:warning=UI dist directory found and will be embedded");
            check_ui_staleness(workspace_root);
        }
    }

    // Re-run when dist contents change (after UI rebuild)
    println!("cargo:rerun-if-changed=../keyrx_ui/dist/index.html");
    // Re-run when key source entry points change (to detect staleness)
    for entry in &[
        "../keyrx_ui/src/App.tsx",
        "../keyrx_ui/src/main.tsx",
        "../keyrx_ui/index.html",
        "../keyrx_ui/vite.config.ts",
        "../keyrx_ui/package.json",
    ] {
        println!("cargo:rerun-if-changed={entry}");
    }
}

/// Set build metadata (timestamp and git hash)
fn set_build_metadata() {
    // Set build timestamp in JST (UTC+9)
    use chrono::offset::FixedOffset;
    let jst = FixedOffset::east_opt(9 * 3600).expect("JST offset");
    let now_jst = chrono::Utc::now().with_timezone(&jst);
    let build_date = now_jst.format("%Y-%m-%d %H:%M JST").to_string();
    println!("cargo:rustc-env=BUILD_DATE={}", build_date);
    println!(
        "cargo:rustc-env=BUILD_TIMESTAMP={}",
        chrono::Utc::now().to_rfc3339()
    );

    // Set git commit hash if available
    if let Ok(output) = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
    {
        if output.status.success() {
            let git_hash = String::from_utf8_lossy(&output.stdout);
            println!("cargo:rustc-env=GIT_HASH={}", git_hash.trim());
        } else {
            println!("cargo:rustc-env=GIT_HASH=unknown");
        }
    } else {
        println!("cargo:rustc-env=GIT_HASH=unknown");
    }
}

/// Warn when UI source files are newer than dist (stale embedded UI)
///
/// Compares the newest modification time in `keyrx_ui/src/` and key config
/// files against `keyrx_ui/dist/index.html`. If source is newer, the daemon
/// would embed outdated UI chunks, causing "Failed to fetch dynamically
/// imported module" errors at runtime.
fn check_ui_staleness(workspace_root: &Path) {
    let dist_index = workspace_root.join("keyrx_ui/dist/index.html");

    let Ok(dist_meta) = std::fs::metadata(&dist_index) else {
        return;
    };
    let Ok(dist_mtime) = dist_meta.modified() else {
        return;
    };

    // Check source directory and config files that affect build output
    let src_dir = workspace_root.join("keyrx_ui/src");
    let mut newest_src = newest_mtime_in_dir(&src_dir);

    let config_files = [
        workspace_root.join("keyrx_ui/package.json"),
        workspace_root.join("keyrx_ui/vite.config.ts"),
        workspace_root.join("keyrx_ui/index.html"),
    ];
    for config_file in &config_files {
        if let Ok(meta) = std::fs::metadata(config_file) {
            if let Ok(mtime) = meta.modified() {
                newest_src = Some(newest_src.map_or(mtime, |n| n.max(mtime)));
            }
        }
    }

    if let Some(src_mtime) = newest_src {
        if src_mtime > dist_mtime {
            emit_stale_ui_warning();
        }
    }
}

fn emit_stale_ui_warning() {
    let lines = [
        "STALE UI DIST DETECTED",
        "UI source files are newer than dist/index.html.",
        "The daemon will embed OUTDATED UI files.",
        "This causes 'Failed to fetch dynamically imported module' errors.",
        "",
        "Fix: run 'make build' or 'scripts/build.sh'",
        "     (rebuilds WASM -> UI -> Daemon in correct sequence)",
    ];
    for line in &lines {
        println!("cargo:warning={line}");
    }
}

/// Find the newest file modification time in a directory (recursive)
fn newest_mtime_in_dir(dir: &Path) -> Option<SystemTime> {
    let entries = std::fs::read_dir(dir).ok()?;
    let mut newest: Option<SystemTime> = None;

    for entry in entries.flatten() {
        let path = entry.path();
        let mtime = if path.is_dir() {
            newest_mtime_in_dir(&path)
        } else {
            path.metadata().ok().and_then(|m| m.modified().ok())
        };
        if let Some(t) = mtime {
            newest = Some(newest.map_or(t, |n| n.max(t)));
        }
    }

    newest
}
