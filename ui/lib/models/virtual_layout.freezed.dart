// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'virtual_layout.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
  'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models',
);

KeyPosition _$KeyPositionFromJson(Map<String, dynamic> json) {
  return _KeyPosition.fromJson(json);
}

/// @nodoc
mixin _$KeyPosition {
  double get x => throw _privateConstructorUsedError;
  double get y => throw _privateConstructorUsedError;

  /// Serializes this KeyPosition to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of KeyPosition
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $KeyPositionCopyWith<KeyPosition> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $KeyPositionCopyWith<$Res> {
  factory $KeyPositionCopyWith(
    KeyPosition value,
    $Res Function(KeyPosition) then,
  ) = _$KeyPositionCopyWithImpl<$Res, KeyPosition>;
  @useResult
  $Res call({double x, double y});
}

/// @nodoc
class _$KeyPositionCopyWithImpl<$Res, $Val extends KeyPosition>
    implements $KeyPositionCopyWith<$Res> {
  _$KeyPositionCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of KeyPosition
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? x = null, Object? y = null}) {
    return _then(
      _value.copyWith(
            x: null == x
                ? _value.x
                : x // ignore: cast_nullable_to_non_nullable
                      as double,
            y: null == y
                ? _value.y
                : y // ignore: cast_nullable_to_non_nullable
                      as double,
          )
          as $Val,
    );
  }
}

/// @nodoc
abstract class _$$KeyPositionImplCopyWith<$Res>
    implements $KeyPositionCopyWith<$Res> {
  factory _$$KeyPositionImplCopyWith(
    _$KeyPositionImpl value,
    $Res Function(_$KeyPositionImpl) then,
  ) = __$$KeyPositionImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({double x, double y});
}

/// @nodoc
class __$$KeyPositionImplCopyWithImpl<$Res>
    extends _$KeyPositionCopyWithImpl<$Res, _$KeyPositionImpl>
    implements _$$KeyPositionImplCopyWith<$Res> {
  __$$KeyPositionImplCopyWithImpl(
    _$KeyPositionImpl _value,
    $Res Function(_$KeyPositionImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of KeyPosition
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? x = null, Object? y = null}) {
    return _then(
      _$KeyPositionImpl(
        x: null == x
            ? _value.x
            : x // ignore: cast_nullable_to_non_nullable
                  as double,
        y: null == y
            ? _value.y
            : y // ignore: cast_nullable_to_non_nullable
                  as double,
      ),
    );
  }
}

/// @nodoc
@JsonSerializable()
class _$KeyPositionImpl extends _KeyPosition {
  const _$KeyPositionImpl({required this.x, required this.y}) : super._();

  factory _$KeyPositionImpl.fromJson(Map<String, dynamic> json) =>
      _$$KeyPositionImplFromJson(json);

  @override
  final double x;
  @override
  final double y;

  @override
  String toString() {
    return 'KeyPosition(x: $x, y: $y)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$KeyPositionImpl &&
            (identical(other.x, x) || other.x == x) &&
            (identical(other.y, y) || other.y == y));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, x, y);

  /// Create a copy of KeyPosition
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$KeyPositionImplCopyWith<_$KeyPositionImpl> get copyWith =>
      __$$KeyPositionImplCopyWithImpl<_$KeyPositionImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$KeyPositionImplToJson(this);
  }
}

abstract class _KeyPosition extends KeyPosition {
  const factory _KeyPosition({
    required final double x,
    required final double y,
  }) = _$KeyPositionImpl;
  const _KeyPosition._() : super._();

  factory _KeyPosition.fromJson(Map<String, dynamic> json) =
      _$KeyPositionImpl.fromJson;

  @override
  double get x;
  @override
  double get y;

