// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'runtime_config.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
  'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models',
);

ProfileSlot _$ProfileSlotFromJson(Map<String, dynamic> json) {
  return _ProfileSlot.fromJson(json);
}

/// @nodoc
mixin _$ProfileSlot {
  String get id => throw _privateConstructorUsedError;
  @JsonKey(name: 'hardware_profile_id')
  String get hardwareProfileId => throw _privateConstructorUsedError;
  @JsonKey(name: 'keymap_id')
  String get keymapId => throw _privateConstructorUsedError;
  bool get active => throw _privateConstructorUsedError;
  int get priority => throw _privateConstructorUsedError;

  /// Serializes this ProfileSlot to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of ProfileSlot
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $ProfileSlotCopyWith<ProfileSlot> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $ProfileSlotCopyWith<$Res> {
  factory $ProfileSlotCopyWith(
    ProfileSlot value,
    $Res Function(ProfileSlot) then,
  ) = _$ProfileSlotCopyWithImpl<$Res, ProfileSlot>;
  @useResult
  $Res call({
    String id,
    @JsonKey(name: 'hardware_profile_id') String hardwareProfileId,
    @JsonKey(name: 'keymap_id') String keymapId,
    bool active,
    int priority,
  });
}

/// @nodoc
class _$ProfileSlotCopyWithImpl<$Res, $Val extends ProfileSlot>
    implements $ProfileSlotCopyWith<$Res> {
  _$ProfileSlotCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of ProfileSlot
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? hardwareProfileId = null,
    Object? keymapId = null,
    Object? active = null,
    Object? priority = null,
  }) {
    return _then(
      _value.copyWith(
            id: null == id
                ? _value.id
                : id // ignore: cast_nullable_to_non_nullable
                      as String,
            hardwareProfileId: null == hardwareProfileId
                ? _value.hardwareProfileId
                : hardwareProfileId // ignore: cast_nullable_to_non_nullable
                      as String,
            keymapId: null == keymapId
                ? _value.keymapId
                : keymapId // ignore: cast_nullable_to_non_nullable
                      as String,
            active: null == active
                ? _value.active
                : active // ignore: cast_nullable_to_non_nullable
                      as bool,
            priority: null == priority
                ? _value.priority
                : priority // ignore: cast_nullable_to_non_nullable
                      as int,
          )
          as $Val,
    );
  }
}

/// @nodoc
abstract class _$$ProfileSlotImplCopyWith<$Res>
    implements $ProfileSlotCopyWith<$Res> {
  factory _$$ProfileSlotImplCopyWith(
    _$ProfileSlotImpl value,
    $Res Function(_$ProfileSlotImpl) then,
  ) = __$$ProfileSlotImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({
    String id,
    @JsonKey(name: 'hardware_profile_id') String hardwareProfileId,
    @JsonKey(name: 'keymap_id') String keymapId,
    bool active,
    int priority,
  });
}

/// @nodoc
class __$$ProfileSlotImplCopyWithImpl<$Res>
    extends _$ProfileSlotCopyWithImpl<$Res, _$ProfileSlotImpl>
    implements _$$ProfileSlotImplCopyWith<$Res> {
  __$$ProfileSlotImplCopyWithImpl(
    _$ProfileSlotImpl _value,
    $Res Function(_$ProfileSlotImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of ProfileSlot
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? hardwareProfileId = null,
    Object? keymapId = null,
    Object? active = null,
    Object? priority = null,
  }) {
    return _then(
      _$ProfileSlotImpl(
        id: null == id
            ? _value.id
            : id // ignore: cast_nullable_to_non_nullable
                  as String,
        hardwareProfileId: null == hardwareProfileId
            ? _value.hardwareProfileId
            : hardwareProfileId // ignore: cast_nullable_to_non_nullable
                  as String,
        keymapId: null == keymapId
            ? _value.keymapId
            : keymapId // ignore: cast_nullable_to_non_nullable
                  as String,
        active: null == active
            ? _value.active
            : active // ignore: cast_nullable_to_non_nullable
                  as bool,
        priority: null == priority
            ? _value.priority
            : priority // ignore: cast_nullable_to_non_nullable
                  as int,
      ),
    );
  }
}

