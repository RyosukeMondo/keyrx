/// Contracts for platform permission handling used by the UI.
library;

/// Permission outcomes relevant to microphone access.
enum PermissionState { granted, denied, permanentlyDenied, restricted }

/// Result of a permission request/check.
class PermissionResult {
  const PermissionResult({
    required this.state,
    this.shouldShowRationale = false,
  });

  final PermissionState state;
  final bool shouldShowRationale;

  bool get isGranted => state == PermissionState.granted;
}

/// Abstraction over platform permissions (e.g., permission_handler).
abstract class PermissionService {
  /// Request microphone permission from the user.
  Future<PermissionResult> requestMicrophone();

  /// Check current microphone permission without prompting.
  Future<PermissionResult> checkMicrophone();
}
