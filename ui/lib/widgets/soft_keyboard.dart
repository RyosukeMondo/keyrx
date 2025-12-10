// Soft keyboard palette widget showing all available output keys.
//
// Displays all KeyCode values in a searchable grid, allowing users to
// select keys for mapping to physical positions.

import 'package:flutter/material.dart';

/// Data model for a key in the soft keyboard palette.
class KeyInfo {
  /// Key variant name (matches Rust KeyCode enum)
  final String variant;

  /// Display name shown to user
  final String display;

  /// Category for grouping
  final String category;

  const KeyInfo({
    required this.variant,
    required this.display,
    required this.category,
  });
}

/// Widget displaying a palette of all available output keys.
///
/// Shows all KeyCode values in a searchable grid, organized by category.
/// Users can search/filter keys and select them for remapping.
class SoftKeyboard extends StatefulWidget {
  /// Callback when a key is selected
  final void Function(String keyVariant)? onKeySelected;

  /// Currently selected key variant (highlighted)
  final String? selectedKey;

  /// Key button size in pixels
  final double keySize;

  /// Spacing between keys in pixels
  final double keySpacing;

  const SoftKeyboard({
    super.key,
    this.onKeySelected,
    this.selectedKey,
    this.keySize = 56.0,
    this.keySpacing = 8.0,
  });

  @override
  State<SoftKeyboard> createState() => _SoftKeyboardState();
}

class _SoftKeyboardState extends State<SoftKeyboard> {
  /// Search query for filtering keys
  String _searchQuery = '';

