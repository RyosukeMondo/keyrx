// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'profile.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$PhysicalPosition {

/// Row index (0-based)
 int get row;/// Column index (0-based)
 int get col;
/// Create a copy of PhysicalPosition
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PhysicalPositionCopyWith<PhysicalPosition> get copyWith => _$PhysicalPositionCopyWithImpl<PhysicalPosition>(this as PhysicalPosition, _$identity);

  /// Serializes this PhysicalPosition to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PhysicalPosition&&(identical(other.row, row) || other.row == row)&&(identical(other.col, col) || other.col == col));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,row,col);



}

/// @nodoc
abstract mixin class $PhysicalPositionCopyWith<$Res>  {
  factory $PhysicalPositionCopyWith(PhysicalPosition value, $Res Function(PhysicalPosition) _then) = _$PhysicalPositionCopyWithImpl;
@useResult
$Res call({
 int row, int col
});




}
/// @nodoc
class _$PhysicalPositionCopyWithImpl<$Res>
    implements $PhysicalPositionCopyWith<$Res> {
  _$PhysicalPositionCopyWithImpl(this._self, this._then);

  final PhysicalPosition _self;
  final $Res Function(PhysicalPosition) _then;

/// Create a copy of PhysicalPosition
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? row = null,Object? col = null,}) {
  return _then(_self.copyWith(
row: null == row ? _self.row : row // ignore: cast_nullable_to_non_nullable
as int,col: null == col ? _self.col : col // ignore: cast_nullable_to_non_nullable
as int,
  ));
}

}