  /// Create a copy of KeyPosition
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$KeyPositionImplCopyWith<_$KeyPositionImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

KeySize _$KeySizeFromJson(Map<String, dynamic> json) {
  return _KeySize.fromJson(json);
}

/// @nodoc
mixin _$KeySize {
  double get width => throw _privateConstructorUsedError;
  double get height => throw _privateConstructorUsedError;

  /// Serializes this KeySize to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of KeySize
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $KeySizeCopyWith<KeySize> get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $KeySizeCopyWith<$Res> {
  factory $KeySizeCopyWith(KeySize value, $Res Function(KeySize) then) =
      _$KeySizeCopyWithImpl<$Res, KeySize>;
  @useResult
  $Res call({double width, double height});
}

/// @nodoc
class _$KeySizeCopyWithImpl<$Res, $Val extends KeySize>
    implements $KeySizeCopyWith<$Res> {
  _$KeySizeCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of KeySize
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? width = null, Object? height = null}) {
    return _then(
      _value.copyWith(
            width: null == width
                ? _value.width
                : width // ignore: cast_nullable_to_non_nullable
                      as double,
            height: null == height
                ? _value.height
                : height // ignore: cast_nullable_to_non_nullable
                      as double,
          )
          as $Val,
    );
  }
}

/// @nodoc
abstract class _$$KeySizeImplCopyWith<$Res> implements $KeySizeCopyWith<$Res> {
  factory _$$KeySizeImplCopyWith(
    _$KeySizeImpl value,
    $Res Function(_$KeySizeImpl) then,
  ) = __$$KeySizeImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({double width, double height});
}

/// @nodoc
class __$$KeySizeImplCopyWithImpl<$Res>
    extends _$KeySizeCopyWithImpl<$Res, _$KeySizeImpl>
    implements _$$KeySizeImplCopyWith<$Res> {
  __$$KeySizeImplCopyWithImpl(
    _$KeySizeImpl _value,
    $Res Function(_$KeySizeImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of KeySize
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? width = null, Object? height = null}) {
    return _then(
      _$KeySizeImpl(
        width: null == width
            ? _value.width
            : width // ignore: cast_nullable_to_non_nullable
                  as double,
        height: null == height
            ? _value.height
            : height // ignore: cast_nullable_to_non_nullable
                  as double,
      ),
    );
  }
}

/// @nodoc
@JsonSerializable()
class _$KeySizeImpl extends _KeySize {
  const _$KeySizeImpl({required this.width, required this.height}) : super._();

  factory _$KeySizeImpl.fromJson(Map<String, dynamic> json) =>
      _$$KeySizeImplFromJson(json);

  @override
  final double width;
  @override
  final double height;

  @override
  String toString() {
    return 'KeySize(width: $width, height: $height)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$KeySizeImpl &&
            (identical(other.width, width) || other.width == width) &&
            (identical(other.height, height) || other.height == height));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, width, height);

  /// Create a copy of KeySize
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$KeySizeImplCopyWith<_$KeySizeImpl> get copyWith =>
      __$$KeySizeImplCopyWithImpl<_$KeySizeImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$KeySizeImplToJson(this);
  }
}

abstract class _KeySize extends KeySize {
  const factory _KeySize({
    required final double width,
    required final double height,
  }) = _$KeySizeImpl;
  const _KeySize._() : super._();

  factory _KeySize.fromJson(Map<String, dynamic> json) = _$KeySizeImpl.fromJson;

  @override
  double get width;
  @override
  double get height;

