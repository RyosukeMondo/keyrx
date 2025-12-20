/// Auto-save service with debouncing and retry logic.
library;

import 'dart:async';

import 'package:core_logic/models/hardware_profile.dart';
import 'package:core_logic/models/runtime_config.dart';
import 'package:core_logic/services/config_result.dart';
import 'package:core_logic/utils/logger.dart';

import '../repositories/keyflow_repository.dart';
import '../services/keyflow_analytics.dart';

/// Mutation operation types for auto-save.
sealed class SaveOperation {
  const SaveOperation();
}

class AddSlotOperation extends SaveOperation {
  const AddSlotOperation(this.device, this.slot, {this.priorityOverride});

  final DeviceInstanceId device;
  final ProfileSlot slot;
  final int? priorityOverride;
}

class UpdateSlotOperation extends SaveOperation {
  const UpdateSlotOperation(this.device, this.slot);

  final DeviceInstanceId device;
  final ProfileSlot slot;
}

class RemoveSlotOperation extends SaveOperation {
  const RemoveSlotOperation(this.device, this.slotId);

  final DeviceInstanceId device;
  final String slotId;
}

/// Service responsible for auto-saving mutations with debouncing and retry.
///
/// Provides:
/// - 500ms debounce after last change
/// - Exponential backoff retry (max 3 retries)
/// - Save state tracking (idle/saving/saved/error)
/// - Structured logging
class AutoSaveService {
  AutoSaveService({
    required KeyFlowRepository repository,
    Duration debounceDelay = const Duration(milliseconds: 500),
    int maxRetries = 3,
    StructuredLogger? logger,
    KeyFlowAnalytics? analytics,
  }) : _repository = repository,
       _debounceDelay = debounceDelay,
       _maxRetries = maxRetries,
       _logger = logger ?? getLogger('auto_save_service'),
       _analytics = analytics {
    _logger.info(
      'auto_save.init',
      context: {
        'debounce_ms': _debounceDelay.inMilliseconds,
        'max_retries': _maxRetries,
      },
    );
  }

  final KeyFlowRepository _repository;
  final Duration _debounceDelay;
  final int _maxRetries;
  final StructuredLogger _logger;
  final KeyFlowAnalytics? _analytics;

  Timer? _debounceTimer;
  SaveOperation? _pendingOperation;
  bool _saving = false;
  bool _disposed = false;

  final _stateController = StreamController<SaveState>.broadcast();

  /// Stream of save state changes.
  Stream<SaveState> get stateStream => _stateController.stream;

  /// Current save state.
  SaveState _currentState = SaveState.idle();
  SaveState get currentState => _currentState;

  /// Schedule a mutation to be saved with debouncing.
  ///
  /// If a save is already pending, it will be replaced with this new operation.
  void scheduleSave(SaveOperation operation) {
    if (_disposed) {
      _logger.warn('auto_save.schedule_after_dispose');
      return;
    }

    _pendingOperation = operation;
    _debounceTimer?.cancel();

    _logger.debug(
      'auto_save.schedule',
      context: {
        'operation': operation.runtimeType.toString(),
        'delay_ms': _debounceDelay.inMilliseconds,
      },
    );

    _debounceTimer = Timer(_debounceDelay, _executeSave);
  }

