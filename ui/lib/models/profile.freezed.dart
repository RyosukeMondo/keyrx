// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'profile.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
  'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models',
);

PhysicalPosition _$PhysicalPositionFromJson(Map<String, dynamic> json) {
  return _PhysicalPosition.fromJson(json);
}

/// @nodoc
mixin _$PhysicalPosition {
  /// Row index (0-based)
  int get row => throw _privateConstructorUsedError;

  /// Column index (0-based)
  int get col => throw _privateConstructorUsedError;

  /// Serializes this PhysicalPosition to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of PhysicalPosition
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $PhysicalPositionCopyWith<PhysicalPosition> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $PhysicalPositionCopyWith<$Res> {
  factory $PhysicalPositionCopyWith(
    PhysicalPosition value,
    $Res Function(PhysicalPosition) then,
  ) = _$PhysicalPositionCopyWithImpl<$Res, PhysicalPosition>;
  @useResult
  $Res call({int row, int col});
}

/// @nodoc
class _$PhysicalPositionCopyWithImpl<$Res, $Val extends PhysicalPosition>
    implements $PhysicalPositionCopyWith<$Res> {
  _$PhysicalPositionCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of PhysicalPosition
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? row = null, Object? col = null}) {
    return _then(
      _value.copyWith(
            row: null == row
                ? _value.row
                : row // ignore: cast_nullable_to_non_nullable
                      as int,
            col: null == col
                ? _value.col
                : col // ignore: cast_nullable_to_non_nullable
                      as int,
          )
          as $Val,
    );
  }
}

/// @nodoc
abstract class _$$PhysicalPositionImplCopyWith<$Res>
    implements $PhysicalPositionCopyWith<$Res> {
  factory _$$PhysicalPositionImplCopyWith(
    _$PhysicalPositionImpl value,
    $Res Function(_$PhysicalPositionImpl) then,
  ) = __$$PhysicalPositionImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({int row, int col});
}

/// @nodoc
class __$$PhysicalPositionImplCopyWithImpl<$Res>
    extends _$PhysicalPositionCopyWithImpl<$Res, _$PhysicalPositionImpl>
    implements _$$PhysicalPositionImplCopyWith<$Res> {
  __$$PhysicalPositionImplCopyWithImpl(
    _$PhysicalPositionImpl _value,
    $Res Function(_$PhysicalPositionImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of PhysicalPosition
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? row = null, Object? col = null}) {
    return _then(
      _$PhysicalPositionImpl(
        row: null == row
            ? _value.row
            : row // ignore: cast_nullable_to_non_nullable
                  as int,
        col: null == col
            ? _value.col
            : col // ignore: cast_nullable_to_non_nullable
                  as int,
      ),
    );
  }
}

/// @nodoc
@JsonSerializable()
class _$PhysicalPositionImpl extends _PhysicalPosition {
  const _$PhysicalPositionImpl({required this.row, required this.col})
    : super._();

  factory _$PhysicalPositionImpl.fromJson(Map<String, dynamic> json) =>
      _$$PhysicalPositionImplFromJson(json);

  /// Row index (0-based)
  @override
  final int row;

  /// Column index (0-based)
  @override
  final int col;

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$PhysicalPositionImpl &&
            (identical(other.row, row) || other.row == row) &&
            (identical(other.col, col) || other.col == col));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, row, col);

  /// Create a copy of PhysicalPosition
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$PhysicalPositionImplCopyWith<_$PhysicalPositionImpl> get copyWith =>
      __$$PhysicalPositionImplCopyWithImpl<_$PhysicalPositionImpl>(
        this,
        _$identity,
      );

  @override
  Map<String, dynamic> toJson() {
    return _$$PhysicalPositionImplToJson(this);
  }
}

abstract class _PhysicalPosition extends PhysicalPosition {
  const factory _PhysicalPosition({
    required final int row,
    required final int col,
  }) = _$PhysicalPositionImpl;
  const _PhysicalPosition._() : super._();

  factory _PhysicalPosition.fromJson(Map<String, dynamic> json) =
      _$PhysicalPositionImpl.fromJson;

  /// Row index (0-based)
  @override
  int get row;

  /// Column index (0-based)
  @override
  int get col;

