#!/usr/bin/env python3
"""
Professional Asset Generator for KeyRx Flutter Application
Uses Stable Diffusion API and rembg for high-quality UI assets
"""

import requests
import json
import base64
from pathlib import Path
from PIL import Image
import io
import subprocess
import argparse
from typing import Optional, Tuple
from dataclasses import dataclass


@dataclass
class AssetConfig:
    """Configuration for asset generation"""
    name: str
    prompt: str
    negative_prompt: str
    width: int
    height: int
    steps: int = 20
    cfg_scale: float = 7.0
    remove_bg: bool = False
    output_path: str = ""


class StableDiffusionGenerator:
    """Interface to local Stable Diffusion API"""

    def __init__(self, api_url: str = "http://localhost:7860"):
        self.api_url = api_url
        self.txt2img_endpoint = f"{api_url}/sdapi/v1/txt2img"
        self.models_endpoint = f"{api_url}/sdapi/v1/sd-models"
        self.options_endpoint = f"{api_url}/sdapi/v1/options"

    def get_available_models(self):
        """Get list of available models"""
        response = requests.get(self.models_endpoint)
        return response.json()

    def set_model(self, model_name: str):
        """Set the active model"""
        payload = {"sd_model_checkpoint": model_name}
        response = requests.post(self.options_endpoint, json=payload)
        return response.status_code == 200

    def generate_image(self, config: AssetConfig) -> Image.Image:
        """Generate image using txt2img"""
        payload = {
            "prompt": config.prompt,
            "negative_prompt": config.negative_prompt,
            "width": config.width,
            "height": config.height,
            "steps": config.steps,
            "cfg_scale": config.cfg_scale,
            "sampler_name": "DPM++ 2M SDE",
            "sampler_index": "DPM++ 2M SDE",
            "restore_faces": False,
            "seed": -1,
            "batch_size": 1,
            "n_iter": 1,
        }

        print(f"Generating: {config.name}")
        print(f"  Prompt: {config.prompt}")
        print(f"  Size: {config.width}x{config.height}")

        response = requests.post(self.txt2img_endpoint, json=payload)
        response.raise_for_status()

        result = response.json()
        image_data = base64.b64decode(result['images'][0])
        image = Image.open(io.BytesIO(image_data))

        return image

    def remove_background(self, image: Image.Image) -> Image.Image:
        """Remove background using rembg"""
        print("  Removing background with rembg...")

        # Save to temp file
        temp_input = "/tmp/temp_input.png"
        temp_output = "/tmp/temp_output.png"
        image.save(temp_input, "PNG")

        # Run rembg
        subprocess.run(["rembg", "i", temp_input, temp_output], check=True)

        # Load result
        result = Image.open(temp_output)
        return result

    def save_image(self, image: Image.Image, path: str):
        """Save image to file"""
        Path(path).parent.mkdir(parents=True, exist_ok=True)
        image.save(path, "PNG")
        print(f"  Saved: {path}")


class IconGenerator:
    """Generate app icons for different platforms"""

    def __init__(self, sd_gen: StableDiffusionGenerator):
        self.sd = sd_gen

    def generate_linux_icons(self, base_image: Image.Image, output_dir: str):
        """Generate Linux icon set (16, 32, 48, 64, 128, 256, 512)"""
        sizes = [16, 32, 48, 64, 128, 256, 512]

        output_path = Path(output_dir)
        output_path.mkdir(parents=True, exist_ok=True)

        for size in sizes:
            resized = base_image.resize((size, size), Image.Resampling.LANCZOS)
            icon_path = output_path / f"icon_{size}x{size}.png"
            resized.save(icon_path, "PNG")
            print(f"  Generated: {icon_path}")

    def generate_windows_ico(self, base_image: Image.Image, output_path: str):
        """Generate Windows ICO file with multiple sizes"""
        sizes = [(16, 16), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)]

        # Create icon with multiple sizes
        icons = []
        for size in sizes:
            resized = base_image.resize(size, Image.Resampling.LANCZOS)
            icons.append(resized)

        # Save as ICO
        Path(output_path).parent.mkdir(parents=True, exist_ok=True)
        icons[0].save(output_path, format='ICO', sizes=sizes)
        print(f"  Generated: {output_path}")


