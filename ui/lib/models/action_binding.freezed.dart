// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'action_binding.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
  'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models',
);

ActionBinding _$ActionBindingFromJson(Map<String, dynamic> json) {
  switch (json['type']) {
    case 'standard_key':
      return ActionBindingStandardKey.fromJson(json);
    case 'macro':
      return ActionBindingMacro.fromJson(json);
    case 'layer_toggle':
      return ActionBindingLayerToggle.fromJson(json);
    case 'transparent':
      return ActionBindingTransparent.fromJson(json);

    default:
      throw CheckedFromJsonException(
        json,
        'type',
        'ActionBinding',
        'Invalid union type "${json['type']}"!',
      );
  }
}

/// @nodoc
mixin _$ActionBinding {
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(@JsonKey(name: 'value') String value) standardKey,
    required TResult Function(@JsonKey(name: 'value') String value) macro,
    required TResult Function(@JsonKey(name: 'value') String value) layerToggle,
    required TResult Function() transparent,
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(@JsonKey(name: 'value') String value)? standardKey,
    TResult? Function(@JsonKey(name: 'value') String value)? macro,
    TResult? Function(@JsonKey(name: 'value') String value)? layerToggle,
    TResult? Function()? transparent,
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(@JsonKey(name: 'value') String value)? standardKey,
    TResult Function(@JsonKey(name: 'value') String value)? macro,
    TResult Function(@JsonKey(name: 'value') String value)? layerToggle,
    TResult Function()? transparent,
    required TResult orElse(),
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(ActionBindingStandardKey value) standardKey,
    required TResult Function(ActionBindingMacro value) macro,
    required TResult Function(ActionBindingLayerToggle value) layerToggle,
    required TResult Function(ActionBindingTransparent value) transparent,
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(ActionBindingStandardKey value)? standardKey,
    TResult? Function(ActionBindingMacro value)? macro,
    TResult? Function(ActionBindingLayerToggle value)? layerToggle,
    TResult? Function(ActionBindingTransparent value)? transparent,
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(ActionBindingStandardKey value)? standardKey,
    TResult Function(ActionBindingMacro value)? macro,
    TResult Function(ActionBindingLayerToggle value)? layerToggle,
    TResult Function(ActionBindingTransparent value)? transparent,
    required TResult orElse(),
  }) => throw _privateConstructorUsedError;

  /// Serializes this ActionBinding to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $ActionBindingCopyWith<$Res> {
  factory $ActionBindingCopyWith(
    ActionBinding value,
    $Res Function(ActionBinding) then,
  ) = _$ActionBindingCopyWithImpl<$Res, ActionBinding>;
}

/// @nodoc
class _$ActionBindingCopyWithImpl<$Res, $Val extends ActionBinding>
    implements $ActionBindingCopyWith<$Res> {
  _$ActionBindingCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of ActionBinding
  /// with the given fields replaced by the non-null parameter values.
}

/// @nodoc
abstract class _$$ActionBindingStandardKeyImplCopyWith<$Res> {
  factory _$$ActionBindingStandardKeyImplCopyWith(
    _$ActionBindingStandardKeyImpl value,
    $Res Function(_$ActionBindingStandardKeyImpl) then,
  ) = __$$ActionBindingStandardKeyImplCopyWithImpl<$Res>;
  @useResult
  $Res call({@JsonKey(name: 'value') String value});
}

/// @nodoc
class __$$ActionBindingStandardKeyImplCopyWithImpl<$Res>
    extends _$ActionBindingCopyWithImpl<$Res, _$ActionBindingStandardKeyImpl>
    implements _$$ActionBindingStandardKeyImplCopyWith<$Res> {
  __$$ActionBindingStandardKeyImplCopyWithImpl(
    _$ActionBindingStandardKeyImpl _value,
    $Res Function(_$ActionBindingStandardKeyImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of ActionBinding
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? value = null}) {
    return _then(
      _$ActionBindingStandardKeyImpl(
        value: null == value
            ? _value.value
            : value // ignore: cast_nullable_to_non_nullable
                  as String,
      ),
    );
  }
}