  /// Create a copy of PhysicalPosition
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$PhysicalPositionImplCopyWith<_$PhysicalPositionImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

KeyAction _$KeyActionFromJson(Map<String, dynamic> json) {
  switch (json['type']) {
    case 'key':
      return KeyActionKey.fromJson(json);
    case 'chord':
      return KeyActionChord.fromJson(json);
    case 'script':
      return KeyActionScript.fromJson(json);
    case 'block':
      return KeyActionBlock.fromJson(json);
    case 'pass':
      return KeyActionPass.fromJson(json);

    default:
      throw CheckedFromJsonException(
        json,
        'type',
        'KeyAction',
        'Invalid union type "${json['type']}"!',
      );
  }
}

/// @nodoc
mixin _$KeyAction {
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String key) key,
    required TResult Function(List<String> keys) chord,
    required TResult Function(String script) script,
    required TResult Function() block,
    required TResult Function() pass,
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String key)? key,
    TResult? Function(List<String> keys)? chord,
    TResult? Function(String script)? script,
    TResult? Function()? block,
    TResult? Function()? pass,
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String key)? key,
    TResult Function(List<String> keys)? chord,
    TResult Function(String script)? script,
    TResult Function()? block,
    TResult Function()? pass,
    required TResult orElse(),
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(KeyActionKey value) key,
    required TResult Function(KeyActionChord value) chord,
    required TResult Function(KeyActionScript value) script,
    required TResult Function(KeyActionBlock value) block,
    required TResult Function(KeyActionPass value) pass,
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(KeyActionKey value)? key,
    TResult? Function(KeyActionChord value)? chord,
    TResult? Function(KeyActionScript value)? script,
    TResult? Function(KeyActionBlock value)? block,
    TResult? Function(KeyActionPass value)? pass,
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(KeyActionKey value)? key,
    TResult Function(KeyActionChord value)? chord,
    TResult Function(KeyActionScript value)? script,
    TResult Function(KeyActionBlock value)? block,
    TResult Function(KeyActionPass value)? pass,
    required TResult orElse(),
  }) => throw _privateConstructorUsedError;

  /// Serializes this KeyAction to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $KeyActionCopyWith<$Res> {
  factory $KeyActionCopyWith(KeyAction value, $Res Function(KeyAction) then) =
      _$KeyActionCopyWithImpl<$Res, KeyAction>;
}

/// @nodoc
class _$KeyActionCopyWithImpl<$Res, $Val extends KeyAction>
    implements $KeyActionCopyWith<$Res> {
  _$KeyActionCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of KeyAction
  /// with the given fields replaced by the non-null parameter values.
}

/// @nodoc
abstract class _$$KeyActionKeyImplCopyWith<$Res> {
  factory _$$KeyActionKeyImplCopyWith(
    _$KeyActionKeyImpl value,
    $Res Function(_$KeyActionKeyImpl) then,
  ) = __$$KeyActionKeyImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String key});
}

/// @nodoc
class __$$KeyActionKeyImplCopyWithImpl<$Res>
    extends _$KeyActionCopyWithImpl<$Res, _$KeyActionKeyImpl>
    implements _$$KeyActionKeyImplCopyWith<$Res> {
  __$$KeyActionKeyImplCopyWithImpl(
    _$KeyActionKeyImpl _value,
    $Res Function(_$KeyActionKeyImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of KeyAction
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? key = null}) {
    return _then(
      _$KeyActionKeyImpl(
        key: null == key
            ? _value.key
            : key // ignore: cast_nullable_to_non_nullable
                  as String,
      ),
    );
  }
}

/// @nodoc
@JsonSerializable()
class _$KeyActionKeyImpl implements KeyActionKey {
  const _$KeyActionKeyImpl({required this.key, final String? $type})
    : $type = $type ?? 'key';

  factory _$KeyActionKeyImpl.fromJson(Map<String, dynamic> json) =>
      _$$KeyActionKeyImplFromJson(json);

  /// The output key to emit
  @override
  final String key;

  @JsonKey(name: 'type')
  final String $type;

