// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'runtime_config.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$ProfileSlot {

 String get id;@JsonKey(name: 'hardware_profile_id') HardwareProfileId get hardwareProfileId;@JsonKey(name: 'keymap_id') KeymapId get keymapId; bool get active; int get priority;
/// Create a copy of ProfileSlot
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ProfileSlotCopyWith<ProfileSlot> get copyWith => _$ProfileSlotCopyWithImpl<ProfileSlot>(this as ProfileSlot, _$identity);

  /// Serializes this ProfileSlot to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ProfileSlot&&(identical(other.id, id) || other.id == id)&&(identical(other.hardwareProfileId, hardwareProfileId) || other.hardwareProfileId == hardwareProfileId)&&(identical(other.keymapId, keymapId) || other.keymapId == keymapId)&&(identical(other.active, active) || other.active == active)&&(identical(other.priority, priority) || other.priority == priority));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,id,hardwareProfileId,keymapId,active,priority);

@override
String toString() {
  return 'ProfileSlot(id: $id, hardwareProfileId: $hardwareProfileId, keymapId: $keymapId, active: $active, priority: $priority)';
}


}

/// @nodoc
abstract mixin class $ProfileSlotCopyWith<$Res>  {
  factory $ProfileSlotCopyWith(ProfileSlot value, $Res Function(ProfileSlot) _then) = _$ProfileSlotCopyWithImpl;
@useResult
$Res call({
 String id,@JsonKey(name: 'hardware_profile_id') HardwareProfileId hardwareProfileId,@JsonKey(name: 'keymap_id') KeymapId keymapId, bool active, int priority
});




}
/// @nodoc
class _$ProfileSlotCopyWithImpl<$Res>
    implements $ProfileSlotCopyWith<$Res> {
  _$ProfileSlotCopyWithImpl(this._self, this._then);

  final ProfileSlot _self;
  final $Res Function(ProfileSlot) _then;

/// Create a copy of ProfileSlot
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? id = null,Object? hardwareProfileId = null,Object? keymapId = null,Object? active = null,Object? priority = null,}) {
  return _then(_self.copyWith(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,hardwareProfileId: null == hardwareProfileId ? _self.hardwareProfileId : hardwareProfileId // ignore: cast_nullable_to_non_nullable
as HardwareProfileId,keymapId: null == keymapId ? _self.keymapId : keymapId // ignore: cast_nullable_to_non_nullable
as KeymapId,active: null == active ? _self.active : active // ignore: cast_nullable_to_non_nullable
as bool,priority: null == priority ? _self.priority : priority // ignore: cast_nullable_to_non_nullable
as int,
  ));
}

}


/// Adds pattern-matching-related methods to [ProfileSlot].
extension ProfileSlotPatterns on ProfileSlot {
/// A variant of `map` that fallback to returning `orElse`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _ProfileSlot value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _ProfileSlot() when $default != null:
return $default(_that);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// Callbacks receives the raw object, upcasted.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case final Subclass2 value:
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _ProfileSlot value)  $default,){
final _that = this;
switch (_that) {
case _ProfileSlot():
return $default(_that);case _:
  throw StateError('Unexpected subclass');

}
}
/// A variant of `map` that fallback to returning `null`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _ProfileSlot value)?  $default,){
final _that = this;
switch (_that) {
case _ProfileSlot() when $default != null:
return $default(_that);case _:
  return null;

}
}
/// A variant of `when` that fallback to an `orElse` callback.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String id, @JsonKey(name: 'hardware_profile_id')  HardwareProfileId hardwareProfileId, @JsonKey(name: 'keymap_id')  KeymapId keymapId,  bool active,  int priority)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _ProfileSlot() when $default != null:
return $default(_that.id,_that.hardwareProfileId,_that.keymapId,_that.active,_that.priority);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// As opposed to `map`, this offers destructuring.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case Subclass2(:final field2):
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String id, @JsonKey(name: 'hardware_profile_id')  HardwareProfileId hardwareProfileId, @JsonKey(name: 'keymap_id')  KeymapId keymapId,  bool active,  int priority)  $default,) {final _that = this;
switch (_that) {
case _ProfileSlot():
return $default(_that.id,_that.hardwareProfileId,_that.keymapId,_that.active,_that.priority);case _:
  throw StateError('Unexpected subclass');

}
}
/// A variant of `when` that fallback to returning `null`
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String id, @JsonKey(name: 'hardware_profile_id')  HardwareProfileId hardwareProfileId, @JsonKey(name: 'keymap_id')  KeymapId keymapId,  bool active,  int priority)?  $default,) {final _that = this;
switch (_that) {
case _ProfileSlot() when $default != null:
return $default(_that.id,_that.hardwareProfileId,_that.keymapId,_that.active,_that.priority);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _ProfileSlot extends ProfileSlot {
  const _ProfileSlot({required this.id, @JsonKey(name: 'hardware_profile_id') required this.hardwareProfileId, @JsonKey(name: 'keymap_id') required this.keymapId, this.active = false, this.priority = 0}): super._();
  factory _ProfileSlot.fromJson(Map<String, dynamic> json) => _$ProfileSlotFromJson(json);

@override final  String id;
@override@JsonKey(name: 'hardware_profile_id') final  HardwareProfileId hardwareProfileId;
@override@JsonKey(name: 'keymap_id') final  KeymapId keymapId;
@override@JsonKey() final  bool active;
@override@JsonKey() final  int priority;

/// Create a copy of ProfileSlot
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$ProfileSlotCopyWith<_ProfileSlot> get copyWith => __$ProfileSlotCopyWithImpl<_ProfileSlot>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$ProfileSlotToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _ProfileSlot&&(identical(other.id, id) || other.id == id)&&(identical(other.hardwareProfileId, hardwareProfileId) || other.hardwareProfileId == hardwareProfileId)&&(identical(other.keymapId, keymapId) || other.keymapId == keymapId)&&(identical(other.active, active) || other.active == active)&&(identical(other.priority, priority) || other.priority == priority));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,id,hardwareProfileId,keymapId,active,priority);

@override
String toString() {
  return 'ProfileSlot(id: $id, hardwareProfileId: $hardwareProfileId, keymapId: $keymapId, active: $active, priority: $priority)';
}


}