/// @nodoc
@JsonSerializable()
class _$ActionBindingStandardKeyImpl implements ActionBindingStandardKey {
  const _$ActionBindingStandardKeyImpl({
    @JsonKey(name: 'value') required this.value,
    final String? $type,
  }) : $type = $type ?? 'standard_key';

  factory _$ActionBindingStandardKeyImpl.fromJson(Map<String, dynamic> json) =>
      _$$ActionBindingStandardKeyImplFromJson(json);

  @override
  @JsonKey(name: 'value')
  final String value;

  @JsonKey(name: 'type')
  final String $type;

  @override
  String toString() {
    return 'ActionBinding.standardKey(value: $value)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ActionBindingStandardKeyImpl &&
            (identical(other.value, value) || other.value == value));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, value);

  /// Create a copy of ActionBinding
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$ActionBindingStandardKeyImplCopyWith<_$ActionBindingStandardKeyImpl>
  get copyWith =>
      __$$ActionBindingStandardKeyImplCopyWithImpl<
        _$ActionBindingStandardKeyImpl
      >(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(@JsonKey(name: 'value') String value) standardKey,
    required TResult Function(@JsonKey(name: 'value') String value) macro,
    required TResult Function(@JsonKey(name: 'value') String value) layerToggle,
    required TResult Function() transparent,
  }) {
    return standardKey(value);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(@JsonKey(name: 'value') String value)? standardKey,
    TResult? Function(@JsonKey(name: 'value') String value)? macro,
    TResult? Function(@JsonKey(name: 'value') String value)? layerToggle,
    TResult? Function()? transparent,
  }) {
    return standardKey?.call(value);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(@JsonKey(name: 'value') String value)? standardKey,
    TResult Function(@JsonKey(name: 'value') String value)? macro,
    TResult Function(@JsonKey(name: 'value') String value)? layerToggle,
    TResult Function()? transparent,
    required TResult orElse(),
  }) {
    if (standardKey != null) {
      return standardKey(value);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(ActionBindingStandardKey value) standardKey,
    required TResult Function(ActionBindingMacro value) macro,
    required TResult Function(ActionBindingLayerToggle value) layerToggle,
    required TResult Function(ActionBindingTransparent value) transparent,
  }) {
    return standardKey(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(ActionBindingStandardKey value)? standardKey,
    TResult? Function(ActionBindingMacro value)? macro,
    TResult? Function(ActionBindingLayerToggle value)? layerToggle,
    TResult? Function(ActionBindingTransparent value)? transparent,
  }) {
    return standardKey?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(ActionBindingStandardKey value)? standardKey,
    TResult Function(ActionBindingMacro value)? macro,
    TResult Function(ActionBindingLayerToggle value)? layerToggle,
    TResult Function(ActionBindingTransparent value)? transparent,
    required TResult orElse(),
  }) {
    if (standardKey != null) {
      return standardKey(this);
    }
    return orElse();
  }

  @override
  Map<String, dynamic> toJson() {
    return _$$ActionBindingStandardKeyImplToJson(this);
  }
}

abstract class ActionBindingStandardKey implements ActionBinding {
  const factory ActionBindingStandardKey({
    @JsonKey(name: 'value') required final String value,
  }) = _$ActionBindingStandardKeyImpl;

  factory ActionBindingStandardKey.fromJson(Map<String, dynamic> json) =
      _$ActionBindingStandardKeyImpl.fromJson;

  @JsonKey(name: 'value')
  String get value;

