// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'hardware_profile.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
  'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models',
);

DeviceInstanceId _$DeviceInstanceIdFromJson(Map<String, dynamic> json) {
  return _DeviceInstanceId.fromJson(json);
}

/// @nodoc
mixin _$DeviceInstanceId {
  @JsonKey(name: 'vendor_id')
  int get vendorId => throw _privateConstructorUsedError;
  @JsonKey(name: 'product_id')
  int get productId => throw _privateConstructorUsedError;
  String? get serial => throw _privateConstructorUsedError;

  /// Serializes this DeviceInstanceId to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of DeviceInstanceId
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $DeviceInstanceIdCopyWith<DeviceInstanceId> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $DeviceInstanceIdCopyWith<$Res> {
  factory $DeviceInstanceIdCopyWith(
    DeviceInstanceId value,
    $Res Function(DeviceInstanceId) then,
  ) = _$DeviceInstanceIdCopyWithImpl<$Res, DeviceInstanceId>;
  @useResult
  $Res call({
    @JsonKey(name: 'vendor_id') int vendorId,
    @JsonKey(name: 'product_id') int productId,
    String? serial,
  });
}

/// @nodoc
class _$DeviceInstanceIdCopyWithImpl<$Res, $Val extends DeviceInstanceId>
    implements $DeviceInstanceIdCopyWith<$Res> {
  _$DeviceInstanceIdCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of DeviceInstanceId
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? vendorId = null,
    Object? productId = null,
    Object? serial = freezed,
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
            serial: freezed == serial
                ? _value.serial
                : serial // ignore: cast_nullable_to_non_nullable
                      as String?,
          )
          as $Val,
    );
  }
}

/// @nodoc
abstract class _$$DeviceInstanceIdImplCopyWith<$Res>
    implements $DeviceInstanceIdCopyWith<$Res> {
  factory _$$DeviceInstanceIdImplCopyWith(
    _$DeviceInstanceIdImpl value,
    $Res Function(_$DeviceInstanceIdImpl) then,
  ) = __$$DeviceInstanceIdImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({
    @JsonKey(name: 'vendor_id') int vendorId,
    @JsonKey(name: 'product_id') int productId,
    String? serial,
  });
}

/// @nodoc
class __$$DeviceInstanceIdImplCopyWithImpl<$Res>
    extends _$DeviceInstanceIdCopyWithImpl<$Res, _$DeviceInstanceIdImpl>
    implements _$$DeviceInstanceIdImplCopyWith<$Res> {
  __$$DeviceInstanceIdImplCopyWithImpl(
    _$DeviceInstanceIdImpl _value,
    $Res Function(_$DeviceInstanceIdImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of DeviceInstanceId
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? vendorId = null,
    Object? productId = null,
    Object? serial = freezed,
  }) {
    return _then(
      _$DeviceInstanceIdImpl(
        vendorId: null == vendorId
            ? _value.vendorId
            : vendorId // ignore: cast_nullable_to_non_nullable
                  as int,
        productId: null == productId
            ? _value.productId
            : productId // ignore: cast_nullable_to_non_nullable
                  as int,
        serial: freezed == serial
            ? _value.serial
            : serial // ignore: cast_nullable_to_non_nullable
                  as String?,
      ),
    );
  }
}

/// @nodoc
@JsonSerializable()
class _$DeviceInstanceIdImpl extends _DeviceInstanceId {
  const _$DeviceInstanceIdImpl({
    @JsonKey(name: 'vendor_id') required this.vendorId,
    @JsonKey(name: 'product_id') required this.productId,
    this.serial,
  }) : super._();

  factory _$DeviceInstanceIdImpl.fromJson(Map<String, dynamic> json) =>
      _$$DeviceInstanceIdImplFromJson(json);

  @override
  @JsonKey(name: 'vendor_id')
  final int vendorId;
  @override
  @JsonKey(name: 'product_id')
  final int productId;
  @override
  final String? serial;

  @override
  String toString() {
    return 'DeviceInstanceId(vendorId: $vendorId, productId: $productId, serial: $serial)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$DeviceInstanceIdImpl &&
            (identical(other.vendorId, vendorId) ||
                other.vendorId == vendorId) &&
            (identical(other.productId, productId) ||
                other.productId == productId) &&
            (identical(other.serial, serial) || other.serial == serial));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, vendorId, productId, serial);

  /// Create a copy of DeviceInstanceId
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$DeviceInstanceIdImplCopyWith<_$DeviceInstanceIdImpl> get copyWith =>
      __$$DeviceInstanceIdImplCopyWithImpl<_$DeviceInstanceIdImpl>(
        this,
        _$identity,
      );

  @override
  Map<String, dynamic> toJson() {
    return _$$DeviceInstanceIdImplToJson(this);
  }
}

abstract class _DeviceInstanceId extends DeviceInstanceId {
  const factory _DeviceInstanceId({
    @JsonKey(name: 'vendor_id') required final int vendorId,
    @JsonKey(name: 'product_id') required final int productId,
    final String? serial,
  }) = _$DeviceInstanceIdImpl;
  const _DeviceInstanceId._() : super._();

