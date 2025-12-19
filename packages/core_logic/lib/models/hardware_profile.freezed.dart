// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'hardware_profile.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$DeviceInstanceId {

@JsonKey(name: 'vendor_id') int get vendorId;@JsonKey(name: 'product_id') int get productId; String? get serial;
/// Create a copy of DeviceInstanceId
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$DeviceInstanceIdCopyWith<DeviceInstanceId> get copyWith => _$DeviceInstanceIdCopyWithImpl<DeviceInstanceId>(this as DeviceInstanceId, _$identity);

  /// Serializes this DeviceInstanceId to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DeviceInstanceId&&(identical(other.vendorId, vendorId) || other.vendorId == vendorId)&&(identical(other.productId, productId) || other.productId == productId)&&(identical(other.serial, serial) || other.serial == serial));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,vendorId,productId,serial);

@override
String toString() {
  return 'DeviceInstanceId(vendorId: $vendorId, productId: $productId, serial: $serial)';
}


}

/// @nodoc
abstract mixin class $DeviceInstanceIdCopyWith<$Res>  {
  factory $DeviceInstanceIdCopyWith(DeviceInstanceId value, $Res Function(DeviceInstanceId) _then) = _$DeviceInstanceIdCopyWithImpl;
@useResult
$Res call({
@JsonKey(name: 'vendor_id') int vendorId,@JsonKey(name: 'product_id') int productId, String? serial
});




}
/// @nodoc
class _$DeviceInstanceIdCopyWithImpl<$Res>
    implements $DeviceInstanceIdCopyWith<$Res> {
  _$DeviceInstanceIdCopyWithImpl(this._self, this._then);

  final DeviceInstanceId _self;
  final $Res Function(DeviceInstanceId) _then;

/// Create a copy of DeviceInstanceId
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? vendorId = null,Object? productId = null,Object? serial = freezed,}) {
  return _then(_self.copyWith(
vendorId: null == vendorId ? _self.vendorId : vendorId // ignore: cast_nullable_to_non_nullable
as int,productId: null == productId ? _self.productId : productId // ignore: cast_nullable_to_non_nullable
as int,serial: freezed == serial ? _self.serial : serial // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}

}


/// Adds pattern-matching-related methods to [DeviceInstanceId].
extension DeviceInstanceIdPatterns on DeviceInstanceId {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _DeviceInstanceId value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _DeviceInstanceId() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _DeviceInstanceId value)  $default,){
final _that = this;
switch (_that) {
case _DeviceInstanceId():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _DeviceInstanceId value)?  $default,){
final _that = this;
switch (_that) {
case _DeviceInstanceId() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function(@JsonKey(name: 'vendor_id')  int vendorId, @JsonKey(name: 'product_id')  int productId,  String? serial)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _DeviceInstanceId() when $default != null:
return $default(_that.vendorId,_that.productId,_that.serial);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function(@JsonKey(name: 'vendor_id')  int vendorId, @JsonKey(name: 'product_id')  int productId,  String? serial)  $default,) {final _that = this;
switch (_that) {
case _DeviceInstanceId():
return $default(_that.vendorId,_that.productId,_that.serial);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function(@JsonKey(name: 'vendor_id')  int vendorId, @JsonKey(name: 'product_id')  int productId,  String? serial)?  $default,) {final _that = this;
switch (_that) {
case _DeviceInstanceId() when $default != null:
return $default(_that.vendorId,_that.productId,_that.serial);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _DeviceInstanceId extends DeviceInstanceId {
  const _DeviceInstanceId({@JsonKey(name: 'vendor_id') required this.vendorId, @JsonKey(name: 'product_id') required this.productId, this.serial}): super._();
  factory _DeviceInstanceId.fromJson(Map<String, dynamic> json) => _$DeviceInstanceIdFromJson(json);

@override@JsonKey(name: 'vendor_id') final  int vendorId;
@override@JsonKey(name: 'product_id') final  int productId;
@override final  String? serial;

/// Create a copy of DeviceInstanceId
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$DeviceInstanceIdCopyWith<_DeviceInstanceId> get copyWith => __$DeviceInstanceIdCopyWithImpl<_DeviceInstanceId>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$DeviceInstanceIdToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _DeviceInstanceId&&(identical(other.vendorId, vendorId) || other.vendorId == vendorId)&&(identical(other.productId, productId) || other.productId == productId)&&(identical(other.serial, serial) || other.serial == serial));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,vendorId,productId,serial);

@override
String toString() {
  return 'DeviceInstanceId(vendorId: $vendorId, productId: $productId, serial: $serial)';
}


}

/// @nodoc
abstract mixin class _$DeviceInstanceIdCopyWith<$Res> implements $DeviceInstanceIdCopyWith<$Res> {
  factory _$DeviceInstanceIdCopyWith(_DeviceInstanceId value, $Res Function(_DeviceInstanceId) _then) = __$DeviceInstanceIdCopyWithImpl;
@override @useResult
$Res call({
@JsonKey(name: 'vendor_id') int vendorId,@JsonKey(name: 'product_id') int productId, String? serial
});




}
/// @nodoc
class __$DeviceInstanceIdCopyWithImpl<$Res>
    implements _$DeviceInstanceIdCopyWith<$Res> {
  __$DeviceInstanceIdCopyWithImpl(this._self, this._then);

  final _DeviceInstanceId _self;
  final $Res Function(_DeviceInstanceId) _then;

/// Create a copy of DeviceInstanceId
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? vendorId = null,Object? productId = null,Object? serial = freezed,}) {
  return _then(_DeviceInstanceId(
vendorId: null == vendorId ? _self.vendorId : vendorId // ignore: cast_nullable_to_non_nullable
as int,productId: null == productId ? _self.productId : productId // ignore: cast_nullable_to_non_nullable
as int,serial: freezed == serial ? _self.serial : serial // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}


/// @nodoc
mixin _$HardwareProfile {

 HardwareProfileId get id;@JsonKey(name: 'virtual_layout_id') VirtualLayoutId get virtualLayoutId;@JsonKey(name: 'vendor_id') int get vendorId;@JsonKey(name: 'product_id') int get productId;@JsonKey(name: 'serial_number') String? get serialNumber; String? get name; Map<int, VirtualKeyId> get wiring;
/// Create a copy of HardwareProfile
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$HardwareProfileCopyWith<HardwareProfile> get copyWith => _$HardwareProfileCopyWithImpl<HardwareProfile>(this as HardwareProfile, _$identity);

  /// Serializes this HardwareProfile to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is HardwareProfile&&(identical(other.id, id) || other.id == id)&&(identical(other.virtualLayoutId, virtualLayoutId) || other.virtualLayoutId == virtualLayoutId)&&(identical(other.vendorId, vendorId) || other.vendorId == vendorId)&&(identical(other.productId, productId) || other.productId == productId)&&(identical(other.serialNumber, serialNumber) || other.serialNumber == serialNumber)&&(identical(other.name, name) || other.name == name)&&const DeepCollectionEquality().equals(other.wiring, wiring));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,id,virtualLayoutId,vendorId,productId,serialNumber,name,const DeepCollectionEquality().hash(wiring));

@override
String toString() {
  return 'HardwareProfile(id: $id, virtualLayoutId: $virtualLayoutId, vendorId: $vendorId, productId: $productId, serialNumber: $serialNumber, name: $name, wiring: $wiring)';
}


}

/// @nodoc
abstract mixin class $HardwareProfileCopyWith<$Res>  {
  factory $HardwareProfileCopyWith(HardwareProfile value, $Res Function(HardwareProfile) _then) = _$HardwareProfileCopyWithImpl;
@useResult
$Res call({
 HardwareProfileId id,@JsonKey(name: 'virtual_layout_id') VirtualLayoutId virtualLayoutId,@JsonKey(name: 'vendor_id') int vendorId,@JsonKey(name: 'product_id') int productId,@JsonKey(name: 'serial_number') String? serialNumber, String? name, Map<int, VirtualKeyId> wiring
});




}
/// @nodoc
class _$HardwareProfileCopyWithImpl<$Res>
    implements $HardwareProfileCopyWith<$Res> {
  _$HardwareProfileCopyWithImpl(this._self, this._then);

  final HardwareProfile _self;
  final $Res Function(HardwareProfile) _then;

/// Create a copy of HardwareProfile
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? id = null,Object? virtualLayoutId = null,Object? vendorId = null,Object? productId = null,Object? serialNumber = freezed,Object? name = freezed,Object? wiring = null,}) {
  return _then(_self.copyWith(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as HardwareProfileId,virtualLayoutId: null == virtualLayoutId ? _self.virtualLayoutId : virtualLayoutId // ignore: cast_nullable_to_non_nullable
as VirtualLayoutId,vendorId: null == vendorId ? _self.vendorId : vendorId // ignore: cast_nullable_to_non_nullable
as int,productId: null == productId ? _self.productId : productId // ignore: cast_nullable_to_non_nullable
as int,serialNumber: freezed == serialNumber ? _self.serialNumber : serialNumber // ignore: cast_nullable_to_non_nullable
as String?,name: freezed == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as String?,wiring: null == wiring ? _self.wiring : wiring // ignore: cast_nullable_to_non_nullable
as Map<int, VirtualKeyId>,
  ));
}

}


/// Adds pattern-matching-related methods to [HardwareProfile].
extension HardwareProfilePatterns on HardwareProfile {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _HardwareProfile value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _HardwareProfile() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _HardwareProfile value)  $default,){
final _that = this;
switch (_that) {
case _HardwareProfile():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _HardwareProfile value)?  $default,){
final _that = this;
switch (_that) {
case _HardwareProfile() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( HardwareProfileId id, @JsonKey(name: 'virtual_layout_id')  VirtualLayoutId virtualLayoutId, @JsonKey(name: 'vendor_id')  int vendorId, @JsonKey(name: 'product_id')  int productId, @JsonKey(name: 'serial_number')  String? serialNumber,  String? name,  Map<int, VirtualKeyId> wiring)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _HardwareProfile() when $default != null:
return $default(_that.id,_that.virtualLayoutId,_that.vendorId,_that.productId,_that.serialNumber,_that.name,_that.wiring);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( HardwareProfileId id, @JsonKey(name: 'virtual_layout_id')  VirtualLayoutId virtualLayoutId, @JsonKey(name: 'vendor_id')  int vendorId, @JsonKey(name: 'product_id')  int productId, @JsonKey(name: 'serial_number')  String? serialNumber,  String? name,  Map<int, VirtualKeyId> wiring)  $default,) {final _that = this;
switch (_that) {
case _HardwareProfile():
return $default(_that.id,_that.virtualLayoutId,_that.vendorId,_that.productId,_that.serialNumber,_that.name,_that.wiring);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( HardwareProfileId id, @JsonKey(name: 'virtual_layout_id')  VirtualLayoutId virtualLayoutId, @JsonKey(name: 'vendor_id')  int vendorId, @JsonKey(name: 'product_id')  int productId, @JsonKey(name: 'serial_number')  String? serialNumber,  String? name,  Map<int, VirtualKeyId> wiring)?  $default,) {final _that = this;
switch (_that) {
case _HardwareProfile() when $default != null:
return $default(_that.id,_that.virtualLayoutId,_that.vendorId,_that.productId,_that.serialNumber,_that.name,_that.wiring);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _HardwareProfile extends HardwareProfile {
  const _HardwareProfile({required this.id, @JsonKey(name: 'virtual_layout_id') required this.virtualLayoutId, @JsonKey(name: 'vendor_id') this.vendorId = 0, @JsonKey(name: 'product_id') this.productId = 0, @JsonKey(name: 'serial_number') this.serialNumber, this.name, final  Map<int, VirtualKeyId> wiring = const <int, VirtualKeyId>{}}): _wiring = wiring,super._();
  factory _HardwareProfile.fromJson(Map<String, dynamic> json) => _$HardwareProfileFromJson(json);

@override final  HardwareProfileId id;
@override@JsonKey(name: 'virtual_layout_id') final  VirtualLayoutId virtualLayoutId;
@override@JsonKey(name: 'vendor_id') final  int vendorId;
@override@JsonKey(name: 'product_id') final  int productId;
@override@JsonKey(name: 'serial_number') final  String? serialNumber;
@override final  String? name;
 final  Map<int, VirtualKeyId> _wiring;
@override@JsonKey() Map<int, VirtualKeyId> get wiring {
  if (_wiring is EqualUnmodifiableMapView) return _wiring;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableMapView(_wiring);
}


/// Create a copy of HardwareProfile
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$HardwareProfileCopyWith<_HardwareProfile> get copyWith => __$HardwareProfileCopyWithImpl<_HardwareProfile>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$HardwareProfileToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _HardwareProfile&&(identical(other.id, id) || other.id == id)&&(identical(other.virtualLayoutId, virtualLayoutId) || other.virtualLayoutId == virtualLayoutId)&&(identical(other.vendorId, vendorId) || other.vendorId == vendorId)&&(identical(other.productId, productId) || other.productId == productId)&&(identical(other.serialNumber, serialNumber) || other.serialNumber == serialNumber)&&(identical(other.name, name) || other.name == name)&&const DeepCollectionEquality().equals(other._wiring, _wiring));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,id,virtualLayoutId,vendorId,productId,serialNumber,name,const DeepCollectionEquality().hash(_wiring));

@override
String toString() {
  return 'HardwareProfile(id: $id, virtualLayoutId: $virtualLayoutId, vendorId: $vendorId, productId: $productId, serialNumber: $serialNumber, name: $name, wiring: $wiring)';
}


}

/// @nodoc
abstract mixin class _$HardwareProfileCopyWith<$Res> implements $HardwareProfileCopyWith<$Res> {
  factory _$HardwareProfileCopyWith(_HardwareProfile value, $Res Function(_HardwareProfile) _then) = __$HardwareProfileCopyWithImpl;
@override @useResult
$Res call({
 HardwareProfileId id,@JsonKey(name: 'virtual_layout_id') VirtualLayoutId virtualLayoutId,@JsonKey(name: 'vendor_id') int vendorId,@JsonKey(name: 'product_id') int productId,@JsonKey(name: 'serial_number') String? serialNumber, String? name, Map<int, VirtualKeyId> wiring
});




}
/// @nodoc
class __$HardwareProfileCopyWithImpl<$Res>
    implements _$HardwareProfileCopyWith<$Res> {
  __$HardwareProfileCopyWithImpl(this._self, this._then);

  final _HardwareProfile _self;
  final $Res Function(_HardwareProfile) _then;

/// Create a copy of HardwareProfile
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? id = null,Object? virtualLayoutId = null,Object? vendorId = null,Object? productId = null,Object? serialNumber = freezed,Object? name = freezed,Object? wiring = null,}) {
  return _then(_HardwareProfile(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as HardwareProfileId,virtualLayoutId: null == virtualLayoutId ? _self.virtualLayoutId : virtualLayoutId // ignore: cast_nullable_to_non_nullable
as VirtualLayoutId,vendorId: null == vendorId ? _self.vendorId : vendorId // ignore: cast_nullable_to_non_nullable
as int,productId: null == productId ? _self.productId : productId // ignore: cast_nullable_to_non_nullable
as int,serialNumber: freezed == serialNumber ? _self.serialNumber : serialNumber // ignore: cast_nullable_to_non_nullable
as String?,name: freezed == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as String?,wiring: null == wiring ? _self._wiring : wiring // ignore: cast_nullable_to_non_nullable
as Map<int, VirtualKeyId>,
  ));
}


}

// dart format on
