#!/usr/bin/env bash
# Synchronize version across all sources (Cargo.toml is SSOT)
#
# Single Source of Truth: Cargo.toml [workspace.package] version
# Propagates to:
#   - keyrx_ui/package.json
#   - keyrx_daemon/keyrx_installer.wxs
#   - scripts/build_windows_installer.ps1
#   - keyrx_ui/src/version.ts (via generate-version.js)
#
# Usage:
#   ./scripts/sync-version.sh              # Sync and update all files
#   ./scripts/sync-version.sh --check      # Check only (no modifications)
#   ./scripts/sync-version.sh --dry-run    # Show what would change
#
# Exit codes:
#   0 - All versions synchronized
#   1 - Version mismatch detected (check mode) or update failed
#   2 - Missing required tools

set -euo pipefail

# Colors
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly BLUE='\033[0;34m'
readonly NC='\033[0m'

# Paths
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CARGO_TOML="$PROJECT_ROOT/Cargo.toml"
PACKAGE_JSON="$PROJECT_ROOT/keyrx_ui/package.json"
INSTALLER_WXS="$PROJECT_ROOT/keyrx_daemon/keyrx_installer.wxs"
INSTALLER_PS1="$PROJECT_ROOT/scripts/build_windows_installer.ps1"

# Modes
CHECK_MODE=false
DRY_RUN=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --check)
            CHECK_MODE=true
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [--check] [--dry-run]"
            echo ""
            echo "Synchronize version across all project files (SSOT: Cargo.toml)"
            echo ""
            echo "Options:"
            echo "  --check      Check version consistency without modifying files"
            echo "  --dry-run    Show what would be changed without modifying files"
            echo "  -h, --help   Show this help message"
            exit 0
            ;;
        *)
            echo -e "${RED}ERROR: Unknown option: $1${NC}" >&2
            exit 1
            ;;
    esac
done

