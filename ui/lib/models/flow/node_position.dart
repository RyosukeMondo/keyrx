import 'dart:ui';

/// Represents a node position on the flow canvas.
class NodePosition {
  NodePosition({
    required this.x,
    required this.y,
    this.isManual = false,
    DateTime? updatedAt,
  }) : updatedAt = updatedAt ?? DateTime.now();

  final double x;
  final double y;
  final bool isManual;
  final DateTime updatedAt;

  Offset get offset => Offset(x, y);

  NodePosition copyWith({
    double? x,
    double? y,
    bool? isManual,
    DateTime? updatedAt,
  }) {
    return NodePosition(
      x: x ?? this.x,
      y: y ?? this.y,
      isManual: isManual ?? this.isManual,
      updatedAt: updatedAt ?? this.updatedAt,
    );
  }

  Map<String, dynamic> toJson() => {
    'x': x,
    'y': y,
    'isManual': isManual,
    'updatedAt': updatedAt.toIso8601String(),
  };

  factory NodePosition.fromJson(Map<String, dynamic> json) {
    return NodePosition(
      x: (json['x'] as num).toDouble(),
      y: (json['y'] as num).toDouble(),
      isManual: json['isManual'] as bool? ?? false,
      updatedAt:
          DateTime.tryParse(json['updatedAt'] as String? ?? '') ??
          DateTime.fromMillisecondsSinceEpoch(0),
    );
  }

  @override
  int get hashCode => Object.hash(x, y, isManual);

  @override
  bool operator ==(Object other) {
    if (identical(this, other)) return true;
    return other is NodePosition &&
        other.x == x &&
        other.y == y &&
        other.isManual == isManual;
  }

  @override
  String toString() =>
      'NodePosition(x: $x, y: $y, isManual: $isManual, updatedAt: $updatedAt)';
}