  @override
  String toString() {
    return 'KeyAction.key(key: $key)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$KeyActionKeyImpl &&
            (identical(other.key, key) || other.key == key));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, key);

  /// Create a copy of KeyAction
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$KeyActionKeyImplCopyWith<_$KeyActionKeyImpl> get copyWith =>
      __$$KeyActionKeyImplCopyWithImpl<_$KeyActionKeyImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String key) key,
    required TResult Function(List<String> keys) chord,
    required TResult Function(String script) script,
    required TResult Function() block,
    required TResult Function() pass,
  }) {
    return key(this.key);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String key)? key,
    TResult? Function(List<String> keys)? chord,
    TResult? Function(String script)? script,
    TResult? Function()? block,
    TResult? Function()? pass,
  }) {
    return key?.call(this.key);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String key)? key,
    TResult Function(List<String> keys)? chord,
    TResult Function(String script)? script,
    TResult Function()? block,
    TResult Function()? pass,
    required TResult orElse(),
  }) {
    if (key != null) {
      return key(this.key);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(KeyActionKey value) key,
    required TResult Function(KeyActionChord value) chord,
    required TResult Function(KeyActionScript value) script,
    required TResult Function(KeyActionBlock value) block,
    required TResult Function(KeyActionPass value) pass,
  }) {
    return key(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(KeyActionKey value)? key,
    TResult? Function(KeyActionChord value)? chord,
    TResult? Function(KeyActionScript value)? script,
    TResult? Function(KeyActionBlock value)? block,
    TResult? Function(KeyActionPass value)? pass,
  }) {
    return key?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(KeyActionKey value)? key,
    TResult Function(KeyActionChord value)? chord,
    TResult Function(KeyActionScript value)? script,
    TResult Function(KeyActionBlock value)? block,
    TResult Function(KeyActionPass value)? pass,
    required TResult orElse(),
  }) {
    if (key != null) {
      return key(this);
    }
    return orElse();
  }

  @override
  Map<String, dynamic> toJson() {
    return _$$KeyActionKeyImplToJson(this);
  }
}

abstract class KeyActionKey implements KeyAction {
  const factory KeyActionKey({required final String key}) = _$KeyActionKeyImpl;

  factory KeyActionKey.fromJson(Map<String, dynamic> json) =
      _$KeyActionKeyImpl.fromJson;

  /// The output key to emit
  String get key;

  /// Create a copy of KeyAction
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$KeyActionKeyImplCopyWith<_$KeyActionKeyImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$KeyActionChordImplCopyWith<$Res> {
  factory _$$KeyActionChordImplCopyWith(
    _$KeyActionChordImpl value,
    $Res Function(_$KeyActionChordImpl) then,
  ) = __$$KeyActionChordImplCopyWithImpl<$Res>;
  @useResult
  $Res call({List<String> keys});
}

/// @nodoc
class __$$KeyActionChordImplCopyWithImpl<$Res>
    extends _$KeyActionCopyWithImpl<$Res, _$KeyActionChordImpl>
    implements _$$KeyActionChordImplCopyWith<$Res> {
  __$$KeyActionChordImplCopyWithImpl(
    _$KeyActionChordImpl _value,
    $Res Function(_$KeyActionChordImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of KeyAction
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? keys = null}) {
    return _then(
      _$KeyActionChordImpl(
        keys: null == keys
            ? _value._keys
            : keys // ignore: cast_nullable_to_non_nullable
                  as List<String>,
      ),
    );
  }
}

/// @nodoc
@JsonSerializable()
class _$KeyActionChordImpl implements KeyActionChord {
  const _$KeyActionChordImpl({
    required final List<String> keys,
    final String? $type,
  }) : _keys = keys,
       $type = $type ?? 'chord';

  factory _$KeyActionChordImpl.fromJson(Map<String, dynamic> json) =>
      _$$KeyActionChordImplFromJson(json);

  /// Keys to press together
  final List<String> _keys;

  /// Keys to press together
  @override
  List<String> get keys {
    if (_keys is EqualUnmodifiableListView) return _keys;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_keys);
  }

  @JsonKey(name: 'type')
  final String $type;

  @override
  String toString() {
    return 'KeyAction.chord(keys: $keys)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$KeyActionChordImpl &&
            const DeepCollectionEquality().equals(other._keys, _keys));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode =>
      Object.hash(runtimeType, const DeepCollectionEquality().hash(_keys));

