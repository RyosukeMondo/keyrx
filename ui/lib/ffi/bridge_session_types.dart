/// Result types for session and recording FFI methods.
library;

import 'dart:convert';

/// Session info returned by list sessions.
class SessionInfo {
  const SessionInfo({
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

/// Session list result.
class SessionListResult {
  const SessionListResult({required this.sessions, this.errorMessage});

  factory SessionListResult.error(String message) =>
      SessionListResult(sessions: const [], errorMessage: message);

  factory SessionListResult.parse(String raw) {
    final trimmed = raw.trim();
    if (trimmed.toLowerCase().startsWith('error:')) {
      return SessionListResult.error(trimmed.substring('error:'.length).trim());
    }

    final payload = trimmed.toLowerCase().startsWith('ok:')
        ? trimmed.substring(trimmed.indexOf(':') + 1)
        : trimmed;

    try {
      final decoded = json.decode(payload);
      if (decoded is! List) {
        return SessionListResult.error('invalid session list payload');
      }

      final sessions = decoded
          .map((e) {
            if (e is! Map<String, dynamic>) return null;
            return SessionInfo(
              path: e['path']?.toString() ?? '',
              name: e['name']?.toString() ?? '',
              created: e['created']?.toString() ?? '',
              eventCount: (e['eventCount'] as num?)?.toInt() ?? 0,
              durationMs: (e['durationMs'] as num?)?.toDouble() ?? 0,
            );
          })
          .whereType<SessionInfo>()
          .toList();

      return SessionListResult(sessions: sessions);
    } catch (e) {
      return SessionListResult.error('$e');
    }
  }

  final List<SessionInfo> sessions;
  final String? errorMessage;

  bool get hasError => errorMessage != null;
}

/// Decision breakdown from session analysis.
class DecisionBreakdown {
  const DecisionBreakdown({
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

/// Session analysis result.
class SessionAnalysis {
  const SessionAnalysis({
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
  final DecisionBreakdown decisionBreakdown;
}

/// Session analysis result wrapper.
class SessionAnalysisResult {
  const SessionAnalysisResult({this.analysis, this.errorMessage});

  factory SessionAnalysisResult.error(String message) =>
      SessionAnalysisResult(errorMessage: message);

  factory SessionAnalysisResult.parse(String raw) {
    final trimmed = raw.trim();
    if (trimmed.toLowerCase().startsWith('error:')) {
      return SessionAnalysisResult.error(
        trimmed.substring('error:'.length).trim(),
      );
    }

    final payload = trimmed.toLowerCase().startsWith('ok:')
        ? trimmed.substring(trimmed.indexOf(':') + 1)
        : trimmed;

    try {
      final decoded = json.decode(payload);
      if (decoded is! Map<String, dynamic>) {
        return SessionAnalysisResult.error('invalid analysis payload');
      }

      final breakdown = decoded['decisionBreakdown'] as Map<String, dynamic>?;
      final analysis = SessionAnalysis(
        sessionPath: decoded['sessionPath']?.toString() ?? '',
        eventCount: (decoded['eventCount'] as num?)?.toInt() ?? 0,
        durationMs: (decoded['durationMs'] as num?)?.toDouble() ?? 0,
        avgLatencyUs: (decoded['avgLatencyUs'] as num?)?.toInt() ?? 0,
        minLatencyUs: (decoded['minLatencyUs'] as num?)?.toInt() ?? 0,
        maxLatencyUs: (decoded['maxLatencyUs'] as num?)?.toInt() ?? 0,
        decisionBreakdown: DecisionBreakdown(
          passThrough: (breakdown?['passThrough'] as num?)?.toInt() ?? 0,
          remap: (breakdown?['remap'] as num?)?.toInt() ?? 0,
          block: (breakdown?['block'] as num?)?.toInt() ?? 0,
          tap: (breakdown?['tap'] as num?)?.toInt() ?? 0,
          hold: (breakdown?['hold'] as num?)?.toInt() ?? 0,
          combo: (breakdown?['combo'] as num?)?.toInt() ?? 0,
          layer: (breakdown?['layer'] as num?)?.toInt() ?? 0,
          modifier: (breakdown?['modifier'] as num?)?.toInt() ?? 0,
        ),
      );

      return SessionAnalysisResult(analysis: analysis);
    } catch (e) {
      return SessionAnalysisResult.error('$e');
    }
  }

  final SessionAnalysis? analysis;
  final String? errorMessage;

  bool get hasError => errorMessage != null;
}

/// Mismatch detail from replay verification.
class ReplayMismatch {
  const ReplayMismatch({
    required this.seq,
    required this.recorded,
    required this.actual,
  });

  final int seq;
  final String recorded;
  final String actual;
}

/// Session replay result.
class ReplayResult {
  const ReplayResult({
    required this.totalEvents,
    required this.matched,
    required this.mismatched,
    required this.success,
    required this.mismatches,
    this.errorMessage,
  });

  factory ReplayResult.error(String message) => ReplayResult(
    totalEvents: 0,
    matched: 0,
    mismatched: 0,
    success: false,
    mismatches: const [],
    errorMessage: message,
  );

  factory ReplayResult.parse(String raw) {
    final trimmed = raw.trim();
    if (trimmed.toLowerCase().startsWith('error:')) {
      return ReplayResult.error(trimmed.substring('error:'.length).trim());
    }

    final payload = trimmed.toLowerCase().startsWith('ok:')
        ? trimmed.substring(trimmed.indexOf(':') + 1)
        : trimmed;

    try {
      final decoded = json.decode(payload);
      if (decoded is! Map<String, dynamic>) {
        return ReplayResult.error('invalid replay payload');
      }

      final mismatchList = decoded['mismatches'] as List<dynamic>? ?? [];
      final mismatches = mismatchList
          .map((e) {
            if (e is! Map<String, dynamic>) return null;
            return ReplayMismatch(
              seq: (e['seq'] as num?)?.toInt() ?? 0,
              recorded: e['recorded']?.toString() ?? '',
              actual: e['actual']?.toString() ?? '',
            );
          })
          .whereType<ReplayMismatch>()
          .toList();

      return ReplayResult(
        totalEvents: (decoded['totalEvents'] as num?)?.toInt() ?? 0,
        matched: (decoded['matched'] as num?)?.toInt() ?? 0,
        mismatched: (decoded['mismatched'] as num?)?.toInt() ?? 0,
        success: decoded['success'] as bool? ?? false,
        mismatches: mismatches,
      );
    } catch (e) {
      return ReplayResult.error('$e');
    }
  }

  final int totalEvents;
  final int matched;
  final int mismatched;
  final bool success;
  final List<ReplayMismatch> mismatches;
  final String? errorMessage;

  bool get hasError => errorMessage != null;
}

/// Recording start result.
class RecordingStartResult {
  const RecordingStartResult({
    required this.success,
    this.outputPath,
    this.errorMessage,
  });

  factory RecordingStartResult.error(String message) =>
      RecordingStartResult(success: false, errorMessage: message);

  factory RecordingStartResult.parse(String raw) {
    final trimmed = raw.trim();
    if (trimmed.toLowerCase().startsWith('error:')) {
      return RecordingStartResult.error(
        trimmed.substring('error:'.length).trim(),
      );
    }

    final payload = trimmed.toLowerCase().startsWith('ok:')
        ? trimmed.substring(trimmed.indexOf(':') + 1)
        : trimmed;

    try {
      final decoded = json.decode(payload);
      if (decoded is! Map<String, dynamic>) {
        return RecordingStartResult.error('invalid recording payload');
      }

      return RecordingStartResult(
        success: decoded['success'] as bool? ?? false,
        outputPath: decoded['outputPath']?.toString(),
        errorMessage: decoded['error']?.toString(),
      );
    } catch (e) {
      return RecordingStartResult.error('$e');
    }
  }

  final bool success;
  final String? outputPath;
  final String? errorMessage;

  bool get hasError => errorMessage != null || !success;
}

/// Recording stop result.
class RecordingStopResult {
  const RecordingStopResult({
    required this.success,
    this.path,
    required this.eventCount,
    required this.durationMs,
    this.errorMessage,
  });

  factory RecordingStopResult.error(String message) => RecordingStopResult(
    success: false,
    eventCount: 0,
    durationMs: 0,
    errorMessage: message,
  );

  factory RecordingStopResult.parse(String raw) {
    final trimmed = raw.trim();
    if (trimmed.toLowerCase().startsWith('error:')) {
      return RecordingStopResult.error(
        trimmed.substring('error:'.length).trim(),
      );
    }

    final payload = trimmed.toLowerCase().startsWith('ok:')
        ? trimmed.substring(trimmed.indexOf(':') + 1)
        : trimmed;

    try {
      final decoded = json.decode(payload);
      if (decoded is! Map<String, dynamic>) {
        return RecordingStopResult.error('invalid recording stop payload');
      }

      return RecordingStopResult(
        success: decoded['success'] as bool? ?? false,
        path: decoded['path']?.toString(),
        eventCount: (decoded['eventCount'] as num?)?.toInt() ?? 0,
        durationMs: (decoded['durationMs'] as num?)?.toDouble() ?? 0,
        errorMessage: decoded['error']?.toString(),
      );
    } catch (e) {
      return RecordingStopResult.error('$e');
    }
  }

  final bool success;
  final String? path;
  final int eventCount;
  final double durationMs;
  final String? errorMessage;

  bool get hasError => errorMessage != null || !success;
}
