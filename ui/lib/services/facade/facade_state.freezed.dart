// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'facade_state.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
  'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models',
);

/// @nodoc
mixin _$FacadeState {
  /// Current engine status.
  EngineStatus get engine => throw _privateConstructorUsedError;

  /// Current device connection status.
  DeviceStatus get device => throw _privateConstructorUsedError;

  /// Current script validation status.
  ValidationStatus get validation => throw _privateConstructorUsedError;

  /// Current device discovery status.
  DiscoveryStatus get discovery => throw _privateConstructorUsedError;

  /// Path to the currently loaded script, if any.
  String? get scriptPath => throw _privateConstructorUsedError;

  /// Path to the selected/connected device, if any.
  String? get selectedDevicePath => throw _privateConstructorUsedError;

  /// Number of validation errors, if validation was performed.
  int? get validationErrorCount => throw _privateConstructorUsedError;

  /// Number of validation warnings, if validation was performed.
  int? get validationWarningCount => throw _privateConstructorUsedError;

  /// Number of discovered devices during last discovery.
  int? get discoveredDeviceCount => throw _privateConstructorUsedError;

  /// Last error message, if any operation failed.
  String? get lastError => throw _privateConstructorUsedError;

  /// Timestamp of this state snapshot.
  DateTime get timestamp => throw _privateConstructorUsedError;

  /// Create a copy of FacadeState
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $FacadeStateCopyWith<FacadeState> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $FacadeStateCopyWith<$Res> {
  factory $FacadeStateCopyWith(
    FacadeState value,
    $Res Function(FacadeState) then,
  ) = _$FacadeStateCopyWithImpl<$Res, FacadeState>;
  @useResult
  $Res call({
    EngineStatus engine,
    DeviceStatus device,
    ValidationStatus validation,
    DiscoveryStatus discovery,
    String? scriptPath,
    String? selectedDevicePath,
    int? validationErrorCount,
    int? validationWarningCount,
    int? discoveredDeviceCount,
    String? lastError,
    DateTime timestamp,
  });
}

/// @nodoc
class _$FacadeStateCopyWithImpl<$Res, $Val extends FacadeState>
    implements $FacadeStateCopyWith<$Res> {
  _$FacadeStateCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of FacadeState
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? engine = null,
    Object? device = null,
    Object? validation = null,
    Object? discovery = null,
    Object? scriptPath = freezed,
    Object? selectedDevicePath = freezed,
    Object? validationErrorCount = freezed,
    Object? validationWarningCount = freezed,
    Object? discoveredDeviceCount = freezed,
    Object? lastError = freezed,
    Object? timestamp = null,
  }) {
    return _then(
      _value.copyWith(
            engine: null == engine
                ? _value.engine
                : engine // ignore: cast_nullable_to_non_nullable
                      as EngineStatus,
            device: null == device
                ? _value.device
                : device // ignore: cast_nullable_to_non_nullable
                      as DeviceStatus,
            validation: null == validation
                ? _value.validation
                : validation // ignore: cast_nullable_to_non_nullable
                      as ValidationStatus,
            discovery: null == discovery
                ? _value.discovery
                : discovery // ignore: cast_nullable_to_non_nullable
                      as DiscoveryStatus,
            scriptPath: freezed == scriptPath
                ? _value.scriptPath
                : scriptPath // ignore: cast_nullable_to_non_nullable
                      as String?,
            selectedDevicePath: freezed == selectedDevicePath
                ? _value.selectedDevicePath
                : selectedDevicePath // ignore: cast_nullable_to_non_nullable
                      as String?,
            validationErrorCount: freezed == validationErrorCount
                ? _value.validationErrorCount
                : validationErrorCount // ignore: cast_nullable_to_non_nullable
                      as int?,
            validationWarningCount: freezed == validationWarningCount
                ? _value.validationWarningCount
                : validationWarningCount // ignore: cast_nullable_to_non_nullable
                      as int?,
            discoveredDeviceCount: freezed == discoveredDeviceCount
                ? _value.discoveredDeviceCount
                : discoveredDeviceCount // ignore: cast_nullable_to_non_nullable
                      as int?,
            lastError: freezed == lastError
                ? _value.lastError
                : lastError // ignore: cast_nullable_to_non_nullable
                      as String?,
            timestamp: null == timestamp
                ? _value.timestamp
                : timestamp // ignore: cast_nullable_to_non_nullable
                      as DateTime,
          )
          as $Val,
    );
  }
}

