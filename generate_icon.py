#!/usr/bin/env python3
"""
Generate app icon: simple keycap with [Rx] text
Uses localhost:7860 for image generation and rembg for transparent background
"""
import io
import base64
import json
import requests
from PIL import Image, ImageDraw, ImageFont

def generate_with_api():
    """Generate image using Stable Diffusion API at localhost:7860"""
    url = "http://localhost:7860/sdapi/v1/txt2img"

    payload = {
        "prompt": "professional 3D render of a single mechanical keyboard keycap, cherry mx style, with '[Rx]' text engraved on top, studio lighting, white keycap with black text, centered composition, product photography, high quality, sharp focus, clean background",
        "negative_prompt": "multiple keys, keyboard, blurry, low quality, text artifacts, watermark, signature, multiple objects, cluttered",
        "steps": 30,
        "width": 1024,
        "height": 1024,
        "cfg_scale": 7,
        "sampler_name": "DPM++ 2M Karras",
    }

    try:
        response = requests.post(url, json=payload, timeout=120)
        response.raise_for_status()
        r = response.json()

        # Save the generated image
        image_data = base64.b64decode(r['images'][0])
        image = Image.open(io.BytesIO(image_data))
        image.save('/tmp/keycap_raw.png')
        print("Generated image saved to /tmp/keycap_raw.png")
        return image
    except Exception as e:
        print(f"API generation failed: {e}")
        return None

def generate_simple_keycap():
    """Generate a simple keycap image using PIL"""
    size = 1024
    img = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    # Draw keycap shape (rounded rectangle with perspective)
    keycap_color = (240, 240, 245, 255)  # Light gray keycap
    border_color = (180, 180, 190, 255)  # Darker border

    # Top face of keycap (slightly smaller trapezoid for perspective)
    top_margin = size // 6
    bottom_margin = size // 8
    top_points = [
        (top_margin, top_margin),
        (size - top_margin, top_margin),
        (size - bottom_margin, size - bottom_margin),
        (bottom_margin, size - bottom_margin)
    ]

    # Draw shadow/depth
    shadow_offset = 15
    shadow_points = [(x + shadow_offset, y + shadow_offset) for x, y in top_points]
    draw.polygon(shadow_points, fill=(100, 100, 110, 180))

    # Draw main keycap
    draw.polygon(top_points, fill=keycap_color, outline=border_color, width=3)

    # Add slight highlight on top edge
    highlight_points = top_points[:2] + [
        (size - top_margin - 30, top_margin + 30),
        (top_margin + 30, top_margin + 30)
    ]
    draw.polygon(highlight_points, fill=(255, 255, 255, 80))

    # Draw [Rx] text
    try:
        # Try to use a nice font
        font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf", 180)
    except:
        font = ImageFont.load_default()

    text = "[Rx]"
    # Get text bounding box
    bbox = draw.textbbox((0, 0), text, font=font)
    text_width = bbox[2] - bbox[0]
    text_height = bbox[3] - bbox[1]

    # Center text on keycap
    text_x = (size - text_width) // 2
    text_y = (size - text_height) // 2 - 20

    # Draw text shadow
    draw.text((text_x + 3, text_y + 3), text, fill=(0, 0, 0, 100), font=font)
    # Draw main text
    draw.text((text_x, text_y), text, fill=(40, 40, 50, 255), font=font)

    img.save('/tmp/keycap_raw.png')
    print("Generated simple keycap image saved to /tmp/keycap_raw.png")
    return img

def remove_background():
    """Remove background using rembg"""
    try:
        import subprocess
        result = subprocess.run(
            ['rembg', 'i', '/tmp/keycap_raw.png', '/tmp/keycap_transparent.png'],
            capture_output=True,
            text=True,
            timeout=60
        )
        if result.returncode == 0:
            print("Background removed successfully: /tmp/keycap_transparent.png")
            return True
        else:
            print(f"rembg failed: {result.stderr}")
            return False
    except FileNotFoundError:
        print("rembg not found, trying to install...")
        return False
    except Exception as e:
        print(f"Background removal failed: {e}")
        return False

def main():
    print("Generating keycap icon with [Rx] text...")

    # Try API first, fall back to simple generation
    image = generate_with_api()
    if image is None:
        print("Falling back to simple PIL generation...")
        image = generate_simple_keycap()

    # Remove background
    if not remove_background():
        print("Using original image without background removal")
        # Just copy the file
        import shutil
        shutil.copy('/tmp/keycap_raw.png', '/tmp/keycap_transparent.png')

    # Copy to project directory
    import shutil
    shutil.copy('/tmp/keycap_transparent.png', '/home/rmondo/repos/keyrx/icon.png')
    print("\nIcon saved to: /home/rmondo/repos/keyrx/icon.png")
    print("\nNext steps:")
    print("1. Check the icon: xdg-open /home/rmondo/repos/keyrx/icon.png")
    print("2. Generate Flutter icons: flutter pub run flutter_launcher_icons")

if __name__ == '__main__':
    main()
