// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'device_identity.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
  'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models',
);

DeviceIdentity _$DeviceIdentityFromJson(Map<String, dynamic> json) {
  return _DeviceIdentity.fromJson(json);
}

/// @nodoc
mixin _$DeviceIdentity {
  /// USB Vendor ID (e.g., 0x046D for Logitech)
  @JsonKey(name: 'vendor_id')
  int get vendorId => throw _privateConstructorUsedError;

  /// USB Product ID (e.g., 0xC52B for specific device model)
  @JsonKey(name: 'product_id')
  int get productId => throw _privateConstructorUsedError;

  /// Device serial number extracted from USB descriptors or generated
  @JsonKey(name: 'serial_number')
  String get serialNumber => throw _privateConstructorUsedError;

  /// Optional user-assigned label for easier identification in UI
  @JsonKey(name: 'user_label', includeIfNull: false)
  String? get userLabel => throw _privateConstructorUsedError;

  /// Serializes this DeviceIdentity to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of DeviceIdentity
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $DeviceIdentityCopyWith<DeviceIdentity> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $DeviceIdentityCopyWith<$Res> {
  factory $DeviceIdentityCopyWith(
    DeviceIdentity value,
    $Res Function(DeviceIdentity) then,
  ) = _$DeviceIdentityCopyWithImpl<$Res, DeviceIdentity>;
  @useResult
  $Res call({
    @JsonKey(name: 'vendor_id') int vendorId,
    @JsonKey(name: 'product_id') int productId,
    @JsonKey(name: 'serial_number') String serialNumber,
    @JsonKey(name: 'user_label', includeIfNull: false) String? userLabel,
  });
}

/// @nodoc
class _$DeviceIdentityCopyWithImpl<$Res, $Val extends DeviceIdentity>
    implements $DeviceIdentityCopyWith<$Res> {
  _$DeviceIdentityCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of DeviceIdentity
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? vendorId = null,
    Object? productId = null,
    Object? serialNumber = null,
    Object? userLabel = freezed,
  }) {
    return _then(
      _value.copyWith(
            vendorId: null == vendorId
                ? _value.vendorId
                : vendorId // ignore: cast_nullable_to_non_nullable
                      as int,
            productId: null == productId
                ? _value.productId
                : productId // ignore: cast_nullable_to_non_nullable
                      as int,
            serialNumber: null == serialNumber
                ? _value.serialNumber
                : serialNumber // ignore: cast_nullable_to_non_nullable
                      as String,
            userLabel: freezed == userLabel
                ? _value.userLabel
                : userLabel // ignore: cast_nullable_to_non_nullable
                      as String?,
          )
          as $Val,
    );
  }
}

/// @nodoc
abstract class _$$DeviceIdentityImplCopyWith<$Res>
    implements $DeviceIdentityCopyWith<$Res> {
  factory _$$DeviceIdentityImplCopyWith(
    _$DeviceIdentityImpl value,
    $Res Function(_$DeviceIdentityImpl) then,
  ) = __$$DeviceIdentityImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({
    @JsonKey(name: 'vendor_id') int vendorId,
    @JsonKey(name: 'product_id') int productId,
    @JsonKey(name: 'serial_number') String serialNumber,
    @JsonKey(name: 'user_label', includeIfNull: false) String? userLabel,
  });
}

/// @nodoc
class __$$DeviceIdentityImplCopyWithImpl<$Res>
    extends _$DeviceIdentityCopyWithImpl<$Res, _$DeviceIdentityImpl>
    implements _$$DeviceIdentityImplCopyWith<$Res> {
  __$$DeviceIdentityImplCopyWithImpl(
    _$DeviceIdentityImpl _value,
    $Res Function(_$DeviceIdentityImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of DeviceIdentity
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? vendorId = null,
    Object? productId = null,
    Object? serialNumber = null,
    Object? userLabel = freezed,
  }) {
    return _then(
      _$DeviceIdentityImpl(
        vendorId: null == vendorId
            ? _value.vendorId
            : vendorId // ignore: cast_nullable_to_non_nullable
                  as int,
        productId: null == productId
            ? _value.productId
            : productId // ignore: cast_nullable_to_non_nullable
                  as int,
        serialNumber: null == serialNumber
            ? _value.serialNumber
            : serialNumber // ignore: cast_nullable_to_non_nullable
                  as String,
        userLabel: freezed == userLabel
            ? _value.userLabel
            : userLabel // ignore: cast_nullable_to_non_nullable
                  as String?,
      ),
    );
  }
}

