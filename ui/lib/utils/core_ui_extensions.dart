import 'dart:ui';

import 'package:core_logic/models/flow/edge_style.dart';
import 'package:core_logic/models/flow/flow_graph.dart';
import 'package:core_logic/models/flow/node_position.dart';

extension NodePositionUi on NodePosition {
  Offset get offset => Offset(x, y);
}

extension EdgePaletteUi on EdgePalette {
  Color get validColor => Color(valid);
  Color get invalidColor => Color(invalid);
  Color get warningColor => Color(warning);
  Color get inactiveColor => Color(inactive);
  Color get hoverGlowColor => Color(hoverGlow);
  Color get priorityBackgroundColor => Color(priorityBackground);
  Color get priorityTextColor => Color(priorityText);
}

extension EdgeStyleUi on EdgeStyle {
  Color colorForStatus(
    EdgeStatus status, {
    bool active = true,
    bool hovered = false,
  }) {
    if (hovered) return Color(palette.hoverGlow);
    if (!active) return Color(palette.inactive);

    switch (status) {
      case EdgeStatus.valid:
        return Color(palette.valid);
      case EdgeStatus.invalid:
        return Color(palette.invalid);
      case EdgeStatus.warning:
        return Color(palette.warning);
    }
  }
}
