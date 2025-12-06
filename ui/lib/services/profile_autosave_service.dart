/// Debounced autosave service for profiles.
///
/// Queues profile saves with debounce, retries transient failures with
/// exponential backoff, and emits status updates for UI feedback.
library;

import 'dart:async';
import 'dart:math' as math;

import '../config/timing_config.dart';
import '../models/profile.dart';
import 'profile_registry_service.dart';
import 'storage_path_resolver.dart';

/// Autosave state lifecycle.
enum AutosaveState { idle, queued, saving, success, error }

/// Snapshot of the current autosave status.
class AutosaveStatus {
  const AutosaveStatus({
    required this.state,
    this.profileId,
    this.attempt = 0,
    this.lastSavedAt,
    this.errorMessage,
    this.targetDirectory,
  });

  /// Current lifecycle state.
  final AutosaveState state;

  /// Profile identifier associated with the status.
  final String? profileId;

  /// Current attempt number (1-based).
  final int attempt;

  /// Last successful save time.
  final DateTime? lastSavedAt;

  /// Error message when [state] == [AutosaveState.error].
  final String? errorMessage;

  /// Directory path where the profile is being saved.
  final String? targetDirectory;

  /// Convenience factory for idle state.
  factory AutosaveStatus.idle({DateTime? lastSavedAt, String? profileId}) {
    return AutosaveStatus(
      state: AutosaveState.idle,
      profileId: profileId,
      lastSavedAt: lastSavedAt,
    );
  }

  /// Convenience factory for queued state.
  factory AutosaveStatus.queued(String profileId) {
    return AutosaveStatus(
      state: AutosaveState.queued,
      profileId: profileId,
    );
  }

  /// Convenience factory for saving state.
  factory AutosaveStatus.saving({
    required String profileId,
    required int attempt,
    required String targetDirectory,
  }) {
    return AutosaveStatus(
      state: AutosaveState.saving,
      profileId: profileId,
      attempt: attempt,
      targetDirectory: targetDirectory,
    );
  }

  /// Convenience factory for success state.
  factory AutosaveStatus.success({
    required String profileId,
    required DateTime savedAt,
    required String targetDirectory,
  }) {
    return AutosaveStatus(
      state: AutosaveState.success,
      profileId: profileId,
      lastSavedAt: savedAt,
      targetDirectory: targetDirectory,
    );
  }

  /// Convenience factory for error state.
  factory AutosaveStatus.error({
    required String profileId,
    required String message,
    required int attempt,
    String? targetDirectory,
    DateTime? lastSavedAt,
  }) {
    return AutosaveStatus(
      state: AutosaveState.error,
      profileId: profileId,
      attempt: attempt,
      errorMessage: message,
      lastSavedAt: lastSavedAt,
      targetDirectory: targetDirectory,
    );
  }
}

/// Provides debounced autosave with retry/backoff semantics.
class ProfileAutosaveService {
  ProfileAutosaveService({
    required ProfileRegistryService profileRegistryService,
    required StoragePathResolver storagePathResolver,
    Duration? debounceDuration,
    Duration? initialBackoff,
    int maxRetries = 3,
    Future<void> Function(Duration duration)? delayFn,
    DateTime Function()? now,
    Future<ProfileRegistryOperationResult> Function(Profile profile)?
        saveOperation,
  })  : _storagePathResolver = storagePathResolver,
        _debounceDuration =
            debounceDuration ??
            const Duration(milliseconds: TimingConfig.debounceMs),
        _initialBackoff =
            initialBackoff ?? const Duration(milliseconds: 200),
        _maxRetries = maxRetries,
        _delayFn = delayFn ?? Future.delayed,
        _now = now ?? DateTime.now,
        _saveOperation =
            saveOperation ?? profileRegistryService.saveProfile {
    _emit(AutosaveStatus.idle());
  }

  final StoragePathResolver _storagePathResolver;
  final Duration _debounceDuration;
  final Duration _initialBackoff;
  final int _maxRetries;
  final Future<void> Function(Duration duration) _delayFn;
  final DateTime Function() _now;
  final Future<ProfileRegistryOperationResult> Function(Profile profile)
      _saveOperation;

  Timer? _debounceTimer;
  Profile? _pendingProfile;
  bool _saving = false;
  bool _disposed = false;
  AutosaveStatus _status = const AutosaveStatus(state: AutosaveState.idle);

