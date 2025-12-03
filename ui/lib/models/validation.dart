/// Validation models for script validation results.
///
/// Dart representations of the Rust validation types for FFI integration.
library;

/// Result of script validation from the FFI layer.
class ValidationResult {
  const ValidationResult({
    required this.isValid,
    required this.errors,
    required this.warnings,
    this.coverage,
  });

  factory ValidationResult.fromJson(Map<String, dynamic> json) {
    return ValidationResult(
      isValid: json['is_valid'] as bool? ?? false,
      errors: (json['errors'] as List<dynamic>?)
              ?.map((e) => ValidationError.fromJson(e as Map<String, dynamic>))
              .toList() ??
          const [],
      warnings: (json['warnings'] as List<dynamic>?)
              ?.map(
                  (e) => ValidationWarning.fromJson(e as Map<String, dynamic>))
              .toList() ??
          const [],
      coverage: json['coverage'] != null
          ? CoverageReport.fromJson(json['coverage'] as Map<String, dynamic>)
          : null,
    );
  }

  factory ValidationResult.error(String message) => ValidationResult(
        isValid: false,
        errors: [ValidationError(code: 'E000', message: message)],
        warnings: const [],
      );

  final bool isValid;
  final List<ValidationError> errors;
  final List<ValidationWarning> warnings;
  final CoverageReport? coverage;

  bool get hasErrors => errors.isNotEmpty;
  bool get hasWarnings => warnings.isNotEmpty;
}

/// A validation error from script validation.
class ValidationError {
  const ValidationError({
    required this.code,
    required this.message,
    this.location,
    this.suggestions = const [],
  });

  factory ValidationError.fromJson(Map<String, dynamic> json) {
    return ValidationError(
      code: json['code'] as String? ?? 'E000',
      message: json['message'] as String? ?? 'Unknown error',
      location: json['location'] != null
          ? SourceLocation.fromJson(json['location'] as Map<String, dynamic>)
          : null,
      suggestions: (json['suggestions'] as List<dynamic>?)
              ?.map((e) => e as String)
              .toList() ??
          const [],
    );
  }

  final String code;
  final String message;
  final SourceLocation? location;
  final List<String> suggestions;

  bool get hasSuggestions => suggestions.isNotEmpty;
}

/// A validation warning from script validation.
class ValidationWarning {
  const ValidationWarning({
    required this.code,
    required this.category,
    required this.message,
    this.location,
  });

  factory ValidationWarning.fromJson(Map<String, dynamic> json) {
    return ValidationWarning(
      code: json['code'] as String? ?? 'W000',
      category: WarningCategory.fromString(json['category'] as String?),
      message: json['message'] as String? ?? 'Unknown warning',
      location: json['location'] != null
          ? SourceLocation.fromJson(json['location'] as Map<String, dynamic>)
          : null,
    );
  }

  final String code;
  final WarningCategory category;
  final String message;
  final SourceLocation? location;
}

/// Categories of validation warnings.
enum WarningCategory {
  conflict,
  safety,
  performance;

  static WarningCategory fromString(String? value) {
    return switch (value?.toLowerCase()) {
      'conflict' => WarningCategory.conflict,
      'safety' => WarningCategory.safety,
      'performance' => WarningCategory.performance,
      _ => WarningCategory.conflict,
    };
  }
}

/// Source location for errors and warnings.
class SourceLocation {
  const SourceLocation({
    required this.line,
    this.column,
    this.context,
  });

  factory SourceLocation.fromJson(Map<String, dynamic> json) {
    return SourceLocation(
      line: (json['line'] as num?)?.toInt() ?? 0,
      column: (json['column'] as num?)?.toInt(),
      context: json['context'] as String?,
    );
  }

  final int line;
  final int? column;
  final String? context;
}

/// Coverage report showing affected keys.
class CoverageReport {
  const CoverageReport({
    this.remapped = const [],
    this.blocked = const [],
    this.tapHold = const [],
    this.comboTriggers = const [],
    this.unaffected = const [],
    this.layers = const {},
  });

  factory CoverageReport.fromJson(Map<String, dynamic> json) {
    return CoverageReport(
      remapped: _parseKeyList(json['remapped']),
      blocked: _parseKeyList(json['blocked']),
      tapHold: _parseKeyList(json['tap_hold']),
      comboTriggers: _parseKeyList(json['combo_triggers']),
      unaffected: _parseKeyList(json['unaffected']),
      layers: (json['layers'] as Map<String, dynamic>?)?.map(
            (k, v) =>
                MapEntry(k, LayerCoverage.fromJson(v as Map<String, dynamic>)),
          ) ??
          const {},
    );
  }

  final List<String> remapped;
  final List<String> blocked;
  final List<String> tapHold;
  final List<String> comboTriggers;
  final List<String> unaffected;
  final Map<String, LayerCoverage> layers;

  int get affectedCount =>
      remapped.length + blocked.length + tapHold.length + comboTriggers.length;

  static List<String> _parseKeyList(dynamic value) {
    if (value is! List) return const [];
    return value.map((e) {
      if (e is String) return e;
      if (e is Map<String, dynamic>) return e['name'] as String? ?? '';
      return '';
    }).where((s) => s.isNotEmpty).toList();
  }
}

/// Per-layer coverage information.
class LayerCoverage {
  const LayerCoverage({
    this.remapped = const [],
    this.blocked = const [],
  });

  factory LayerCoverage.fromJson(Map<String, dynamic> json) {
    return LayerCoverage(
      remapped: CoverageReport._parseKeyList(json['remapped']),
      blocked: CoverageReport._parseKeyList(json['blocked']),
    );
  }

  final List<String> remapped;
  final List<String> blocked;
}

/// Options for validation.
class ValidationOptions {
  const ValidationOptions({
    this.strict = false,
    this.noWarnings = false,
    this.includeCoverage = false,
    this.includeVisual = false,
  });

  final bool strict;
  final bool noWarnings;
  final bool includeCoverage;
  final bool includeVisual;

  Map<String, dynamic> toJson() => {
        'strict': strict,
        'no_warnings': noWarnings,
        'include_coverage': includeCoverage,
        'include_visual': includeVisual,
      };
}