  factory _DeviceInstanceId.fromJson(Map<String, dynamic> json) =
      _$DeviceInstanceIdImpl.fromJson;

  @override
  @JsonKey(name: 'vendor_id')
  int get vendorId;
  @override
  @JsonKey(name: 'product_id')
  int get productId;
  @override
  String? get serial;

  /// Create a copy of DeviceInstanceId
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$DeviceInstanceIdImplCopyWith<_$DeviceInstanceIdImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

HardwareProfile _$HardwareProfileFromJson(Map<String, dynamic> json) {
  return _HardwareProfile.fromJson(json);
}

/// @nodoc
mixin _$HardwareProfile {
  String get id => throw _privateConstructorUsedError;
  @JsonKey(name: 'vendor_id')
  int get vendorId => throw _privateConstructorUsedError;
  @JsonKey(name: 'product_id')
  int get productId => throw _privateConstructorUsedError;
  @JsonKey(name: 'serial_number')
  String? get serialNumber => throw _privateConstructorUsedError;
  String? get name => throw _privateConstructorUsedError;
  @JsonKey(name: 'virtual_layout_id')
  String get virtualLayoutId => throw _privateConstructorUsedError;
  Map<int, String> get wiring => throw _privateConstructorUsedError;

  /// Serializes this HardwareProfile to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of HardwareProfile
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $HardwareProfileCopyWith<HardwareProfile> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $HardwareProfileCopyWith<$Res> {
  factory $HardwareProfileCopyWith(
    HardwareProfile value,
    $Res Function(HardwareProfile) then,
  ) = _$HardwareProfileCopyWithImpl<$Res, HardwareProfile>;
  @useResult
  $Res call({
    String id,
    @JsonKey(name: 'vendor_id') int vendorId,
    @JsonKey(name: 'product_id') int productId,
    @JsonKey(name: 'serial_number') String? serialNumber,
    String? name,
    @JsonKey(name: 'virtual_layout_id') String virtualLayoutId,
    Map<int, String> wiring,
  });
}

/// @nodoc
class _$HardwareProfileCopyWithImpl<$Res, $Val extends HardwareProfile>
    implements $HardwareProfileCopyWith<$Res> {
  _$HardwareProfileCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of HardwareProfile
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? vendorId = null,
    Object? productId = null,
    Object? serialNumber = freezed,
    Object? name = freezed,
    Object? virtualLayoutId = null,
    Object? wiring = null,
  }) {
    return _then(
      _value.copyWith(
            id: null == id
                ? _value.id
                : id // ignore: cast_nullable_to_non_nullable
                      as String,
            vendorId: null == vendorId
                ? _value.vendorId
                : vendorId // ignore: cast_nullable_to_non_nullable
                      as int,
            productId: null == productId
                ? _value.productId
                : productId // ignore: cast_nullable_to_non_nullable
                      as int,
            serialNumber: freezed == serialNumber
                ? _value.serialNumber
                : serialNumber // ignore: cast_nullable_to_non_nullable
                      as String?,
            name: freezed == name
                ? _value.name
                : name // ignore: cast_nullable_to_non_nullable
                      as String?,
            virtualLayoutId: null == virtualLayoutId
                ? _value.virtualLayoutId
                : virtualLayoutId // ignore: cast_nullable_to_non_nullable
                      as String,
            wiring: null == wiring
                ? _value.wiring
                : wiring // ignore: cast_nullable_to_non_nullable
                      as Map<int, String>,
          )
          as $Val,
    );
  }
}

/// @nodoc
abstract class _$$HardwareProfileImplCopyWith<$Res>
    implements $HardwareProfileCopyWith<$Res> {
  factory _$$HardwareProfileImplCopyWith(
    _$HardwareProfileImpl value,
    $Res Function(_$HardwareProfileImpl) then,
  ) = __$$HardwareProfileImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({
    String id,
    @JsonKey(name: 'vendor_id') int vendorId,
    @JsonKey(name: 'product_id') int productId,
    @JsonKey(name: 'serial_number') String? serialNumber,
    String? name,
    @JsonKey(name: 'virtual_layout_id') String virtualLayoutId,
    Map<int, String> wiring,
  });
}