/// @nodoc
abstract class _$$FacadeStateImplCopyWith<$Res>
    implements $FacadeStateCopyWith<$Res> {
  factory _$$FacadeStateImplCopyWith(
    _$FacadeStateImpl value,
    $Res Function(_$FacadeStateImpl) then,
  ) = __$$FacadeStateImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({
    EngineStatus engine,
    DeviceStatus device,
    ValidationStatus validation,
    DiscoveryStatus discovery,
    String? scriptPath,
    String? selectedDevicePath,
    int? validationErrorCount,
    int? validationWarningCount,
    int? discoveredDeviceCount,
    String? lastError,
    DateTime timestamp,
  });
}

/// @nodoc
class __$$FacadeStateImplCopyWithImpl<$Res>
    extends _$FacadeStateCopyWithImpl<$Res, _$FacadeStateImpl>
    implements _$$FacadeStateImplCopyWith<$Res> {
  __$$FacadeStateImplCopyWithImpl(
    _$FacadeStateImpl _value,
    $Res Function(_$FacadeStateImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of FacadeState
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? engine = null,
    Object? device = null,
    Object? validation = null,
    Object? discovery = null,
    Object? scriptPath = freezed,
    Object? selectedDevicePath = freezed,
    Object? validationErrorCount = freezed,
    Object? validationWarningCount = freezed,
    Object? discoveredDeviceCount = freezed,
    Object? lastError = freezed,
    Object? timestamp = null,
  }) {
    return _then(
      _$FacadeStateImpl(
        engine: null == engine
            ? _value.engine
            : engine // ignore: cast_nullable_to_non_nullable
                  as EngineStatus,
        device: null == device
            ? _value.device
            : device // ignore: cast_nullable_to_non_nullable
                  as DeviceStatus,
        validation: null == validation
            ? _value.validation
            : validation // ignore: cast_nullable_to_non_nullable
                  as ValidationStatus,
        discovery: null == discovery
            ? _value.discovery
            : discovery // ignore: cast_nullable_to_non_nullable
                  as DiscoveryStatus,
        scriptPath: freezed == scriptPath
            ? _value.scriptPath
            : scriptPath // ignore: cast_nullable_to_non_nullable
                  as String?,
        selectedDevicePath: freezed == selectedDevicePath
            ? _value.selectedDevicePath
            : selectedDevicePath // ignore: cast_nullable_to_non_nullable
                  as String?,
        validationErrorCount: freezed == validationErrorCount
            ? _value.validationErrorCount
            : validationErrorCount // ignore: cast_nullable_to_non_nullable
                  as int?,
        validationWarningCount: freezed == validationWarningCount
            ? _value.validationWarningCount
            : validationWarningCount // ignore: cast_nullable_to_non_nullable
                  as int?,
        discoveredDeviceCount: freezed == discoveredDeviceCount
            ? _value.discoveredDeviceCount
            : discoveredDeviceCount // ignore: cast_nullable_to_non_nullable
                  as int?,
        lastError: freezed == lastError
            ? _value.lastError
            : lastError // ignore: cast_nullable_to_non_nullable
                  as String?,
        timestamp: null == timestamp
            ? _value.timestamp
            : timestamp // ignore: cast_nullable_to_non_nullable
                  as DateTime,
      ),
    );
  }
}

/// @nodoc

class _$FacadeStateImpl extends _FacadeState {
  const _$FacadeStateImpl({
    required this.engine,
    required this.device,
    required this.validation,
    required this.discovery,
    this.scriptPath,
    this.selectedDevicePath,
    this.validationErrorCount,
    this.validationWarningCount,
    this.discoveredDeviceCount,
    this.lastError,
    required this.timestamp,
  }) : super._();

  /// Current engine status.
  @override
  final EngineStatus engine;

  /// Current device connection status.
  @override
  final DeviceStatus device;

  /// Current script validation status.
  @override
  final ValidationStatus validation;

  /// Current device discovery status.
  @override
  final DiscoveryStatus discovery;