def main():
    parser = argparse.ArgumentParser(description="Generate professional assets for KeyRx Flutter app")
    parser.add_argument("--api-url", default="http://localhost:7860", help="Stable Diffusion API URL")
    parser.add_argument("--model", default="dreamshaperXL_lightningDPMSDE", help="Model to use")
    parser.add_argument("--asset-type", choices=["all", "icons", "buttons", "backgrounds"],
                       default="all", help="Type of assets to generate")
    args = parser.parse_args()

    # Initialize generator
    sd = StableDiffusionGenerator(args.api_url)
    icon_gen = IconGenerator(sd)

    # Set model
    print(f"Setting model: {args.model}")
    sd.set_model(args.model)

    # Base directory
    base_dir = Path(__file__).parent.parent / "assets"

    # Asset configurations for KeyRx (keyboard remapping app)
    assets = {
        "icons": [
            AssetConfig(
                name="app_icon",
                prompt="modern minimal keyboard icon logo, tech startup style, flat design, professional, "
                       "gradient blue to purple, clean lines, centered, isometric view, "
                       "gaming keyboard aesthetic, high contrast, vector style, simple geometric shapes",
                negative_prompt="text, letters, words, watermark, signature, blur, noise, realistic, "
                               "photographic, complex, cluttered, 3d render",
                width=1024,
                height=1024,
                steps=25,
                cfg_scale=7.5,
                remove_bg=True,
                output_path=str(base_dir / "icons" / "app_icon_base.png")
            ),
            AssetConfig(
                name="key_icon",
                prompt="single keyboard key icon, mechanical keyboard style, minimalist, "
                       "gradient metallic, professional UI design, clean, centered, simple",
                negative_prompt="text, letters, keyboard layout, multiple keys, realistic, complex",
                width=512,
                height=512,
                steps=20,
                remove_bg=True,
                output_path=str(base_dir / "icons" / "key_icon.png")
            ),
            AssetConfig(
                name="settings_icon",
                prompt="modern gear icon, minimal flat design, professional UI, "
                       "gradient blue purple, clean lines, simple geometric",
                negative_prompt="text, realistic, complex, 3d, photographic",
                width=512,
                height=512,
                steps=20,
                remove_bg=True,
                output_path=str(base_dir / "icons" / "settings_icon.png")
            ),
            AssetConfig(
                name="profile_icon",
                prompt="document layers icon, flat design, minimal, professional UI, "
                       "gradient accent colors, clean modern",
                negative_prompt="text, realistic, complex, photographic, 3d",
                width=512,
                height=512,
                steps=20,
                remove_bg=True,
                output_path=str(base_dir / "icons" / "profile_icon.png")
            ),
        ],
        "buttons": [
            AssetConfig(
                name="primary_button",
                prompt="modern button background, gradient blue to purple, "
                       "smooth professional UI design, subtle glow, flat design, "
                       "rounded corners aesthetic, clean minimal",
                negative_prompt="text, icons, symbols, realistic, complex, border",
                width=512,
                height=128,
                steps=15,
                output_path=str(base_dir / "buttons" / "primary_button.png")
            ),
            AssetConfig(
                name="secondary_button",
                prompt="elegant button background, subtle gradient gray, "
                       "professional UI, modern flat design, minimal clean",
                negative_prompt="text, icons, bright colors, realistic",
                width=512,
                height=128,
                steps=15,
                output_path=str(base_dir / "buttons" / "secondary_button.png")
            ),
            AssetConfig(
                name="danger_button",
                prompt="alert button background, gradient red to dark red, "
                       "modern UI design, smooth professional, warning aesthetic",
                negative_prompt="text, icons, symbols, realistic",
                width=512,
                height=128,
                steps=15,
                output_path=str(base_dir / "buttons" / "danger_button.png")
            ),
        ],
        "backgrounds": [
            AssetConfig(
                name="main_background",
                prompt="abstract tech background, dark theme, subtle gradient, "
                       "minimal geometric patterns, professional software UI, "
                       "deep blue purple gradient, clean modern, "
                       "subtle circuit board pattern, tech aesthetic, elegant",
                negative_prompt="busy, cluttered, bright, colorful, realistic objects, "
                               "text, icons, photo, people",
                width=1024,
                height=1024,
                steps=25,
                output_path=str(base_dir / "backgrounds" / "main_background.png")
            ),
            AssetConfig(
                name="panel_background",
                prompt="subtle panel background texture, dark elegant, "
                       "minimal gradient, professional UI, soft edges, "
                       "tech aesthetic, clean modern",
                negative_prompt="bright, colorful, complex, realistic, text, icons",
                width=1024,
                height=1024,
                steps=20,
                output_path=str(base_dir / "backgrounds" / "panel_background.png")
            ),
        ],
    }

    # Generate assets based on type
    asset_types = ["icons", "buttons", "backgrounds"] if args.asset_type == "all" else [args.asset_type]

    for asset_type in asset_types:
        print(f"\n{'='*60}")
        print(f"Generating {asset_type.upper()}")
        print(f"{'='*60}\n")

        for config in assets[asset_type]:
            # Generate image
            image = sd.generate_image(config)

            # Remove background if needed
            if config.remove_bg:
                image = sd.remove_background(image)

            # Save base image
            sd.save_image(image, config.output_path)

            # Special handling for app icon - generate platform-specific versions
            if config.name == "app_icon":
                print("\n  Generating platform-specific icons...")

                # Linux icons
                linux_icon_dir = str(base_dir.parent / "linux" / "icons")
                icon_gen.generate_linux_icons(image, linux_icon_dir)

                # Windows icon
                windows_icon_path = str(base_dir.parent / "windows" / "runner" / "resources" / "app_icon.ico")
                icon_gen.generate_windows_ico(image, windows_icon_path)

            print()

    print(f"\n{'='*60}")
    print("Asset generation complete!")
    print(f"{'='*60}")
    print(f"\nGenerated assets in: {base_dir}")
    print("\nNext steps:")
    print("1. Review generated assets")
    print("2. Update pubspec.yaml to reference new assets")
    print("3. Rebuild Flutter app")


if __name__ == "__main__":
    main()