/// @nodoc
@JsonSerializable()
class _$ProfileSlotImpl extends _ProfileSlot {
  const _$ProfileSlotImpl({
    required this.id,
    @JsonKey(name: 'hardware_profile_id') required this.hardwareProfileId,
    @JsonKey(name: 'keymap_id') required this.keymapId,
    this.active = false,
    this.priority = 0,
  }) : super._();

  factory _$ProfileSlotImpl.fromJson(Map<String, dynamic> json) =>
      _$$ProfileSlotImplFromJson(json);

  @override
  final String id;
  @override
  @JsonKey(name: 'hardware_profile_id')
  final String hardwareProfileId;
  @override
  @JsonKey(name: 'keymap_id')
  final String keymapId;
  @override
  @JsonKey()
  final bool active;
  @override
  @JsonKey()
  final int priority;

  @override
  String toString() {
    return 'ProfileSlot(id: $id, hardwareProfileId: $hardwareProfileId, keymapId: $keymapId, active: $active, priority: $priority)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ProfileSlotImpl &&
            (identical(other.id, id) || other.id == id) &&
            (identical(other.hardwareProfileId, hardwareProfileId) ||
                other.hardwareProfileId == hardwareProfileId) &&
            (identical(other.keymapId, keymapId) ||
                other.keymapId == keymapId) &&
            (identical(other.active, active) || other.active == active) &&
            (identical(other.priority, priority) ||
                other.priority == priority));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(
    runtimeType,
    id,
    hardwareProfileId,
    keymapId,
    active,
    priority,
  );

  /// Create a copy of ProfileSlot
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$ProfileSlotImplCopyWith<_$ProfileSlotImpl> get copyWith =>
      __$$ProfileSlotImplCopyWithImpl<_$ProfileSlotImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$ProfileSlotImplToJson(this);
  }
}

abstract class _ProfileSlot extends ProfileSlot {
  const factory _ProfileSlot({
    required final String id,
    @JsonKey(name: 'hardware_profile_id')
    required final String hardwareProfileId,
    @JsonKey(name: 'keymap_id') required final String keymapId,
    final bool active,
    final int priority,
  }) = _$ProfileSlotImpl;
  const _ProfileSlot._() : super._();

  factory _ProfileSlot.fromJson(Map<String, dynamic> json) =
      _$ProfileSlotImpl.fromJson;

  @override
  String get id;
  @override
  @JsonKey(name: 'hardware_profile_id')
  String get hardwareProfileId;
  @override
  @JsonKey(name: 'keymap_id')
  String get keymapId;
  @override
  bool get active;
  @override
  int get priority;

  /// Create a copy of ProfileSlot
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ProfileSlotImplCopyWith<_$ProfileSlotImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

DeviceSlots _$DeviceSlotsFromJson(Map<String, dynamic> json) {
  return _DeviceSlots.fromJson(json);
}

/// @nodoc
mixin _$DeviceSlots {
  DeviceInstanceId get device => throw _privateConstructorUsedError;
  List<ProfileSlot> get slots => throw _privateConstructorUsedError;

  /// Serializes this DeviceSlots to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of DeviceSlots
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $DeviceSlotsCopyWith<DeviceSlots> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $DeviceSlotsCopyWith<$Res> {
  factory $DeviceSlotsCopyWith(
    DeviceSlots value,
    $Res Function(DeviceSlots) then,
  ) = _$DeviceSlotsCopyWithImpl<$Res, DeviceSlots>;
  @useResult
  $Res call({DeviceInstanceId device, List<ProfileSlot> slots});

  $DeviceInstanceIdCopyWith<$Res> get device;
}

/// @nodoc
class _$DeviceSlotsCopyWithImpl<$Res, $Val extends DeviceSlots>
    implements $DeviceSlotsCopyWith<$Res> {
  _$DeviceSlotsCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of DeviceSlots
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? device = null, Object? slots = null}) {
    return _then(
      _value.copyWith(
            device: null == device
                ? _value.device
                : device // ignore: cast_nullable_to_non_nullable
                      as DeviceInstanceId,
            slots: null == slots
                ? _value.slots
                : slots // ignore: cast_nullable_to_non_nullable
                      as List<ProfileSlot>,
          )
          as $Val,
    );
  }

