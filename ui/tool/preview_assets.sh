#!/bin/bash
# Quick asset preview script

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/.."

echo "==========================================="
echo "KeyRx Asset Preview"
echo "==========================================="
echo ""

echo "Generated Assets:"
echo ""

echo "Icons:"
for icon in assets/icons/*.png; do
    if [ -f "$icon" ]; then
        size=$(wc -c < "$icon" | numfmt --to=iec)
        echo "  ✓ $(basename "$icon") - $size"
    fi
done

echo ""
echo "Buttons:"
for button in assets/buttons/*.png; do
    if [ -f "$button" ]; then
        size=$(wc -c < "$button" | numfmt --to=iec)
        echo "  ✓ $(basename "$button") - $size"
    fi
done

echo ""
echo "Backgrounds:"
for bg in assets/backgrounds/*.png; do
    if [ -f "$bg" ]; then
        size=$(wc -c < "$bg" | numfmt --to=iec)
        echo "  ✓ $(basename "$bg") - $size"
    fi
done

echo ""
echo "Platform Icons:"
linux_icons=$(ls linux/icons/*.png 2>/dev/null | wc -l)
echo "  ✓ Linux: $linux_icons sizes"

if [ -f "windows/runner/resources/app_icon.ico" ]; then
    win_size=$(wc -c < "windows/runner/resources/app_icon.ico" | numfmt --to=iec)
    echo "  ✓ Windows: app_icon.ico - $win_size"
fi

echo ""
echo "Total asset size:"
total_size=$(du -sh assets/ | cut -f1)
echo "  $total_size"

echo ""
echo "==========================================="
echo ""
echo "To view assets:"
echo "  - Open assets/ directory in file manager"
echo "  - Use image viewer: eog assets/icons/app_icon_base.png"
echo ""
echo "To use in Flutter:"
echo "  import 'package:keyrx_ui/ui/assets/app_assets.dart';"
echo "  Image.asset(AppAssets.appIconBase)"
echo ""
echo "To regenerate:"
echo "  ./tool/regenerate_asset.sh all"