  /// Create a copy of KeyAction
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$KeyActionChordImplCopyWith<_$KeyActionChordImpl> get copyWith =>
      __$$KeyActionChordImplCopyWithImpl<_$KeyActionChordImpl>(
        this,
        _$identity,
      );

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String key) key,
    required TResult Function(List<String> keys) chord,
    required TResult Function(String script) script,
    required TResult Function() block,
    required TResult Function() pass,
  }) {
    return chord(keys);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String key)? key,
    TResult? Function(List<String> keys)? chord,
    TResult? Function(String script)? script,
    TResult? Function()? block,
    TResult? Function()? pass,
  }) {
    return chord?.call(keys);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String key)? key,
    TResult Function(List<String> keys)? chord,
    TResult Function(String script)? script,
    TResult Function()? block,
    TResult Function()? pass,
    required TResult orElse(),
  }) {
    if (chord != null) {
      return chord(keys);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(KeyActionKey value) key,
    required TResult Function(KeyActionChord value) chord,
    required TResult Function(KeyActionScript value) script,
    required TResult Function(KeyActionBlock value) block,
    required TResult Function(KeyActionPass value) pass,
  }) {
    return chord(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(KeyActionKey value)? key,
    TResult? Function(KeyActionChord value)? chord,
    TResult? Function(KeyActionScript value)? script,
    TResult? Function(KeyActionBlock value)? block,
    TResult? Function(KeyActionPass value)? pass,
  }) {
    return chord?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(KeyActionKey value)? key,
    TResult Function(KeyActionChord value)? chord,
    TResult Function(KeyActionScript value)? script,
    TResult Function(KeyActionBlock value)? block,
    TResult Function(KeyActionPass value)? pass,
    required TResult orElse(),
  }) {
    if (chord != null) {
      return chord(this);
    }
    return orElse();
  }

  @override
  Map<String, dynamic> toJson() {
    return _$$KeyActionChordImplToJson(this);
  }
}

abstract class KeyActionChord implements KeyAction {
  const factory KeyActionChord({required final List<String> keys}) =
      _$KeyActionChordImpl;

  factory KeyActionChord.fromJson(Map<String, dynamic> json) =
      _$KeyActionChordImpl.fromJson;

  /// Keys to press together
  List<String> get keys;

  /// Create a copy of KeyAction
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$KeyActionChordImplCopyWith<_$KeyActionChordImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$KeyActionScriptImplCopyWith<$Res> {
  factory _$$KeyActionScriptImplCopyWith(
    _$KeyActionScriptImpl value,
    $Res Function(_$KeyActionScriptImpl) then,
  ) = __$$KeyActionScriptImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String script});
}

/// @nodoc
class __$$KeyActionScriptImplCopyWithImpl<$Res>
    extends _$KeyActionCopyWithImpl<$Res, _$KeyActionScriptImpl>
    implements _$$KeyActionScriptImplCopyWith<$Res> {
  __$$KeyActionScriptImplCopyWithImpl(
    _$KeyActionScriptImpl _value,
    $Res Function(_$KeyActionScriptImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of KeyAction
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? script = null}) {
    return _then(
      _$KeyActionScriptImpl(
        script: null == script
            ? _value.script
            : script // ignore: cast_nullable_to_non_nullable
                  as String,
      ),
    );
  }
}

/// @nodoc
@JsonSerializable()
class _$KeyActionScriptImpl implements KeyActionScript {
  const _$KeyActionScriptImpl({required this.script, final String? $type})
    : $type = $type ?? 'script';

  factory _$KeyActionScriptImpl.fromJson(Map<String, dynamic> json) =>
      _$$KeyActionScriptImplFromJson(json);

  /// Script identifier or command to run
  @override
  final String script;

  @JsonKey(name: 'type')
  final String $type;