  /// Create a copy of KeySize
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$KeySizeImplCopyWith<_$KeySizeImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

VirtualKeyDef _$VirtualKeyDefFromJson(Map<String, dynamic> json) {
  return _VirtualKeyDef.fromJson(json);
}

/// @nodoc
mixin _$VirtualKeyDef {
  String get id => throw _privateConstructorUsedError;
  String get label => throw _privateConstructorUsedError;
  KeyPosition? get position => throw _privateConstructorUsedError;
  KeySize? get size => throw _privateConstructorUsedError;

  /// Serializes this VirtualKeyDef to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of VirtualKeyDef
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $VirtualKeyDefCopyWith<VirtualKeyDef> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $VirtualKeyDefCopyWith<$Res> {
  factory $VirtualKeyDefCopyWith(
    VirtualKeyDef value,
    $Res Function(VirtualKeyDef) then,
  ) = _$VirtualKeyDefCopyWithImpl<$Res, VirtualKeyDef>;
  @useResult
  $Res call({String id, String label, KeyPosition? position, KeySize? size});

  $KeyPositionCopyWith<$Res>? get position;
  $KeySizeCopyWith<$Res>? get size;
}

/// @nodoc
class _$VirtualKeyDefCopyWithImpl<$Res, $Val extends VirtualKeyDef>
    implements $VirtualKeyDefCopyWith<$Res> {
  _$VirtualKeyDefCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of VirtualKeyDef
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? label = null,
    Object? position = freezed,
    Object? size = freezed,
  }) {
    return _then(
      _value.copyWith(
            id: null == id
                ? _value.id
                : id // ignore: cast_nullable_to_non_nullable
                      as String,
            label: null == label
                ? _value.label
                : label // ignore: cast_nullable_to_non_nullable
                      as String,
            position: freezed == position
                ? _value.position
                : position // ignore: cast_nullable_to_non_nullable
                      as KeyPosition?,
            size: freezed == size
                ? _value.size
                : size // ignore: cast_nullable_to_non_nullable
                      as KeySize?,
          )
          as $Val,
    );
  }

  /// Create a copy of VirtualKeyDef
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $KeyPositionCopyWith<$Res>? get position {
    if (_value.position == null) {
      return null;
    }

    return $KeyPositionCopyWith<$Res>(_value.position!, (value) {
      return _then(_value.copyWith(position: value) as $Val);
    });
  }

  /// Create a copy of VirtualKeyDef
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $KeySizeCopyWith<$Res>? get size {
    if (_value.size == null) {
      return null;
    }

    return $KeySizeCopyWith<$Res>(_value.size!, (value) {
      return _then(_value.copyWith(size: value) as $Val);
    });
  }
}

/// @nodoc
abstract class _$$VirtualKeyDefImplCopyWith<$Res>
    implements $VirtualKeyDefCopyWith<$Res> {
  factory _$$VirtualKeyDefImplCopyWith(
    _$VirtualKeyDefImpl value,
    $Res Function(_$VirtualKeyDefImpl) then,
  ) = __$$VirtualKeyDefImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({String id, String label, KeyPosition? position, KeySize? size});

  @override
  $KeyPositionCopyWith<$Res>? get position;
  @override
  $KeySizeCopyWith<$Res>? get size;
}

/// @nodoc
class __$$VirtualKeyDefImplCopyWithImpl<$Res>
    extends _$VirtualKeyDefCopyWithImpl<$Res, _$VirtualKeyDefImpl>
    implements _$$VirtualKeyDefImplCopyWith<$Res> {
  __$$VirtualKeyDefImplCopyWithImpl(
    _$VirtualKeyDefImpl _value,
    $Res Function(_$VirtualKeyDefImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of VirtualKeyDef
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? label = null,
    Object? position = freezed,
    Object? size = freezed,
  }) {
    return _then(
      _$VirtualKeyDefImpl(
        id: null == id
            ? _value.id
            : id // ignore: cast_nullable_to_non_nullable
                  as String,
        label: null == label
            ? _value.label
            : label // ignore: cast_nullable_to_non_nullable
                  as String,
        position: freezed == position
            ? _value.position
            : position // ignore: cast_nullable_to_non_nullable
                  as KeyPosition?,
        size: freezed == size
            ? _value.size
            : size // ignore: cast_nullable_to_non_nullable
                  as KeySize?,
      ),
    );
  }
}

/// @nodoc
@JsonSerializable()
class _$VirtualKeyDefImpl extends _VirtualKeyDef {
  const _$VirtualKeyDefImpl({
    required this.id,
    required this.label,
    this.position,
    this.size,
  }) : super._();

