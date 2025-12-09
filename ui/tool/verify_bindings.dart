#!/usr/bin/env dart
// ignore_for_file: avoid_print
// Dart wrapper for FFI binding verification
//
// This script can be used as a Flutter pre-build check.
// It calls the Python verification script and reports results.
//
// Usage:
//   dart tool/verify_bindings.dart
//
// Exit codes:
//   0 - Bindings are in sync
//   1 - Bindings are out of sync or error occurred

import 'dart:io';

Future<void> main() async {
  print('=================================================================');
  print('Flutter FFI Bindings Verification');
  print('=================================================================');

  // Get project root (this script is in ui/tool/)
  final scriptDir = Directory.current;
  final uiDir = scriptDir.parent;
  final projectRoot = uiDir.parent;

  // Path to verification script
  final verifyScript = File('${projectRoot.path}/scripts/verify_bindings.py');

  if (!verifyScript.existsSync()) {
    stderr.writeln('ERROR: Verification script not found!');
    stderr.writeln('Expected: ${verifyScript.path}');
    exit(1);
  }

  // Run the Python verification script
  try {
    final result = await Process.run('python3', [
      verifyScript.path,
    ], workingDirectory: projectRoot.path);

    // Print output
    if (result.stdout.toString().isNotEmpty) {
      print(result.stdout);
    }
    if (result.stderr.toString().isNotEmpty) {
      stderr.write(result.stderr);
    }

    // Exit with same code as verification script
    exit(result.exitCode);
  } catch (e) {
    stderr.writeln('ERROR: Failed to run verification script: $e');
    exit(1);
  }
}
