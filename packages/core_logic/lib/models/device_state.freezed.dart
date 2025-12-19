// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'device_state.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
  'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models',
);

DeviceState _$DeviceStateFromJson(Map<String, dynamic> json) {
  return _DeviceState.fromJson(json);
}

/// @nodoc
mixin _$DeviceState {
  /// Device identity
  DeviceIdentity get identity => throw _privateConstructorUsedError;

  /// Whether the device is enabled (participates in KeyRx)
  /// When false, device is in total passthrough mode
  @JsonKey(name: 'device_enabled')
  bool get deviceEnabled => throw _privateConstructorUsedError;

  /// Whether remapping is enabled for this device
  /// Only effective when deviceEnabled is true
  @JsonKey(name: 'remap_enabled')
  bool get remapEnabled => throw _privateConstructorUsedError;

  /// Connection timestamp (ISO 8601)
  @JsonKey(name: 'connected_at')
  String get connectedAt => throw _privateConstructorUsedError;

  /// Last update timestamp (ISO 8601)
  @JsonKey(name: 'updated_at')
  String get updatedAt => throw _privateConstructorUsedError;

  /// Assigned profile ID (if any)
  @JsonKey(name: 'profile_id')
  String? get profileId => throw _privateConstructorUsedError;

  /// Serializes this DeviceState to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of DeviceState
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $DeviceStateCopyWith<DeviceState> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $DeviceStateCopyWith<$Res> {
  factory $DeviceStateCopyWith(
    DeviceState value,
    $Res Function(DeviceState) then,
  ) = _$DeviceStateCopyWithImpl<$Res, DeviceState>;
  @useResult
  $Res call({
    DeviceIdentity identity,
    @JsonKey(name: 'device_enabled') bool deviceEnabled,
    @JsonKey(name: 'remap_enabled') bool remapEnabled,
    @JsonKey(name: 'connected_at') String connectedAt,
    @JsonKey(name: 'updated_at') String updatedAt,
    @JsonKey(name: 'profile_id') String? profileId,
  });

  $DeviceIdentityCopyWith<$Res> get identity;
}

/// @nodoc
class _$DeviceStateCopyWithImpl<$Res, $Val extends DeviceState>
    implements $DeviceStateCopyWith<$Res> {
  _$DeviceStateCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of DeviceState
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? identity = null,
    Object? deviceEnabled = null,
    Object? remapEnabled = null,
    Object? connectedAt = null,
    Object? updatedAt = null,
    Object? profileId = freezed,
  }) {
    return _then(
      _value.copyWith(
            identity: null == identity
                ? _value.identity
                : identity // ignore: cast_nullable_to_non_nullable
                      as DeviceIdentity,
            deviceEnabled: null == deviceEnabled
                ? _value.deviceEnabled
                : deviceEnabled // ignore: cast_nullable_to_non_nullable
                      as bool,
            remapEnabled: null == remapEnabled
                ? _value.remapEnabled
                : remapEnabled // ignore: cast_nullable_to_non_nullable
                      as bool,
            connectedAt: null == connectedAt
                ? _value.connectedAt
                : connectedAt // ignore: cast_nullable_to_non_nullable
                      as String,
            updatedAt: null == updatedAt
                ? _value.updatedAt
                : updatedAt // ignore: cast_nullable_to_non_nullable
                      as String,
            profileId: freezed == profileId
                ? _value.profileId
                : profileId // ignore: cast_nullable_to_non_nullable
                      as String?,
          )
          as $Val,
    );
  }

  /// Create a copy of DeviceState
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $DeviceIdentityCopyWith<$Res> get identity {
    return $DeviceIdentityCopyWith<$Res>(_value.identity, (value) {
      return _then(_value.copyWith(identity: value) as $Val);
    });
  }
}

/// @nodoc
abstract class _$$DeviceStateImplCopyWith<$Res>
    implements $DeviceStateCopyWith<$Res> {
  factory _$$DeviceStateImplCopyWith(
    _$DeviceStateImpl value,
    $Res Function(_$DeviceStateImpl) then,
  ) = __$$DeviceStateImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({
    DeviceIdentity identity,
    @JsonKey(name: 'device_enabled') bool deviceEnabled,
    @JsonKey(name: 'remap_enabled') bool remapEnabled,
    @JsonKey(name: 'connected_at') String connectedAt,
    @JsonKey(name: 'updated_at') String updatedAt,
    @JsonKey(name: 'profile_id') String? profileId,
  });

  @override
  $DeviceIdentityCopyWith<$Res> get identity;
}