/// @nodoc
@JsonSerializable()
class _$DeviceIdentityImpl extends _DeviceIdentity {
  const _$DeviceIdentityImpl({
    @JsonKey(name: 'vendor_id') required this.vendorId,
    @JsonKey(name: 'product_id') required this.productId,
    @JsonKey(name: 'serial_number') required this.serialNumber,
    @JsonKey(name: 'user_label', includeIfNull: false) this.userLabel,
  }) : super._();

  factory _$DeviceIdentityImpl.fromJson(Map<String, dynamic> json) =>
      _$$DeviceIdentityImplFromJson(json);

  /// USB Vendor ID (e.g., 0x046D for Logitech)
  @override
  @JsonKey(name: 'vendor_id')
  final int vendorId;

  /// USB Product ID (e.g., 0xC52B for specific device model)
  @override
  @JsonKey(name: 'product_id')
  final int productId;

  /// Device serial number extracted from USB descriptors or generated
  @override
  @JsonKey(name: 'serial_number')
  final String serialNumber;

  /// Optional user-assigned label for easier identification in UI
  @override
  @JsonKey(name: 'user_label', includeIfNull: false)
  final String? userLabel;

  @override
  String toString() {
    return 'DeviceIdentity(vendorId: $vendorId, productId: $productId, serialNumber: $serialNumber, userLabel: $userLabel)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$DeviceIdentityImpl &&
            (identical(other.vendorId, vendorId) ||
                other.vendorId == vendorId) &&
            (identical(other.productId, productId) ||
                other.productId == productId) &&
            (identical(other.serialNumber, serialNumber) ||
                other.serialNumber == serialNumber) &&
            (identical(other.userLabel, userLabel) ||
                other.userLabel == userLabel));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode =>
      Object.hash(runtimeType, vendorId, productId, serialNumber, userLabel);

  /// Create a copy of DeviceIdentity
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$DeviceIdentityImplCopyWith<_$DeviceIdentityImpl> get copyWith =>
      __$$DeviceIdentityImplCopyWithImpl<_$DeviceIdentityImpl>(
        this,
        _$identity,
      );

  @override
  Map<String, dynamic> toJson() {
    return _$$DeviceIdentityImplToJson(this);
  }
}

abstract class _DeviceIdentity extends DeviceIdentity {
  const factory _DeviceIdentity({
    @JsonKey(name: 'vendor_id') required final int vendorId,
    @JsonKey(name: 'product_id') required final int productId,
    @JsonKey(name: 'serial_number') required final String serialNumber,
    @JsonKey(name: 'user_label', includeIfNull: false) final String? userLabel,
  }) = _$DeviceIdentityImpl;
  const _DeviceIdentity._() : super._();

  factory _DeviceIdentity.fromJson(Map<String, dynamic> json) =
      _$DeviceIdentityImpl.fromJson;

  /// USB Vendor ID (e.g., 0x046D for Logitech)
  @override
  @JsonKey(name: 'vendor_id')
  int get vendorId;

  /// USB Product ID (e.g., 0xC52B for specific device model)
  @override
  @JsonKey(name: 'product_id')
  int get productId;

  /// Device serial number extracted from USB descriptors or generated
  @override
  @JsonKey(name: 'serial_number')
  String get serialNumber;

  /// Optional user-assigned label for easier identification in UI
  @override
  @JsonKey(name: 'user_label', includeIfNull: false)
  String? get userLabel;

  /// Create a copy of DeviceIdentity
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$DeviceIdentityImplCopyWith<_$DeviceIdentityImpl> get copyWith =>
      throw _privateConstructorUsedError;
}