  @override
  String toString() {
    return 'KeyAction.script(script: $script)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$KeyActionScriptImpl &&
            (identical(other.script, script) || other.script == script));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, script);

  /// Create a copy of KeyAction
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$KeyActionScriptImplCopyWith<_$KeyActionScriptImpl> get copyWith =>
      __$$KeyActionScriptImplCopyWithImpl<_$KeyActionScriptImpl>(
        this,
        _$identity,
      );

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String key) key,
    required TResult Function(List<String> keys) chord,
    required TResult Function(String script) script,
    required TResult Function() block,
    required TResult Function() pass,
  }) {
    return script(this.script);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String key)? key,
    TResult? Function(List<String> keys)? chord,
    TResult? Function(String script)? script,
    TResult? Function()? block,
    TResult? Function()? pass,
  }) {
    return script?.call(this.script);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String key)? key,
    TResult Function(List<String> keys)? chord,
    TResult Function(String script)? script,
    TResult Function()? block,
    TResult Function()? pass,
    required TResult orElse(),
  }) {
    if (script != null) {
      return script(this.script);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(KeyActionKey value) key,
    required TResult Function(KeyActionChord value) chord,
    required TResult Function(KeyActionScript value) script,
    required TResult Function(KeyActionBlock value) block,
    required TResult Function(KeyActionPass value) pass,
  }) {
    return script(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(KeyActionKey value)? key,
    TResult? Function(KeyActionChord value)? chord,
    TResult? Function(KeyActionScript value)? script,
    TResult? Function(KeyActionBlock value)? block,
    TResult? Function(KeyActionPass value)? pass,
  }) {
    return script?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(KeyActionKey value)? key,
    TResult Function(KeyActionChord value)? chord,
    TResult Function(KeyActionScript value)? script,
    TResult Function(KeyActionBlock value)? block,
    TResult Function(KeyActionPass value)? pass,
    required TResult orElse(),
  }) {
    if (script != null) {
      return script(this);
    }
    return orElse();
  }

  @override
  Map<String, dynamic> toJson() {
    return _$$KeyActionScriptImplToJson(this);
  }
}

abstract class KeyActionScript implements KeyAction {
  const factory KeyActionScript({required final String script}) =
      _$KeyActionScriptImpl;

  factory KeyActionScript.fromJson(Map<String, dynamic> json) =
      _$KeyActionScriptImpl.fromJson;

  /// Script identifier or command to run
  String get script;

  /// Create a copy of KeyAction
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$KeyActionScriptImplCopyWith<_$KeyActionScriptImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$KeyActionBlockImplCopyWith<$Res> {
  factory _$$KeyActionBlockImplCopyWith(
    _$KeyActionBlockImpl value,
    $Res Function(_$KeyActionBlockImpl) then,
  ) = __$$KeyActionBlockImplCopyWithImpl<$Res>;
}

/// @nodoc
class __$$KeyActionBlockImplCopyWithImpl<$Res>
    extends _$KeyActionCopyWithImpl<$Res, _$KeyActionBlockImpl>
    implements _$$KeyActionBlockImplCopyWith<$Res> {
  __$$KeyActionBlockImplCopyWithImpl(
    _$KeyActionBlockImpl _value,
    $Res Function(_$KeyActionBlockImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of KeyAction
  /// with the given fields replaced by the non-null parameter values.
}

/// @nodoc
@JsonSerializable()
class _$KeyActionBlockImpl implements KeyActionBlock {
  const _$KeyActionBlockImpl({final String? $type}) : $type = $type ?? 'block';

  factory _$KeyActionBlockImpl.fromJson(Map<String, dynamic> json) =>
      _$$KeyActionBlockImplFromJson(json);

  @JsonKey(name: 'type')
  final String $type;

  @override
  String toString() {
    return 'KeyAction.block()';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType && other is _$KeyActionBlockImpl);
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => runtimeType.hashCode;

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String key) key,
    required TResult Function(List<String> keys) chord,
    required TResult Function(String script) script,
    required TResult Function() block,
    required TResult Function() pass,
  }) {
    return block();
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String key)? key,
    TResult? Function(List<String> keys)? chord,
    TResult? Function(String script)? script,
    TResult? Function()? block,
    TResult? Function()? pass,
  }) {
    return block?.call();
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String key)? key,
    TResult Function(List<String> keys)? chord,
    TResult Function(String script)? script,
    TResult Function()? block,
    TResult Function()? pass,
    required TResult orElse(),
  }) {
    if (block != null) {
      return block();
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(KeyActionKey value) key,
    required TResult Function(KeyActionChord value) chord,
    required TResult Function(KeyActionScript value) script,
    required TResult Function(KeyActionBlock value) block,
    required TResult Function(KeyActionPass value) pass,
  }) {
    return block(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(KeyActionKey value)? key,
    TResult? Function(KeyActionChord value)? chord,
    TResult? Function(KeyActionScript value)? script,
    TResult? Function(KeyActionBlock value)? block,
    TResult? Function(KeyActionPass value)? pass,
  }) {
    return block?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(KeyActionKey value)? key,
    TResult Function(KeyActionChord value)? chord,
    TResult Function(KeyActionScript value)? script,
    TResult Function(KeyActionBlock value)? block,
    TResult Function(KeyActionPass value)? pass,
    required TResult orElse(),
  }) {
    if (block != null) {
      return block(this);
    }
    return orElse();
  }

  @override
  Map<String, dynamic> toJson() {
    return _$$KeyActionBlockImplToJson(this);
  }
}

