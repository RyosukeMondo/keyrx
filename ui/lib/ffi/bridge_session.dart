/// Session and recording FFI methods.
///
/// Provides session recording, listing, analysis, and replay functionality
/// for the KeyRx bridge.
library;

import 'dart:ffi';

import 'package:ffi/ffi.dart';

import 'bindings.dart';
import 'bridge_session_types.dart';

export 'bridge_session_types.dart';

/// Mixin providing session and recording FFI methods.
mixin BridgeSessionMixin {
  KeyrxBindings? get bindings;
  Object? get loadFailure;

  /// Start recording key events to a file.
  RecordingStartResult startRecording(String path) {
    final startFn = bindings?.startRecording;
    if (startFn == null) {
      return RecordingStartResult.error('startRecording not available');
    }

    final pathPtr = path.toNativeUtf8();
    Pointer<Char>? ptr;
    try {
      ptr = startFn(pathPtr.cast<Char>());
      if (ptr == nullptr) {
        return RecordingStartResult.error('startRecording returned null');
      }

      final raw = ptr.cast<Utf8>().toDartString();
      return RecordingStartResult.parse(raw);
    } catch (e) {
      return RecordingStartResult.error('$e');
    } finally {
      calloc.free(pathPtr);
      if (ptr != null && ptr != nullptr) {
        try {
          bindings?.freeString(ptr);
        } catch (_) {}
      }
    }
  }

  /// Stop recording and save the session.
  RecordingStopResult stopRecording() {
    final stopFn = bindings?.stopRecording;
    if (stopFn == null) {
      return RecordingStopResult.error('stopRecording not available');
    }

    Pointer<Char>? ptr;
    try {
      ptr = stopFn();
      if (ptr == nullptr) {
        return RecordingStopResult.error('stopRecording returned null');
      }

      final raw = ptr.cast<Utf8>().toDartString();
      return RecordingStopResult.parse(raw);
    } catch (e) {
      return RecordingStopResult.error('$e');
    } finally {
      if (ptr != null && ptr != nullptr) {
        try {
          bindings?.freeString(ptr);
        } catch (_) {}
      }
    }
  }

  /// List session files in a directory.
  SessionListResult listSessions(String dirPath) {
    final listFn = bindings?.listSessions;
    if (listFn == null) {
      return SessionListResult.error('listSessions not available');
    }

    final dirPtr = dirPath.toNativeUtf8();
    Pointer<Char>? ptr;
    try {
      ptr = listFn(dirPtr.cast<Char>());
      if (ptr == nullptr) {
        return SessionListResult.error('listSessions returned null');
      }

      final raw = ptr.cast<Utf8>().toDartString();
      return SessionListResult.parse(raw);
    } catch (e) {
      return SessionListResult.error('$e');
    } finally {
      calloc.free(dirPtr);
      if (ptr != null && ptr != nullptr) {
        try {
          bindings?.freeString(ptr);
        } catch (_) {}
      }
    }
  }

  /// Analyze a session file.
  SessionAnalysisResult analyzeSession(String path) {
    final analyzeFn = bindings?.analyzeSession;
    if (analyzeFn == null) {
      return SessionAnalysisResult.error('analyzeSession not available');
    }

    final pathPtr = path.toNativeUtf8();
    Pointer<Char>? ptr;
    try {
      ptr = analyzeFn(pathPtr.cast<Char>());
      if (ptr == nullptr) {
        return SessionAnalysisResult.error('analyzeSession returned null');
      }

      final raw = ptr.cast<Utf8>().toDartString();
      return SessionAnalysisResult.parse(raw);
    } catch (e) {
      return SessionAnalysisResult.error('$e');
    } finally {
      calloc.free(pathPtr);
      if (ptr != null && ptr != nullptr) {
        try {
          bindings?.freeString(ptr);
        } catch (_) {}
      }
    }
  }

  /// Replay a session file with optional verification.
  ReplayResult replaySession(String path, {bool verify = false}) {
    final replayFn = bindings?.replaySession;
    if (replayFn == null) {
      return ReplayResult.error('replaySession not available');
    }

    final pathPtr = path.toNativeUtf8();
    Pointer<Char>? ptr;
    try {
      ptr = replayFn(pathPtr.cast<Char>(), verify);
      if (ptr == nullptr) {
        return ReplayResult.error('replaySession returned null');
      }

      final raw = ptr.cast<Utf8>().toDartString();
      return ReplayResult.parse(raw);
    } catch (e) {
      return ReplayResult.error('$e');
    } finally {
      calloc.free(pathPtr);
      if (ptr != null && ptr != nullptr) {
        try {
          bindings?.freeString(ptr);
        } catch (_) {}
      }
    }
  }
}
