use std::path::PathBuf;

fn main() {
    // Verify that the UI dist directory exists
    let ui_dist_path = PathBuf::from("../keyrx_ui/dist");

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
        }
    }

    // Tell cargo to re-run this build script if the UI dist directory changes
    println!("cargo:rerun-if-changed=../keyrx_ui/dist");

    // Set build timestamp
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
        }
    }
}
