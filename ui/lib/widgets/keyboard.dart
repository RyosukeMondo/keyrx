/// Visual keyboard widget.
///
/// Renders a keyboard layout with selectable keys
/// for the visual keymap editor.
library;

import 'package:flutter/material.dart';

import 'visual_keyboard.dart';

/// Visual keyboard widget for keymap editing.
///
/// This is a wrapper around [VisualKeyboard] for backward compatibility.
class KeyboardWidget extends StatelessWidget {
  const KeyboardWidget({super.key, this.onKeySelected, this.selectedKey});
  final void Function(String key)? onKeySelected;
  final String? selectedKey;

  @override
  Widget build(BuildContext context) => VisualKeyboard(
    onKeyTap: onKeySelected != null ? (key) => onKeySelected!(key.id) : null,
    selectedKeys: selectedKey != null ? {selectedKey!} : {},
  );
}
