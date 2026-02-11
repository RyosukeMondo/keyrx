#!/usr/bin/env bash
# Package KeyRx for Distribution
# Consolidates all packaging operations: deb, rpm, tarball, Windows installer
# Usage: ./scripts/package.sh [--windows|--deb|--rpm|--tarball|--all]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"
VERSION="${VERSION:-0.1.5}"

# Source common utilities
source "$SCRIPT_DIR/lib/common.sh"

# Colors and logging
info() { echo -e "\033[0;36m[INFO]\033[0m $*"; }
success() { echo -e "\033[0;32m[SUCCESS]\033[0m $*"; }
error() { echo -e "\033[0;31m[ERROR]\033[0m $*" >&2; }
warn() { echo -e "\033[0;33m[WARN]\033[0m $*"; }

# Package Windows installer using Inno Setup
package_windows() {
    info "Building Windows installer..."

    # Check if Inno Setup is installed
    if ! command -v iscc &> /dev/null && ! [ -f "/c/Program Files (x86)/Inno Setup 6/iscc.exe" ]; then
        error "Inno Setup not found. Install from: https://jrsoftware.org/isdl.php"
        return 1
    fi

    local ISCC_PATH
    if command -v iscc &> /dev/null; then
        ISCC_PATH="iscc"
    else
        ISCC_PATH="/c/Program Files (x86)/Inno Setup 6/iscc.exe"
    fi

    # Build release binaries if needed
    if [ ! -f "$REPO_ROOT/target/release/keyrx_daemon.exe" ]; then
        info "Building release binaries..."
        cargo build --release --bin keyrx_daemon --bin keyrx_compiler
    fi

    # Build UI if needed
    if [ ! -d "$REPO_ROOT/keyrx_ui/dist" ]; then
        info "Building UI..."
        (cd "$REPO_ROOT/keyrx_ui" && npm run build)
    fi

    # Compile installer
    info "Compiling Inno Setup installer..."
    cd "$REPO_ROOT"
    "$ISCC_PATH" keyrx-installer.iss

    success "Windows installer created: installer-output/keyrx-setup-v$VERSION-windows-x64.exe"
}

# Package Debian/Ubuntu .deb
package_deb() {
    info "Building .deb package..."

    local DEB_DIR="$REPO_ROOT/target/debian"
    local PKG_NAME="keyrx_$VERSION-1_amd64"

    # Build release binary
    cargo build --release --bin keyrx_daemon --bin keyrx_compiler

    # Create package structure
    mkdir -p "$DEB_DIR/$PKG_NAME/DEBIAN"
    mkdir -p "$DEB_DIR/$PKG_NAME/usr/local/bin"
    mkdir -p "$DEB_DIR/$PKG_NAME/usr/share/keyrx"
    mkdir -p "$DEB_DIR/$PKG_NAME/usr/share/doc/keyrx"

    # Copy binaries
    cp "$REPO_ROOT/target/release/keyrx_daemon" "$DEB_DIR/$PKG_NAME/usr/local/bin/"
    cp "$REPO_ROOT/target/release/keyrx_compiler" "$DEB_DIR/$PKG_NAME/usr/local/bin/"

    # Copy docs
    cp "$REPO_ROOT/README.md" "$DEB_DIR/$PKG_NAME/usr/share/doc/keyrx/"
    cp "$REPO_ROOT/LICENSE" "$DEB_DIR/$PKG_NAME/usr/share/doc/keyrx/"

    # Create control file
    cat > "$DEB_DIR/$PKG_NAME/DEBIAN/control" <<EOF
Package: keyrx
Version: $VERSION
Section: utils
Priority: optional
Architecture: amd64
Maintainer: KeyRx Contributors <https://github.com/RyosukeMondo/keyrx>
Description: Advanced keyboard remapping daemon
 KeyRx provides flexible keyboard remapping with layers, macros,
 and per-device configuration.
EOF

    # Build package
    dpkg-deb --build "$DEB_DIR/$PKG_NAME"

    success ".deb package created: target/debian/$PKG_NAME.deb"
}