  final StreamController<AutosaveStatus> _statusController =
      StreamController<AutosaveStatus>.broadcast();

  /// Current autosave status snapshot.
  AutosaveStatus get status => _status;

  /// Stream of autosave status updates.
  Stream<AutosaveStatus> get statusStream => _statusController.stream;

  /// Queue a profile for autosave with debounce.
  void queueSave(Profile profile) {
    if (_disposed) return;

    _pendingProfile = profile;
    _debounceTimer?.cancel();
    _debounceTimer = Timer(_debounceDuration, _flushPending);

    _emit(AutosaveStatus.queued(profile.id));
  }

  /// Cancel timers and close status streams.
  Future<void> dispose() async {
    _disposed = true;
    _debounceTimer?.cancel();
    await _statusController.close();
  }

  Future<void> _flushPending() async {
    if (_disposed || _saving) {
      return;
    }

    final profile = _pendingProfile;
    _pendingProfile = null;

    if (profile == null) {
      return;
    }

    await _saveWithRetry(profile);
  }

  Future<void> _saveWithRetry(Profile profile) async {
    _saving = true;

    final targetDirectory = await _resolveTargetDirectory(profile.id);
    if (targetDirectory == null) {
      _saving = false;
      return;
    }

    final totalAttempts = 1 + _maxRetries;
    ProfileRegistryOperationResult? result;

    for (var attempt = 1; attempt <= totalAttempts; attempt++) {
      _emit(
        AutosaveStatus.saving(
          profileId: profile.id,
          attempt: attempt,
          targetDirectory: targetDirectory,
        ),
      );

      try {
        result = await _saveOperation(profile);
      } catch (e) {
        result = ProfileRegistryOperationResult.error(e.toString());
      }

      if (result.success) {
        final savedAt = _now();
        _emit(
          AutosaveStatus.success(
            profileId: profile.id,
            savedAt: savedAt,
            targetDirectory: targetDirectory,
          ),
        );
        _saving = false;
        break;
      }

      final shouldRetry =
          attempt < totalAttempts && _isTransientError(result.errorMessage);
      if (!shouldRetry) {
        _emit(
          AutosaveStatus.error(
            profileId: profile.id,
            message: result.errorMessage ?? 'Autosave failed',
            attempt: attempt,
            targetDirectory: targetDirectory,
            lastSavedAt: _status.lastSavedAt,
          ),
        );
        _saving = false;
        break;
      }

      final backoff = _backoffForAttempt(attempt);
      await _delayFn(backoff);
    }

    _saving = false;

    // If another profile update was queued during the save, process it next.
    if (_pendingProfile != null && !_disposed) {
      _debounceTimer?.cancel();
      _debounceTimer = Timer(_debounceDuration, _flushPending);
    }
  }

  Duration _backoffForAttempt(int attempt) {
    // attempt is 1-based; backoff grows exponentially after the first attempt.
    if (attempt <= 1) return _initialBackoff;
    final factor = math.pow(2, attempt - 1).toInt();
    return Duration(milliseconds: _initialBackoff.inMilliseconds * factor);
  }

  Future<String?> _resolveTargetDirectory(String profileId) async {
    try {
      return await _storagePathResolver.ensureProfilesDirectory();
    } catch (e) {
      _emit(
        AutosaveStatus.error(
          profileId: profileId,
          message: 'Unable to prepare profiles directory: $e',
          attempt: 1,
          targetDirectory: null,
          lastSavedAt: _status.lastSavedAt,
        ),
      );
      return null;
    }
  }

  bool _isTransientError(String? message) {
    if (message == null) return true;
    final lower = message.toLowerCase();

    // Treat validation and invalid data errors as non-transient to avoid loops.
    if (lower.contains('validation') || lower.contains('invalid')) {
      return false;
    }

    // Consider IO/timeouts/transient keywords as retryable.
    return lower.contains('io') ||
        lower.contains('timeout') ||
        lower.contains('temporar') ||
        lower.contains('retry') ||
        lower.contains('unavailable') ||
        lower.contains('busy');
  }

  void _emit(AutosaveStatus status) {
    _status = status;
    if (!_disposed) {
      _statusController.add(status);
    }
  }
}