abstract class KeyActionBlock implements KeyAction {
  const factory KeyActionBlock() = _$KeyActionBlockImpl;

  factory KeyActionBlock.fromJson(Map<String, dynamic> json) =
      _$KeyActionBlockImpl.fromJson;
}

/// @nodoc
abstract class _$$KeyActionPassImplCopyWith<$Res> {
  factory _$$KeyActionPassImplCopyWith(
    _$KeyActionPassImpl value,
    $Res Function(_$KeyActionPassImpl) then,
  ) = __$$KeyActionPassImplCopyWithImpl<$Res>;
}

/// @nodoc
class __$$KeyActionPassImplCopyWithImpl<$Res>
    extends _$KeyActionCopyWithImpl<$Res, _$KeyActionPassImpl>
    implements _$$KeyActionPassImplCopyWith<$Res> {
  __$$KeyActionPassImplCopyWithImpl(
    _$KeyActionPassImpl _value,
    $Res Function(_$KeyActionPassImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of KeyAction
  /// with the given fields replaced by the non-null parameter values.
}

/// @nodoc
@JsonSerializable()
class _$KeyActionPassImpl implements KeyActionPass {
  const _$KeyActionPassImpl({final String? $type}) : $type = $type ?? 'pass';

  factory _$KeyActionPassImpl.fromJson(Map<String, dynamic> json) =>
      _$$KeyActionPassImplFromJson(json);

  @JsonKey(name: 'type')
  final String $type;

  @override
  String toString() {
    return 'KeyAction.pass()';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType && other is _$KeyActionPassImpl);
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => runtimeType.hashCode;

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String key) key,
    required TResult Function(List<String> keys) chord,
    required TResult Function(String script) script,
    required TResult Function() block,
    required TResult Function() pass,
  }) {
    return pass();
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String key)? key,
    TResult? Function(List<String> keys)? chord,
    TResult? Function(String script)? script,
    TResult? Function()? block,
    TResult? Function()? pass,
  }) {
    return pass?.call();
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String key)? key,
    TResult Function(List<String> keys)? chord,
    TResult Function(String script)? script,
    TResult Function()? block,
    TResult Function()? pass,
    required TResult orElse(),
  }) {
    if (pass != null) {
      return pass();
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(KeyActionKey value) key,
    required TResult Function(KeyActionChord value) chord,
    required TResult Function(KeyActionScript value) script,
    required TResult Function(KeyActionBlock value) block,
    required TResult Function(KeyActionPass value) pass,
  }) {
    return pass(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(KeyActionKey value)? key,
    TResult? Function(KeyActionChord value)? chord,
    TResult? Function(KeyActionScript value)? script,
    TResult? Function(KeyActionBlock value)? block,
    TResult? Function(KeyActionPass value)? pass,
  }) {
    return pass?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(KeyActionKey value)? key,
    TResult Function(KeyActionChord value)? chord,
    TResult Function(KeyActionScript value)? script,
    TResult Function(KeyActionBlock value)? block,
    TResult Function(KeyActionPass value)? pass,
    required TResult orElse(),
  }) {
    if (pass != null) {
      return pass(this);
    }
    return orElse();
  }

  @override
  Map<String, dynamic> toJson() {
    return _$$KeyActionPassImplToJson(this);
  }
}

