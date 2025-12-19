/// Session and recording FFI methods.
///
/// Provides session recording, listing, analysis, and replay functionality
/// for the KeyRx bridge.
library;

import 'bindings.dart';
import 'bridge_session_types.dart';

export 'bridge_session_types.dart';

/// Mixin providing session and recording FFI methods.
mixin BridgeSessionMixin {
  KeyrxBindings? get bindings;
  Object? get loadFailure;

  /// Start recording key events to a file.
  RecordingStartResult startRecording(String path) {
    if (bindings == null) {
      return RecordingStartResult.error('startRecording not available');
    }

    try {
      final raw = bindings!.compat.startRecording(path);
      return RecordingStartResult.parse(raw);
    } catch (e) {
      return RecordingStartResult.error('$e');
    }
  }

  /// Stop recording and save the session.
  RecordingStopResult stopRecording() {
    if (bindings == null) {
      return RecordingStopResult.error('stopRecording not available');
    }

    try {
      final raw = bindings!.compat.stopRecording();
      return RecordingStopResult.parse(raw);
    } catch (e) {
      return RecordingStopResult.error('$e');
    }
  }

  /// List session files in a directory.
  SessionListResult listSessions(String dirPath) {
    if (bindings == null) {
      return SessionListResult.error('listSessions not available');
    }

    try {
      final raw = bindings!.compat.listSessions(dirPath);
      return SessionListResult.parse(raw);
    } catch (e) {
      return SessionListResult.error('$e');
    }
  }

  /// Analyze a session file.
  SessionAnalysisResult analyzeSession(String path) {
    if (bindings == null) {
      return SessionAnalysisResult.error('analyzeSession not available');
    }

    try {
      final raw = bindings!.compat.analyzeSession(path);
      return SessionAnalysisResult.parse(raw);
    } catch (e) {
      return SessionAnalysisResult.error('$e');
    }
  }

  /// Replay a session file with optional verification.
  ReplayResult replaySession(String path, {bool verify = false}) {
    if (bindings == null) {
      return ReplayResult.error('replaySession not available');
    }

    try {
      final raw = bindings!.compat.replaySession(path, verify);
      return ReplayResult.parse(raw);
    } catch (e) {
      return ReplayResult.error('$e');
    }
  }
}