/// @nodoc
class __$$HardwareProfileImplCopyWithImpl<$Res>
    extends _$HardwareProfileCopyWithImpl<$Res, _$HardwareProfileImpl>
    implements _$$HardwareProfileImplCopyWith<$Res> {
  __$$HardwareProfileImplCopyWithImpl(
    _$HardwareProfileImpl _value,
    $Res Function(_$HardwareProfileImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of HardwareProfile
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? vendorId = null,
    Object? productId = null,
    Object? serialNumber = freezed,
    Object? name = freezed,
    Object? virtualLayoutId = null,
    Object? wiring = null,
  }) {
    return _then(
      _$HardwareProfileImpl(
        id: null == id
            ? _value.id
            : id // ignore: cast_nullable_to_non_nullable
                  as String,
        vendorId: null == vendorId
            ? _value.vendorId
            : vendorId // ignore: cast_nullable_to_non_nullable
                  as int,
        productId: null == productId
            ? _value.productId
            : productId // ignore: cast_nullable_to_non_nullable
                  as int,
        serialNumber: freezed == serialNumber
            ? _value.serialNumber
            : serialNumber // ignore: cast_nullable_to_non_nullable
                  as String?,
        name: freezed == name
            ? _value.name
            : name // ignore: cast_nullable_to_non_nullable
                  as String?,
        virtualLayoutId: null == virtualLayoutId
            ? _value.virtualLayoutId
            : virtualLayoutId // ignore: cast_nullable_to_non_nullable
                  as String,
        wiring: null == wiring
            ? _value._wiring
            : wiring // ignore: cast_nullable_to_non_nullable
                  as Map<int, String>,
      ),
    );
  }
}

/// @nodoc
@JsonSerializable()
class _$HardwareProfileImpl extends _HardwareProfile {
  const _$HardwareProfileImpl({
    required this.id,
    @JsonKey(name: 'vendor_id') required this.vendorId,
    @JsonKey(name: 'product_id') required this.productId,
    @JsonKey(name: 'serial_number') this.serialNumber,
    this.name,
    @JsonKey(name: 'virtual_layout_id') required this.virtualLayoutId,
    final Map<int, String> wiring = const <int, VirtualKeyId>{},
  }) : _wiring = wiring,
       super._();

  factory _$HardwareProfileImpl.fromJson(Map<String, dynamic> json) =>
      _$$HardwareProfileImplFromJson(json);

  @override
  final String id;
  @override
  @JsonKey(name: 'vendor_id')
  final int vendorId;
  @override
  @JsonKey(name: 'product_id')
  final int productId;
  @override
  @JsonKey(name: 'serial_number')
  final String? serialNumber;
  @override
  final String? name;
  @override
  @JsonKey(name: 'virtual_layout_id')
  final String virtualLayoutId;
  final Map<int, String> _wiring;
  @override
  @JsonKey()
  Map<int, String> get wiring {
    if (_wiring is EqualUnmodifiableMapView) return _wiring;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableMapView(_wiring);
  }

  @override
  String toString() {
    return 'HardwareProfile(id: $id, vendorId: $vendorId, productId: $productId, serialNumber: $serialNumber, name: $name, virtualLayoutId: $virtualLayoutId, wiring: $wiring)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$HardwareProfileImpl &&
            (identical(other.id, id) || other.id == id) &&
            (identical(other.vendorId, vendorId) ||
                other.vendorId == vendorId) &&
            (identical(other.productId, productId) ||
                other.productId == productId) &&
            (identical(other.serialNumber, serialNumber) ||
                other.serialNumber == serialNumber) &&
            (identical(other.name, name) || other.name == name) &&
            (identical(other.virtualLayoutId, virtualLayoutId) ||
                other.virtualLayoutId == virtualLayoutId) &&
            const DeepCollectionEquality().equals(other._wiring, _wiring));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(
    runtimeType,
    id,
    vendorId,
    productId,
    serialNumber,
    name,
    virtualLayoutId,
    const DeepCollectionEquality().hash(_wiring),
  );

  /// Create a copy of HardwareProfile
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$HardwareProfileImplCopyWith<_$HardwareProfileImpl> get copyWith =>
      __$$HardwareProfileImplCopyWithImpl<_$HardwareProfileImpl>(
        this,
        _$identity,
      );

  @override
  Map<String, dynamic> toJson() {
    return _$$HardwareProfileImplToJson(this);
  }
}

abstract class _HardwareProfile extends HardwareProfile {
  const factory _HardwareProfile({
    required final String id,
    @JsonKey(name: 'vendor_id') required final int vendorId,
    @JsonKey(name: 'product_id') required final int productId,
    @JsonKey(name: 'serial_number') final String? serialNumber,
    final String? name,
    @JsonKey(name: 'virtual_layout_id') required final String virtualLayoutId,
    final Map<int, String> wiring,
  }) = _$HardwareProfileImpl;
  const _HardwareProfile._() : super._();

  factory _HardwareProfile.fromJson(Map<String, dynamic> json) =
      _$HardwareProfileImpl.fromJson;

  @override
  String get id;
  @override
  @JsonKey(name: 'vendor_id')
  int get vendorId;
  @override
  @JsonKey(name: 'product_id')
  int get productId;
  @override
  @JsonKey(name: 'serial_number')
  String? get serialNumber;
  @override
  String? get name;
  @override
  @JsonKey(name: 'virtual_layout_id')
  String get virtualLayoutId;
  @override
  Map<int, String> get wiring;

  /// Create a copy of HardwareProfile
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$HardwareProfileImplCopyWith<_$HardwareProfileImpl> get copyWith =>
      throw _privateConstructorUsedError;
}