/// Adds pattern-matching-related methods to [PhysicalPosition].
extension PhysicalPositionPatterns on PhysicalPosition {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _PhysicalPosition value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _PhysicalPosition() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _PhysicalPosition value)  $default,){
final _that = this;
switch (_that) {
case _PhysicalPosition():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _PhysicalPosition value)?  $default,){
final _that = this;
switch (_that) {
case _PhysicalPosition() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( int row,  int col)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _PhysicalPosition() when $default != null:
return $default(_that.row,_that.col);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( int row,  int col)  $default,) {final _that = this;
switch (_that) {
case _PhysicalPosition():
return $default(_that.row,_that.col);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( int row,  int col)?  $default,) {final _that = this;
switch (_that) {
case _PhysicalPosition() when $default != null:
return $default(_that.row,_that.col);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _PhysicalPosition extends PhysicalPosition {
  const _PhysicalPosition({required this.row, required this.col}): super._();
  factory _PhysicalPosition.fromJson(Map<String, dynamic> json) => _$PhysicalPositionFromJson(json);

/// Row index (0-based)
@override final  int row;
/// Column index (0-based)
@override final  int col;

/// Create a copy of PhysicalPosition
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$PhysicalPositionCopyWith<_PhysicalPosition> get copyWith => __$PhysicalPositionCopyWithImpl<_PhysicalPosition>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$PhysicalPositionToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _PhysicalPosition&&(identical(other.row, row) || other.row == row)&&(identical(other.col, col) || other.col == col));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,row,col);



}

/// @nodoc
abstract mixin class _$PhysicalPositionCopyWith<$Res> implements $PhysicalPositionCopyWith<$Res> {
  factory _$PhysicalPositionCopyWith(_PhysicalPosition value, $Res Function(_PhysicalPosition) _then) = __$PhysicalPositionCopyWithImpl;
@override @useResult
$Res call({
 int row, int col
});




}
/// @nodoc
class __$PhysicalPositionCopyWithImpl<$Res>
    implements _$PhysicalPositionCopyWith<$Res> {
  __$PhysicalPositionCopyWithImpl(this._self, this._then);

  final _PhysicalPosition _self;
  final $Res Function(_PhysicalPosition) _then;

/// Create a copy of PhysicalPosition
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? row = null,Object? col = null,}) {
  return _then(_PhysicalPosition(
row: null == row ? _self.row : row // ignore: cast_nullable_to_non_nullable
as int,col: null == col ? _self.col : col // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

KeyAction _$KeyActionFromJson(
  Map<String, dynamic> json
) {
        switch (json['type']) {
                  case 'key':
          return KeyActionKey.fromJson(
            json
          );
                case 'chord':
          return KeyActionChord.fromJson(
            json
          );
                case 'script':
          return KeyActionScript.fromJson(
            json
          );
                case 'block':
          return KeyActionBlock.fromJson(
            json
          );
                case 'pass':
          return KeyActionPass.fromJson(
            json
          );
        
          default:
            throw CheckedFromJsonException(
  json,
  'type',
  'KeyAction',
  'Invalid union type "${json['type']}"!'
);
        }
      
}

/// @nodoc
mixin _$KeyAction {



  /// Serializes this KeyAction to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is KeyAction);
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'KeyAction()';
}


}

/// @nodoc
class $KeyActionCopyWith<$Res>  {
$KeyActionCopyWith(KeyAction _, $Res Function(KeyAction) __);
}


/// Adds pattern-matching-related methods to [KeyAction].
extension KeyActionPatterns on KeyAction {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( KeyActionKey value)?  key,TResult Function( KeyActionChord value)?  chord,TResult Function( KeyActionScript value)?  script,TResult Function( KeyActionBlock value)?  block,TResult Function( KeyActionPass value)?  pass,required TResult orElse(),}){
final _that = this;
switch (_that) {
case KeyActionKey() when key != null:
return key(_that);case KeyActionChord() when chord != null:
return chord(_that);case KeyActionScript() when script != null:
return script(_that);case KeyActionBlock() when block != null:
return block(_that);case KeyActionPass() when pass != null:
return pass(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( KeyActionKey value)  key,required TResult Function( KeyActionChord value)  chord,required TResult Function( KeyActionScript value)  script,required TResult Function( KeyActionBlock value)  block,required TResult Function( KeyActionPass value)  pass,}){
final _that = this;
switch (_that) {
case KeyActionKey():
return key(_that);case KeyActionChord():
return chord(_that);case KeyActionScript():
return script(_that);case KeyActionBlock():
return block(_that);case KeyActionPass():
return pass(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( KeyActionKey value)?  key,TResult? Function( KeyActionChord value)?  chord,TResult? Function( KeyActionScript value)?  script,TResult? Function( KeyActionBlock value)?  block,TResult? Function( KeyActionPass value)?  pass,}){
final _that = this;
switch (_that) {
case KeyActionKey() when key != null:
return key(_that);case KeyActionChord() when chord != null:
return chord(_that);case KeyActionScript() when script != null:
return script(_that);case KeyActionBlock() when block != null:
return block(_that);case KeyActionPass() when pass != null:
return pass(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String key)?  key,TResult Function( List<String> keys)?  chord,TResult Function( String script)?  script,TResult Function()?  block,TResult Function()?  pass,required TResult orElse(),}) {final _that = this;
switch (_that) {
case KeyActionKey() when key != null:
return key(_that.key);case KeyActionChord() when chord != null:
return chord(_that.keys);case KeyActionScript() when script != null:
return script(_that.script);case KeyActionBlock() when block != null:
return block();case KeyActionPass() when pass != null:
return pass();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String key)  key,required TResult Function( List<String> keys)  chord,required TResult Function( String script)  script,required TResult Function()  block,required TResult Function()  pass,}) {final _that = this;
switch (_that) {
case KeyActionKey():
return key(_that.key);case KeyActionChord():
return chord(_that.keys);case KeyActionScript():
return script(_that.script);case KeyActionBlock():
return block();case KeyActionPass():
return pass();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String key)?  key,TResult? Function( List<String> keys)?  chord,TResult? Function( String script)?  script,TResult? Function()?  block,TResult? Function()?  pass,}) {final _that = this;
switch (_that) {
case KeyActionKey() when key != null:
return key(_that.key);case KeyActionChord() when chord != null:
return chord(_that.keys);case KeyActionScript() when script != null:
return script(_that.script);case KeyActionBlock() when block != null:
return block();case KeyActionPass() when pass != null:
return pass();case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class KeyActionKey implements KeyAction {
  const KeyActionKey({required this.key, final  String? $type}): $type = $type ?? 'key';
  factory KeyActionKey.fromJson(Map<String, dynamic> json) => _$KeyActionKeyFromJson(json);

/// The output key to emit
 final  String key;

@JsonKey(name: 'type')
final String $type;


/// Create a copy of KeyAction
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$KeyActionKeyCopyWith<KeyActionKey> get copyWith => _$KeyActionKeyCopyWithImpl<KeyActionKey>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$KeyActionKeyToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is KeyActionKey&&(identical(other.key, key) || other.key == key));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,key);

@override
String toString() {
  return 'KeyAction.key(key: $key)';
}


}

/// @nodoc
abstract mixin class $KeyActionKeyCopyWith<$Res> implements $KeyActionCopyWith<$Res> {
  factory $KeyActionKeyCopyWith(KeyActionKey value, $Res Function(KeyActionKey) _then) = _$KeyActionKeyCopyWithImpl;
@useResult
$Res call({
 String key
});




}
/// @nodoc
class _$KeyActionKeyCopyWithImpl<$Res>
    implements $KeyActionKeyCopyWith<$Res> {
  _$KeyActionKeyCopyWithImpl(this._self, this._then);

  final KeyActionKey _self;
  final $Res Function(KeyActionKey) _then;

/// Create a copy of KeyAction
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? key = null,}) {
  return _then(KeyActionKey(
key: null == key ? _self.key : key // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc
@JsonSerializable()

class KeyActionChord implements KeyAction {
  const KeyActionChord({required final  List<String> keys, final  String? $type}): _keys = keys,$type = $type ?? 'chord';
  factory KeyActionChord.fromJson(Map<String, dynamic> json) => _$KeyActionChordFromJson(json);

/// Keys to press together
 final  List<String> _keys;
/// Keys to press together
 List<String> get keys {
  if (_keys is EqualUnmodifiableListView) return _keys;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_keys);
}


@JsonKey(name: 'type')
final String $type;


/// Create a copy of KeyAction
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$KeyActionChordCopyWith<KeyActionChord> get copyWith => _$KeyActionChordCopyWithImpl<KeyActionChord>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$KeyActionChordToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is KeyActionChord&&const DeepCollectionEquality().equals(other._keys, _keys));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_keys));

@override
String toString() {
  return 'KeyAction.chord(keys: $keys)';
}


}

