/// Windows Scan Codes (Set 1) for standard US Layout.
///
/// Used for manual key mapping when physical input is intercepted by the OS.
library;

/// Map of Visual Key IDs to Windows Scan Codes.
///
/// Based on standard USB HID to Scan Code Set 1 translation.
const Map<String, int> keyIdToWindowsScanCode = {
  // Row 0 (Function)
  'Escape': 0x01,
  'F1': 0x3B,
  'F2': 0x3C,
  'F3': 0x3D,
  'F4': 0x3E,
  'F5': 0x3F,
  'F6': 0x40,
  'F7': 0x41,
  'F8': 0x42,
  'F9': 0x43,
  'F10': 0x44,
  'F11': 0x57,
  'F12': 0x58,

  // Row 1 (Numbers)
  'Grave': 0x29,
  'Key1': 0x02,
  'Key2': 0x03,
  'Key3': 0x04,
  'Key4': 0x05,
  'Key5': 0x06,
  'Key6': 0x07,
  'Key7': 0x08,
  'Key8': 0x09,
  'Key9': 0x0A,
  'Key0': 0x0B,
  'Minus': 0x0C,
  'Equal': 0x0D,
  'Backspace': 0x0E,

  // Row 2 (QWERTY)
  'Tab': 0x0F,
  'KeyQ': 0x10,
  'KeyW': 0x11,
  'KeyE': 0x12,
  'KeyR': 0x13,
  'KeyT': 0x14,
  'KeyY': 0x15,
  'KeyU': 0x16,
  'KeyI': 0x17,
  'KeyO': 0x18,
  'KeyP': 0x19,
  'BracketLeft': 0x1A,
  'BracketRight': 0x1B,
  'Backslash': 0x2B,

  // Row 3 (ASDF)
  'CapsLock': 0x3A,
  'KeyA': 0x1E,
  'KeyS': 0x1F,
  'KeyD': 0x20,
  'KeyF': 0x21,
  'KeyG': 0x22,
  'KeyH': 0x23,
  'KeyJ': 0x24,
  'KeyK': 0x25,
  'KeyL': 0x26,
  'Semicolon': 0x27,
  'Quote': 0x28,
  'Enter': 0x1C,

  // Row 4 (ZXCV)
  'ShiftLeft': 0x2A,
  'KeyZ': 0x2C,
  'KeyX': 0x2D,
  'KeyC': 0x2E,
  'KeyV': 0x2F,
  'KeyB': 0x30,
  'KeyN': 0x31,
  'KeyM': 0x32,
  'Comma': 0x33,
  'Period': 0x34,
  'Slash': 0x35,
  'ShiftRight': 0x36,

  // Row 5 (Bottom)
  'ControlLeft': 0x1D,
  'MetaLeft': 0xE05B, // Extended
  'AltLeft': 0x38,
  'Space': 0x39,
  'AltRight': 0xE038, // Extended
  'MetaRight': 0xE05C, // Extended
  'ContextMenu': 0xE05D, // Extended
  'ControlRight': 0xE01D, // Extended

  // Navigation Cluster (if needed)
  'Insert': 0xE052,
  'Home': 0xE047,
  'PageUp': 0xE049,
  'Delete': 0xE053,
  'End': 0xE04F,
  'PageDown': 0xE051,
  'ArrowUp': 0xE048,
  'ArrowLeft': 0xE04B,
  'ArrowDown': 0xE050,
  'ArrowRight': 0xE04D,

  // Numpad
  'NumLock': 0x45,
  'NumpadDivide': 0xE035,
  'NumpadMultiply': 0x37,
  'NumpadSubtract': 0x4A,
  'NumpadAdd': 0x4E,
  'NumpadEnter': 0xE01C,
  'NumpadDecimal': 0x53,
  'Numpad0': 0x52,
  'Numpad1': 0x4F,
  'Numpad2': 0x50,
  'Numpad3': 0x51,
  'Numpad4': 0x4B,
  'Numpad5': 0x4C,
  'Numpad6': 0x4D,
  'Numpad7': 0x47,
  'Numpad8': 0x48,
  'Numpad9': 0x49,

  // Misc
  'PrintScreen': 0xE037,
  'ScrollLock': 0x46,
  'Pause': 0xE11D, // Very special sequence usually

  // Japanese Keys
  'NonConvert': 0x7B, // Muhenkan
  'Convert': 0x79, // Henkan
  'KanaMode': 0x70, // Katakana/Hiragana/Romaji
  'IntlRo': 0x73, // Ro (next to RShift)
  'IntlYen': 0x7D, // Yen (next to Backspace)
};
