/// Migration prompt page for V1 to V2 profile migration.
///
/// Displays a dialog explaining the migration process and allowing users
/// to migrate their old profiles to the new revolutionary mapping system.
library;

import 'package:flutter/material.dart';

/// Migration status enum
enum MigrationStatus {
  notStarted,
  inProgress,
  completed,
  failed,
}

/// Migration report data model
class MigrationReport {
  final int totalCount;
  final int migratedCount;
  final int failedCount;
  final String? backupPath;
  final List<MigrationFailure> failures;

  MigrationReport({
    required this.totalCount,
    required this.migratedCount,
    required this.failedCount,
    this.backupPath,
    this.failures = const [],
  });

  factory MigrationReport.fromJson(Map<String, dynamic> json) {
    return MigrationReport(
      totalCount: json['total_count'] as int,
      migratedCount: json['migrated_count'] as int,
      failedCount: json['failed_count'] as int,
      backupPath: json['backup_path'] as String?,
      failures: (json['failures'] as List<dynamic>?)
              ?.map((f) => MigrationFailure.fromJson(f as Map<String, dynamic>))
              .toList() ??
          [],
    );
  }

  bool get isSuccess => failedCount == 0 && totalCount > 0;
  bool get isPartial => migratedCount > 0 && failedCount > 0;
  double get successRate =>
      totalCount > 0 ? (migratedCount / totalCount) * 100 : 0;
}

/// Migration failure details
class MigrationFailure {
  final String path;
  final String error;

  MigrationFailure({
    required this.path,
    required this.error,
  });

  factory MigrationFailure.fromJson(Map<String, dynamic> json) {
    return MigrationFailure(
      path: json['path'] as String,
      error: json['error'] as String,
    );
  }
}

/// Page displaying migration prompt and handling migration process.
///
/// Shows:
/// - Explanation of what migration does
/// - Progress indicator during migration
/// - Results summary after completion
/// - Option to skip migration
class MigrationPromptPage extends StatefulWidget {
  const MigrationPromptPage({
    super.key,
    required this.onMigrate,
    required this.onSkip,
  });

  /// Callback to execute migration. Returns migration report.
  final Future<MigrationReport> Function() onMigrate;

  /// Callback when user skips migration
  final VoidCallback onSkip;

  @override
  State<MigrationPromptPage> createState() => _MigrationPromptPageState();
}