/// @nodoc
abstract mixin class $KeyActionChordCopyWith<$Res> implements $KeyActionCopyWith<$Res> {
  factory $KeyActionChordCopyWith(KeyActionChord value, $Res Function(KeyActionChord) _then) = _$KeyActionChordCopyWithImpl;
@useResult
$Res call({
 List<String> keys
});




}
/// @nodoc
class _$KeyActionChordCopyWithImpl<$Res>
    implements $KeyActionChordCopyWith<$Res> {
  _$KeyActionChordCopyWithImpl(this._self, this._then);

  final KeyActionChord _self;
  final $Res Function(KeyActionChord) _then;

/// Create a copy of KeyAction
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? keys = null,}) {
  return _then(KeyActionChord(
keys: null == keys ? _self._keys : keys // ignore: cast_nullable_to_non_nullable
as List<String>,
  ));
}


}

/// @nodoc
@JsonSerializable()

class KeyActionScript implements KeyAction {
  const KeyActionScript({required this.script, final  String? $type}): $type = $type ?? 'script';
  factory KeyActionScript.fromJson(Map<String, dynamic> json) => _$KeyActionScriptFromJson(json);

/// Script identifier or command to run
 final  String script;

@JsonKey(name: 'type')
final String $type;


/// Create a copy of KeyAction
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$KeyActionScriptCopyWith<KeyActionScript> get copyWith => _$KeyActionScriptCopyWithImpl<KeyActionScript>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$KeyActionScriptToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is KeyActionScript&&(identical(other.script, script) || other.script == script));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,script);

@override
String toString() {
  return 'KeyAction.script(script: $script)';
}


}