  /// Create a copy of ActionBinding
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ActionBindingStandardKeyImplCopyWith<_$ActionBindingStandardKeyImpl>
  get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$ActionBindingMacroImplCopyWith<$Res> {
  factory _$$ActionBindingMacroImplCopyWith(
    _$ActionBindingMacroImpl value,
    $Res Function(_$ActionBindingMacroImpl) then,
  ) = __$$ActionBindingMacroImplCopyWithImpl<$Res>;
  @useResult
  $Res call({@JsonKey(name: 'value') String value});
}

/// @nodoc
class __$$ActionBindingMacroImplCopyWithImpl<$Res>
    extends _$ActionBindingCopyWithImpl<$Res, _$ActionBindingMacroImpl>
    implements _$$ActionBindingMacroImplCopyWith<$Res> {
  __$$ActionBindingMacroImplCopyWithImpl(
    _$ActionBindingMacroImpl _value,
    $Res Function(_$ActionBindingMacroImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of ActionBinding
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? value = null}) {
    return _then(
      _$ActionBindingMacroImpl(
        value: null == value
            ? _value.value
            : value // ignore: cast_nullable_to_non_nullable
                  as String,
      ),
    );
  }
}

/// @nodoc
@JsonSerializable()
class _$ActionBindingMacroImpl implements ActionBindingMacro {
  const _$ActionBindingMacroImpl({
    @JsonKey(name: 'value') required this.value,
    final String? $type,
  }) : $type = $type ?? 'macro';

  factory _$ActionBindingMacroImpl.fromJson(Map<String, dynamic> json) =>
      _$$ActionBindingMacroImplFromJson(json);

  @override
  @JsonKey(name: 'value')
  final String value;

  @JsonKey(name: 'type')
  final String $type;

  @override
  String toString() {
    return 'ActionBinding.macro(value: $value)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ActionBindingMacroImpl &&
            (identical(other.value, value) || other.value == value));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, value);

  /// Create a copy of ActionBinding
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$ActionBindingMacroImplCopyWith<_$ActionBindingMacroImpl> get copyWith =>
      __$$ActionBindingMacroImplCopyWithImpl<_$ActionBindingMacroImpl>(
        this,
        _$identity,
      );

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(@JsonKey(name: 'value') String value) standardKey,
    required TResult Function(@JsonKey(name: 'value') String value) macro,
    required TResult Function(@JsonKey(name: 'value') String value) layerToggle,
    required TResult Function() transparent,
  }) {
    return macro(value);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(@JsonKey(name: 'value') String value)? standardKey,
    TResult? Function(@JsonKey(name: 'value') String value)? macro,
    TResult? Function(@JsonKey(name: 'value') String value)? layerToggle,
    TResult? Function()? transparent,
  }) {
    return macro?.call(value);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(@JsonKey(name: 'value') String value)? standardKey,
    TResult Function(@JsonKey(name: 'value') String value)? macro,
    TResult Function(@JsonKey(name: 'value') String value)? layerToggle,
    TResult Function()? transparent,
    required TResult orElse(),
  }) {
    if (macro != null) {
      return macro(value);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(ActionBindingStandardKey value) standardKey,
    required TResult Function(ActionBindingMacro value) macro,
    required TResult Function(ActionBindingLayerToggle value) layerToggle,
    required TResult Function(ActionBindingTransparent value) transparent,
  }) {
    return macro(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(ActionBindingStandardKey value)? standardKey,
    TResult? Function(ActionBindingMacro value)? macro,
    TResult? Function(ActionBindingLayerToggle value)? layerToggle,
    TResult? Function(ActionBindingTransparent value)? transparent,
  }) {
    return macro?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(ActionBindingStandardKey value)? standardKey,
    TResult Function(ActionBindingMacro value)? macro,
    TResult Function(ActionBindingLayerToggle value)? layerToggle,
    TResult Function(ActionBindingTransparent value)? transparent,
    required TResult orElse(),
  }) {
    if (macro != null) {
      return macro(this);
    }
    return orElse();
  }

  @override
  Map<String, dynamic> toJson() {
    return _$$ActionBindingMacroImplToJson(this);
  }
}

abstract class ActionBindingMacro implements ActionBinding {
  const factory ActionBindingMacro({
    @JsonKey(name: 'value') required final String value,
  }) = _$ActionBindingMacroImpl;

  factory ActionBindingMacro.fromJson(Map<String, dynamic> json) =
      _$ActionBindingMacroImpl.fromJson;

  @JsonKey(name: 'value')
  String get value;