/// @nodoc
abstract mixin class _$ProfileSlotCopyWith<$Res> implements $ProfileSlotCopyWith<$Res> {
  factory _$ProfileSlotCopyWith(_ProfileSlot value, $Res Function(_ProfileSlot) _then) = __$ProfileSlotCopyWithImpl;
@override @useResult
$Res call({
 String id,@JsonKey(name: 'hardware_profile_id') HardwareProfileId hardwareProfileId,@JsonKey(name: 'keymap_id') KeymapId keymapId, bool active, int priority
});




}
/// @nodoc
class __$ProfileSlotCopyWithImpl<$Res>
    implements _$ProfileSlotCopyWith<$Res> {
  __$ProfileSlotCopyWithImpl(this._self, this._then);

  final _ProfileSlot _self;
  final $Res Function(_ProfileSlot) _then;

/// Create a copy of ProfileSlot
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? id = null,Object? hardwareProfileId = null,Object? keymapId = null,Object? active = null,Object? priority = null,}) {
  return _then(_ProfileSlot(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,hardwareProfileId: null == hardwareProfileId ? _self.hardwareProfileId : hardwareProfileId // ignore: cast_nullable_to_non_nullable
as HardwareProfileId,keymapId: null == keymapId ? _self.keymapId : keymapId // ignore: cast_nullable_to_non_nullable
as KeymapId,active: null == active ? _self.active : active // ignore: cast_nullable_to_non_nullable
as bool,priority: null == priority ? _self.priority : priority // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}


/// @nodoc
mixin _$DeviceSlots {

 DeviceInstanceId get device; List<ProfileSlot> get slots;
/// Create a copy of DeviceSlots
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$DeviceSlotsCopyWith<DeviceSlots> get copyWith => _$DeviceSlotsCopyWithImpl<DeviceSlots>(this as DeviceSlots, _$identity);

  /// Serializes this DeviceSlots to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DeviceSlots&&(identical(other.device, device) || other.device == device)&&const DeepCollectionEquality().equals(other.slots, slots));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,device,const DeepCollectionEquality().hash(slots));

@override
String toString() {
  return 'DeviceSlots(device: $device, slots: $slots)';
}


}

/// @nodoc
abstract mixin class $DeviceSlotsCopyWith<$Res>  {
  factory $DeviceSlotsCopyWith(DeviceSlots value, $Res Function(DeviceSlots) _then) = _$DeviceSlotsCopyWithImpl;
@useResult
$Res call({
 DeviceInstanceId device, List<ProfileSlot> slots
});


$DeviceInstanceIdCopyWith<$Res> get device;

}
/// @nodoc
class _$DeviceSlotsCopyWithImpl<$Res>
    implements $DeviceSlotsCopyWith<$Res> {
  _$DeviceSlotsCopyWithImpl(this._self, this._then);

  final DeviceSlots _self;
  final $Res Function(DeviceSlots) _then;

/// Create a copy of DeviceSlots
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? device = null,Object? slots = null,}) {
  return _then(_self.copyWith(
device: null == device ? _self.device : device // ignore: cast_nullable_to_non_nullable
as DeviceInstanceId,slots: null == slots ? _self.slots : slots // ignore: cast_nullable_to_non_nullable
as List<ProfileSlot>,
  ));
}
/// Create a copy of DeviceSlots
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$DeviceInstanceIdCopyWith<$Res> get device {
  
  return $DeviceInstanceIdCopyWith<$Res>(_self.device, (value) {
    return _then(_self.copyWith(device: value));
  });
}
}


/// Adds pattern-matching-related methods to [DeviceSlots].
extension DeviceSlotsPatterns on DeviceSlots {
/// A variant of `map` that fallback to returning `orElse`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _DeviceSlots value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _DeviceSlots() when $default != null:
return $default(_that);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// Callbacks receives the raw object, upcasted.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case final Subclass2 value:
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _DeviceSlots value)  $default,){
final _that = this;
switch (_that) {
case _DeviceSlots():
return $default(_that);case _:
  throw StateError('Unexpected subclass');

}
}
/// A variant of `map` that fallback to returning `null`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _DeviceSlots value)?  $default,){
final _that = this;
switch (_that) {
case _DeviceSlots() when $default != null:
return $default(_that);case _:
  return null;

}
}
/// A variant of `when` that fallback to an `orElse` callback.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( DeviceInstanceId device,  List<ProfileSlot> slots)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _DeviceSlots() when $default != null:
return $default(_that.device,_that.slots);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// As opposed to `map`, this offers destructuring.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case Subclass2(:final field2):
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( DeviceInstanceId device,  List<ProfileSlot> slots)  $default,) {final _that = this;
switch (_that) {
case _DeviceSlots():
return $default(_that.device,_that.slots);case _:
  throw StateError('Unexpected subclass');

}
}
/// A variant of `when` that fallback to returning `null`
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( DeviceInstanceId device,  List<ProfileSlot> slots)?  $default,) {final _that = this;
switch (_that) {
case _DeviceSlots() when $default != null:
return $default(_that.device,_that.slots);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _DeviceSlots extends DeviceSlots {
  const _DeviceSlots({required this.device, final  List<ProfileSlot> slots = const <ProfileSlot>[]}): _slots = slots,super._();
  factory _DeviceSlots.fromJson(Map<String, dynamic> json) => _$DeviceSlotsFromJson(json);

@override final  DeviceInstanceId device;
 final  List<ProfileSlot> _slots;
@override@JsonKey() List<ProfileSlot> get slots {
  if (_slots is EqualUnmodifiableListView) return _slots;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_slots);
}


/// Create a copy of DeviceSlots
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$DeviceSlotsCopyWith<_DeviceSlots> get copyWith => __$DeviceSlotsCopyWithImpl<_DeviceSlots>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$DeviceSlotsToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _DeviceSlots&&(identical(other.device, device) || other.device == device)&&const DeepCollectionEquality().equals(other._slots, _slots));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,device,const DeepCollectionEquality().hash(_slots));