abstract class KeyActionPass implements KeyAction {
  const factory KeyActionPass() = _$KeyActionPassImpl;

  factory KeyActionPass.fromJson(Map<String, dynamic> json) =
      _$KeyActionPassImpl.fromJson;
}

Profile _$ProfileFromJson(Map<String, dynamic> json) {
  return _Profile.fromJson(json);
}

/// @nodoc
mixin _$Profile {
  /// Unique identifier (UUID v4)
  String get id => throw _privateConstructorUsedError;

  /// Human-readable profile name
  String get name => throw _privateConstructorUsedError;

  /// Layout type this profile is designed for
  @JsonKey(name: 'layout_type')
  LayoutType get layoutType => throw _privateConstructorUsedError;

  /// Key mappings: physical position key → action
  /// Only contains entries for remapped keys (sparse map)
  /// Serialized as {"row,col": action} in JSON
  Map<String, KeyAction> get mappings => throw _privateConstructorUsedError;

  /// Creation timestamp (ISO 8601)
  @JsonKey(name: 'created_at')
  String get createdAt => throw _privateConstructorUsedError;

  /// Last modification timestamp (ISO 8601)
  @JsonKey(name: 'updated_at')
  String get updatedAt => throw _privateConstructorUsedError;

  /// Serializes this Profile to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of Profile
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $ProfileCopyWith<Profile> get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $ProfileCopyWith<$Res> {
  factory $ProfileCopyWith(Profile value, $Res Function(Profile) then) =
      _$ProfileCopyWithImpl<$Res, Profile>;
  @useResult
  $Res call({
    String id,
    String name,
    @JsonKey(name: 'layout_type') LayoutType layoutType,
    Map<String, KeyAction> mappings,
    @JsonKey(name: 'created_at') String createdAt,
    @JsonKey(name: 'updated_at') String updatedAt,
  });
}

/// @nodoc
class _$ProfileCopyWithImpl<$Res, $Val extends Profile>
    implements $ProfileCopyWith<$Res> {
  _$ProfileCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of Profile
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? name = null,
    Object? layoutType = null,
    Object? mappings = null,
    Object? createdAt = null,
    Object? updatedAt = null,
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
                      as LayoutType,
            mappings: null == mappings
                ? _value.mappings
                : mappings // ignore: cast_nullable_to_non_nullable
                      as Map<String, KeyAction>,
            createdAt: null == createdAt
                ? _value.createdAt
                : createdAt // ignore: cast_nullable_to_non_nullable
                      as String,
            updatedAt: null == updatedAt
                ? _value.updatedAt
                : updatedAt // ignore: cast_nullable_to_non_nullable
                      as String,
          )
          as $Val,
    );
  }
}

/// @nodoc
abstract class _$$ProfileImplCopyWith<$Res> implements $ProfileCopyWith<$Res> {
  factory _$$ProfileImplCopyWith(
    _$ProfileImpl value,
    $Res Function(_$ProfileImpl) then,
  ) = __$$ProfileImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({
    String id,
    String name,
    @JsonKey(name: 'layout_type') LayoutType layoutType,
    Map<String, KeyAction> mappings,
    @JsonKey(name: 'created_at') String createdAt,
    @JsonKey(name: 'updated_at') String updatedAt,
  });
}

/// @nodoc
class __$$ProfileImplCopyWithImpl<$Res>
    extends _$ProfileCopyWithImpl<$Res, _$ProfileImpl>
    implements _$$ProfileImplCopyWith<$Res> {
  __$$ProfileImplCopyWithImpl(
    _$ProfileImpl _value,
    $Res Function(_$ProfileImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of Profile
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? name = null,
    Object? layoutType = null,
    Object? mappings = null,
    Object? createdAt = null,
    Object? updatedAt = null,
  }) {
    return _then(
      _$ProfileImpl(
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
                  as LayoutType,
        mappings: null == mappings
            ? _value._mappings
            : mappings // ignore: cast_nullable_to_non_nullable
                  as Map<String, KeyAction>,
        createdAt: null == createdAt
            ? _value.createdAt
            : createdAt // ignore: cast_nullable_to_non_nullable
                  as String,
        updatedAt: null == updatedAt
            ? _value.updatedAt
            : updatedAt // ignore: cast_nullable_to_non_nullable
                  as String,
      ),
    );
  }
}

