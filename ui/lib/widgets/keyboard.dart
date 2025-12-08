/// Visual keyboard widget.
///
/// Renders a keyboard layout with selectable keys
/// for the visual keymap editor.
library;

import 'package:flutter/material.dart';

import 'editor/key_grid.dart';

/// Visual keyboard widget for keymap editing.
///
/// This is a wrapper around [KeyGrid] for backward compatibility.
class KeyboardWidget extends StatelessWidget {
  final void Function(String key)? onKeySelected;
  final String? selectedKey;

  const KeyboardWidget({super.key, this.onKeySelected, this.selectedKey});

  @override
  Widget build(BuildContext context) {
    return KeyGrid(onKeySelected: onKeySelected, selectedKey: selectedKey);
  }
}