@override
String toString() {
  return 'DeviceSlots(device: $device, slots: $slots)';
}


}

/// @nodoc
abstract mixin class _$DeviceSlotsCopyWith<$Res> implements $DeviceSlotsCopyWith<$Res> {
  factory _$DeviceSlotsCopyWith(_DeviceSlots value, $Res Function(_DeviceSlots) _then) = __$DeviceSlotsCopyWithImpl;
@override @useResult
$Res call({
 DeviceInstanceId device, List<ProfileSlot> slots
});


@override $DeviceInstanceIdCopyWith<$Res> get device;

}
/// @nodoc
class __$DeviceSlotsCopyWithImpl<$Res>
    implements _$DeviceSlotsCopyWith<$Res> {
  __$DeviceSlotsCopyWithImpl(this._self, this._then);

  final _DeviceSlots _self;
  final $Res Function(_DeviceSlots) _then;

/// Create a copy of DeviceSlots
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? device = null,Object? slots = null,}) {
  return _then(_DeviceSlots(
device: null == device ? _self.device : device // ignore: cast_nullable_to_non_nullable
as DeviceInstanceId,slots: null == slots ? _self._slots : slots // ignore: cast_nullable_to_non_nullable
as List<ProfileSlot>,
  ));
}

/// Create a copy of DeviceSlots
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$DeviceInstanceIdCopyWith<$Res> get device {
  
  return $DeviceInstanceIdCopyWith<$Res>(_self.device, (value) {
    return _then(_self.copyWith(device: value));
  });
}
}