  /// Path to the currently loaded script, if any.
  @override
  final String? scriptPath;

  /// Path to the selected/connected device, if any.
  @override
  final String? selectedDevicePath;

  /// Number of validation errors, if validation was performed.
  @override
  final int? validationErrorCount;

  /// Number of validation warnings, if validation was performed.
  @override
  final int? validationWarningCount;

  /// Number of discovered devices during last discovery.
  @override
  final int? discoveredDeviceCount;

  /// Last error message, if any operation failed.
  @override
  final String? lastError;

  /// Timestamp of this state snapshot.
  @override
  final DateTime timestamp;

  @override
  String toString() {
    return 'FacadeState(engine: $engine, device: $device, validation: $validation, discovery: $discovery, scriptPath: $scriptPath, selectedDevicePath: $selectedDevicePath, validationErrorCount: $validationErrorCount, validationWarningCount: $validationWarningCount, discoveredDeviceCount: $discoveredDeviceCount, lastError: $lastError, timestamp: $timestamp)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$FacadeStateImpl &&
            (identical(other.engine, engine) || other.engine == engine) &&
            (identical(other.device, device) || other.device == device) &&
            (identical(other.validation, validation) ||
                other.validation == validation) &&
            (identical(other.discovery, discovery) ||
                other.discovery == discovery) &&
            (identical(other.scriptPath, scriptPath) ||
                other.scriptPath == scriptPath) &&
            (identical(other.selectedDevicePath, selectedDevicePath) ||
                other.selectedDevicePath == selectedDevicePath) &&
            (identical(other.validationErrorCount, validationErrorCount) ||
                other.validationErrorCount == validationErrorCount) &&
            (identical(other.validationWarningCount, validationWarningCount) ||
                other.validationWarningCount == validationWarningCount) &&
            (identical(other.discoveredDeviceCount, discoveredDeviceCount) ||
                other.discoveredDeviceCount == discoveredDeviceCount) &&
            (identical(other.lastError, lastError) ||
                other.lastError == lastError) &&
            (identical(other.timestamp, timestamp) ||
                other.timestamp == timestamp));
  }

  @override
  int get hashCode => Object.hash(
    runtimeType,
    engine,
    device,
    validation,
    discovery,
    scriptPath,
    selectedDevicePath,
    validationErrorCount,
    validationWarningCount,
    discoveredDeviceCount,
    lastError,
    timestamp,
  );

  /// Create a copy of FacadeState
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$FacadeStateImplCopyWith<_$FacadeStateImpl> get copyWith =>
      __$$FacadeStateImplCopyWithImpl<_$FacadeStateImpl>(this, _$identity);
}

abstract class _FacadeState extends FacadeState {
  const factory _FacadeState({
    required final EngineStatus engine,
    required final DeviceStatus device,
    required final ValidationStatus validation,
    required final DiscoveryStatus discovery,
    final String? scriptPath,
    final String? selectedDevicePath,
    final int? validationErrorCount,
    final int? validationWarningCount,
    final int? discoveredDeviceCount,
    final String? lastError,
    required final DateTime timestamp,
  }) = _$FacadeStateImpl;
  const _FacadeState._() : super._();

  /// Current engine status.
  @override
  EngineStatus get engine;

  /// Current device connection status.
  @override
  DeviceStatus get device;

  /// Current script validation status.
  @override
  ValidationStatus get validation;

  /// Current device discovery status.
  @override
  DiscoveryStatus get discovery;

  /// Path to the currently loaded script, if any.
  @override
  String? get scriptPath;

  /// Path to the selected/connected device, if any.
  @override
  String? get selectedDevicePath;

  /// Number of validation errors, if validation was performed.
  @override
  int? get validationErrorCount;

  /// Number of validation warnings, if validation was performed.
  @override
  int? get validationWarningCount;

  /// Number of discovered devices during last discovery.
  @override
  int? get discoveredDeviceCount;

  /// Last error message, if any operation failed.
  @override
  String? get lastError;

  /// Timestamp of this state snapshot.
  @override
  DateTime get timestamp;

  /// Create a copy of FacadeState
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$FacadeStateImplCopyWith<_$FacadeStateImpl> get copyWith =>
      throw _privateConstructorUsedError;
}
