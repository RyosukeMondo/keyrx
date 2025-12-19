// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'device_config.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
  'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models',
);

DeviceConfig _$DeviceConfigFromJson(Map<String, dynamic> json) {
  return _DeviceConfig.fromJson(json);
}

/// @nodoc
mixin _$DeviceConfig {
  /// List of known devices and their configurations.
  List<DeviceIdentity> get devices => throw _privateConstructorUsedError;

  /// Serializes this DeviceConfig to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of DeviceConfig
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $DeviceConfigCopyWith<DeviceConfig> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $DeviceConfigCopyWith<$Res> {
  factory $DeviceConfigCopyWith(
    DeviceConfig value,
    $Res Function(DeviceConfig) then,
  ) = _$DeviceConfigCopyWithImpl<$Res, DeviceConfig>;
  @useResult
  $Res call({List<DeviceIdentity> devices});
}

/// @nodoc
class _$DeviceConfigCopyWithImpl<$Res, $Val extends DeviceConfig>
    implements $DeviceConfigCopyWith<$Res> {
  _$DeviceConfigCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of DeviceConfig
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? devices = null}) {
    return _then(
      _value.copyWith(
            devices: null == devices
                ? _value.devices
                : devices // ignore: cast_nullable_to_non_nullable
                      as List<DeviceIdentity>,
          )
          as $Val,
    );
  }
}

/// @nodoc
abstract class _$$DeviceConfigImplCopyWith<$Res>
    implements $DeviceConfigCopyWith<$Res> {
  factory _$$DeviceConfigImplCopyWith(
    _$DeviceConfigImpl value,
    $Res Function(_$DeviceConfigImpl) then,
  ) = __$$DeviceConfigImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({List<DeviceIdentity> devices});
}

/// @nodoc
class __$$DeviceConfigImplCopyWithImpl<$Res>
    extends _$DeviceConfigCopyWithImpl<$Res, _$DeviceConfigImpl>
    implements _$$DeviceConfigImplCopyWith<$Res> {
  __$$DeviceConfigImplCopyWithImpl(
    _$DeviceConfigImpl _value,
    $Res Function(_$DeviceConfigImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of DeviceConfig
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? devices = null}) {
    return _then(
      _$DeviceConfigImpl(
        devices: null == devices
            ? _value._devices
            : devices // ignore: cast_nullable_to_non_nullable
                  as List<DeviceIdentity>,
      ),
    );
  }
}

/// @nodoc
@JsonSerializable()
class _$DeviceConfigImpl implements _DeviceConfig {
  const _$DeviceConfigImpl({final List<DeviceIdentity> devices = const []})
    : _devices = devices;

  factory _$DeviceConfigImpl.fromJson(Map<String, dynamic> json) =>
      _$$DeviceConfigImplFromJson(json);

  /// List of known devices and their configurations.
  final List<DeviceIdentity> _devices;

  /// List of known devices and their configurations.
  @override
  @JsonKey()
  List<DeviceIdentity> get devices {
    if (_devices is EqualUnmodifiableListView) return _devices;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_devices);
  }

  @override
  String toString() {
    return 'DeviceConfig(devices: $devices)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$DeviceConfigImpl &&
            const DeepCollectionEquality().equals(other._devices, _devices));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode =>
      Object.hash(runtimeType, const DeepCollectionEquality().hash(_devices));

  /// Create a copy of DeviceConfig
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$DeviceConfigImplCopyWith<_$DeviceConfigImpl> get copyWith =>
      __$$DeviceConfigImplCopyWithImpl<_$DeviceConfigImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$DeviceConfigImplToJson(this);
  }
}

abstract class _DeviceConfig implements DeviceConfig {
  const factory _DeviceConfig({final List<DeviceIdentity> devices}) =
      _$DeviceConfigImpl;

  factory _DeviceConfig.fromJson(Map<String, dynamic> json) =
      _$DeviceConfigImpl.fromJson;

  /// List of known devices and their configurations.
  @override
  List<DeviceIdentity> get devices;

  /// Create a copy of DeviceConfig
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$DeviceConfigImplCopyWith<_$DeviceConfigImpl> get copyWith =>
      throw _privateConstructorUsedError;
}
