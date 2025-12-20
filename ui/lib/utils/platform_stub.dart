/// Stub for platform-specific APIs on web.
///
/// Provides stub implementations of dart:io APIs that are not available on web.
library;

/// Stub Directory class for web platform.
class Directory {
  Directory();

  Future<bool> exists() async {
    throw UnsupportedError('Directory not available on web');
  }
}

/// Stub Platform class for web platform.
class Platform {
  static Map<String, String> get environment => {};
  static bool get isWindows => false;
}