/// @nodoc
abstract mixin class $KeyActionScriptCopyWith<$Res> implements $KeyActionCopyWith<$Res> {
  factory $KeyActionScriptCopyWith(KeyActionScript value, $Res Function(KeyActionScript) _then) = _$KeyActionScriptCopyWithImpl;
@useResult
$Res call({
 String script
});




}
/// @nodoc
class _$KeyActionScriptCopyWithImpl<$Res>
    implements $KeyActionScriptCopyWith<$Res> {
  _$KeyActionScriptCopyWithImpl(this._self, this._then);

  final KeyActionScript _self;
  final $Res Function(KeyActionScript) _then;

/// Create a copy of KeyAction
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? script = null,}) {
  return _then(KeyActionScript(
script: null == script ? _self.script : script // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc
@JsonSerializable()

class KeyActionBlock implements KeyAction {
  const KeyActionBlock({final  String? $type}): $type = $type ?? 'block';
  factory KeyActionBlock.fromJson(Map<String, dynamic> json) => _$KeyActionBlockFromJson(json);



@JsonKey(name: 'type')
final String $type;



@override
Map<String, dynamic> toJson() {
  return _$KeyActionBlockToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is KeyActionBlock);
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'KeyAction.block()';
}


}




/// @nodoc
@JsonSerializable()

class KeyActionPass implements KeyAction {
  const KeyActionPass({final  String? $type}): $type = $type ?? 'pass';
  factory KeyActionPass.fromJson(Map<String, dynamic> json) => _$KeyActionPassFromJson(json);



@JsonKey(name: 'type')
final String $type;



@override
Map<String, dynamic> toJson() {
  return _$KeyActionPassToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is KeyActionPass);
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'KeyAction.pass()';
}


}





/// @nodoc
mixin _$Profile {

/// Unique identifier (UUID v4)
 String get id;/// Human-readable profile name
 String get name;/// Layout type this profile is designed for
@JsonKey(name: 'layout_type') LayoutType get layoutType;/// Creation timestamp (ISO 8601)
@JsonKey(name: 'created_at') String get createdAt;/// Last modification timestamp (ISO 8601)
@JsonKey(name: 'updated_at') String get updatedAt;/// Key mappings: physical position key → action
/// Only contains entries for remapped keys (sparse map)
/// Serialized as {"row,col": action} in JSON
 Map<String, KeyAction> get mappings;
/// Create a copy of Profile
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ProfileCopyWith<Profile> get copyWith => _$ProfileCopyWithImpl<Profile>(this as Profile, _$identity);

  /// Serializes this Profile to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is Profile&&(identical(other.id, id) || other.id == id)&&(identical(other.name, name) || other.name == name)&&(identical(other.layoutType, layoutType) || other.layoutType == layoutType)&&(identical(other.createdAt, createdAt) || other.createdAt == createdAt)&&(identical(other.updatedAt, updatedAt) || other.updatedAt == updatedAt)&&const DeepCollectionEquality().equals(other.mappings, mappings));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,id,name,layoutType,createdAt,updatedAt,const DeepCollectionEquality().hash(mappings));

@override
String toString() {
  return 'Profile(id: $id, name: $name, layoutType: $layoutType, createdAt: $createdAt, updatedAt: $updatedAt, mappings: $mappings)';
}


}

/// @nodoc
abstract mixin class $ProfileCopyWith<$Res>  {
  factory $ProfileCopyWith(Profile value, $Res Function(Profile) _then) = _$ProfileCopyWithImpl;
@useResult
$Res call({
 String id, String name,@JsonKey(name: 'layout_type') LayoutType layoutType,@JsonKey(name: 'created_at') String createdAt,@JsonKey(name: 'updated_at') String updatedAt, Map<String, KeyAction> mappings
});




}
/// @nodoc
class _$ProfileCopyWithImpl<$Res>
    implements $ProfileCopyWith<$Res> {
  _$ProfileCopyWithImpl(this._self, this._then);

  final Profile _self;
  final $Res Function(Profile) _then;

/// Create a copy of Profile
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? id = null,Object? name = null,Object? layoutType = null,Object? createdAt = null,Object? updatedAt = null,Object? mappings = null,}) {
  return _then(_self.copyWith(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,name: null == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as String,layoutType: null == layoutType ? _self.layoutType : layoutType // ignore: cast_nullable_to_non_nullable
as LayoutType,createdAt: null == createdAt ? _self.createdAt : createdAt // ignore: cast_nullable_to_non_nullable
as String,updatedAt: null == updatedAt ? _self.updatedAt : updatedAt // ignore: cast_nullable_to_non_nullable
as String,mappings: null == mappings ? _self.mappings : mappings // ignore: cast_nullable_to_non_nullable
as Map<String, KeyAction>,
  ));
}

}