/// @nodoc
mixin _$RuntimeConfig {

 List<DeviceSlots> get devices;
/// Create a copy of RuntimeConfig
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$RuntimeConfigCopyWith<RuntimeConfig> get copyWith => _$RuntimeConfigCopyWithImpl<RuntimeConfig>(this as RuntimeConfig, _$identity);

  /// Serializes this RuntimeConfig to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is RuntimeConfig&&const DeepCollectionEquality().equals(other.devices, devices));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(devices));

@override
String toString() {
  return 'RuntimeConfig(devices: $devices)';
}


}

/// @nodoc
abstract mixin class $RuntimeConfigCopyWith<$Res>  {
  factory $RuntimeConfigCopyWith(RuntimeConfig value, $Res Function(RuntimeConfig) _then) = _$RuntimeConfigCopyWithImpl;
@useResult
$Res call({
 List<DeviceSlots> devices
});




}
/// @nodoc
class _$RuntimeConfigCopyWithImpl<$Res>
    implements $RuntimeConfigCopyWith<$Res> {
  _$RuntimeConfigCopyWithImpl(this._self, this._then);

  final RuntimeConfig _self;
  final $Res Function(RuntimeConfig) _then;

/// Create a copy of RuntimeConfig
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? devices = null,}) {
  return _then(_self.copyWith(
devices: null == devices ? _self.devices : devices // ignore: cast_nullable_to_non_nullable
as List<DeviceSlots>,
  ));
}

}


/// Adds pattern-matching-related methods to [RuntimeConfig].
extension RuntimeConfigPatterns on RuntimeConfig {
/// A variant of `map` that fallback to returning `orElse`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _RuntimeConfig value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _RuntimeConfig() when $default != null:
return $default(_that);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// Callbacks receives the raw object, upcasted.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case final Subclass2 value:
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _RuntimeConfig value)  $default,){
final _that = this;
switch (_that) {
case _RuntimeConfig():
return $default(_that);case _:
  throw StateError('Unexpected subclass');

}
}
/// A variant of `map` that fallback to returning `null`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _RuntimeConfig value)?  $default,){
final _that = this;
switch (_that) {
case _RuntimeConfig() when $default != null:
return $default(_that);case _:
  return null;

}
}
/// A variant of `when` that fallback to an `orElse` callback.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( List<DeviceSlots> devices)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _RuntimeConfig() when $default != null:
return $default(_that.devices);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// As opposed to `map`, this offers destructuring.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case Subclass2(:final field2):
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( List<DeviceSlots> devices)  $default,) {final _that = this;
switch (_that) {
case _RuntimeConfig():
return $default(_that.devices);case _:
  throw StateError('Unexpected subclass');

}
}
/// A variant of `when` that fallback to returning `null`
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( List<DeviceSlots> devices)?  $default,) {final _that = this;
switch (_that) {
case _RuntimeConfig() when $default != null:
return $default(_that.devices);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _RuntimeConfig extends RuntimeConfig {
  const _RuntimeConfig({final  List<DeviceSlots> devices = const <DeviceSlots>[]}): _devices = devices,super._();
  factory _RuntimeConfig.fromJson(Map<String, dynamic> json) => _$RuntimeConfigFromJson(json);

 final  List<DeviceSlots> _devices;
@override@JsonKey() List<DeviceSlots> get devices {
  if (_devices is EqualUnmodifiableListView) return _devices;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_devices);
}


/// Create a copy of RuntimeConfig
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$RuntimeConfigCopyWith<_RuntimeConfig> get copyWith => __$RuntimeConfigCopyWithImpl<_RuntimeConfig>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$RuntimeConfigToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _RuntimeConfig&&const DeepCollectionEquality().equals(other._devices, _devices));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_devices));

@override
String toString() {
  return 'RuntimeConfig(devices: $devices)';
}


}

/// @nodoc
abstract mixin class _$RuntimeConfigCopyWith<$Res> implements $RuntimeConfigCopyWith<$Res> {
  factory _$RuntimeConfigCopyWith(_RuntimeConfig value, $Res Function(_RuntimeConfig) _then) = __$RuntimeConfigCopyWithImpl;
@override @useResult
$Res call({
 List<DeviceSlots> devices
});




}
/// @nodoc
class __$RuntimeConfigCopyWithImpl<$Res>
    implements _$RuntimeConfigCopyWith<$Res> {
  __$RuntimeConfigCopyWithImpl(this._self, this._then);

  final _RuntimeConfig _self;
  final $Res Function(_RuntimeConfig) _then;

/// Create a copy of RuntimeConfig
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? devices = null,}) {
  return _then(_RuntimeConfig(
devices: null == devices ? _self._devices : devices // ignore: cast_nullable_to_non_nullable
as List<DeviceSlots>,
  ));
}


}

// dart format on