  /// Execute the pending save operation with retry logic.
  Future<void> _executeSave() async {
    final operation = _pendingOperation;
    if (operation == null || _saving) {
      return;
    }

    _saving = true;
    _updateState(SaveState.saving());
    _logger.info(
      'auto_save.start',
      context: {'operation': operation.runtimeType.toString()},
    );

    var attempt = 0;
    ConfigOperationResult<dynamic>? result;

    while (attempt < _maxRetries) {
      try {
        result = await _performOperation(operation);

        if (result.isSuccess) {
          _logger.info('auto_save.success', context: {'attempt': attempt + 1});
          _updateState(SaveState.saved());

          // Track analytics
          _analytics?.trackAutoSave(
            operation: operation.runtimeType.toString(),
            success: true,
            retryCount: attempt > 0 ? attempt : null,
          );

          _pendingOperation = null;
          _saving = false;

          // Reset to idle after 2 seconds
          Timer(const Duration(seconds: 2), () {
            if (_currentState.status == SaveStatus.saved) {
              _updateState(SaveState.idle());
            }
          });
          return;
        }

        attempt++;
        if (attempt < _maxRetries) {
          final delayMs = _calculateBackoffDelay(attempt);
          _logger.warn(
            'auto_save.retry',
            message: result.errorMessage,
            context: {'attempt': attempt, 'delay_ms': delayMs},
          );
          await Future<void>.delayed(Duration(milliseconds: delayMs));
        }
      } on Exception catch (e, st) {
        _logger.error(
          'auto_save.exception',
          message: '$e',
          error: e,
          stackTrace: st,
          context: {'attempt': attempt + 1},
        );
        attempt++;
        if (attempt < _maxRetries) {
          final delayMs = _calculateBackoffDelay(attempt);
          await Future<void>.delayed(Duration(milliseconds: delayMs));
        }
      }
    }

    // All retries failed
    final errorMessage = result?.errorMessage ?? 'Save failed after retries';
    _logger.error(
      'auto_save.failed',
      message: errorMessage,
      context: {'attempts': _maxRetries},
    );

    // Track analytics
    _analytics?.trackAutoSave(
      operation: operation.runtimeType.toString(),
      success: false,
      retryCount: _maxRetries,
      errorMessage: errorMessage,
    );

    _updateState(SaveState.error(errorMessage));
    _saving = false;
  }

  Future<ConfigOperationResult<dynamic>> _performOperation(
    SaveOperation operation,
  ) async => switch (operation) {
    final AddSlotOperation op => await _repository.addSlot(
      op.device,
      op.slot,
      priorityOverride: op.priorityOverride,
    ),
    final UpdateSlotOperation op => await _repository.updateSlot(
      op.device,
      op.slot,
    ),
    final RemoveSlotOperation op => await _repository.removeSlot(
      op.device,
      op.slotId,
    ),
  };

  /// Calculate exponential backoff delay in milliseconds.
  int _calculateBackoffDelay(int attempt) {
    // Base delay: 100ms, exponential: 100, 200, 400, ...
    return 100 * (1 << (attempt - 1));
  }

  void _updateState(SaveState newState) {
    _currentState = newState;
    if (!_stateController.isClosed) {
      _stateController.add(newState);
    }
  }

  /// Retry the last failed save operation.
  Future<void> retry() async {
    if (_currentState.status != SaveStatus.error || _pendingOperation == null) {
      _logger.warn('auto_save.retry_invalid_state');
      return;
    }

    _logger.info('auto_save.manual_retry');
    await _executeSave();
  }

  /// Cancel any pending save and clean up resources.
  void dispose() {
    _disposed = true;
    _debounceTimer?.cancel();
    _stateController.close();
    _logger.info('auto_save.disposed');
  }
}

/// Save state tracking.
enum SaveStatus { idle, saving, saved, error }

class SaveState {
  const SaveState({required this.status, this.errorMessage});

  factory SaveState.idle() => const SaveState(status: SaveStatus.idle);
  factory SaveState.saving() => const SaveState(status: SaveStatus.saving);
  factory SaveState.saved() => const SaveState(status: SaveStatus.saved);
  factory SaveState.error(String message) =>
      SaveState(status: SaveStatus.error, errorMessage: message);

  final SaveStatus status;
  final String? errorMessage;

  bool get isIdle => status == SaveStatus.idle;
  bool get isSaving => status == SaveStatus.saving;
  bool get isSaved => status == SaveStatus.saved;
  bool get isError => status == SaveStatus.error;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is SaveState &&
          status == other.status &&
          errorMessage == other.errorMessage;

  @override
  int get hashCode => Object.hash(status, errorMessage);

  @override
  String toString() =>
      'SaveState($status${errorMessage != null ? ', $errorMessage' : ''})';
}