/// Adds pattern-matching-related methods to [Profile].
extension ProfilePatterns on Profile {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _Profile value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _Profile() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _Profile value)  $default,){
final _that = this;
switch (_that) {
case _Profile():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _Profile value)?  $default,){
final _that = this;
switch (_that) {
case _Profile() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String id,  String name, @JsonKey(name: 'layout_type')  LayoutType layoutType, @JsonKey(name: 'created_at')  String createdAt, @JsonKey(name: 'updated_at')  String updatedAt,  Map<String, KeyAction> mappings)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _Profile() when $default != null:
return $default(_that.id,_that.name,_that.layoutType,_that.createdAt,_that.updatedAt,_that.mappings);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String id,  String name, @JsonKey(name: 'layout_type')  LayoutType layoutType, @JsonKey(name: 'created_at')  String createdAt, @JsonKey(name: 'updated_at')  String updatedAt,  Map<String, KeyAction> mappings)  $default,) {final _that = this;
switch (_that) {
case _Profile():
return $default(_that.id,_that.name,_that.layoutType,_that.createdAt,_that.updatedAt,_that.mappings);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String id,  String name, @JsonKey(name: 'layout_type')  LayoutType layoutType, @JsonKey(name: 'created_at')  String createdAt, @JsonKey(name: 'updated_at')  String updatedAt,  Map<String, KeyAction> mappings)?  $default,) {final _that = this;
switch (_that) {
case _Profile() when $default != null:
return $default(_that.id,_that.name,_that.layoutType,_that.createdAt,_that.updatedAt,_that.mappings);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _Profile extends Profile {
  const _Profile({required this.id, required this.name, @JsonKey(name: 'layout_type') required this.layoutType, @JsonKey(name: 'created_at') required this.createdAt, @JsonKey(name: 'updated_at') required this.updatedAt, final  Map<String, KeyAction> mappings = const {}}): _mappings = mappings,super._();
  factory _Profile.fromJson(Map<String, dynamic> json) => _$ProfileFromJson(json);

/// Unique identifier (UUID v4)
@override final  String id;
/// Human-readable profile name
@override final  String name;
/// Layout type this profile is designed for
@override@JsonKey(name: 'layout_type') final  LayoutType layoutType;
/// Creation timestamp (ISO 8601)
@override@JsonKey(name: 'created_at') final  String createdAt;
/// Last modification timestamp (ISO 8601)
@override@JsonKey(name: 'updated_at') final  String updatedAt;
/// Key mappings: physical position key → action
/// Only contains entries for remapped keys (sparse map)
/// Serialized as {"row,col": action} in JSON
 final  Map<String, KeyAction> _mappings;
/// Key mappings: physical position key → action
/// Only contains entries for remapped keys (sparse map)
/// Serialized as {"row,col": action} in JSON
@override@JsonKey() Map<String, KeyAction> get mappings {
  if (_mappings is EqualUnmodifiableMapView) return _mappings;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableMapView(_mappings);
}


/// Create a copy of Profile
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$ProfileCopyWith<_Profile> get copyWith => __$ProfileCopyWithImpl<_Profile>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$ProfileToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _Profile&&(identical(other.id, id) || other.id == id)&&(identical(other.name, name) || other.name == name)&&(identical(other.layoutType, layoutType) || other.layoutType == layoutType)&&(identical(other.createdAt, createdAt) || other.createdAt == createdAt)&&(identical(other.updatedAt, updatedAt) || other.updatedAt == updatedAt)&&const DeepCollectionEquality().equals(other._mappings, _mappings));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,id,name,layoutType,createdAt,updatedAt,const DeepCollectionEquality().hash(_mappings));

@override
String toString() {
  return 'Profile(id: $id, name: $name, layoutType: $layoutType, createdAt: $createdAt, updatedAt: $updatedAt, mappings: $mappings)';
}


}

/// @nodoc
abstract mixin class _$ProfileCopyWith<$Res> implements $ProfileCopyWith<$Res> {
  factory _$ProfileCopyWith(_Profile value, $Res Function(_Profile) _then) = __$ProfileCopyWithImpl;
@override @useResult
$Res call({
 String id, String name,@JsonKey(name: 'layout_type') LayoutType layoutType,@JsonKey(name: 'created_at') String createdAt,@JsonKey(name: 'updated_at') String updatedAt, Map<String, KeyAction> mappings
});




}
/// @nodoc
class __$ProfileCopyWithImpl<$Res>
    implements _$ProfileCopyWith<$Res> {
  __$ProfileCopyWithImpl(this._self, this._then);

  final _Profile _self;
  final $Res Function(_Profile) _then;

/// Create a copy of Profile
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? id = null,Object? name = null,Object? layoutType = null,Object? createdAt = null,Object? updatedAt = null,Object? mappings = null,}) {
  return _then(_Profile(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,name: null == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as String,layoutType: null == layoutType ? _self.layoutType : layoutType // ignore: cast_nullable_to_non_nullable
as LayoutType,createdAt: null == createdAt ? _self.createdAt : createdAt // ignore: cast_nullable_to_non_nullable
as String,updatedAt: null == updatedAt ? _self.updatedAt : updatedAt // ignore: cast_nullable_to_non_nullable
as String,mappings: null == mappings ? _self._mappings : mappings // ignore: cast_nullable_to_non_nullable
as Map<String, KeyAction>,
  ));
}


}

// dart format on