  /// All available keys, organized by category
  static const List<KeyInfo> _allKeys = [
    // Letters A-Z
    KeyInfo(variant: 'A', display: 'A', category: 'Letters'),
    KeyInfo(variant: 'B', display: 'B', category: 'Letters'),
    KeyInfo(variant: 'C', display: 'C', category: 'Letters'),
    KeyInfo(variant: 'D', display: 'D', category: 'Letters'),
    KeyInfo(variant: 'E', display: 'E', category: 'Letters'),
    KeyInfo(variant: 'F', display: 'F', category: 'Letters'),
    KeyInfo(variant: 'G', display: 'G', category: 'Letters'),
    KeyInfo(variant: 'H', display: 'H', category: 'Letters'),
    KeyInfo(variant: 'I', display: 'I', category: 'Letters'),
    KeyInfo(variant: 'J', display: 'J', category: 'Letters'),
    KeyInfo(variant: 'K', display: 'K', category: 'Letters'),
    KeyInfo(variant: 'L', display: 'L', category: 'Letters'),
    KeyInfo(variant: 'M', display: 'M', category: 'Letters'),
    KeyInfo(variant: 'N', display: 'N', category: 'Letters'),
    KeyInfo(variant: 'O', display: 'O', category: 'Letters'),
    KeyInfo(variant: 'P', display: 'P', category: 'Letters'),
    KeyInfo(variant: 'Q', display: 'Q', category: 'Letters'),
    KeyInfo(variant: 'R', display: 'R', category: 'Letters'),
    KeyInfo(variant: 'S', display: 'S', category: 'Letters'),
    KeyInfo(variant: 'T', display: 'T', category: 'Letters'),
    KeyInfo(variant: 'U', display: 'U', category: 'Letters'),
    KeyInfo(variant: 'V', display: 'V', category: 'Letters'),
    KeyInfo(variant: 'W', display: 'W', category: 'Letters'),
    KeyInfo(variant: 'X', display: 'X', category: 'Letters'),
    KeyInfo(variant: 'Y', display: 'Y', category: 'Letters'),
    KeyInfo(variant: 'Z', display: 'Z', category: 'Letters'),

    // Numbers 0-9
    KeyInfo(variant: 'Key0', display: '0', category: 'Numbers'),
    KeyInfo(variant: 'Key1', display: '1', category: 'Numbers'),
    KeyInfo(variant: 'Key2', display: '2', category: 'Numbers'),
    KeyInfo(variant: 'Key3', display: '3', category: 'Numbers'),
    KeyInfo(variant: 'Key4', display: '4', category: 'Numbers'),
    KeyInfo(variant: 'Key5', display: '5', category: 'Numbers'),
    KeyInfo(variant: 'Key6', display: '6', category: 'Numbers'),
    KeyInfo(variant: 'Key7', display: '7', category: 'Numbers'),
    KeyInfo(variant: 'Key8', display: '8', category: 'Numbers'),
    KeyInfo(variant: 'Key9', display: '9', category: 'Numbers'),

    // Function keys F1-F12
    KeyInfo(variant: 'F1', display: 'F1', category: 'Function'),
    KeyInfo(variant: 'F2', display: 'F2', category: 'Function'),
    KeyInfo(variant: 'F3', display: 'F3', category: 'Function'),
    KeyInfo(variant: 'F4', display: 'F4', category: 'Function'),
    KeyInfo(variant: 'F5', display: 'F5', category: 'Function'),
    KeyInfo(variant: 'F6', display: 'F6', category: 'Function'),
    KeyInfo(variant: 'F7', display: 'F7', category: 'Function'),
    KeyInfo(variant: 'F8', display: 'F8', category: 'Function'),
    KeyInfo(variant: 'F9', display: 'F9', category: 'Function'),
    KeyInfo(variant: 'F10', display: 'F10', category: 'Function'),
    KeyInfo(variant: 'F11', display: 'F11', category: 'Function'),
    KeyInfo(variant: 'F12', display: 'F12', category: 'Function'),

    // Modifier keys
    KeyInfo(variant: 'LeftShift', display: 'LShift', category: 'Modifiers'),
    KeyInfo(variant: 'RightShift', display: 'RShift', category: 'Modifiers'),
    KeyInfo(variant: 'LeftCtrl', display: 'LCtrl', category: 'Modifiers'),
    KeyInfo(variant: 'RightCtrl', display: 'RCtrl', category: 'Modifiers'),
    KeyInfo(variant: 'LeftAlt', display: 'LAlt', category: 'Modifiers'),
    KeyInfo(variant: 'RightAlt', display: 'RAlt', category: 'Modifiers'),
    KeyInfo(variant: 'LeftMeta', display: 'LWin', category: 'Modifiers'),
    KeyInfo(variant: 'RightMeta', display: 'RWin', category: 'Modifiers'),

    // Navigation keys
    KeyInfo(variant: 'Up', display: '↑', category: 'Navigation'),
    KeyInfo(variant: 'Down', display: '↓', category: 'Navigation'),
    KeyInfo(variant: 'Left', display: '←', category: 'Navigation'),
    KeyInfo(variant: 'Right', display: '→', category: 'Navigation'),
    KeyInfo(variant: 'Home', display: 'Home', category: 'Navigation'),
    KeyInfo(variant: 'End', display: 'End', category: 'Navigation'),
    KeyInfo(variant: 'PageUp', display: 'PgUp', category: 'Navigation'),
    KeyInfo(variant: 'PageDown', display: 'PgDn', category: 'Navigation'),

    // Editing keys
    KeyInfo(variant: 'Insert', display: 'Ins', category: 'Editing'),
    KeyInfo(variant: 'Delete', display: 'Del', category: 'Editing'),
    KeyInfo(variant: 'Backspace', display: 'Bksp', category: 'Editing'),

    // Whitespace keys
    KeyInfo(variant: 'Space', display: 'Space', category: 'Whitespace'),
    KeyInfo(variant: 'Tab', display: 'Tab', category: 'Whitespace'),
    KeyInfo(variant: 'Enter', display: 'Enter', category: 'Whitespace'),

    // Lock keys
    KeyInfo(variant: 'CapsLock', display: 'Caps', category: 'Lock'),
    KeyInfo(variant: 'NumLock', display: 'Num', category: 'Lock'),
    KeyInfo(variant: 'ScrollLock', display: 'Scroll', category: 'Lock'),

    // Special keys
    KeyInfo(variant: 'Escape', display: 'Esc', category: 'Special'),
    KeyInfo(variant: 'PrintScreen', display: 'PrtSc', category: 'Special'),
    KeyInfo(variant: 'Pause', display: 'Pause', category: 'Special'),

    // Punctuation and symbols
    KeyInfo(variant: 'Grave', display: '`', category: 'Symbols'),
    KeyInfo(variant: 'Minus', display: '-', category: 'Symbols'),
    KeyInfo(variant: 'Equal', display: '=', category: 'Symbols'),
    KeyInfo(variant: 'LeftBracket', display: '[', category: 'Symbols'),
    KeyInfo(variant: 'RightBracket', display: ']', category: 'Symbols'),
    KeyInfo(variant: 'Backslash', display: '\\', category: 'Symbols'),
    KeyInfo(variant: 'Semicolon', display: ';', category: 'Symbols'),
    KeyInfo(variant: 'Apostrophe', display: "'", category: 'Symbols'),
    KeyInfo(variant: 'Comma', display: ',', category: 'Symbols'),
    KeyInfo(variant: 'Period', display: '.', category: 'Symbols'),
    KeyInfo(variant: 'Slash', display: '/', category: 'Symbols'),

    // Numpad keys
    KeyInfo(variant: 'Numpad0', display: 'Num0', category: 'Numpad'),
    KeyInfo(variant: 'Numpad1', display: 'Num1', category: 'Numpad'),
    KeyInfo(variant: 'Numpad2', display: 'Num2', category: 'Numpad'),
    KeyInfo(variant: 'Numpad3', display: 'Num3', category: 'Numpad'),
    KeyInfo(variant: 'Numpad4', display: 'Num4', category: 'Numpad'),
    KeyInfo(variant: 'Numpad5', display: 'Num5', category: 'Numpad'),
    KeyInfo(variant: 'Numpad6', display: 'Num6', category: 'Numpad'),
    KeyInfo(variant: 'Numpad7', display: 'Num7', category: 'Numpad'),
    KeyInfo(variant: 'Numpad8', display: 'Num8', category: 'Numpad'),
    KeyInfo(variant: 'Numpad9', display: 'Num9', category: 'Numpad'),
    KeyInfo(variant: 'NumpadAdd', display: 'Num+', category: 'Numpad'),
    KeyInfo(variant: 'NumpadSubtract', display: 'Num-', category: 'Numpad'),
    KeyInfo(variant: 'NumpadMultiply', display: 'Num*', category: 'Numpad'),
    KeyInfo(variant: 'NumpadDivide', display: 'Num/', category: 'Numpad'),
    KeyInfo(variant: 'NumpadEnter', display: 'NumEnt', category: 'Numpad'),
    KeyInfo(variant: 'NumpadDecimal', display: 'Num.', category: 'Numpad'),

    // Media keys
    KeyInfo(variant: 'VolumeUp', display: 'Vol+', category: 'Media'),
    KeyInfo(variant: 'VolumeDown', display: 'Vol-', category: 'Media'),
    KeyInfo(variant: 'VolumeMute', display: 'Mute', category: 'Media'),
    KeyInfo(variant: 'MediaPlayPause', display: 'Play', category: 'Media'),
    KeyInfo(variant: 'MediaStop', display: 'Stop', category: 'Media'),
    KeyInfo(variant: 'MediaNext', display: 'Next', category: 'Media'),
    KeyInfo(variant: 'MediaPrev', display: 'Prev', category: 'Media'),
  ];

