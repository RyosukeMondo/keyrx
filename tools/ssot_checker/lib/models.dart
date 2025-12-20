enum ViolationType {
  flutterInCoreLogic,
  businessLogicInUI,
  interfaceInconsistency,
  duplicateImplementation,
}

class SSOTViolation {
  const SSOTViolation({
    required this.type,
    required this.file,
    required this.line,
    required this.message,
    required this.suggestion,
  });

  final ViolationType type;
  final String file;
  final int? line;
  final String message;
  final String suggestion;

  @override
  String toString() {
    final location = line != null ? '$file:$line' : file;
    return '[$type] $location: $message\n  → $suggestion';
  }
}