  /// Create a copy of DeviceSlots
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $DeviceInstanceIdCopyWith<$Res> get device {
    return $DeviceInstanceIdCopyWith<$Res>(_value.device, (value) {
      return _then(_value.copyWith(device: value) as $Val);
    });
  }
}

/// @nodoc
abstract class _$$DeviceSlotsImplCopyWith<$Res>
    implements $DeviceSlotsCopyWith<$Res> {
  factory _$$DeviceSlotsImplCopyWith(
    _$DeviceSlotsImpl value,
    $Res Function(_$DeviceSlotsImpl) then,
  ) = __$$DeviceSlotsImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({DeviceInstanceId device, List<ProfileSlot> slots});

  @override
  $DeviceInstanceIdCopyWith<$Res> get device;
}

/// @nodoc
class __$$DeviceSlotsImplCopyWithImpl<$Res>
    extends _$DeviceSlotsCopyWithImpl<$Res, _$DeviceSlotsImpl>
    implements _$$DeviceSlotsImplCopyWith<$Res> {
  __$$DeviceSlotsImplCopyWithImpl(
    _$DeviceSlotsImpl _value,
    $Res Function(_$DeviceSlotsImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of DeviceSlots
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? device = null, Object? slots = null}) {
    return _then(
      _$DeviceSlotsImpl(
        device: null == device
            ? _value.device
            : device // ignore: cast_nullable_to_non_nullable
                  as DeviceInstanceId,
        slots: null == slots
            ? _value._slots
            : slots // ignore: cast_nullable_to_non_nullable
                  as List<ProfileSlot>,
      ),
    );
  }
}

/// @nodoc
@JsonSerializable()
class _$DeviceSlotsImpl extends _DeviceSlots {
  const _$DeviceSlotsImpl({
    required this.device,
    final List<ProfileSlot> slots = const <ProfileSlot>[],
  }) : _slots = slots,
       super._();

  factory _$DeviceSlotsImpl.fromJson(Map<String, dynamic> json) =>
      _$$DeviceSlotsImplFromJson(json);

  @override
  final DeviceInstanceId device;
  final List<ProfileSlot> _slots;
  @override
  @JsonKey()
  List<ProfileSlot> get slots {
    if (_slots is EqualUnmodifiableListView) return _slots;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_slots);
  }

  @override
  String toString() {
    return 'DeviceSlots(device: $device, slots: $slots)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$DeviceSlotsImpl &&
            (identical(other.device, device) || other.device == device) &&
            const DeepCollectionEquality().equals(other._slots, _slots));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(
    runtimeType,
    device,
    const DeepCollectionEquality().hash(_slots),
  );

  /// Create a copy of DeviceSlots
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$DeviceSlotsImplCopyWith<_$DeviceSlotsImpl> get copyWith =>
      __$$DeviceSlotsImplCopyWithImpl<_$DeviceSlotsImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$DeviceSlotsImplToJson(this);
  }
}

abstract class _DeviceSlots extends DeviceSlots {
  const factory _DeviceSlots({
    required final DeviceInstanceId device,
    final List<ProfileSlot> slots,
  }) = _$DeviceSlotsImpl;
  const _DeviceSlots._() : super._();

  factory _DeviceSlots.fromJson(Map<String, dynamic> json) =
      _$DeviceSlotsImpl.fromJson;

  @override
  DeviceInstanceId get device;
  @override
  List<ProfileSlot> get slots;