  factory _$VirtualKeyDefImpl.fromJson(Map<String, dynamic> json) =>
      _$$VirtualKeyDefImplFromJson(json);

  @override
  final String id;
  @override
  final String label;
  @override
  final KeyPosition? position;
  @override
  final KeySize? size;

  @override
  String toString() {
    return 'VirtualKeyDef(id: $id, label: $label, position: $position, size: $size)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$VirtualKeyDefImpl &&
            (identical(other.id, id) || other.id == id) &&
            (identical(other.label, label) || other.label == label) &&
            (identical(other.position, position) ||
                other.position == position) &&
            (identical(other.size, size) || other.size == size));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, id, label, position, size);

  /// Create a copy of VirtualKeyDef
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$VirtualKeyDefImplCopyWith<_$VirtualKeyDefImpl> get copyWith =>
      __$$VirtualKeyDefImplCopyWithImpl<_$VirtualKeyDefImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$VirtualKeyDefImplToJson(this);
  }
}

abstract class _VirtualKeyDef extends VirtualKeyDef {
  const factory _VirtualKeyDef({
    required final String id,
    required final String label,
    final KeyPosition? position,
    final KeySize? size,
  }) = _$VirtualKeyDefImpl;
  const _VirtualKeyDef._() : super._();

  factory _VirtualKeyDef.fromJson(Map<String, dynamic> json) =
      _$VirtualKeyDefImpl.fromJson;

  @override
  String get id;
  @override
  String get label;
  @override
  KeyPosition? get position;
  @override
  KeySize? get size;

  /// Create a copy of VirtualKeyDef
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$VirtualKeyDefImplCopyWith<_$VirtualKeyDefImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

VirtualLayout _$VirtualLayoutFromJson(Map<String, dynamic> json) {
  return _VirtualLayout.fromJson(json);
}

/// @nodoc
mixin _$VirtualLayout {
  String get id => throw _privateConstructorUsedError;
  String get name => throw _privateConstructorUsedError;
  @JsonKey(name: 'layout_type')
  VirtualLayoutType get layoutType => throw _privateConstructorUsedError;
  List<VirtualKeyDef> get keys => throw _privateConstructorUsedError;

  /// Serializes this VirtualLayout to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of VirtualLayout
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $VirtualLayoutCopyWith<VirtualLayout> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $VirtualLayoutCopyWith<$Res> {
  factory $VirtualLayoutCopyWith(
    VirtualLayout value,
    $Res Function(VirtualLayout) then,
  ) = _$VirtualLayoutCopyWithImpl<$Res, VirtualLayout>;
  @useResult
  $Res call({
    String id,
    String name,
    @JsonKey(name: 'layout_type') VirtualLayoutType layoutType,
    List<VirtualKeyDef> keys,
  });
}

/// @nodoc
class _$VirtualLayoutCopyWithImpl<$Res, $Val extends VirtualLayout>
    implements $VirtualLayoutCopyWith<$Res> {
  _$VirtualLayoutCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of VirtualLayout
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? name = null,
    Object? layoutType = null,
    Object? keys = null,
  }) {
    return _then(
      _value.copyWith(
            id: null == id
                ? _value.id
                : id // ignore: cast_nullable_to_non_nullable
                      as String,
            name: null == name
                ? _value.name
                : name // ignore: cast_nullable_to_non_nullable
                      as String,
            layoutType: null == layoutType
                ? _value.layoutType
                : layoutType // ignore: cast_nullable_to_non_nullable
                      as VirtualLayoutType,
            keys: null == keys
                ? _value.keys
                : keys // ignore: cast_nullable_to_non_nullable
                      as List<VirtualKeyDef>,
          )
          as $Val,
    );
  }
}

/// @nodoc
abstract class _$$VirtualLayoutImplCopyWith<$Res>
    implements $VirtualLayoutCopyWith<$Res> {
  factory _$$VirtualLayoutImplCopyWith(
    _$VirtualLayoutImpl value,
    $Res Function(_$VirtualLayoutImpl) then,
  ) = __$$VirtualLayoutImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({
    String id,
    String name,
    @JsonKey(name: 'layout_type') VirtualLayoutType layoutType,
    List<VirtualKeyDef> keys,
  });
}