  /// Create a copy of ActionBinding
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ActionBindingMacroImplCopyWith<_$ActionBindingMacroImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$ActionBindingLayerToggleImplCopyWith<$Res> {
  factory _$$ActionBindingLayerToggleImplCopyWith(
    _$ActionBindingLayerToggleImpl value,
    $Res Function(_$ActionBindingLayerToggleImpl) then,
  ) = __$$ActionBindingLayerToggleImplCopyWithImpl<$Res>;
  @useResult
  $Res call({@JsonKey(name: 'value') String value});
}

/// @nodoc
class __$$ActionBindingLayerToggleImplCopyWithImpl<$Res>
    extends _$ActionBindingCopyWithImpl<$Res, _$ActionBindingLayerToggleImpl>
    implements _$$ActionBindingLayerToggleImplCopyWith<$Res> {
  __$$ActionBindingLayerToggleImplCopyWithImpl(
    _$ActionBindingLayerToggleImpl _value,
    $Res Function(_$ActionBindingLayerToggleImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of ActionBinding
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? value = null}) {
    return _then(
      _$ActionBindingLayerToggleImpl(
        value: null == value
            ? _value.value
            : value // ignore: cast_nullable_to_non_nullable
                  as String,
      ),
    );
  }
}

/// @nodoc
@JsonSerializable()
class _$ActionBindingLayerToggleImpl implements ActionBindingLayerToggle {
  const _$ActionBindingLayerToggleImpl({
    @JsonKey(name: 'value') required this.value,
    final String? $type,
  }) : $type = $type ?? 'layer_toggle';

  factory _$ActionBindingLayerToggleImpl.fromJson(Map<String, dynamic> json) =>
      _$$ActionBindingLayerToggleImplFromJson(json);

  @override
  @JsonKey(name: 'value')
  final String value;

  @JsonKey(name: 'type')
  final String $type;

  @override
  String toString() {
    return 'ActionBinding.layerToggle(value: $value)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ActionBindingLayerToggleImpl &&
            (identical(other.value, value) || other.value == value));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, value);

  /// Create a copy of ActionBinding
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$ActionBindingLayerToggleImplCopyWith<_$ActionBindingLayerToggleImpl>
  get copyWith =>
      __$$ActionBindingLayerToggleImplCopyWithImpl<
        _$ActionBindingLayerToggleImpl
      >(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(@JsonKey(name: 'value') String value) standardKey,
    required TResult Function(@JsonKey(name: 'value') String value) macro,
    required TResult Function(@JsonKey(name: 'value') String value) layerToggle,
    required TResult Function() transparent,
  }) {
    return layerToggle(value);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(@JsonKey(name: 'value') String value)? standardKey,
    TResult? Function(@JsonKey(name: 'value') String value)? macro,
    TResult? Function(@JsonKey(name: 'value') String value)? layerToggle,
    TResult? Function()? transparent,
  }) {
    return layerToggle?.call(value);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(@JsonKey(name: 'value') String value)? standardKey,
    TResult Function(@JsonKey(name: 'value') String value)? macro,
    TResult Function(@JsonKey(name: 'value') String value)? layerToggle,
    TResult Function()? transparent,
    required TResult orElse(),
  }) {
    if (layerToggle != null) {
      return layerToggle(value);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(ActionBindingStandardKey value) standardKey,
    required TResult Function(ActionBindingMacro value) macro,
    required TResult Function(ActionBindingLayerToggle value) layerToggle,
    required TResult Function(ActionBindingTransparent value) transparent,
  }) {
    return layerToggle(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(ActionBindingStandardKey value)? standardKey,
    TResult? Function(ActionBindingMacro value)? macro,
    TResult? Function(ActionBindingLayerToggle value)? layerToggle,
    TResult? Function(ActionBindingTransparent value)? transparent,
  }) {
    return layerToggle?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(ActionBindingStandardKey value)? standardKey,
    TResult Function(ActionBindingMacro value)? macro,
    TResult Function(ActionBindingLayerToggle value)? layerToggle,
    TResult Function(ActionBindingTransparent value)? transparent,
    required TResult orElse(),
  }) {
    if (layerToggle != null) {
      return layerToggle(this);
    }
    return orElse();
  }

  @override
  Map<String, dynamic> toJson() {
    return _$$ActionBindingLayerToggleImplToJson(this);
  }
}

abstract class ActionBindingLayerToggle implements ActionBinding {
  const factory ActionBindingLayerToggle({
    @JsonKey(name: 'value') required final String value,
  }) = _$ActionBindingLayerToggleImpl;