class _MigrationPromptPageState extends State<MigrationPromptPage> {
  MigrationStatus _status = MigrationStatus.notStarted;
  MigrationReport? _report;
  String? _errorMessage;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Profile Migration'),
        automaticallyImplyLeading: false,
      ),
      body: Center(
        child: ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: 600),
          child: Padding(
            padding: const EdgeInsets.all(24),
            child: _buildContent(),
          ),
        ),
      ),
    );
  }

  Widget _buildContent() {
    switch (_status) {
      case MigrationStatus.notStarted:
        return _buildPrompt();
      case MigrationStatus.inProgress:
        return _buildProgress();
      case MigrationStatus.completed:
        return _buildResults();
      case MigrationStatus.failed:
        return _buildError();
    }
  }

  /// Build initial migration prompt
  Widget _buildPrompt() {
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Icon(
          Icons.upgrade,
          size: 64,
          color: Colors.blue,
        ),
        const SizedBox(height: 24),
        Text(
          'Welcome to Revolutionary Mapping',
          style: Theme.of(context).textTheme.headlineMedium,
        ),
        const SizedBox(height: 16),
        Text(
          'We detected old profile data from a previous version. '
          'Would you like to migrate your profiles to the new system?',
          style: Theme.of(context).textTheme.bodyLarge,
        ),
        const SizedBox(height: 24),
        _buildFeatureList(),
        const SizedBox(height: 24),
        _buildWarningBox(),
        const SizedBox(height: 32),
        _buildActionButtons(),
      ],
    );
  }

  Widget _buildFeatureList() {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'What\'s new in Revolutionary Mapping:',
              style: Theme.of(context).textTheme.titleMedium?.copyWith(
                    fontWeight: FontWeight.bold,
                  ),
            ),
            const SizedBox(height: 12),
            _buildFeatureItem(
              Icons.devices,
              'Per-Device Control',
              'Assign different profiles to identical devices',
            ),
            _buildFeatureItem(
              Icons.grid_on,
              'Layout-Aware Profiles',
              'Profiles work with any device of the same layout type',
            ),
            _buildFeatureItem(
              Icons.dynamic_form,
              'Dynamic Layouts',
              'Support for keyboards, macro pads, and Stream Decks',
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildFeatureItem(IconData icon, String title, String description) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 12),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Icon(icon, size: 20, color: Colors.blue),
          const SizedBox(width: 12),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  title,
                  style: const TextStyle(fontWeight: FontWeight.w600),
                ),
                Text(
                  description,
                  style: TextStyle(
                    fontSize: 12,
                    color: Colors.grey[400],
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildWarningBox() {
    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: Colors.orange.withValues(alpha: 0.1),
        border: Border.all(color: Colors.orange),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Icon(Icons.info_outline, color: Colors.orange),
          const SizedBox(width: 12),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                const Text(
                  'Migration Information',
                  style: TextStyle(
                    fontWeight: FontWeight.bold,
                    color: Colors.orange,
                  ),
                ),
                const SizedBox(height: 8),
                Text(
                  'A backup of your old profiles will be created automatically. '
                  'The migration process will convert your device-specific profiles '
                  'to the new layout-based system.',
                  style: TextStyle(fontSize: 13, color: Colors.grey[300]),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildActionButtons() {
    return Row(
      mainAxisAlignment: MainAxisAlignment.end,
      children: [
        TextButton(
          onPressed: _handleSkip,
          child: const Text('Skip'),
        ),
        const SizedBox(width: 16),
        ElevatedButton.icon(
          onPressed: _handleMigrate,
          icon: const Icon(Icons.upgrade),
          label: const Text('Migrate Profiles'),
        ),
      ],
    );
  }

  /// Build migration progress UI
  Widget _buildProgress() {
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        const CircularProgressIndicator(),
        const SizedBox(height: 24),
        Text(
          'Migrating profiles...',
          style: Theme.of(context).textTheme.titleLarge,
        ),
        const SizedBox(height: 12),
        Text(
          'Please wait while we convert your profiles to the new format.',
          style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                color: Colors.grey[400],
              ),
          textAlign: TextAlign.center,
        ),
      ],
    );
  }

  /// Build migration results UI
  Widget _buildResults() {
    if (_report == null) {
      return _buildError();
    }

    final report = _report!;
    final isSuccess = report.isSuccess;
    final isPartial = report.isPartial;

    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Icon(
          isSuccess
              ? Icons.check_circle
              : isPartial
                  ? Icons.warning
                  : Icons.error,
          size: 64,
          color: isSuccess
              ? Colors.green
              : isPartial
                  ? Colors.orange
                  : Colors.red,
        ),
        const SizedBox(height: 24),
        Text(
          isSuccess
              ? 'Migration Completed'
              : isPartial
                  ? 'Migration Partially Completed'
                  : 'Migration Failed',
          style: Theme.of(context).textTheme.headlineMedium,
        ),
        const SizedBox(height: 16),
        _buildResultsSummary(report),
        if (report.failures.isNotEmpty) ...[
          const SizedBox(height: 24),
          _buildFailuresList(report.failures),
        ],
        const SizedBox(height: 32),
        Align(
          alignment: Alignment.centerRight,
          child: ElevatedButton(
            onPressed: _handleContinue,
            child: const Text('Continue'),
          ),
        ),
      ],
    );
  }

  Widget _buildResultsSummary(MigrationReport report) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            _buildSummaryRow('Total Profiles', '${report.totalCount}'),
            _buildSummaryRow('Migrated', '${report.migratedCount}',
                color: Colors.green),
            if (report.failedCount > 0)
              _buildSummaryRow('Failed', '${report.failedCount}',
                  color: Colors.red),
            _buildSummaryRow(
                'Success Rate', '${report.successRate.toStringAsFixed(1)}%'),
            if (report.backupPath != null) ...[
              const Divider(height: 24),
              Row(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  const Icon(Icons.folder, size: 16),
                  const SizedBox(width: 8),
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        const Text(
                          'Backup Location:',
                          style: TextStyle(fontWeight: FontWeight.w600),
                        ),
                        const SizedBox(height: 4),
                        Text(
                          report.backupPath!,
                          style: TextStyle(
                            fontSize: 12,
                            color: Colors.grey[400],
                            fontFamily: 'monospace',
                          ),
                        ),
                      ],
                    ),
                  ),
                ],
              ),
            ],
          ],
        ),
      ),
    );
  }

  Widget _buildSummaryRow(String label, String value, {Color? color}) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(label),
          Text(
            value,
            style: TextStyle(
              fontWeight: FontWeight.bold,
              color: color,
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildFailuresList(List<MigrationFailure> failures) {
    return Card(
      color: Colors.red.withValues(alpha: 0.1),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                const Icon(Icons.error_outline, color: Colors.red, size: 20),
                const SizedBox(width: 8),
                const Text(
                  'Failed Migrations:',
                  style: TextStyle(
                    fontWeight: FontWeight.bold,
                    color: Colors.red,
                  ),
                ),
              ],
            ),
            const SizedBox(height: 12),
            ...failures.map((failure) => Padding(
                  padding: const EdgeInsets.only(bottom: 8),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        failure.path,
                        style: const TextStyle(
                          fontFamily: 'monospace',
                          fontSize: 12,
                        ),
                      ),
                      Text(
                        failure.error,
                        style: TextStyle(
                          fontSize: 11,
                          color: Colors.grey[400],
                        ),
                      ),
                    ],
                  ),
                )),
          ],
        ),
      ),
    );
  }

  /// Build error UI
  Widget _buildError() {
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        const Icon(Icons.error, size: 64, color: Colors.red),
        const SizedBox(height: 24),
        Text(
          'Migration Failed',
          style: Theme.of(context).textTheme.headlineMedium,
        ),
        const SizedBox(height: 16),
        Text(
          _errorMessage ?? 'An unexpected error occurred during migration.',
          style: Theme.of(context).textTheme.bodyMedium,
          textAlign: TextAlign.center,
        ),
        const SizedBox(height: 32),
        Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            TextButton(
              onPressed: _handleSkip,
              child: const Text('Skip Migration'),
            ),
            const SizedBox(width: 16),
            ElevatedButton(
              onPressed: _handleRetry,
              child: const Text('Retry'),
            ),
          ],
        ),
      ],
    );
  }

  /// Handle migrate button press
  Future<void> _handleMigrate() async {
    setState(() {
      _status = MigrationStatus.inProgress;
    });

    try {
      final report = await widget.onMigrate();
      setState(() {
        _report = report;
        _status = MigrationStatus.completed;
      });
    } catch (e) {
      setState(() {
        _errorMessage = e.toString();
        _status = MigrationStatus.failed;
      });
    }
  }

  /// Handle skip button press
  void _handleSkip() {
    widget.onSkip();
  }

  /// Handle continue button press after successful migration
  void _handleContinue() {
    widget.onSkip(); // Use skip callback to close the migration flow
  }

  /// Handle retry after failure
  void _handleRetry() {
    setState(() {
      _status = MigrationStatus.notStarted;
      _errorMessage = null;
      _report = null;
    });
  }
}

