# KeyRx Asset Generator

Professional asset generation tool using Stable Diffusion for the KeyRx Flutter application.

## Features

- Generate app icons for Linux (multiple sizes: 16x16 to 512x512)
- Generate Windows app icon (.ico format with embedded sizes)
- Generate UI button backgrounds with professional gradients
- Generate background images for the application
- Automatic background removal using rembg for icons
- Professional, clean, minimal design aesthetic

## Prerequisites

1. **Stable Diffusion WebUI** running locally on `http://localhost:7860`
2. **Python 3** with required packages:
   ```bash
   pip install requests pillow
   ```
3. **rembg** for background removal:
   ```bash
   pip install rembg
   ```

## Usage

### Generate All Assets

```bash
cd ui
python3 tool/generate_assets.py --asset-type all
```

### Generate Specific Asset Types

```bash
# Icons only
python3 tool/generate_assets.py --asset-type icons

# Buttons only
python3 tool/generate_assets.py --asset-type buttons

# Backgrounds only
python3 tool/generate_assets.py --asset-type backgrounds
```

### Use Different SD Model

```bash
python3 tool/generate_assets.py --model "v1-5-pruned-emaonly"
```

### Custom API URL

```bash
python3 tool/generate_assets.py --api-url "http://192.168.1.100:7860"
```

## Generated Assets

### Icons
- `assets/icons/app_icon_base.png` - Base application icon (1024x1024)
- `assets/icons/key_icon.png` - Keyboard key icon
- `assets/icons/settings_icon.png` - Settings gear icon
- `assets/icons/profile_icon.png` - Profile/layers icon
- `linux/icons/icon_*x*.png` - Linux app icons (7 sizes)
- `windows/runner/resources/app_icon.ico` - Windows app icon

### Buttons
- `assets/buttons/primary_button.png` - Primary action button (blue-purple gradient)
- `assets/buttons/secondary_button.png` - Secondary button (gray gradient)
- `assets/buttons/danger_button.png` - Danger/delete button (red gradient)

### Backgrounds
- `assets/backgrounds/main_background.png` - Main app background
- `assets/backgrounds/panel_background.png` - Panel background texture

## Using Assets in Flutter

### Import the helper classes:

```dart
import 'package:keyrx_ui/ui/assets/app_assets.dart';
import 'package:keyrx_ui/ui/assets/asset_button.dart';
import 'package:keyrx_ui/ui/assets/asset_icon.dart';
```

### Use custom buttons:

```dart
AssetButton(
  text: 'Save Configuration',
  type: AssetButtonType.primary,
  onPressed: () => _saveConfig(),
)
```

### Use custom icons:

```dart
AssetIcon(
  type: AssetIconType.settings,
  size: 32,
)
```

### Use backgrounds:

```dart
Container(
  decoration: BoxDecoration(
    image: DecorationImage(
      image: AssetImage(AppAssets.mainBackground),
      fit: BoxFit.cover,
    ),
  ),
  child: YourWidget(),
)
```

## Customization

Edit `generate_assets.py` to customize:

- **Prompts**: Modify the `AssetConfig` objects to change the visual style
- **Sizes**: Adjust width/height parameters
- **Quality**: Modify steps and cfg_scale parameters
- **Add new assets**: Add more `AssetConfig` entries to any category

## Prompt Engineering Tips

For best results:
- Use clear, descriptive prompts
- Specify "flat design", "minimal", "professional" for UI elements
- Add "clean", "simple", "modern" for better results
- Use negative prompts to exclude unwanted elements
- For icons, always use `remove_bg=True`
- Experiment with different cfg_scale values (7-9 work well)

## Troubleshooting

### Server Error (500)
- Check if Stable Diffusion WebUI is running
- Try reducing image size
- Switch to a different model
- Reduce the number of steps

### Background Removal Fails
- Ensure rembg is installed: `pip install rembg`
- Check if the rembg model has been downloaded
- Verify the input image has a clear subject

### Icons Look Blurry
- Increase the base resolution (1024x1024 minimum)
- Increase the number of steps (25-30)
- Use a higher quality model like DreamShaper XL

## Examples

See `example_usage.dart` for complete Flutter integration examples.
