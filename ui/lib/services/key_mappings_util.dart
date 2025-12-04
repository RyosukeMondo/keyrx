/// Utility functions and constants for key validation.
library;

class KeyMappings {
  static List<String> allowedKeys = [
    'esc', 'f1', 'f2', 'f3', 'f4', 'f5', 'f6', 'f7', 'f8', 'f9', 'f10', 'f11',
    'f12', '`', '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '-', '=',
    'backspace', 'tab', 'q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p', '[',
    ']', '\\', 'capslock', 'a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l', ';',
    "'", 'enter', 'lshift', 'z', 'x', 'c', 'v', 'b', 'n', 'm', ',', '.', '/',
    'rshift', 'lctrl', 'lwin', 'lalt', 'space', 'ralt', 'rwin', 'menu', 'rctrl',
  ];

  static bool isKnownKey(String key) {
    if (key.isEmpty) return false;
    final normalized = key.toLowerCase();
    return allowedKeys.contains(normalized);
  }

  /// Allow runtime updates from the native bridge.
  static void updateAllowedKeys(List<String> keys) {
    if (keys.isNotEmpty) {
      final normalized =
          keys
              .map((k) => k.toLowerCase())
              .where((k) => k.isNotEmpty)
              .toSet()
              .toList()
            ..sort();
      allowedKeys = normalized;
    }
  }
}
