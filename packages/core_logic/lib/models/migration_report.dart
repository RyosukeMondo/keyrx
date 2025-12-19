/// Migration report data model
class MigrationReport {
  MigrationReport({
    required this.totalCount,
    required this.migratedCount,
    required this.failedCount,
    this.backupPath,
    this.failures = const [],
  });

  factory MigrationReport.fromJson(Map<String, dynamic> json) =>
      MigrationReport(
        totalCount: json['total_count'] as int,
        migratedCount: json['migrated_count'] as int,
        failedCount: json['failed_count'] as int,
        backupPath: json['backup_path'] as String?,
        failures:
            (json['failures'] as List<dynamic>?)
                ?.map(
                  (f) => MigrationFailure.fromJson(f as Map<String, dynamic>),
                )
                .toList() ??
            [],
      );
  final int totalCount;
  final int migratedCount;
  final int failedCount;
  final String? backupPath;
  final List<MigrationFailure> failures;

  bool get isSuccess => failedCount == 0 && totalCount > 0;
  bool get isPartial => migratedCount > 0 && failedCount > 0;
  double get successRate =>
      totalCount > 0 ? (migratedCount / totalCount) * 100 : 0;
}

/// Migration failure details
class MigrationFailure {
  MigrationFailure({required this.path, required this.error});

  factory MigrationFailure.fromJson(Map<String, dynamic> json) =>
      MigrationFailure(
        path: json['path'] as String,
        error: json['error'] as String,
      );
  final String path;
  final String error;
}