# Logging functions
log_info() { echo -e "${BLUE}[INFO]${NC} $*"; }
log_success() { echo -e "${GREEN}[OK]${NC} $*"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $*"; }
log_error() { echo -e "${RED}[ERROR]${NC} $*" >&2; }

# Check required tools
if ! command -v grep &>/dev/null || ! command -v sed &>/dev/null; then
    log_error "Missing required tools: grep, sed"
    exit 2
fi

# Extract version from Cargo.toml [workspace.package] (SSOT)
get_cargo_version() {
    grep -A 20 '^\[workspace\.package\]' "$CARGO_TOML" | \
        grep '^version\s*=' | head -1 | \
        sed -E 's/^version\s*=\s*"([^"]+)".*/\1/'
}

# Extract versions from other files
get_package_json_version() {
    grep '"version"' "$PACKAGE_JSON" | head -1 | \
        sed -E 's/.*"version":\s*"([^"]+)".*/\1/'
}

get_installer_wxs_version() {
    grep 'Version=' "$INSTALLER_WXS" | head -1 | \
        sed -E 's/.*Version="([^"]+)".*/\1/' | \
        sed 's/\.0$//'  # Remove .0 suffix (WiX uses 4-part)
}

get_installer_ps1_version() {
    grep '^\s*\[string\]\$Version\s*=' "$INSTALLER_PS1" | \
        sed -E 's/.*=\s*"([^"]+)".*/\1/'
}

# Update functions
update_package_json() {
    local new="$1" old="$2"
    [[ "$DRY_RUN" == true ]] && { log_info "Would update package.json: $old → $new"; return 0; }

    if [[ "$OSTYPE" == "darwin"* ]]; then
        sed -i '' -E "s/(\"version\":\s*\")$old/\1$new/" "$PACKAGE_JSON"
    else
        sed -i -E "s/(\"version\":\s*\")$old/\1$new/" "$PACKAGE_JSON"
    fi
    log_success "Updated package.json: $old → $new"
}

update_installer_wxs() {
    local new="$1" old="$2"
    local wix_new="${new}.0" wix_old="${old}.0"  # WiX needs 4-part version
    [[ "$DRY_RUN" == true ]] && { log_info "Would update keyrx_installer.wxs: $wix_old → $wix_new"; return 0; }

    if [[ "$OSTYPE" == "darwin"* ]]; then
        sed -i '' -E "s/(Version=\")$wix_old/\1$wix_new/" "$INSTALLER_WXS"
    else
        sed -i -E "s/(Version=\")$wix_old/\1$wix_new/" "$INSTALLER_WXS"
    fi
    log_success "Updated keyrx_installer.wxs: $wix_old → $wix_new"
}

update_installer_ps1() {
    local new="$1" old="$2"
    [[ "$DRY_RUN" == true ]] && { log_info "Would update build_windows_installer.ps1: $old → $new"; return 0; }

    if [[ "$OSTYPE" == "darwin"* ]]; then
        sed -i '' -E "s/(\[string\]\\\$Version\s*=\s*\")$old/\1$new/" "$INSTALLER_PS1"
    else
        sed -i -E "s/(\[string\]\\\$Version\s*=\s*\")$old/\1$new/" "$INSTALLER_PS1"
    fi
    log_success "Updated build_windows_installer.ps1: $old → $new"
}

# Main logic
main() {
    log_info "Version Synchronization Tool"
    log_info "=============================="
    echo ""

    # Get SSOT version
    CARGO_VERSION=$(get_cargo_version)
    if [[ -z "$CARGO_VERSION" ]]; then
        log_error "Failed to extract version from Cargo.toml"
        exit 1
    fi
    log_info "SSOT (Cargo.toml): $CARGO_VERSION"

    # Get current versions
    PKG_VERSION=$(get_package_json_version)
    WXS_VERSION=$(get_installer_wxs_version)
    PS1_VERSION=$(get_installer_ps1_version)

    echo ""
    log_info "Current versions:"
    echo "  Cargo.toml:                  $CARGO_VERSION (SSOT)"
    echo "  package.json:                $PKG_VERSION"
    echo "  keyrx_installer.wxs:         $WXS_VERSION"
    echo "  build_windows_installer.ps1: $PS1_VERSION"
    echo ""

    # Check for mismatches
    MISMATCH=false
    [[ "$PKG_VERSION" != "$CARGO_VERSION" ]] && { log_warn "package.json mismatch: $PKG_VERSION != $CARGO_VERSION"; MISMATCH=true; }
    [[ "$WXS_VERSION" != "$CARGO_VERSION" ]] && { log_warn "keyrx_installer.wxs mismatch: $WXS_VERSION != $CARGO_VERSION"; MISMATCH=true; }
    [[ "$PS1_VERSION" != "$CARGO_VERSION" ]] && { log_warn "build_windows_installer.ps1 mismatch: $PS1_VERSION != $CARGO_VERSION"; MISMATCH=true; }

    # Handle check mode
    if [[ "$CHECK_MODE" == true ]]; then
        [[ "$MISMATCH" == true ]] && { log_error "Version mismatch detected. Run without --check to fix."; exit 1; }
        log_success "All versions synchronized!"
        exit 0
    fi

    # Update files if mismatched
    if [[ "$MISMATCH" == true ]]; then
        echo ""
        log_info "Synchronizing versions to $CARGO_VERSION..."
        echo ""

        [[ "$PKG_VERSION" != "$CARGO_VERSION" ]] && update_package_json "$CARGO_VERSION" "$PKG_VERSION"
        [[ "$WXS_VERSION" != "$CARGO_VERSION" ]] && update_installer_wxs "$CARGO_VERSION" "$WXS_VERSION"
        [[ "$PS1_VERSION" != "$CARGO_VERSION" ]] && update_installer_ps1 "$CARGO_VERSION" "$PS1_VERSION"

        echo ""
        if [[ "$DRY_RUN" == true ]]; then
            log_info "Dry run complete. No files modified."
        else
            log_success "Version synchronization complete!"
            log_info "Regenerating keyrx_ui/src/version.ts..."

            if [[ -f "$SCRIPT_DIR/generate-version.js" ]]; then
                (cd "$PROJECT_ROOT/keyrx_ui" && node ../scripts/generate-version.js) || log_warn "Failed to regenerate version.ts"
            else
                log_warn "generate-version.js not found, skipping version.ts regeneration"
            fi
        fi
    else
        log_success "All versions already synchronized!"
    fi

    exit 0
}

main
