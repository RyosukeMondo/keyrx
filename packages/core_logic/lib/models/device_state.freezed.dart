// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'device_state.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$DeviceState {

/// Device identity
 DeviceIdentity get identity;/// Whether the device is enabled (participates in KeyRx)
/// When false, device is in total passthrough mode
@JsonKey(name: 'device_enabled') bool get deviceEnabled;/// Whether remapping is enabled for this device
/// Only effective when deviceEnabled is true
@JsonKey(name: 'remap_enabled') bool get remapEnabled;/// Connection timestamp (ISO 8601)
@JsonKey(name: 'connected_at') String get connectedAt;/// Last update timestamp (ISO 8601)
@JsonKey(name: 'updated_at') String get updatedAt;/// Assigned profile ID (if any)
@JsonKey(name: 'profile_id') String? get profileId;
/// Create a copy of DeviceState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$DeviceStateCopyWith<DeviceState> get copyWith => _$DeviceStateCopyWithImpl<DeviceState>(this as DeviceState, _$identity);

  /// Serializes this DeviceState to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DeviceState&&(identical(other.identity, identity) || other.identity == identity)&&(identical(other.deviceEnabled, deviceEnabled) || other.deviceEnabled == deviceEnabled)&&(identical(other.remapEnabled, remapEnabled) || other.remapEnabled == remapEnabled)&&(identical(other.connectedAt, connectedAt) || other.connectedAt == connectedAt)&&(identical(other.updatedAt, updatedAt) || other.updatedAt == updatedAt)&&(identical(other.profileId, profileId) || other.profileId == profileId));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,identity,deviceEnabled,remapEnabled,connectedAt,updatedAt,profileId);

@override
String toString() {
  return 'DeviceState(identity: $identity, deviceEnabled: $deviceEnabled, remapEnabled: $remapEnabled, connectedAt: $connectedAt, updatedAt: $updatedAt, profileId: $profileId)';
}


}

