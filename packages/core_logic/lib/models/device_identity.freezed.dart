// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'device_identity.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$DeviceIdentity {

/// USB Vendor ID (e.g., 0x046D for Logitech)
@JsonKey(name: 'vendor_id') int get vendorId;/// USB Product ID (e.g., 0xC52B for specific device model)
@JsonKey(name: 'product_id') int get productId;/// Device serial number extracted from USB descriptors or generated
@JsonKey(name: 'serial_number') String get serialNumber;/// Optional user-assigned label for easier identification in UI
@JsonKey(name: 'user_label', includeIfNull: false) String? get userLabel;
/// Create a copy of DeviceIdentity
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$DeviceIdentityCopyWith<DeviceIdentity> get copyWith => _$DeviceIdentityCopyWithImpl<DeviceIdentity>(this as DeviceIdentity, _$identity);

  /// Serializes this DeviceIdentity to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DeviceIdentity&&(identical(other.vendorId, vendorId) || other.vendorId == vendorId)&&(identical(other.productId, productId) || other.productId == productId)&&(identical(other.serialNumber, serialNumber) || other.serialNumber == serialNumber)&&(identical(other.userLabel, userLabel) || other.userLabel == userLabel));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,vendorId,productId,serialNumber,userLabel);

@override
String toString() {
  return 'DeviceIdentity(vendorId: $vendorId, productId: $productId, serialNumber: $serialNumber, userLabel: $userLabel)';
}


}

