#!/usr/bin/env bash
# Comprehensive build script for KeyRX
#
# This script orchestrates the complete build process:
# 1. Generates error documentation
# 2. Generates Dart FFI bindings
# 3. Builds Rust core library
# 4. Builds Flutter UI
#
# Usage:
#   ./scripts/build.sh [options]
#
# Options:
#   --release       Build in release mode (default: debug)
#   --rust-only     Build only Rust core
#   --flutter-only  Build only Flutter UI
#   --verify        Verify bindings without rebuilding
#   --help          Show this help message

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default options
BUILD_MODE="debug"
BUILD_RUST=true
BUILD_FLUTTER=true
VERIFY_ONLY=false

# Get script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --release)
            BUILD_MODE="release"
            shift
            ;;
        --rust-only)
            BUILD_FLUTTER=false
            shift
            ;;
        --flutter-only)
            BUILD_RUST=false
            shift
            ;;
        --verify)
            VERIFY_ONLY=true
            shift
            ;;
        --help)
            sed -n '2,/^$/p' "$0" | sed 's/^# //'
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Change to project root
cd "$PROJECT_ROOT"

echo "==================================================================="
echo -e "${BLUE}KeyRX Build Script${NC}"
echo "==================================================================="
echo "Mode: $BUILD_MODE"
echo "Build Rust: $BUILD_RUST"
echo "Build Flutter: $BUILD_FLUTTER"
echo ""

# Verify bindings only
if [ "$VERIFY_ONLY" = true ]; then
    echo -e "${BLUE}Verifying FFI bindings...${NC}"
    if python3 scripts/verify_bindings.py; then
        echo -e "${GREEN}✓ Bindings verified${NC}"
        exit 0
    else
        echo -e "${RED}✗ Bindings verification failed${NC}"
        exit 1
    fi
fi

# Step 1: Generate error documentation
if [ "$BUILD_RUST" = true ]; then
    echo -e "${BLUE}Step 1: Generating error documentation...${NC}"
    cd core
    cargo run --bin generate_error_docs
    cd ..
    echo -e "${GREEN}✓ Error documentation generated${NC}"
    echo ""
fi

# Step 2: Generate Dart FFI bindings
echo -e "${BLUE}Step 2: Generating Dart FFI bindings...${NC}"
python3 scripts/generate_dart_bindings.py
if command -v dart &> /dev/null; then
    echo "Formatting generated Dart code..."
    cd ui
    dart format lib/ffi/generated/bindings_generated.dart
    cd ..
fi
echo -e "${GREEN}✓ Dart FFI bindings generated${NC}"
echo ""

# Step 3: Verify bindings
echo -e "${BLUE}Step 3: Verifying FFI bindings...${NC}"
if python3 scripts/verify_bindings.py; then
    echo -e "${GREEN}✓ FFI bindings verified${NC}"
else
    echo -e "${RED}✗ FFI bindings verification failed${NC}"
    exit 1
fi
echo ""

# Step 4: Build Rust core
if [ "$BUILD_RUST" = true ]; then
    echo -e "${BLUE}Step 4: Building Rust core library...${NC}"
    cd core
    if [ "$BUILD_MODE" = "release" ]; then
        cargo build --release
    else
        cargo build
    fi
    cd ..
    echo -e "${GREEN}✓ Rust core built${NC}"
    echo ""
fi

# Step 5: Build Flutter UI
if [ "$BUILD_FLUTTER" = true ]; then
    echo -e "${BLUE}Step 5: Building Flutter UI...${NC}"
    cd ui
    if [ "$BUILD_MODE" = "release" ]; then
        flutter build linux --release
    else
        # For debug, just ensure dependencies are fetched
        flutter pub get
    fi
    cd ..
    echo -e "${GREEN}✓ Flutter UI prepared${NC}"
    echo ""
fi

echo "==================================================================="
echo -e "${GREEN}Build completed successfully!${NC}"
echo "==================================================================="

# Show output locations
if [ "$BUILD_RUST" = true ]; then
    if [ "$BUILD_MODE" = "release" ]; then
        echo "Rust library: core/target/release/"
    else
        echo "Rust library: core/target/debug/"
    fi
fi

if [ "$BUILD_FLUTTER" = true ] && [ "$BUILD_MODE" = "release" ]; then
    echo "Flutter bundle: ui/build/linux/x64/release/"
fi