/// @nodoc
abstract mixin class $DeviceStateCopyWith<$Res>  {
  factory $DeviceStateCopyWith(DeviceState value, $Res Function(DeviceState) _then) = _$DeviceStateCopyWithImpl;
@useResult
$Res call({
 DeviceIdentity identity,@JsonKey(name: 'device_enabled') bool deviceEnabled,@JsonKey(name: 'remap_enabled') bool remapEnabled,@JsonKey(name: 'connected_at') String connectedAt,@JsonKey(name: 'updated_at') String updatedAt,@JsonKey(name: 'profile_id') String? profileId
});


$DeviceIdentityCopyWith<$Res> get identity;

}
/// @nodoc
class _$DeviceStateCopyWithImpl<$Res>
    implements $DeviceStateCopyWith<$Res> {
  _$DeviceStateCopyWithImpl(this._self, this._then);

  final DeviceState _self;
  final $Res Function(DeviceState) _then;

/// Create a copy of DeviceState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? identity = null,Object? deviceEnabled = null,Object? remapEnabled = null,Object? connectedAt = null,Object? updatedAt = null,Object? profileId = freezed,}) {
  return _then(_self.copyWith(
identity: null == identity ? _self.identity : identity // ignore: cast_nullable_to_non_nullable
as DeviceIdentity,deviceEnabled: null == deviceEnabled ? _self.deviceEnabled : deviceEnabled // ignore: cast_nullable_to_non_nullable
as bool,remapEnabled: null == remapEnabled ? _self.remapEnabled : remapEnabled // ignore: cast_nullable_to_non_nullable
as bool,connectedAt: null == connectedAt ? _self.connectedAt : connectedAt // ignore: cast_nullable_to_non_nullable
as String,updatedAt: null == updatedAt ? _self.updatedAt : updatedAt // ignore: cast_nullable_to_non_nullable
as String,profileId: freezed == profileId ? _self.profileId : profileId // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}
/// Create a copy of DeviceState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$DeviceIdentityCopyWith<$Res> get identity {
  
  return $DeviceIdentityCopyWith<$Res>(_self.identity, (value) {
    return _then(_self.copyWith(identity: value));
  });
}
}


/// Adds pattern-matching-related methods to [DeviceState].
extension DeviceStatePatterns on DeviceState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _DeviceState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _DeviceState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _DeviceState value)  $default,){
final _that = this;
switch (_that) {
case _DeviceState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _DeviceState value)?  $default,){
final _that = this;
switch (_that) {
case _DeviceState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( DeviceIdentity identity, @JsonKey(name: 'device_enabled')  bool deviceEnabled, @JsonKey(name: 'remap_enabled')  bool remapEnabled, @JsonKey(name: 'connected_at')  String connectedAt, @JsonKey(name: 'updated_at')  String updatedAt, @JsonKey(name: 'profile_id')  String? profileId)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _DeviceState() when $default != null:
return $default(_that.identity,_that.deviceEnabled,_that.remapEnabled,_that.connectedAt,_that.updatedAt,_that.profileId);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( DeviceIdentity identity, @JsonKey(name: 'device_enabled')  bool deviceEnabled, @JsonKey(name: 'remap_enabled')  bool remapEnabled, @JsonKey(name: 'connected_at')  String connectedAt, @JsonKey(name: 'updated_at')  String updatedAt, @JsonKey(name: 'profile_id')  String? profileId)  $default,) {final _that = this;
switch (_that) {
case _DeviceState():
return $default(_that.identity,_that.deviceEnabled,_that.remapEnabled,_that.connectedAt,_that.updatedAt,_that.profileId);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( DeviceIdentity identity, @JsonKey(name: 'device_enabled')  bool deviceEnabled, @JsonKey(name: 'remap_enabled')  bool remapEnabled, @JsonKey(name: 'connected_at')  String connectedAt, @JsonKey(name: 'updated_at')  String updatedAt, @JsonKey(name: 'profile_id')  String? profileId)?  $default,) {final _that = this;
switch (_that) {
case _DeviceState() when $default != null:
return $default(_that.identity,_that.deviceEnabled,_that.remapEnabled,_that.connectedAt,_that.updatedAt,_that.profileId);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _DeviceState extends DeviceState {
  const _DeviceState({required this.identity, @JsonKey(name: 'device_enabled') required this.deviceEnabled, @JsonKey(name: 'remap_enabled') required this.remapEnabled, @JsonKey(name: 'connected_at') required this.connectedAt, @JsonKey(name: 'updated_at') required this.updatedAt, @JsonKey(name: 'profile_id') this.profileId}): super._();
  factory _DeviceState.fromJson(Map<String, dynamic> json) => _$DeviceStateFromJson(json);

/// Device identity
@override final  DeviceIdentity identity;
/// Whether the device is enabled (participates in KeyRx)
/// When false, device is in total passthrough mode
@override@JsonKey(name: 'device_enabled') final  bool deviceEnabled;
/// Whether remapping is enabled for this device
/// Only effective when deviceEnabled is true
@override@JsonKey(name: 'remap_enabled') final  bool remapEnabled;
/// Connection timestamp (ISO 8601)
@override@JsonKey(name: 'connected_at') final  String connectedAt;
/// Last update timestamp (ISO 8601)
@override@JsonKey(name: 'updated_at') final  String updatedAt;
/// Assigned profile ID (if any)
@override@JsonKey(name: 'profile_id') final  String? profileId;

/// Create a copy of DeviceState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$DeviceStateCopyWith<_DeviceState> get copyWith => __$DeviceStateCopyWithImpl<_DeviceState>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$DeviceStateToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _DeviceState&&(identical(other.identity, identity) || other.identity == identity)&&(identical(other.deviceEnabled, deviceEnabled) || other.deviceEnabled == deviceEnabled)&&(identical(other.remapEnabled, remapEnabled) || other.remapEnabled == remapEnabled)&&(identical(other.connectedAt, connectedAt) || other.connectedAt == connectedAt)&&(identical(other.updatedAt, updatedAt) || other.updatedAt == updatedAt)&&(identical(other.profileId, profileId) || other.profileId == profileId));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,identity,deviceEnabled,remapEnabled,connectedAt,updatedAt,profileId);

@override
String toString() {
  return 'DeviceState(identity: $identity, deviceEnabled: $deviceEnabled, remapEnabled: $remapEnabled, connectedAt: $connectedAt, updatedAt: $updatedAt, profileId: $profileId)';
}


}

/// @nodoc
abstract mixin class _$DeviceStateCopyWith<$Res> implements $DeviceStateCopyWith<$Res> {
  factory _$DeviceStateCopyWith(_DeviceState value, $Res Function(_DeviceState) _then) = __$DeviceStateCopyWithImpl;
@override @useResult
$Res call({
 DeviceIdentity identity,@JsonKey(name: 'device_enabled') bool deviceEnabled,@JsonKey(name: 'remap_enabled') bool remapEnabled,@JsonKey(name: 'connected_at') String connectedAt,@JsonKey(name: 'updated_at') String updatedAt,@JsonKey(name: 'profile_id') String? profileId
});


@override $DeviceIdentityCopyWith<$Res> get identity;

}
/// @nodoc
class __$DeviceStateCopyWithImpl<$Res>
    implements _$DeviceStateCopyWith<$Res> {
  __$DeviceStateCopyWithImpl(this._self, this._then);

  final _DeviceState _self;
  final $Res Function(_DeviceState) _then;

/// Create a copy of DeviceState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? identity = null,Object? deviceEnabled = null,Object? remapEnabled = null,Object? connectedAt = null,Object? updatedAt = null,Object? profileId = freezed,}) {
  return _then(_DeviceState(
identity: null == identity ? _self.identity : identity // ignore: cast_nullable_to_non_nullable
as DeviceIdentity,deviceEnabled: null == deviceEnabled ? _self.deviceEnabled : deviceEnabled // ignore: cast_nullable_to_non_nullable
as bool,remapEnabled: null == remapEnabled ? _self.remapEnabled : remapEnabled // ignore: cast_nullable_to_non_nullable
as bool,connectedAt: null == connectedAt ? _self.connectedAt : connectedAt // ignore: cast_nullable_to_non_nullable
as String,updatedAt: null == updatedAt ? _self.updatedAt : updatedAt // ignore: cast_nullable_to_non_nullable
as String,profileId: freezed == profileId ? _self.profileId : profileId // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}

/// Create a copy of DeviceState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$DeviceIdentityCopyWith<$Res> get identity {
  
  return $DeviceIdentityCopyWith<$Res>(_self.identity, (value) {
    return _then(_self.copyWith(identity: value));
  });
}
}

// dart format on
