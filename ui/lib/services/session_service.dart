/// Session management service for listing, analysis, and replay.
///
/// Keeps FFI concerns out of widgets by routing through this service.
library;

import 'dart:async';

import '../ffi/bridge.dart';

/// Session info from listing.
class SessionRecord {
  const SessionRecord({
    required this.path,
    required this.name,
    required this.created,
    required this.eventCount,
    required this.durationMs,
  });

  final String path;
  final String name;
  final String created;
  final int eventCount;
  final double durationMs;
}

/// Decision breakdown from session analysis.
class SessionDecisionBreakdown {
  const SessionDecisionBreakdown({
    required this.passThrough,
    required this.remap,
    required this.block,
    required this.tap,
    required this.hold,
    required this.combo,
    required this.layer,
    required this.modifier,
  });

  final int passThrough;
  final int remap;
  final int block;
  final int tap;
  final int hold;
  final int combo;
  final int layer;
  final int modifier;
}

/// Analysis result for a session.
class SessionAnalysisData {
  const SessionAnalysisData({
    required this.sessionPath,
    required this.eventCount,
    required this.durationMs,
    required this.avgLatencyUs,
    required this.minLatencyUs,
    required this.maxLatencyUs,
    required this.decisionBreakdown,
  });

  final String sessionPath;
  final int eventCount;
  final double durationMs;
  final int avgLatencyUs;
  final int minLatencyUs;
  final int maxLatencyUs;
  final SessionDecisionBreakdown decisionBreakdown;
}

/// Mismatch detail from replay verification.
class SessionReplayMismatch {
  const SessionReplayMismatch({
    required this.seq,
    required this.recorded,
    required this.actual,
  });

  final int seq;
  final String recorded;
  final String actual;
}

/// Result of a replay operation.
class SessionReplayData {
  const SessionReplayData({
    required this.totalEvents,
    required this.matched,
    required this.mismatched,
    required this.success,
    required this.mismatches,
  });

  final int totalEvents;
  final int matched;
  final int mismatched;
  final bool success;
  final List<SessionReplayMismatch> mismatches;
}

/// Result of listing sessions.
class SessionListServiceResult {
  const SessionListServiceResult({
    required this.sessions,
    this.errorMessage,
  });

  factory SessionListServiceResult.error(String message) =>
      SessionListServiceResult(sessions: const [], errorMessage: message);

  final List<SessionRecord> sessions;
  final String? errorMessage;

  bool get hasError => errorMessage != null;
}

/// Result of analyzing a session.
class SessionAnalysisServiceResult {
  const SessionAnalysisServiceResult({
    this.analysis,
    this.errorMessage,
  });

  factory SessionAnalysisServiceResult.error(String message) =>
      SessionAnalysisServiceResult(errorMessage: message);

  final SessionAnalysisData? analysis;
  final String? errorMessage;

  bool get hasError => errorMessage != null;
}

/// Result of replaying a session.
class SessionReplayServiceResult {
  const SessionReplayServiceResult({
    this.data,
    this.errorMessage,
  });

  factory SessionReplayServiceResult.error(String message) =>
      SessionReplayServiceResult(errorMessage: message);

  final SessionReplayData? data;
  final String? errorMessage;

  bool get hasError => errorMessage != null;
}

/// Abstraction for session management operations.
abstract class SessionService {
  /// List sessions in a directory.
  Future<SessionListServiceResult> listSessions(String dirPath);

  /// Analyze a session file.
  Future<SessionAnalysisServiceResult> analyze(String path);

  /// Replay a session with optional verification.
  Future<SessionReplayServiceResult> replay(String path, {bool verify = false});

  /// Dispose any held resources.
  Future<void> dispose();
}

/// Real SessionService that wraps the KeyrxBridge.
class SessionServiceImpl implements SessionService {
  SessionServiceImpl({required KeyrxBridge bridge}) : _bridge = bridge;

  final KeyrxBridge _bridge;

  @override
  Future<SessionListServiceResult> listSessions(String dirPath) async {
    if (_bridge.loadFailure != null) {
      return SessionListServiceResult.error(
        'Engine unavailable: ${_bridge.loadFailure}',
      );
    }

    final result = _bridge.listSessions(dirPath);

    if (result.hasError) {
      return SessionListServiceResult.error(
        result.errorMessage ?? 'Unknown error',
      );
    }

    final sessions = result.sessions.map((s) {
      return SessionRecord(
        path: s.path,
        name: s.name,
        created: s.created,
        eventCount: s.eventCount,
        durationMs: s.durationMs,
      );
    }).toList();

    return SessionListServiceResult(sessions: sessions);
  }

  @override
  Future<SessionAnalysisServiceResult> analyze(String path) async {
    if (_bridge.loadFailure != null) {
      return SessionAnalysisServiceResult.error(
        'Engine unavailable: ${_bridge.loadFailure}',
      );
    }

    final result = _bridge.analyzeSession(path);

    if (result.hasError || result.analysis == null) {
      return SessionAnalysisServiceResult.error(
        result.errorMessage ?? 'Unknown error',
      );
    }

    final a = result.analysis!;
    final breakdown = SessionDecisionBreakdown(
      passThrough: a.decisionBreakdown.passThrough,
      remap: a.decisionBreakdown.remap,
      block: a.decisionBreakdown.block,
      tap: a.decisionBreakdown.tap,
      hold: a.decisionBreakdown.hold,
      combo: a.decisionBreakdown.combo,
      layer: a.decisionBreakdown.layer,
      modifier: a.decisionBreakdown.modifier,
    );

    return SessionAnalysisServiceResult(
      analysis: SessionAnalysisData(
        sessionPath: a.sessionPath,
        eventCount: a.eventCount,
        durationMs: a.durationMs,
        avgLatencyUs: a.avgLatencyUs,
        minLatencyUs: a.minLatencyUs,
        maxLatencyUs: a.maxLatencyUs,
        decisionBreakdown: breakdown,
      ),
    );
  }

  @override
  Future<SessionReplayServiceResult> replay(
    String path, {
    bool verify = false,
  }) async {
    if (_bridge.loadFailure != null) {
      return SessionReplayServiceResult.error(
        'Engine unavailable: ${_bridge.loadFailure}',
      );
    }

    final result = _bridge.replaySession(path, verify: verify);

    if (result.hasError) {
      return SessionReplayServiceResult.error(
        result.errorMessage ?? 'Unknown error',
      );
    }

    final mismatches = result.mismatches.map((m) {
      return SessionReplayMismatch(
        seq: m.seq,
        recorded: m.recorded,
        actual: m.actual,
      );
    }).toList();

    return SessionReplayServiceResult(
      data: SessionReplayData(
        totalEvents: result.totalEvents,
        matched: result.matched,
        mismatched: result.mismatched,
        success: result.success,
        mismatches: mismatches,
      ),
    );
  }

  @override
  Future<void> dispose() async {
    // No resources to dispose
  }
}
