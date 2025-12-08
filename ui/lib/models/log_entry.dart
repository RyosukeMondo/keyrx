class LogEntry {
  const LogEntry({
    required this.level,
    required this.message,
    required this.target,
    required this.fields,
    required this.timestamp,
  });

  final String level;
  final String message;
  final String target;
  final Map<String, dynamic> fields;
  final DateTime timestamp;

  factory LogEntry.fromJson(Map<String, dynamic> json) {
    return LogEntry(
      level: json['level'] as String? ?? 'UNKNOWN',
      message: json['message'] as String? ?? '',
      target: json['target'] as String? ?? '',
      fields: Map<String, dynamic>.from(json['fields'] as Map? ?? {}),
      timestamp: DateTime.now(), // Logs from bridge don't have timestamp yet
    );
  }

  @override
  String toString() => '[$level] $message';
}
