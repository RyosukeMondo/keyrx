#!/bin/bash
# Quick regeneration script for specific asset types

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/.."

show_help() {
    echo "Usage: ./tool/regenerate_asset.sh [ASSET_TYPE]"
    echo ""
    echo "Asset types:"
    echo "  all          - Regenerate all assets"
    echo "  icons        - Regenerate icons only"
    echo "  buttons      - Regenerate button backgrounds"
    echo "  backgrounds  - Regenerate background images"
    echo "  app-icon     - Regenerate app icon and platform-specific versions"
    echo ""
    echo "Options:"
    echo "  -h, --help   - Show this help message"
    echo ""
    echo "Examples:"
    echo "  ./tool/regenerate_asset.sh icons"
    echo "  ./tool/regenerate_asset.sh backgrounds"
}

if [ $# -eq 0 ] || [ "$1" == "-h" ] || [ "$1" == "--help" ]; then
    show_help
    exit 0
fi

ASSET_TYPE=$1

echo "==========================================="
echo "KeyRx Asset Regeneration Tool"
echo "==========================================="
echo ""

# Check if Stable Diffusion is running
echo "Checking Stable Diffusion server..."
if ! curl -s http://localhost:7860/sdapi/v1/sd-models > /dev/null; then
    echo "ERROR: Stable Diffusion server not running on http://localhost:7860"
    echo "Please start Stable Diffusion WebUI first."
    exit 1
fi
echo "✓ Server is running"
echo ""

# Backup old assets
BACKUP_DIR="assets_backup_$(date +%Y%m%d_%H%M%S)"
if [ -d "assets" ]; then
    echo "Backing up current assets to: $BACKUP_DIR"
    cp -r assets "$BACKUP_DIR"
    echo "✓ Backup created"
    echo ""
fi

# Run generation
echo "Generating assets: $ASSET_TYPE"
echo ""
python3 tool/generate_assets.py --asset-type "$ASSET_TYPE"

echo ""
echo "==========================================="
echo "Regeneration complete!"
echo "==========================================="
echo ""
echo "Next steps:"
echo "1. Review the generated assets in the assets/ directory"
echo "2. Run 'flutter pub get' if needed"
echo "3. Hot reload or restart your app to see the new assets"
echo ""
echo "If you want to revert to the previous version:"
echo "  rm -rf assets && mv $BACKUP_DIR assets"
