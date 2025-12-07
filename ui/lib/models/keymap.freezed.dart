// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'keymap.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
  'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models',
);

KeymapLayer _$KeymapLayerFromJson(Map<String, dynamic> json) {
  return _KeymapLayer.fromJson(json);
}

/// @nodoc
mixin _$KeymapLayer {
  String get name => throw _privateConstructorUsedError;
  Map<String, ActionBinding> get bindings => throw _privateConstructorUsedError;

  /// Serializes this KeymapLayer to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of KeymapLayer
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $KeymapLayerCopyWith<KeymapLayer> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $KeymapLayerCopyWith<$Res> {
  factory $KeymapLayerCopyWith(
    KeymapLayer value,
    $Res Function(KeymapLayer) then,
  ) = _$KeymapLayerCopyWithImpl<$Res, KeymapLayer>;
  @useResult
  $Res call({String name, Map<String, ActionBinding> bindings});
}

/// @nodoc
class _$KeymapLayerCopyWithImpl<$Res, $Val extends KeymapLayer>
    implements $KeymapLayerCopyWith<$Res> {
  _$KeymapLayerCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of KeymapLayer
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? name = null, Object? bindings = null}) {
    return _then(
      _value.copyWith(
            name: null == name
                ? _value.name
                : name // ignore: cast_nullable_to_non_nullable
                      as String,
            bindings: null == bindings
                ? _value.bindings
                : bindings // ignore: cast_nullable_to_non_nullable
                      as Map<String, ActionBinding>,
          )
          as $Val,
    );
  }
}

/// @nodoc
abstract class _$$KeymapLayerImplCopyWith<$Res>
    implements $KeymapLayerCopyWith<$Res> {
  factory _$$KeymapLayerImplCopyWith(
    _$KeymapLayerImpl value,
    $Res Function(_$KeymapLayerImpl) then,
  ) = __$$KeymapLayerImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({String name, Map<String, ActionBinding> bindings});
}

/// @nodoc
class __$$KeymapLayerImplCopyWithImpl<$Res>
    extends _$KeymapLayerCopyWithImpl<$Res, _$KeymapLayerImpl>
    implements _$$KeymapLayerImplCopyWith<$Res> {
  __$$KeymapLayerImplCopyWithImpl(
    _$KeymapLayerImpl _value,
    $Res Function(_$KeymapLayerImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of KeymapLayer
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? name = null, Object? bindings = null}) {
    return _then(
      _$KeymapLayerImpl(
        name: null == name
            ? _value.name
            : name // ignore: cast_nullable_to_non_nullable
                  as String,
        bindings: null == bindings
            ? _value._bindings
            : bindings // ignore: cast_nullable_to_non_nullable
                  as Map<String, ActionBinding>,
      ),
    );
  }
}

/// @nodoc
@JsonSerializable()
class _$KeymapLayerImpl extends _KeymapLayer {
  const _$KeymapLayerImpl({
    required this.name,
    final Map<String, ActionBinding> bindings =
        const <VirtualKeyId, ActionBinding>{},
  }) : _bindings = bindings,
       super._();

  factory _$KeymapLayerImpl.fromJson(Map<String, dynamic> json) =>
      _$$KeymapLayerImplFromJson(json);

  @override
  final String name;
  final Map<String, ActionBinding> _bindings;
  @override
  @JsonKey()
  Map<String, ActionBinding> get bindings {
    if (_bindings is EqualUnmodifiableMapView) return _bindings;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableMapView(_bindings);
  }

  @override
  String toString() {
    return 'KeymapLayer(name: $name, bindings: $bindings)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$KeymapLayerImpl &&
            (identical(other.name, name) || other.name == name) &&
            const DeepCollectionEquality().equals(other._bindings, _bindings));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(
    runtimeType,
    name,
    const DeepCollectionEquality().hash(_bindings),
  );

  /// Create a copy of KeymapLayer
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$KeymapLayerImplCopyWith<_$KeymapLayerImpl> get copyWith =>
      __$$KeymapLayerImplCopyWithImpl<_$KeymapLayerImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$KeymapLayerImplToJson(this);
  }
}

abstract class _KeymapLayer extends KeymapLayer {
  const factory _KeymapLayer({
    required final String name,
    final Map<String, ActionBinding> bindings,
  }) = _$KeymapLayerImpl;
  const _KeymapLayer._() : super._();

  factory _KeymapLayer.fromJson(Map<String, dynamic> json) =
      _$KeymapLayerImpl.fromJson;

  @override
  String get name;
  @override
  Map<String, ActionBinding> get bindings;