  factory ActionBindingLayerToggle.fromJson(Map<String, dynamic> json) =
      _$ActionBindingLayerToggleImpl.fromJson;

  @JsonKey(name: 'value')
  String get value;

  /// Create a copy of ActionBinding
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ActionBindingLayerToggleImplCopyWith<_$ActionBindingLayerToggleImpl>
  get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$ActionBindingTransparentImplCopyWith<$Res> {
  factory _$$ActionBindingTransparentImplCopyWith(
    _$ActionBindingTransparentImpl value,
    $Res Function(_$ActionBindingTransparentImpl) then,
  ) = __$$ActionBindingTransparentImplCopyWithImpl<$Res>;
}

/// @nodoc
class __$$ActionBindingTransparentImplCopyWithImpl<$Res>
    extends _$ActionBindingCopyWithImpl<$Res, _$ActionBindingTransparentImpl>
    implements _$$ActionBindingTransparentImplCopyWith<$Res> {
  __$$ActionBindingTransparentImplCopyWithImpl(
    _$ActionBindingTransparentImpl _value,
    $Res Function(_$ActionBindingTransparentImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of ActionBinding
  /// with the given fields replaced by the non-null parameter values.
}

/// @nodoc
@JsonSerializable()
class _$ActionBindingTransparentImpl implements ActionBindingTransparent {
  const _$ActionBindingTransparentImpl({final String? $type})
    : $type = $type ?? 'transparent';

  factory _$ActionBindingTransparentImpl.fromJson(Map<String, dynamic> json) =>
      _$$ActionBindingTransparentImplFromJson(json);

  @JsonKey(name: 'type')
  final String $type;

  @override
  String toString() {
    return 'ActionBinding.transparent()';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ActionBindingTransparentImpl);
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => runtimeType.hashCode;

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(@JsonKey(name: 'value') String value) standardKey,
    required TResult Function(@JsonKey(name: 'value') String value) macro,
    required TResult Function(@JsonKey(name: 'value') String value) layerToggle,
    required TResult Function() transparent,
  }) {
    return transparent();
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(@JsonKey(name: 'value') String value)? standardKey,
    TResult? Function(@JsonKey(name: 'value') String value)? macro,
    TResult? Function(@JsonKey(name: 'value') String value)? layerToggle,
    TResult? Function()? transparent,
  }) {
    return transparent?.call();
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(@JsonKey(name: 'value') String value)? standardKey,
    TResult Function(@JsonKey(name: 'value') String value)? macro,
    TResult Function(@JsonKey(name: 'value') String value)? layerToggle,
    TResult Function()? transparent,
    required TResult orElse(),
  }) {
    if (transparent != null) {
      return transparent();
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(ActionBindingStandardKey value) standardKey,
    required TResult Function(ActionBindingMacro value) macro,
    required TResult Function(ActionBindingLayerToggle value) layerToggle,
    required TResult Function(ActionBindingTransparent value) transparent,
  }) {
    return transparent(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(ActionBindingStandardKey value)? standardKey,
    TResult? Function(ActionBindingMacro value)? macro,
    TResult? Function(ActionBindingLayerToggle value)? layerToggle,
    TResult? Function(ActionBindingTransparent value)? transparent,
  }) {
    return transparent?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(ActionBindingStandardKey value)? standardKey,
    TResult Function(ActionBindingMacro value)? macro,
    TResult Function(ActionBindingLayerToggle value)? layerToggle,
    TResult Function(ActionBindingTransparent value)? transparent,
    required TResult orElse(),
  }) {
    if (transparent != null) {
      return transparent(this);
    }
    return orElse();
  }

  @override
  Map<String, dynamic> toJson() {
    return _$$ActionBindingTransparentImplToJson(this);
  }
}

abstract class ActionBindingTransparent implements ActionBinding {
  const factory ActionBindingTransparent() = _$ActionBindingTransparentImpl;

  factory ActionBindingTransparent.fromJson(Map<String, dynamic> json) =
      _$ActionBindingTransparentImpl.fromJson;
}
