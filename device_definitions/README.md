# Device Definitions

This directory contains TOML device definition files that describe the physical layout and scancode mappings for various input devices.

## Purpose

Device definitions enable KeyRX's revolutionary mapping system to:
- Translate device-specific scancodes to physical positions (row, col)
- Support multiple identical devices with different profiles
- Render accurate visual layouts in the editor
- Provide layout-aware remapping for keyboards, macro pads, and button boxes

## Directory Structure

```
device_definitions/
├── standard/          # Generic keyboard layouts (ANSI, ISO)
├── elgato/           # Elgato Stream Deck definitions
├── [manufacturer]/   # Vendor-specific definitions
└── README.md         # This file
```

## TOML Format Specification

### Required Fields

#### Device Metadata

```toml
name = "Device Name"           # Human-readable device name
vendor_id = 0x1234            # USB Vendor ID (hex, must be non-zero)
product_id = 0x5678           # USB Product ID (hex, must be non-zero)
```

#### Layout Definition

```toml
[layout]
layout_type = "matrix"        # One of: "matrix", "standard", "split"
rows = 5                      # Number of rows (must be > 0)
cols = 15                     # Number of columns (optional, for uniform layouts)
```

For irregular layouts (different column counts per row), use `cols_per_row`:

```toml
[layout]
layout_type = "standard"
rows = 6
cols_per_row = [15, 14, 14, 13, 12, 8]  # Columns for each row
```

#### Matrix Map

The matrix map translates scancodes to physical positions. Scancodes are strings (quoted numbers), positions are `[row, col]` arrays:

```toml
[matrix_map]
"1" = [0, 0]      # ESC key at row 0, col 0
"16" = [2, 1]     # Q key at row 2, col 1
"57" = [5, 3]     # Space at row 5, col 3
```

**Important**:
- Use Linux scancodes / USB HID usage codes
- Scancodes must be quoted strings in TOML
- All positions must be within layout bounds
- Each scancode and position must be unique

### Optional Fields

#### Manufacturer

```toml
manufacturer = "Manufacturer Name"  # Device manufacturer
```

#### Visual Metadata

Provides hints for UI rendering:

```toml
[visual]
key_width = 60       # Key width in pixels
key_height = 60      # Key height in pixels
key_spacing = 4      # Spacing between keys in pixels
```

## Layout Types

### Matrix Layout

Regular grid of buttons/keys with uniform rows and columns:
- Example: Stream Deck (3×5 matrix)
- Use `cols` field for uniform column count
- All positions are (row, col) with 0-based indexing

### Standard Layout

Traditional keyboard layout with irregular row lengths:
- Example: ANSI 104-key keyboard
- Use `cols_per_row` to specify varying column counts
- Common for keyboards with staggered rows

### Split Layout

Split keyboard with two separate halves:
- Example: Ergodox, Kinesis Advantage
- Can use either `cols` or `cols_per_row`
- Each half is mapped independently in the grid

## Scancode Reference

KeyRX uses Linux scancodes which align with USB HID usage codes for most keys:

### Common Scancodes

| Key | Scancode | Key | Scancode | Key | Scancode |
|-----|----------|-----|----------|-----|----------|
| ESC | 1 | A | 30 | Space | 57 |
| 1 | 2 | S | 31 | Left Ctrl | 29 |
| 2 | 3 | D | 32 | Left Shift | 42 |
| Tab | 15 | F | 33 | Left Alt | 56 |
| Q | 16 | Enter | 28 | Right Alt | 100 |
| W | 17 | Caps Lock | 58 | Right Ctrl | 97 |
| E | 18 | Backspace | 14 | Left Meta | 125 |

### Function Keys

| Key | Scancode | Key | Scancode |
|-----|----------|-----|----------|
| F1 | 59 | F7 | 65 |
| F2 | 60 | F8 | 66 |
| F3 | 61 | F9 | 67 |
| F4 | 62 | F10 | 68 |
| F5 | 63 | F11 | 87 |
| F6 | 64 | F12 | 88 |

### Special Keys

| Key | Scancode |
|-----|----------|
| Print Screen | 99 |
| Scroll Lock | 70 |
| Pause | 119 |
| Insert | 110 |
| Delete | 111 |
| Home | 102 |
| End | 107 |
| Page Up | 104 |
| Page Down | 109 |

### Arrow Keys

| Key | Scancode |
|-----|----------|
| Up | 103 |
| Down | 108 |
| Left | 105 |
| Right | 106 |

### Numpad

| Key | Scancode | Key | Scancode |
|-----|----------|-----|----------|
| Num Lock | 69 | Numpad 5 | 76 |
| Numpad / | 98 | Numpad 6 | 77 |
| Numpad * | 55 | Numpad 7 | 71 |
| Numpad - | 74 | Numpad 8 | 72 |
| Numpad + | 78 | Numpad 9 | 73 |
| Numpad Enter | 96 | Numpad 0 | 82 |
| Numpad . | 83 | Numpad 1-4 | 79-81 + 75 |

## Validation Rules

Device definitions are validated on load. The following rules must be satisfied:

1. **IDs**: `vendor_id` and `product_id` must be non-zero
2. **Layout**:
   - `layout_type` must be "matrix", "standard", or "split"
   - `rows` must be > 0
   - `cols` must be > 0 (if specified)
   - `cols_per_row` length must equal `rows` (if specified)
   - All values in `cols_per_row` must be > 0
3. **Matrix Map**:
   - Must contain at least one scancode mapping
   - All positions must be within layout bounds
   - No duplicate scancodes
   - No duplicate positions

## Examples

### Simple Macro Pad (Matrix Layout)

```toml
name = "6-Key Macro Pad"
vendor_id = 0x1234
product_id = 0xABCD
manufacturer = "Custom"

[layout]
layout_type = "matrix"
rows = 2
cols = 3

[visual]
key_width = 80
key_height = 80
key_spacing = 8

[matrix_map]
"1" = [0, 0]
"2" = [0, 1]
"3" = [0, 2]
"4" = [1, 0]
"5" = [1, 1]
"6" = [1, 2]
```

### Stream Deck MK.2 (Matrix Layout)

```toml
name = "Elgato Stream Deck MK.2"
vendor_id = 0x0fd9
product_id = 0x0080
manufacturer = "Elgato"

[layout]
layout_type = "matrix"
rows = 3
cols = 5

[visual]
key_width = 72
key_height = 72
key_spacing = 6

[matrix_map]
"1" = [0, 0]
"2" = [0, 1]
# ... (15 buttons total)
"15" = [2, 4]
```

## Adding New Definitions

To add support for a new device:

1. Create a new `.toml` file in the appropriate vendor directory
2. Determine the device's VID:PID (use `lsusb` on Linux or Device Manager on Windows)
3. Map out the physical layout (rows × columns)
4. Identify scancodes for each button/key (use `evtest` on Linux or Raw Input API on Windows)
5. Create the matrix_map entries
6. Validate the definition by loading it in KeyRX

## Testing Definitions

Use the KeyRX CLI to validate definitions:

```bash
keyrx definitions validate device_definitions/your-device.toml
keyrx definitions list
keyrx definitions show 1234:5678
```

## Contributing

When contributing device definitions:
- Use accurate VID:PID from real devices
- Test with actual hardware when possible
- Include visual metadata for better UI rendering
- Add comments for non-obvious mappings
- Follow the existing file naming convention: `vendor-name/device-model.toml`

## License

Device definitions in this directory are part of KeyRX and are distributed under the same license.