  /// Get filtered keys based on search query
  List<KeyInfo> get _filteredKeys {
    if (_searchQuery.isEmpty) {
      return _allKeys;
    }

    final query = _searchQuery.toLowerCase();
    return _allKeys.where((key) {
      return key.variant.toLowerCase().contains(query) ||
          key.display.toLowerCase().contains(query) ||
          key.category.toLowerCase().contains(query);
    }).toList();
  }

  @override
  Widget build(BuildContext context) {
    final filteredKeys = _filteredKeys;
    final theme = Theme.of(context);

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        // Search bar
        Padding(
          padding: const EdgeInsets.all(16.0),
          child: TextField(
            decoration: InputDecoration(
              hintText: 'Search keys...',
              prefixIcon: const Icon(Icons.search),
              suffixIcon: _searchQuery.isNotEmpty
                  ? IconButton(
                      icon: const Icon(Icons.clear),
                      onPressed: () {
                        setState(() {
                          _searchQuery = '';
                        });
                      },
                    )
                  : null,
              border: OutlineInputBorder(
                borderRadius: BorderRadius.circular(8.0),
              ),
            ),
            onChanged: (value) {
              setState(() {
                _searchQuery = value;
              });
            },
          ),
        ),

        // Results count
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16.0),
          child: Text(
            '${filteredKeys.length} keys',
            style: theme.textTheme.bodySmall?.copyWith(
              color: theme.textTheme.bodySmall?.color?.withValues(alpha: 0.6),
            ),
          ),
        ),

        const SizedBox(height: 8),

        // Key grid
        Expanded(
          child: filteredKeys.isEmpty
              ? Center(
                  child: Column(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Icon(
                        Icons.search_off,
                        size: 48,
                        color: theme.disabledColor,
                      ),
                      const SizedBox(height: 16),
                      Text(
                        'No keys found',
                        style: theme.textTheme.bodyLarge?.copyWith(
                          color: theme.disabledColor,
                        ),
                      ),
                    ],
                  ),
                )
              : GridView.builder(
                  padding: const EdgeInsets.all(16.0),
                  gridDelegate: SliverGridDelegateWithMaxCrossAxisExtent(
                    maxCrossAxisExtent: widget.keySize,
                    mainAxisSpacing: widget.keySpacing,
                    crossAxisSpacing: widget.keySpacing,
                    childAspectRatio: 1.0,
                  ),
                  itemCount: filteredKeys.length,
                  itemBuilder: (context, index) {
                    final keyInfo = filteredKeys[index];
                    return _buildKeyButton(context, keyInfo);
                  },
                ),
        ),
      ],
    );
  }

  /// Build a single key button
  Widget _buildKeyButton(BuildContext context, KeyInfo keyInfo) {
    final theme = Theme.of(context);
    final isSelected = widget.selectedKey == keyInfo.variant;

    return Material(
      color: isSelected
          ? theme.colorScheme.primary
          : theme.colorScheme.surfaceContainerHighest,
      borderRadius: BorderRadius.circular(4.0),
      elevation: isSelected ? 4.0 : 1.0,
      child: InkWell(
        onTap: () {
          widget.onKeySelected?.call(keyInfo.variant);
        },
        borderRadius: BorderRadius.circular(4.0),
        child: Center(
          child: Padding(
            padding: const EdgeInsets.all(4.0),
            child: Text(
              keyInfo.display,
              style: theme.textTheme.bodyMedium?.copyWith(
                color: isSelected
                    ? theme.colorScheme.onPrimary
                    : theme.colorScheme.onSurface,
                fontWeight: FontWeight.w500,
              ),
              textAlign: TextAlign.center,
              maxLines: 2,
              overflow: TextOverflow.ellipsis,
            ),
          ),
        ),
      ),
    );
  }
}
