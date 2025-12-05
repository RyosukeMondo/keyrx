/// Migration wrapper widget that checks and shows migration prompt.
///
/// This widget wraps the main app and shows the migration prompt page
/// if migration is needed and hasn't been completed yet.
library;

import 'package:flutter/material.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../config/config.dart';
import '../pages/migration_prompt_page.dart';

/// Wrapper widget that manages migration flow.
///
/// Shows [MigrationPromptPage] if:
/// - Migration has not been marked as completed
/// - Old profiles directory exists (checked via callback)
///
/// Otherwise shows the child widget (main app).
class MigrationWrapper extends StatefulWidget {
  const MigrationWrapper({
    super.key,
    required this.child,
    required this.onCheckMigrationNeeded,
    required this.onRunMigration,
  });

  /// The main app widget to show after migration check
  final Widget child;

  /// Callback to check if migration is needed (checks for old profiles)
  final Future<bool> Function() onCheckMigrationNeeded;

  /// Callback to run the migration, returns a migration report
  final Future<MigrationReport> Function() onRunMigration;

  @override
  State<MigrationWrapper> createState() => _MigrationWrapperState();
}

class _MigrationWrapperState extends State<MigrationWrapper> {
  bool _isCheckingMigration = true;
  bool _showMigrationPrompt = false;

  @override
  void initState() {
    super.initState();
    _checkMigrationStatus();
  }

  Future<void> _checkMigrationStatus() async {
    try {
      final prefs = await SharedPreferences.getInstance();
      final migrationCompleted =
          prefs.getBool(StorageKeys.migrationCompletedKey) ?? false;

      if (migrationCompleted) {
        // Migration already done, proceed to main app
        setState(() {
          _isCheckingMigration = false;
          _showMigrationPrompt = false;
        });
        return;
      }

      // Check if old profiles exist
      final migrationNeeded = await widget.onCheckMigrationNeeded();

      setState(() {
        _isCheckingMigration = false;
        _showMigrationPrompt = migrationNeeded;
      });
    } catch (e) {
      // On error, skip migration and proceed to main app
      setState(() {
        _isCheckingMigration = false;
        _showMigrationPrompt = false;
      });
    }
  }

  Future<void> _handleSkipMigration() async {
    // Mark migration as completed (even if skipped)
    final prefs = await SharedPreferences.getInstance();
    await prefs.setBool(StorageKeys.migrationCompletedKey, true);

    setState(() {
      _showMigrationPrompt = false;
    });
  }

  Future<MigrationReport> _handleRunMigration() async {
    final report = await widget.onRunMigration();

    // Mark migration as completed
    final prefs = await SharedPreferences.getInstance();
    await prefs.setBool(StorageKeys.migrationCompletedKey, true);

    return report;
  }

  @override
  Widget build(BuildContext context) {
    if (_isCheckingMigration) {
      return const Scaffold(
        body: Center(
          child: CircularProgressIndicator(),
        ),
      );
    }

    if (_showMigrationPrompt) {
      return MigrationPromptPage(
        onMigrate: _handleRunMigration,
        onSkip: _handleSkipMigration,
      );
    }

    return widget.child;
  }
}
