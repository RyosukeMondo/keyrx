# KeyRx Asset Generation System

Professional asset generation for the KeyRx Flutter application using Stable Diffusion.

## What Was Generated

### 1. Application Icons

**Linux Icons** (7 sizes for different contexts):
- 16x16, 32x32, 48x48, 64x64, 128x128, 256x256, 512x512
- Location: `linux/icons/icon_*x*.png`
- Automatically used by Linux desktop environments

**Windows Icon**:
- Multi-resolution ICO file
- Location: `windows/runner/resources/app_icon.ico`
- Automatically used by Windows

**Base App Icon**:
- High-resolution PNG (1024x1024)
- Location: `assets/icons/app_icon_base.png`
- Use for splash screens, about dialogs, etc.

### 2. UI Icons (512x512, transparent background)

- **Key Icon**: Mechanical keyboard key design
- **Settings Icon**: Modern gear icon
- **Profile Icon**: Layered document icon

Location: `assets/icons/`

### 3. Button Backgrounds (512x128)

- **Primary Button**: Blue-purple gradient for main actions
- **Secondary Button**: Gray gradient for secondary actions
- **Danger Button**: Red gradient for destructive actions

Location: `assets/buttons/`

### 4. Backgrounds (1024x1024)

- **Main Background**: Dark tech-themed background with subtle gradients
- **Panel Background**: Elegant panel texture for overlays

Location: `assets/backgrounds/`

## Usage

### Quick Start

```dart
import 'package:keyrx_ui/ui/assets/app_assets.dart';
import 'package:keyrx_ui/ui/assets/asset_button.dart';
import 'package:keyrx_ui/ui/assets/asset_icon.dart';
```

### Using Custom Buttons

```dart
AssetButton(
  text: 'Save Configuration',
  type: AssetButtonType.primary,
  onPressed: () => saveConfig(),
)

AssetButton(
  text: 'Load Default',
  type: AssetButtonType.secondary,
  onPressed: () => loadDefault(),
)

AssetButton(
  text: 'Delete Profile',
  type: AssetButtonType.danger,
  onPressed: () => deleteProfile(),
)
```

### Using Custom Icons

```dart
// Standard size
AssetIcon(
  type: AssetIconType.settings,
  size: 24,
)

// Large icon with tint
AssetIcon(
  type: AssetIconType.key,
  size: 48,
  tintColor: Colors.blue,
)

// In app bar
AppBar(
  leading: const AssetIcon(
    type: AssetIconType.profile,
    size: 24,
  ),
)
```

### Using Backgrounds

```dart
// Full screen background
Container(
  decoration: BoxDecoration(
    image: DecorationImage(
      image: AssetImage(AppAssets.mainBackground),
      fit: BoxFit.cover,
    ),
  ),
  child: YourContent(),
)

// Panel background
Container(
  decoration: BoxDecoration(
    image: DecorationImage(
      image: AssetImage(AppAssets.panelBackground),
      fit: BoxFit.cover,
    ),
    borderRadius: BorderRadius.circular(12),
  ),
  child: YourPanel(),
)
```

### Direct Asset Access

```dart
// All assets are available via AppAssets class
Image.asset(AppAssets.appIconBase)
Image.asset(AppAssets.keyIcon)
Image.asset(AppAssets.settingsIcon)
Image.asset(AppAssets.profileIcon)
Image.asset(AppAssets.primaryButton)
Image.asset(AppAssets.secondaryButton)
Image.asset(AppAssets.dangerButton)
Image.asset(AppAssets.mainBackground)
Image.asset(AppAssets.panelBackground)
```

## Regenerating Assets

### Regenerate All Assets

```bash
cd ui
./tool/regenerate_asset.sh all
```

### Regenerate Specific Type

```bash
./tool/regenerate_asset.sh icons
./tool/regenerate_asset.sh buttons
./tool/regenerate_asset.sh backgrounds
```

### Manual Generation with Python Script

```bash
cd ui
python3 tool/generate_assets.py --asset-type all
python3 tool/generate_assets.py --asset-type icons
python3 tool/generate_assets.py --model "v1-5-pruned-emaonly"
```

## Customization

### Modify Prompts

Edit `tool/generate_assets.py` and change the `AssetConfig` objects:

```python
AssetConfig(
    name="my_custom_icon",
    prompt="your custom prompt here, professional, minimal, clean",
    negative_prompt="unwanted elements, text, blur",
    width=512,
    height=512,
    steps=20,
    remove_bg=True,
    output_path=str(base_dir / "icons" / "my_custom_icon.png")
)
```

### Add New Assets

Add new `AssetConfig` entries to the appropriate category in `generate_assets.py`, then:

1. Run the generator
2. Add the asset path to `pubspec.yaml`
3. Add the constant to `lib/ui/assets/app_assets.dart`
4. Use in your Flutter app

## Design Principles

All generated assets follow these principles:

- **Professional**: Clean, modern design suitable for production apps
- **Minimal**: Simple geometric shapes, no unnecessary details
- **Consistent**: Unified color palette (blue-purple gradients)
- **Scalable**: High resolution base assets that scale well
- **Accessible**: Good contrast ratios for readability
- **Performance**: Optimized PNG files with transparency where needed

## Technical Details

### Generation Process

1. **Prompt Engineering**: Carefully crafted prompts for consistent style
2. **Stable Diffusion**: Uses local SD WebUI API (DreamShaper XL model)
3. **Background Removal**: rembg removes backgrounds from icons
4. **Multi-Resolution**: Icons generated in all required sizes
5. **Format Conversion**: Automatic ICO generation for Windows

### Quality Settings

- **Icons**: 1024x1024, 25 steps, CFG 7.5
- **Buttons**: 512x128, 15 steps, CFG 7.0
- **Backgrounds**: 1024x1024, 20-25 steps, CFG 7.0

### File Sizes

- Total assets: ~3.5 MB
- Icons: ~1.3 MB
- Buttons: ~250 KB
- Backgrounds: ~2.1 MB

## Examples

See complete examples in:
- `lib/ui/assets/example_usage.dart` - Full Flutter integration examples
- `tool/README.md` - Asset generation documentation

## Requirements

- Stable Diffusion WebUI running on http://localhost:7860
- Python 3 with requests and PIL
- rembg for background removal
- Flutter SDK

## Troubleshooting

### Assets Not Showing

1. Run `flutter pub get`
2. Restart the app (hot reload may not work for assets)
3. Check `pubspec.yaml` has correct asset paths

### Generation Errors

1. Verify SD WebUI is running: `curl http://localhost:7860/sdapi/v1/sd-models`
2. Check model is loaded correctly
3. Try reducing image sizes if you get 500 errors
4. Check rembg is installed: `pip install rembg`

### Icons Look Wrong on Platform

- **Linux**: Icons cached by system, may need to clear icon cache
- **Windows**: ICO file must be in correct location, rebuild required
- **Both**: Verify icon files exist and are not corrupted

## Next Steps

1. ✅ Assets generated and integrated
2. Review generated assets visually
3. Customize prompts if needed
4. Integrate into your existing UI components
5. Test on Linux and Windows platforms
6. Adjust colors/gradients to match your theme

## Support

For issues or questions:
- Check `tool/README.md` for detailed documentation
- Review `example_usage.dart` for integration examples
- Regenerate assets if they look incorrect
- Customize prompts for different styles