/// @nodoc
class __$$DeviceStateImplCopyWithImpl<$Res>
    extends _$DeviceStateCopyWithImpl<$Res, _$DeviceStateImpl>
    implements _$$DeviceStateImplCopyWith<$Res> {
  __$$DeviceStateImplCopyWithImpl(
    _$DeviceStateImpl _value,
    $Res Function(_$DeviceStateImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of DeviceState
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? identity = null,
    Object? deviceEnabled = null,
    Object? remapEnabled = null,
    Object? connectedAt = null,
    Object? updatedAt = null,
    Object? profileId = freezed,
  }) {
    return _then(
      _$DeviceStateImpl(
        identity: null == identity
            ? _value.identity
            : identity // ignore: cast_nullable_to_non_nullable
                  as DeviceIdentity,
        deviceEnabled: null == deviceEnabled
            ? _value.deviceEnabled
            : deviceEnabled // ignore: cast_nullable_to_non_nullable
                  as bool,
        remapEnabled: null == remapEnabled
            ? _value.remapEnabled
            : remapEnabled // ignore: cast_nullable_to_non_nullable
                  as bool,
        connectedAt: null == connectedAt
            ? _value.connectedAt
            : connectedAt // ignore: cast_nullable_to_non_nullable
                  as String,
        updatedAt: null == updatedAt
            ? _value.updatedAt
            : updatedAt // ignore: cast_nullable_to_non_nullable
                  as String,
        profileId: freezed == profileId
            ? _value.profileId
            : profileId // ignore: cast_nullable_to_non_nullable
                  as String?,
      ),
    );
  }
}

/// @nodoc
@JsonSerializable()
class _$DeviceStateImpl extends _DeviceState {
  const _$DeviceStateImpl({
    required this.identity,
    @JsonKey(name: 'device_enabled') required this.deviceEnabled,
    @JsonKey(name: 'remap_enabled') required this.remapEnabled,
    @JsonKey(name: 'connected_at') required this.connectedAt,
    @JsonKey(name: 'updated_at') required this.updatedAt,
    @JsonKey(name: 'profile_id') this.profileId,
  }) : super._();

  factory _$DeviceStateImpl.fromJson(Map<String, dynamic> json) =>
      _$$DeviceStateImplFromJson(json);

  /// Device identity
  @override
  final DeviceIdentity identity;

  /// Whether the device is enabled (participates in KeyRx)
  /// When false, device is in total passthrough mode
  @override
  @JsonKey(name: 'device_enabled')
  final bool deviceEnabled;

  /// Whether remapping is enabled for this device
  /// Only effective when deviceEnabled is true
  @override
  @JsonKey(name: 'remap_enabled')
  final bool remapEnabled;

  /// Connection timestamp (ISO 8601)
  @override
  @JsonKey(name: 'connected_at')
  final String connectedAt;

  /// Last update timestamp (ISO 8601)
  @override
  @JsonKey(name: 'updated_at')
  final String updatedAt;

  /// Assigned profile ID (if any)
  @override
  @JsonKey(name: 'profile_id')
  final String? profileId;

  @override
  String toString() {
    return 'DeviceState(identity: $identity, deviceEnabled: $deviceEnabled, remapEnabled: $remapEnabled, connectedAt: $connectedAt, updatedAt: $updatedAt, profileId: $profileId)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$DeviceStateImpl &&
            (identical(other.identity, identity) ||
                other.identity == identity) &&
            (identical(other.deviceEnabled, deviceEnabled) ||
                other.deviceEnabled == deviceEnabled) &&
            (identical(other.remapEnabled, remapEnabled) ||
                other.remapEnabled == remapEnabled) &&
            (identical(other.connectedAt, connectedAt) ||
                other.connectedAt == connectedAt) &&
            (identical(other.updatedAt, updatedAt) ||
                other.updatedAt == updatedAt) &&
            (identical(other.profileId, profileId) ||
                other.profileId == profileId));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(
    runtimeType,
    identity,
    deviceEnabled,
    remapEnabled,
    connectedAt,
    updatedAt,
    profileId,
  );

  /// Create a copy of DeviceState
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$DeviceStateImplCopyWith<_$DeviceStateImpl> get copyWith =>
      __$$DeviceStateImplCopyWithImpl<_$DeviceStateImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$DeviceStateImplToJson(this);
  }
}

abstract class _DeviceState extends DeviceState {
  const factory _DeviceState({
    required final DeviceIdentity identity,
    @JsonKey(name: 'device_enabled') required final bool deviceEnabled,
    @JsonKey(name: 'remap_enabled') required final bool remapEnabled,
    @JsonKey(name: 'connected_at') required final String connectedAt,
    @JsonKey(name: 'updated_at') required final String updatedAt,
    @JsonKey(name: 'profile_id') final String? profileId,
  }) = _$DeviceStateImpl;
  const _DeviceState._() : super._();

  factory _DeviceState.fromJson(Map<String, dynamic> json) =
      _$DeviceStateImpl.fromJson;

  /// Device identity
  @override
  DeviceIdentity get identity;

  /// Whether the device is enabled (participates in KeyRx)
  /// When false, device is in total passthrough mode
  @override
  @JsonKey(name: 'device_enabled')
  bool get deviceEnabled;

  /// Whether remapping is enabled for this device
  /// Only effective when deviceEnabled is true
  @override
  @JsonKey(name: 'remap_enabled')
  bool get remapEnabled;

  /// Connection timestamp (ISO 8601)
  @override
  @JsonKey(name: 'connected_at')
  String get connectedAt;

  /// Last update timestamp (ISO 8601)
  @override
  @JsonKey(name: 'updated_at')
  String get updatedAt;

  /// Assigned profile ID (if any)
  @override
  @JsonKey(name: 'profile_id')
  String? get profileId;

  /// Create a copy of DeviceState
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$DeviceStateImplCopyWith<_$DeviceStateImpl> get copyWith =>
      throw _privateConstructorUsedError;
}