  /// Create a copy of KeymapLayer
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$KeymapLayerImplCopyWith<_$KeymapLayerImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

Keymap _$KeymapFromJson(Map<String, dynamic> json) {
  return _Keymap.fromJson(json);
}

/// @nodoc
mixin _$Keymap {
  String get id => throw _privateConstructorUsedError;
  String get name => throw _privateConstructorUsedError;
  @JsonKey(name: 'virtual_layout_id')
  String get virtualLayoutId => throw _privateConstructorUsedError;
  List<KeymapLayer> get layers => throw _privateConstructorUsedError;

  /// Serializes this Keymap to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of Keymap
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $KeymapCopyWith<Keymap> get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $KeymapCopyWith<$Res> {
  factory $KeymapCopyWith(Keymap value, $Res Function(Keymap) then) =
      _$KeymapCopyWithImpl<$Res, Keymap>;
  @useResult
  $Res call({
    String id,
    String name,
    @JsonKey(name: 'virtual_layout_id') String virtualLayoutId,
    List<KeymapLayer> layers,
  });
}

/// @nodoc
class _$KeymapCopyWithImpl<$Res, $Val extends Keymap>
    implements $KeymapCopyWith<$Res> {
  _$KeymapCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of Keymap
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? name = null,
    Object? virtualLayoutId = null,
    Object? layers = null,
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
            virtualLayoutId: null == virtualLayoutId
                ? _value.virtualLayoutId
                : virtualLayoutId // ignore: cast_nullable_to_non_nullable
                      as String,
            layers: null == layers
                ? _value.layers
                : layers // ignore: cast_nullable_to_non_nullable
                      as List<KeymapLayer>,
          )
          as $Val,
    );
  }
}

/// @nodoc
abstract class _$$KeymapImplCopyWith<$Res> implements $KeymapCopyWith<$Res> {
  factory _$$KeymapImplCopyWith(
    _$KeymapImpl value,
    $Res Function(_$KeymapImpl) then,
  ) = __$$KeymapImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({
    String id,
    String name,
    @JsonKey(name: 'virtual_layout_id') String virtualLayoutId,
    List<KeymapLayer> layers,
  });
}

/// @nodoc
class __$$KeymapImplCopyWithImpl<$Res>
    extends _$KeymapCopyWithImpl<$Res, _$KeymapImpl>
    implements _$$KeymapImplCopyWith<$Res> {
  __$$KeymapImplCopyWithImpl(
    _$KeymapImpl _value,
    $Res Function(_$KeymapImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of Keymap
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? name = null,
    Object? virtualLayoutId = null,
    Object? layers = null,
  }) {
    return _then(
      _$KeymapImpl(
        id: null == id
            ? _value.id
            : id // ignore: cast_nullable_to_non_nullable
                  as String,
        name: null == name
            ? _value.name
            : name // ignore: cast_nullable_to_non_nullable
                  as String,
        virtualLayoutId: null == virtualLayoutId
            ? _value.virtualLayoutId
            : virtualLayoutId // ignore: cast_nullable_to_non_nullable
                  as String,
        layers: null == layers
            ? _value._layers
            : layers // ignore: cast_nullable_to_non_nullable
                  as List<KeymapLayer>,
      ),
    );
  }
}

/// @nodoc
@JsonSerializable()
class _$KeymapImpl extends _Keymap {
  const _$KeymapImpl({
    required this.id,
    required this.name,
    @JsonKey(name: 'virtual_layout_id') required this.virtualLayoutId,
    final List<KeymapLayer> layers = const <KeymapLayer>[],
  }) : _layers = layers,
       super._();

  factory _$KeymapImpl.fromJson(Map<String, dynamic> json) =>
      _$$KeymapImplFromJson(json);

  @override
  final String id;
  @override
  final String name;
  @override
  @JsonKey(name: 'virtual_layout_id')
  final String virtualLayoutId;
  final List<KeymapLayer> _layers;
  @override
  @JsonKey()
  List<KeymapLayer> get layers {
    if (_layers is EqualUnmodifiableListView) return _layers;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_layers);
  }

  @override
  String toString() {
    return 'Keymap(id: $id, name: $name, virtualLayoutId: $virtualLayoutId, layers: $layers)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$KeymapImpl &&
            (identical(other.id, id) || other.id == id) &&
            (identical(other.name, name) || other.name == name) &&
            (identical(other.virtualLayoutId, virtualLayoutId) ||
                other.virtualLayoutId == virtualLayoutId) &&
            const DeepCollectionEquality().equals(other._layers, _layers));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(
    runtimeType,
    id,
    name,
    virtualLayoutId,
    const DeepCollectionEquality().hash(_layers),
  );

  /// Create a copy of Keymap
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$KeymapImplCopyWith<_$KeymapImpl> get copyWith =>
      __$$KeymapImplCopyWithImpl<_$KeymapImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$KeymapImplToJson(this);
  }
}

abstract class _Keymap extends Keymap {
  const factory _Keymap({
    required final String id,
    required final String name,
    @JsonKey(name: 'virtual_layout_id') required final String virtualLayoutId,
    final List<KeymapLayer> layers,
  }) = _$KeymapImpl;
  const _Keymap._() : super._();

  factory _Keymap.fromJson(Map<String, dynamic> json) = _$KeymapImpl.fromJson;

  @override
  String get id;
  @override
  String get name;
  @override
  @JsonKey(name: 'virtual_layout_id')
  String get virtualLayoutId;
  @override
  List<KeymapLayer> get layers;

  /// Create a copy of Keymap
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$KeymapImplCopyWith<_$KeymapImpl> get copyWith =>
      throw _privateConstructorUsedError;
}
