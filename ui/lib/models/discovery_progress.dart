/// Models for device discovery progress.
library;

/// Progress update from an active discovery session.
class DiscoveryProgress {
  const DiscoveryProgress({
    required this.captured,
    required this.total,
    this.nextKey,
  });

  factory DiscoveryProgress.fromJson(Map<String, dynamic> json) {
    return DiscoveryProgress(
      captured: (json['captured'] as num).toInt(),
      total: (json['total'] as num).toInt(),
      nextKey: json['next'] != null
          ? DiscoveryPosition.fromJson(json['next'] as Map<String, dynamic>)
          : null,
    );
  }

  /// Number of keys successfully mapped so far.
  final int captured;

  /// Total number of keys in the configured layout.
  final int total;

  /// The next key position expected to be pressed.
  ///
  /// If null, discovery is complete or waiting for processing.
  final DiscoveryPosition? nextKey;

  bool get isComplete => captured >= total;
  double get progress => total > 0 ? captured / total : 0.0;
}

/// A specific key position in the matrix.
class DiscoveryPosition {
  const DiscoveryPosition({
    required this.row,
    required this.col,
  });

  factory DiscoveryPosition.fromJson(Map<String, dynamic> json) {
    return DiscoveryPosition(
      row: (json['row'] as num).toInt(),
      col: (json['col'] as num).toInt(),
    );
  }

  final int row;
  final int col;

  @override
  String toString() => 'Row $row, Col $col';
}
