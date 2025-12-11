// FFI Introspection Models
//
// Dart models for FFI introspection metadata from Rust.
// These models match the Rust types in core/src/ffi/introspection.rs

import 'dart:convert';

/// Introspection metadata for all FFI functions
class IntrospectionData {
  final int protocolVersion;
  final List<DomainMetadata> domains;
  final int totalFunctions;
  final int totalEvents;

  IntrospectionData({
    required this.protocolVersion,
    required this.domains,
    required this.totalFunctions,
    required this.totalEvents,
  });

  factory IntrospectionData.fromJson(Map<String, dynamic> json) {
    return IntrospectionData(
      protocolVersion: json['protocol_version'] as int,
      domains: (json['domains'] as List)
          .map((d) => DomainMetadata.fromJson(d))
          .toList(),
      totalFunctions: json['total_functions'] as int,
      totalEvents: json['total_events'] as int,
    );
  }

  Map<String, dynamic> toJson() => {
        'protocol_version': protocolVersion,
        'domains': domains.map((d) => d.toJson()).toList(),
        'total_functions': totalFunctions,
        'total_events': totalEvents,
      };
}

/// Domain metadata
class DomainMetadata {
  final String name;
  final String description;
  final String version;
  final List<FunctionMetadata> functions;
  final List<EventMetadata> events;

  DomainMetadata({
    required this.name,
    required this.description,
    required this.version,
    required this.functions,
    required this.events,
  });

  factory DomainMetadata.fromJson(Map<String, dynamic> json) {
    return DomainMetadata(
      name: json['name'] as String,
      description: json['description'] as String,
      version: json['version'] as String,
      functions: (json['functions'] as List)
          .map((f) => FunctionMetadata.fromJson(f))
          .toList(),
      events: (json['events'] as List)
          .map((e) => EventMetadata.fromJson(e))
          .toList(),
    );
  }

  Map<String, dynamic> toJson() => {
        'name': name,
        'description': description,
        'version': version,
        'functions': functions.map((f) => f.toJson()).toList(),
        'events': events.map((e) => e.toJson()).toList(),
      };
}

/// Function metadata
class FunctionMetadata {
  final String name;
  final String rustName;
  final String description;
  final List<ParameterMetadata> parameters;
  final TypeMetadata returns;
  final List<String> errors;
  final List<String> eventsEmitted;
  final bool deprecated;
  final ExampleMetadata? example;

  FunctionMetadata({
    required this.name,
    required this.rustName,
    required this.description,
    required this.parameters,
    required this.returns,
    required this.errors,
    required this.eventsEmitted,
    required this.deprecated,
    this.example,
  });

  factory FunctionMetadata.fromJson(Map<String, dynamic> json) {
    return FunctionMetadata(
      name: json['name'] as String,
      rustName: json['rust_name'] as String,
      description: json['description'] as String,
      parameters: (json['parameters'] as List)
          .map((p) => ParameterMetadata.fromJson(p))
          .toList(),
      returns: TypeMetadata.fromJson(json['returns']),
      errors: (json['errors'] as List).cast<String>(),
      eventsEmitted: (json['events_emitted'] as List).cast<String>(),
      deprecated: json['deprecated'] as bool,
      example: json['example'] != null
          ? ExampleMetadata.fromJson(json['example'])
          : null,
    );
  }

  Map<String, dynamic> toJson() => {
        'name': name,
        'rust_name': rustName,
        'description': description,
        'parameters': parameters.map((p) => p.toJson()).toList(),
        'returns': returns.toJson(),
        'errors': errors,
        'events_emitted': eventsEmitted,
        'deprecated': deprecated,
        if (example != null) 'example': example!.toJson(),
      };
}

/// Parameter metadata
class ParameterMetadata {
  final String name;
  final String typeName;
  final String description;
  final bool required;
  final Map<String, dynamic>? constraints;

  ParameterMetadata({
    required this.name,
    required this.typeName,
    required this.description,
    required this.required,
    this.constraints,
  });