/// @nodoc
@JsonSerializable()
class _$ProfileImpl extends _Profile {
  const _$ProfileImpl({
    required this.id,
    required this.name,
    @JsonKey(name: 'layout_type') required this.layoutType,
    final Map<String, KeyAction> mappings = const {},
    @JsonKey(name: 'created_at') required this.createdAt,
    @JsonKey(name: 'updated_at') required this.updatedAt,
  }) : _mappings = mappings,
       super._();

  factory _$ProfileImpl.fromJson(Map<String, dynamic> json) =>
      _$$ProfileImplFromJson(json);

  /// Unique identifier (UUID v4)
  @override
  final String id;

  /// Human-readable profile name
  @override
  final String name;

  /// Layout type this profile is designed for
  @override
  @JsonKey(name: 'layout_type')
  final LayoutType layoutType;

  /// Key mappings: physical position key → action
  /// Only contains entries for remapped keys (sparse map)
  /// Serialized as {"row,col": action} in JSON
  final Map<String, KeyAction> _mappings;

  /// Key mappings: physical position key → action
  /// Only contains entries for remapped keys (sparse map)
  /// Serialized as {"row,col": action} in JSON
  @override
  @JsonKey()
  Map<String, KeyAction> get mappings {
    if (_mappings is EqualUnmodifiableMapView) return _mappings;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableMapView(_mappings);
  }

  /// Creation timestamp (ISO 8601)
  @override
  @JsonKey(name: 'created_at')
  final String createdAt;

  /// Last modification timestamp (ISO 8601)
  @override
  @JsonKey(name: 'updated_at')
  final String updatedAt;

  @override
  String toString() {
    return 'Profile(id: $id, name: $name, layoutType: $layoutType, mappings: $mappings, createdAt: $createdAt, updatedAt: $updatedAt)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ProfileImpl &&
            (identical(other.id, id) || other.id == id) &&
            (identical(other.name, name) || other.name == name) &&
            (identical(other.layoutType, layoutType) ||
                other.layoutType == layoutType) &&
            const DeepCollectionEquality().equals(other._mappings, _mappings) &&
            (identical(other.createdAt, createdAt) ||
                other.createdAt == createdAt) &&
            (identical(other.updatedAt, updatedAt) ||
                other.updatedAt == updatedAt));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(
    runtimeType,
    id,
    name,
    layoutType,
    const DeepCollectionEquality().hash(_mappings),
    createdAt,
    updatedAt,
  );

  /// Create a copy of Profile
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$ProfileImplCopyWith<_$ProfileImpl> get copyWith =>
      __$$ProfileImplCopyWithImpl<_$ProfileImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$ProfileImplToJson(this);
  }
}

abstract class _Profile extends Profile {
  const factory _Profile({
    required final String id,
    required final String name,
    @JsonKey(name: 'layout_type') required final LayoutType layoutType,
    final Map<String, KeyAction> mappings,
    @JsonKey(name: 'created_at') required final String createdAt,
    @JsonKey(name: 'updated_at') required final String updatedAt,
  }) = _$ProfileImpl;
  const _Profile._() : super._();

  factory _Profile.fromJson(Map<String, dynamic> json) = _$ProfileImpl.fromJson;

  /// Unique identifier (UUID v4)
  @override
  String get id;

  /// Human-readable profile name
  @override
  String get name;

  /// Layout type this profile is designed for
  @override
  @JsonKey(name: 'layout_type')
  LayoutType get layoutType;

  /// Key mappings: physical position key → action
  /// Only contains entries for remapped keys (sparse map)
  /// Serialized as {"row,col": action} in JSON
  @override
  Map<String, KeyAction> get mappings;

  /// Creation timestamp (ISO 8601)
  @override
  @JsonKey(name: 'created_at')
  String get createdAt;

  /// Last modification timestamp (ISO 8601)
  @override
  @JsonKey(name: 'updated_at')
  String get updatedAt;

  /// Create a copy of Profile
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ProfileImplCopyWith<_$ProfileImpl> get copyWith =>
      throw _privateConstructorUsedError;
}