  /// Create a copy of DeviceSlots
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$DeviceSlotsImplCopyWith<_$DeviceSlotsImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

RuntimeConfig _$RuntimeConfigFromJson(Map<String, dynamic> json) {
  return _RuntimeConfig.fromJson(json);
}

/// @nodoc
mixin _$RuntimeConfig {
  List<DeviceSlots> get devices => throw _privateConstructorUsedError;

  /// Serializes this RuntimeConfig to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of RuntimeConfig
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $RuntimeConfigCopyWith<RuntimeConfig> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $RuntimeConfigCopyWith<$Res> {
  factory $RuntimeConfigCopyWith(
    RuntimeConfig value,
    $Res Function(RuntimeConfig) then,
  ) = _$RuntimeConfigCopyWithImpl<$Res, RuntimeConfig>;
  @useResult
  $Res call({List<DeviceSlots> devices});
}

/// @nodoc
class _$RuntimeConfigCopyWithImpl<$Res, $Val extends RuntimeConfig>
    implements $RuntimeConfigCopyWith<$Res> {
  _$RuntimeConfigCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of RuntimeConfig
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? devices = null}) {
    return _then(
      _value.copyWith(
            devices: null == devices
                ? _value.devices
                : devices // ignore: cast_nullable_to_non_nullable
                      as List<DeviceSlots>,
          )
          as $Val,
    );
  }
}

/// @nodoc
abstract class _$$RuntimeConfigImplCopyWith<$Res>
    implements $RuntimeConfigCopyWith<$Res> {
  factory _$$RuntimeConfigImplCopyWith(
    _$RuntimeConfigImpl value,
    $Res Function(_$RuntimeConfigImpl) then,
  ) = __$$RuntimeConfigImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({List<DeviceSlots> devices});
}

/// @nodoc
class __$$RuntimeConfigImplCopyWithImpl<$Res>
    extends _$RuntimeConfigCopyWithImpl<$Res, _$RuntimeConfigImpl>
    implements _$$RuntimeConfigImplCopyWith<$Res> {
  __$$RuntimeConfigImplCopyWithImpl(
    _$RuntimeConfigImpl _value,
    $Res Function(_$RuntimeConfigImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of RuntimeConfig
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? devices = null}) {
    return _then(
      _$RuntimeConfigImpl(
        devices: null == devices
            ? _value._devices
            : devices // ignore: cast_nullable_to_non_nullable
                  as List<DeviceSlots>,
      ),
    );
  }
}

/// @nodoc
@JsonSerializable()
class _$RuntimeConfigImpl extends _RuntimeConfig {
  const _$RuntimeConfigImpl({
    final List<DeviceSlots> devices = const <DeviceSlots>[],
  }) : _devices = devices,
       super._();

  factory _$RuntimeConfigImpl.fromJson(Map<String, dynamic> json) =>
      _$$RuntimeConfigImplFromJson(json);

  final List<DeviceSlots> _devices;
  @override
  @JsonKey()
  List<DeviceSlots> get devices {
    if (_devices is EqualUnmodifiableListView) return _devices;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_devices);
  }

  @override
  String toString() {
    return 'RuntimeConfig(devices: $devices)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$RuntimeConfigImpl &&
            const DeepCollectionEquality().equals(other._devices, _devices));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode =>
      Object.hash(runtimeType, const DeepCollectionEquality().hash(_devices));

  /// Create a copy of RuntimeConfig
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$RuntimeConfigImplCopyWith<_$RuntimeConfigImpl> get copyWith =>
      __$$RuntimeConfigImplCopyWithImpl<_$RuntimeConfigImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$RuntimeConfigImplToJson(this);
  }
}

abstract class _RuntimeConfig extends RuntimeConfig {
  const factory _RuntimeConfig({final List<DeviceSlots> devices}) =
      _$RuntimeConfigImpl;
  const _RuntimeConfig._() : super._();

  factory _RuntimeConfig.fromJson(Map<String, dynamic> json) =
      _$RuntimeConfigImpl.fromJson;

  @override
  List<DeviceSlots> get devices;

  /// Create a copy of RuntimeConfig
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$RuntimeConfigImplCopyWith<_$RuntimeConfigImpl> get copyWith =>
      throw _privateConstructorUsedError;
}
