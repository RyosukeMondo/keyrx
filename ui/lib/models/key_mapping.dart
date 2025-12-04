/// Data models for key mapping configurations.
library;

/// Types of key actions available in the editor.
enum KeyActionType { remap, block, pass }

extension KeyActionTypeLabel on KeyActionType {
  String get label {
    switch (this) {
      case KeyActionType.remap:
        return 'Remap';
      case KeyActionType.block:
        return 'Block';
      case KeyActionType.pass:
        return 'Pass through';
    }
  }
}

/// Represents a single key mapping configuration.
class KeyMapping {
  const KeyMapping({
    required this.from,
    required this.type,
    this.to,
    this.layer,
    this.tapHoldTap,
    this.tapHoldHold,
  });

  final String from;
  final KeyActionType type;
  final String? to;
  final String? layer;
  final String? tapHoldTap;
  final String? tapHoldHold;
}

/// Represents a combo (multiple keys pressed together) configuration.
class ComboMapping {
  const ComboMapping({required this.keys, required this.output});

  final List<String> keys;
  final String output;
}

/// Backward compatibility alias.
/// @deprecated Use [BindingPanel] from widgets/editor/binding_panel.dart instead.
@Deprecated('Use BindingPanel from widgets/editor/binding_panel.dart instead')
typedef KeyConfigPanel = Object;