/// @nodoc
abstract mixin class $DeviceIdentityCopyWith<$Res>  {
  factory $DeviceIdentityCopyWith(DeviceIdentity value, $Res Function(DeviceIdentity) _then) = _$DeviceIdentityCopyWithImpl;
@useResult
$Res call({
@JsonKey(name: 'vendor_id') int vendorId,@JsonKey(name: 'product_id') int productId,@JsonKey(name: 'serial_number') String serialNumber,@JsonKey(name: 'user_label', includeIfNull: false) String? userLabel
});




}
/// @nodoc
class _$DeviceIdentityCopyWithImpl<$Res>
    implements $DeviceIdentityCopyWith<$Res> {
  _$DeviceIdentityCopyWithImpl(this._self, this._then);

  final DeviceIdentity _self;
  final $Res Function(DeviceIdentity) _then;

/// Create a copy of DeviceIdentity
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? vendorId = null,Object? productId = null,Object? serialNumber = null,Object? userLabel = freezed,}) {
  return _then(_self.copyWith(
vendorId: null == vendorId ? _self.vendorId : vendorId // ignore: cast_nullable_to_non_nullable
as int,productId: null == productId ? _self.productId : productId // ignore: cast_nullable_to_non_nullable
as int,serialNumber: null == serialNumber ? _self.serialNumber : serialNumber // ignore: cast_nullable_to_non_nullable
as String,userLabel: freezed == userLabel ? _self.userLabel : userLabel // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}

}


/// Adds pattern-matching-related methods to [DeviceIdentity].
extension DeviceIdentityPatterns on DeviceIdentity {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _DeviceIdentity value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _DeviceIdentity() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _DeviceIdentity value)  $default,){
final _that = this;
switch (_that) {
case _DeviceIdentity():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _DeviceIdentity value)?  $default,){
final _that = this;
switch (_that) {
case _DeviceIdentity() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function(@JsonKey(name: 'vendor_id')  int vendorId, @JsonKey(name: 'product_id')  int productId, @JsonKey(name: 'serial_number')  String serialNumber, @JsonKey(name: 'user_label', includeIfNull: false)  String? userLabel)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _DeviceIdentity() when $default != null:
return $default(_that.vendorId,_that.productId,_that.serialNumber,_that.userLabel);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function(@JsonKey(name: 'vendor_id')  int vendorId, @JsonKey(name: 'product_id')  int productId, @JsonKey(name: 'serial_number')  String serialNumber, @JsonKey(name: 'user_label', includeIfNull: false)  String? userLabel)  $default,) {final _that = this;
switch (_that) {
case _DeviceIdentity():
return $default(_that.vendorId,_that.productId,_that.serialNumber,_that.userLabel);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function(@JsonKey(name: 'vendor_id')  int vendorId, @JsonKey(name: 'product_id')  int productId, @JsonKey(name: 'serial_number')  String serialNumber, @JsonKey(name: 'user_label', includeIfNull: false)  String? userLabel)?  $default,) {final _that = this;
switch (_that) {
case _DeviceIdentity() when $default != null:
return $default(_that.vendorId,_that.productId,_that.serialNumber,_that.userLabel);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _DeviceIdentity extends DeviceIdentity {
  const _DeviceIdentity({@JsonKey(name: 'vendor_id') required this.vendorId, @JsonKey(name: 'product_id') required this.productId, @JsonKey(name: 'serial_number') required this.serialNumber, @JsonKey(name: 'user_label', includeIfNull: false) this.userLabel}): super._();
  factory _DeviceIdentity.fromJson(Map<String, dynamic> json) => _$DeviceIdentityFromJson(json);

/// USB Vendor ID (e.g., 0x046D for Logitech)
@override@JsonKey(name: 'vendor_id') final  int vendorId;
/// USB Product ID (e.g., 0xC52B for specific device model)
@override@JsonKey(name: 'product_id') final  int productId;
/// Device serial number extracted from USB descriptors or generated
@override@JsonKey(name: 'serial_number') final  String serialNumber;
/// Optional user-assigned label for easier identification in UI
@override@JsonKey(name: 'user_label', includeIfNull: false) final  String? userLabel;

/// Create a copy of DeviceIdentity
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$DeviceIdentityCopyWith<_DeviceIdentity> get copyWith => __$DeviceIdentityCopyWithImpl<_DeviceIdentity>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$DeviceIdentityToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _DeviceIdentity&&(identical(other.vendorId, vendorId) || other.vendorId == vendorId)&&(identical(other.productId, productId) || other.productId == productId)&&(identical(other.serialNumber, serialNumber) || other.serialNumber == serialNumber)&&(identical(other.userLabel, userLabel) || other.userLabel == userLabel));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,vendorId,productId,serialNumber,userLabel);

@override
String toString() {
  return 'DeviceIdentity(vendorId: $vendorId, productId: $productId, serialNumber: $serialNumber, userLabel: $userLabel)';
}


}

/// @nodoc
abstract mixin class _$DeviceIdentityCopyWith<$Res> implements $DeviceIdentityCopyWith<$Res> {
  factory _$DeviceIdentityCopyWith(_DeviceIdentity value, $Res Function(_DeviceIdentity) _then) = __$DeviceIdentityCopyWithImpl;
@override @useResult
$Res call({
@JsonKey(name: 'vendor_id') int vendorId,@JsonKey(name: 'product_id') int productId,@JsonKey(name: 'serial_number') String serialNumber,@JsonKey(name: 'user_label', includeIfNull: false) String? userLabel
});




}
/// @nodoc
class __$DeviceIdentityCopyWithImpl<$Res>
    implements _$DeviceIdentityCopyWith<$Res> {
  __$DeviceIdentityCopyWithImpl(this._self, this._then);

  final _DeviceIdentity _self;
  final $Res Function(_DeviceIdentity) _then;

/// Create a copy of DeviceIdentity
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? vendorId = null,Object? productId = null,Object? serialNumber = null,Object? userLabel = freezed,}) {
  return _then(_DeviceIdentity(
vendorId: null == vendorId ? _self.vendorId : vendorId // ignore: cast_nullable_to_non_nullable
as int,productId: null == productId ? _self.productId : productId // ignore: cast_nullable_to_non_nullable
as int,serialNumber: null == serialNumber ? _self.serialNumber : serialNumber // ignore: cast_nullable_to_non_nullable
as String,userLabel: freezed == userLabel ? _self.userLabel : userLabel // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

// dart format on
