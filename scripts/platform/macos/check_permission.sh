#!/usr/bin/env bash
# Check macOS Accessibility permission for E2E testing
# Returns: 0 if permission granted, 1 if denied

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Create temporary directory for the checker binary
TEMP_DIR=$(mktemp -d)
trap 'rm -rf "$TEMP_DIR"' EXIT

# Create src directory for proper Cargo project structure
mkdir -p "$TEMP_DIR/src"

# Create minimal Rust program that checks permission
cat > "$TEMP_DIR/src/main.rs" << 'EOF'
fn main() {
    #[cfg(target_os = "macos")]
    {
        // SAFETY: AXIsProcessTrusted() is a stable macOS API with no side effects
        let has_permission = unsafe { accessibility_sys::AXIsProcessTrusted() };
        std::process::exit(if has_permission { 0 } else { 1 });
    }

    #[cfg(not(target_os = "macos"))]
    {
        eprintln!("This script is only for macOS");
        std::process::exit(2);
    }
}
EOF

# Create minimal Cargo.toml
cat > "$TEMP_DIR/Cargo.toml" << 'EOF'
[package]
name = "permission_checker"
version = "0.1.0"
edition = "2021"

[dependencies]
accessibility-sys = "0.1"
EOF

# Build the checker (suppress build output)
cargo build --release --manifest-path "$TEMP_DIR/Cargo.toml" --quiet 2>/dev/null || {
    echo "Error: Failed to build permission checker" >&2
    exit 1
}

# Run the checker and capture exit code
"$TEMP_DIR/target/release/permission_checker"
exit $?
