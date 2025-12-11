// FFI Introspection Service
//
// Service for fetching and caching FFI metadata from Rust.

import 'dart:ffi';
import 'package:ffi/ffi.dart';
import '../ffi/bindings.dart';
import '../ffi/introspection_models.dart';

class FfiIntrospectionService {
  final KeyrxBindings _bindings;
  IntrospectionData? _cachedData;
  DateTime? _lastFetch;

  FfiIntrospectionService(this._bindings);

  /// Get introspection metadata (cached for 5 minutes)
  Future<IntrospectionData> getMetadata({bool forceRefresh = false}) async {
    if (!forceRefresh &&
        _cachedData != null &&
        _lastFetch != null &&
        DateTime.now().difference(_lastFetch!).inMinutes < 5) {
      return _cachedData!;
    }

    final resultPtr = _bindings.introspectionMetadata();
    if (resultPtr == nullptr) {
      throw Exception('Failed to fetch introspection metadata');
    }

    try {
      final resultStr = resultPtr.cast<Utf8>().toDartString();
      final result =
          FfiResult.fromJson(resultStr, IntrospectionData.fromJson);

      if (!result.success || result.data == null) {
        throw Exception(
            'Failed to fetch introspection metadata: ${result.error}');
      }

      _cachedData = result.data!;
      _lastFetch = DateTime.now();
      return _cachedData!;
    } finally {
      _bindings.freeString(resultPtr.cast());
    }
  }

  /// Get metadata for a specific domain
  Future<DomainMetadata?> getDomainMetadata(String domainName) async {
    final data = await getMetadata();
    return data.domains.firstWhere(
      (d) => d.name == domainName,
      orElse: () => throw Exception('Domain not found: $domainName'),
    );
  }

  /// Get metadata for a specific function
  Future<FunctionMetadata?> getFunctionMetadata(
    String domainName,
    String functionName,
  ) async {
    final domain = await getDomainMetadata(domainName);
    return domain?.functions.firstWhere(
      (f) => f.name == functionName,
      orElse: () => throw Exception(
          'Function not found: $domainName.$functionName'),
    );
  }

  /// Get all domains
  Future<List<DomainMetadata>> getAllDomains() async {
    final data = await getMetadata();
    return data.domains;
  }

  /// Search functions by name
  Future<List<FunctionSearchResult>> searchFunctions(String query) async {
    final data = await getMetadata();
    final results = <FunctionSearchResult>[];
    final lowerQuery = query.toLowerCase();

    for (final domain in data.domains) {
      for (final function in domain.functions) {
        if (function.name.toLowerCase().contains(lowerQuery) ||
            function.description.toLowerCase().contains(lowerQuery)) {
          results.add(FunctionSearchResult(
            domain: domain.name,
            function: function,
          ));
        }
      }
    }

    return results;
  }

  /// Get all events across all domains
  Future<List<EventSearchResult>> getAllEvents() async {
    final data = await getMetadata();
    final results = <EventSearchResult>[];

    for (final domain in data.domains) {
      for (final event in domain.events) {
        results.add(EventSearchResult(
          domain: domain.name,
          event: event,
        ));
      }
    }

    return results;
  }

  /// Clear cache
  void clearCache() {
    _cachedData = null;
    _lastFetch = null;
  }
}

/// Function search result
class FunctionSearchResult {
  final String domain;
  final FunctionMetadata function;

  FunctionSearchResult({
    required this.domain,
    required this.function,
  });

  String get fullName => '$domain.${function.name}';
}

/// Event search result
class EventSearchResult {
  final String domain;
  final EventMetadata event;

  EventSearchResult({
    required this.domain,
    required this.event,
  });

  String get fullName => '$domain.${event.name}';
}