/// @nodoc
class __$$VirtualLayoutImplCopyWithImpl<$Res>
    extends _$VirtualLayoutCopyWithImpl<$Res, _$VirtualLayoutImpl>
    implements _$$VirtualLayoutImplCopyWith<$Res> {
  __$$VirtualLayoutImplCopyWithImpl(
    _$VirtualLayoutImpl _value,
    $Res Function(_$VirtualLayoutImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of VirtualLayout
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? name = null,
    Object? layoutType = null,
    Object? keys = null,
  }) {
    return _then(
      _$VirtualLayoutImpl(
        id: null == id
            ? _value.id
            : id // ignore: cast_nullable_to_non_nullable
                  as String,
        name: null == name
            ? _value.name
            : name // ignore: cast_nullable_to_non_nullable
                  as String,
        layoutType: null == layoutType
            ? _value.layoutType
            : layoutType // ignore: cast_nullable_to_non_nullable
                  as VirtualLayoutType,
        keys: null == keys
            ? _value._keys
            : keys // ignore: cast_nullable_to_non_nullable
                  as List<VirtualKeyDef>,
      ),
    );
  }
}

/// @nodoc
@JsonSerializable()
class _$VirtualLayoutImpl extends _VirtualLayout {
  const _$VirtualLayoutImpl({
    required this.id,
    required this.name,
    @JsonKey(name: 'layout_type') required this.layoutType,
    final List<VirtualKeyDef> keys = const <VirtualKeyDef>[],
  }) : _keys = keys,
       super._();

  factory _$VirtualLayoutImpl.fromJson(Map<String, dynamic> json) =>
      _$$VirtualLayoutImplFromJson(json);

  @override
  final String id;
  @override
  final String name;
  @override
  @JsonKey(name: 'layout_type')
  final VirtualLayoutType layoutType;
  final List<VirtualKeyDef> _keys;
  @override
  @JsonKey()
  List<VirtualKeyDef> get keys {
    if (_keys is EqualUnmodifiableListView) return _keys;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_keys);
  }

  @override
  String toString() {
    return 'VirtualLayout(id: $id, name: $name, layoutType: $layoutType, keys: $keys)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$VirtualLayoutImpl &&
            (identical(other.id, id) || other.id == id) &&
            (identical(other.name, name) || other.name == name) &&
            (identical(other.layoutType, layoutType) ||
                other.layoutType == layoutType) &&
            const DeepCollectionEquality().equals(other._keys, _keys));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(
    runtimeType,
    id,
    name,
    layoutType,
    const DeepCollectionEquality().hash(_keys),
  );

  /// Create a copy of VirtualLayout
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$VirtualLayoutImplCopyWith<_$VirtualLayoutImpl> get copyWith =>
      __$$VirtualLayoutImplCopyWithImpl<_$VirtualLayoutImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$VirtualLayoutImplToJson(this);
  }
}

abstract class _VirtualLayout extends VirtualLayout {
  const factory _VirtualLayout({
    required final String id,
    required final String name,
    @JsonKey(name: 'layout_type') required final VirtualLayoutType layoutType,
    final List<VirtualKeyDef> keys,
  }) = _$VirtualLayoutImpl;
  const _VirtualLayout._() : super._();

  factory _VirtualLayout.fromJson(Map<String, dynamic> json) =
      _$VirtualLayoutImpl.fromJson;

  @override
  String get id;
  @override
  String get name;
  @override
  @JsonKey(name: 'layout_type')
  VirtualLayoutType get layoutType;
  @override
  List<VirtualKeyDef> get keys;

  /// Create a copy of VirtualLayout
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$VirtualLayoutImplCopyWith<_$VirtualLayoutImpl> get copyWith =>
      throw _privateConstructorUsedError;
}