  factory ParameterMetadata.fromJson(Map<String, dynamic> json) {
    return ParameterMetadata(
      name: json['name'] as String,
      typeName: json['type_name'] as String,
      description: json['description'] as String,
      required: json['required'] as bool,
      constraints: json['constraints'] as Map<String, dynamic>?,
    );
  }

  Map<String, dynamic> toJson() => {
        'name': name,
        'type_name': typeName,
        'description': description,
        'required': required,
        if (constraints != null) 'constraints': constraints,
      };
}

/// Type metadata
class TypeMetadata {
  final String typeName;
  final String kind; // "primitive", "object", "array", "enum"
  final String? description;
  final Map<String, TypeMetadata>? properties;
  final TypeMetadata? items;

  TypeMetadata({
    required this.typeName,
    required this.kind,
    this.description,
    this.properties,
    this.items,
  });

  factory TypeMetadata.fromJson(Map<String, dynamic> json) {
    return TypeMetadata(
      typeName: json['type_name'] as String,
      kind: json['kind'] as String,
      description: json['description'] as String?,
      properties: json['properties'] != null
          ? (json['properties'] as Map<String, dynamic>).map(
              (k, v) => MapEntry(k, TypeMetadata.fromJson(v)),
            )
          : null,
      items: json['items'] != null
          ? TypeMetadata.fromJson(json['items'])
          : null,
    );
  }

  Map<String, dynamic> toJson() => {
        'type_name': typeName,
        'kind': kind,
        if (description != null) 'description': description,
        if (properties != null)
          'properties':
              properties!.map((k, v) => MapEntry(k, v.toJson())),
        if (items != null) 'items': items!.toJson(),
      };
}

/// Event metadata
class EventMetadata {
  final String name;
  final String description;
  final TypeMetadata payload;

  EventMetadata({
    required this.name,
    required this.description,
    required this.payload,
  });

  factory EventMetadata.fromJson(Map<String, dynamic> json) {
    return EventMetadata(
      name: json['name'] as String,
      description: json['description'] as String,
      payload: TypeMetadata.fromJson(json['payload']),
    );
  }

  Map<String, dynamic> toJson() => {
        'name': name,
        'description': description,
        'payload': payload.toJson(),
      };
}

/// Example metadata
class ExampleMetadata {
  final Map<String, dynamic> input;
  final Map<String, dynamic> output;

  ExampleMetadata({
    required this.input,
    required this.output,
  });

  factory ExampleMetadata.fromJson(Map<String, dynamic> json) {
    return ExampleMetadata(
      input: json['input'] as Map<String, dynamic>,
      output: json['output'] as Map<String, dynamic>,
    );
  }

  Map<String, dynamic> toJson() => {
        'input': input,
        'output': output,
      };
}

/// FFI Result wrapper
class FfiResult<T> {
  final bool success;
  final T? data;
  final FfiError? error;

  FfiResult.ok(this.data)
      : success = true,
        error = null;

  FfiResult.err(this.error)
      : success = false,
        data = null;

  factory FfiResult.fromJson(
    String json,
    T Function(Map<String, dynamic>) fromJson,
  ) {
    if (json.startsWith('ok:')) {
      final jsonStr = json.substring(3);
      final decoded = jsonDecode(jsonStr) as Map<String, dynamic>;
      return FfiResult.ok(fromJson(decoded));
    } else if (json.startsWith('error:')) {
      final jsonStr = json.substring(6);
      final decoded = jsonDecode(jsonStr) as Map<String, dynamic>;
      return FfiResult.err(FfiError.fromJson(decoded));
    } else {
      throw FormatException('Invalid FFI result format: $json');
    }
  }
}

/// FFI Error
class FfiError {
  final String code;
  final String message;
  final Map<String, dynamic>? details;

  FfiError({
    required this.code,
    required this.message,
    this.details,
  });

  factory FfiError.fromJson(Map<String, dynamic> json) {
    return FfiError(
      code: json['code'] as String,
      message: json['message'] as String,
      details: json['details'] as Map<String, dynamic>?,
    );
  }

  Map<String, dynamic> toJson() => {
        'code': code,
        'message': message,
        if (details != null) 'details': details,
      };

  @override
  String toString() => 'FfiError($code): $message';
}