# Package RPM for Fedora/RHEL
package_rpm() {
    info "Building .rpm package..."

    if ! command -v rpmbuild &> /dev/null; then
        error "rpmbuild not found. Install: sudo dnf install rpm-build"
        return 1
    fi

    # Build release binary
    cargo build --release --bin keyrx_daemon --bin keyrx_compiler

    local RPM_DIR="$HOME/rpmbuild"
    mkdir -p "$RPM_DIR"/{BUILD,RPMS,SOURCES,SPECS,SRPMS}

    # Create spec file
    cat > "$RPM_DIR/SPECS/keyrx.spec" <<EOF
Name:           keyrx
Version:        $VERSION
Release:        1%{?dist}
Summary:        Advanced keyboard remapping daemon

License:        MIT
URL:            https://github.com/RyosukeMondo/keyrx
Source0:        keyrx-$VERSION.tar.gz

%description
KeyRx provides flexible keyboard remapping with layers, macros,
and per-device configuration.

%install
mkdir -p %{buildroot}/usr/local/bin
cp keyrx_daemon %{buildroot}/usr/local/bin/
cp keyrx_compiler %{buildroot}/usr/local/bin/

%files
/usr/local/bin/keyrx_daemon
/usr/local/bin/keyrx_compiler

%changelog
* $(date "+%a %b %d %Y") KeyRx Contributors - $VERSION-1
- Release $VERSION
EOF

    # Build RPM
    rpmbuild -bb "$RPM_DIR/SPECS/keyrx.spec"

    success ".rpm package created in: $RPM_DIR/RPMS/"
}

# Package tarball
package_tarball() {
    info "Building tarball..."

    # Build release binary
    cargo build --release --bin keyrx_daemon --bin keyrx_compiler

    local TAR_DIR="$REPO_ROOT/target/package"
    local PKG_NAME="keyrx-$VERSION-linux-x86_64"

    mkdir -p "$TAR_DIR/$PKG_NAME"

    # Copy binaries
    cp "$REPO_ROOT/target/release/keyrx_daemon" "$TAR_DIR/$PKG_NAME/"
    cp "$REPO_ROOT/target/release/keyrx_compiler" "$TAR_DIR/$PKG_NAME/"

    # Copy docs
    cp "$REPO_ROOT/README.md" "$TAR_DIR/$PKG_NAME/"
    cp "$REPO_ROOT/LICENSE" "$TAR_DIR/$PKG_NAME/"

    # Create install script
    cat > "$TAR_DIR/$PKG_NAME/install.sh" <<'EOF'
#!/bin/bash
set -e
echo "Installing KeyRx..."
sudo cp keyrx_daemon /usr/local/bin/
sudo cp keyrx_compiler /usr/local/bin/
echo "KeyRx installed successfully!"
echo "Run: keyrx_daemon --help"
EOF
    chmod +x "$TAR_DIR/$PKG_NAME/install.sh"

    # Create tarball
    cd "$TAR_DIR"
    tar -czf "$PKG_NAME.tar.gz" "$PKG_NAME"

    success "Tarball created: target/package/$PKG_NAME.tar.gz"
}

# Show usage
usage() {
    cat <<EOF
Usage: $0 [OPTIONS]

Package KeyRx for distribution.

OPTIONS:
    --windows     Build Windows installer (.exe)
    --deb         Build Debian package (.deb)
    --rpm         Build RPM package (.rpm)
    --tarball     Build tarball (.tar.gz)
    --all         Build all packages
    --help        Show this help message

EXAMPLES:
    $0 --windows                  # Build Windows installer only
    $0 --deb --rpm                # Build .deb and .rpm
    $0 --all                      # Build all packages

ENVIRONMENT:
    VERSION       Package version (default: 0.1.5)

EOF
}

# Main
main() {
    local BUILD_WINDOWS=false
    local BUILD_DEB=false
    local BUILD_RPM=false
    local BUILD_TARBALL=false

    # Parse arguments
    if [ $# -eq 0 ]; then
        usage
        exit 0
    fi

    while [ $# -gt 0 ]; do
        case "$1" in
            --windows)
                BUILD_WINDOWS=true
                shift
                ;;
            --deb)
                BUILD_DEB=true
                shift
                ;;
            --rpm)
                BUILD_RPM=true
                shift
                ;;
            --tarball)
                BUILD_TARBALL=true
                shift
                ;;
            --all)
                BUILD_WINDOWS=true
                BUILD_DEB=true
                BUILD_RPM=true
                BUILD_TARBALL=true
                shift
                ;;
            --help)
                usage
                exit 0
                ;;
            *)
                error "Unknown option: $1"
                usage
                exit 1
                ;;
        esac
    done

    # Execute builds
    if [ "$BUILD_WINDOWS" = true ]; then
        package_windows
    fi

    if [ "$BUILD_DEB" = true ]; then
        package_deb
    fi

    if [ "$BUILD_RPM" = true ]; then
        package_rpm
    fi

    if [ "$BUILD_TARBALL" = true ]; then
        package_tarball
    fi

    success "All requested packages built successfully!"
}

main "$@"
