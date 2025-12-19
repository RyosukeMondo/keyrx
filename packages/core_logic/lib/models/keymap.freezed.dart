// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'keymap.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$Combo {

 List<VirtualKeyId> get keys; String get output;
/// Create a copy of Combo
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ComboCopyWith<Combo> get copyWith => _$ComboCopyWithImpl<Combo>(this as Combo, _$identity);

  /// Serializes this Combo to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is Combo&&const DeepCollectionEquality().equals(other.keys, keys)&&(identical(other.output, output) || other.output == output));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(keys),output);

@override
String toString() {
  return 'Combo(keys: $keys, output: $output)';
}


}

/// @nodoc
abstract mixin class $ComboCopyWith<$Res>  {
  factory $ComboCopyWith(Combo value, $Res Function(Combo) _then) = _$ComboCopyWithImpl;
@useResult
$Res call({
 List<VirtualKeyId> keys, String output
});




}
/// @nodoc
class _$ComboCopyWithImpl<$Res>
    implements $ComboCopyWith<$Res> {
  _$ComboCopyWithImpl(this._self, this._then);

  final Combo _self;
  final $Res Function(Combo) _then;

/// Create a copy of Combo
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? keys = null,Object? output = null,}) {
  return _then(_self.copyWith(
keys: null == keys ? _self.keys : keys // ignore: cast_nullable_to_non_nullable
as List<VirtualKeyId>,output: null == output ? _self.output : output // ignore: cast_nullable_to_non_nullable
as String,
  ));
}

}


/// Adds pattern-matching-related methods to [Combo].
extension ComboPatterns on Combo {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _Combo value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _Combo() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _Combo value)  $default,){
final _that = this;
switch (_that) {
case _Combo():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _Combo value)?  $default,){
final _that = this;
switch (_that) {
case _Combo() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( List<VirtualKeyId> keys,  String output)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _Combo() when $default != null:
return $default(_that.keys,_that.output);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( List<VirtualKeyId> keys,  String output)  $default,) {final _that = this;
switch (_that) {
case _Combo():
return $default(_that.keys,_that.output);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( List<VirtualKeyId> keys,  String output)?  $default,) {final _that = this;
switch (_that) {
case _Combo() when $default != null:
return $default(_that.keys,_that.output);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _Combo implements Combo {
  const _Combo({required final  List<VirtualKeyId> keys, required this.output}): _keys = keys;
  factory _Combo.fromJson(Map<String, dynamic> json) => _$ComboFromJson(json);

 final  List<VirtualKeyId> _keys;
@override List<VirtualKeyId> get keys {
  if (_keys is EqualUnmodifiableListView) return _keys;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_keys);
}

@override final  String output;

/// Create a copy of Combo
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$ComboCopyWith<_Combo> get copyWith => __$ComboCopyWithImpl<_Combo>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$ComboToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _Combo&&const DeepCollectionEquality().equals(other._keys, _keys)&&(identical(other.output, output) || other.output == output));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_keys),output);

@override
String toString() {
  return 'Combo(keys: $keys, output: $output)';
}


}

/// @nodoc
abstract mixin class _$ComboCopyWith<$Res> implements $ComboCopyWith<$Res> {
  factory _$ComboCopyWith(_Combo value, $Res Function(_Combo) _then) = __$ComboCopyWithImpl;
@override @useResult
$Res call({
 List<VirtualKeyId> keys, String output
});




}
/// @nodoc
class __$ComboCopyWithImpl<$Res>
    implements _$ComboCopyWith<$Res> {
  __$ComboCopyWithImpl(this._self, this._then);

  final _Combo _self;
  final $Res Function(_Combo) _then;

/// Create a copy of Combo
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? keys = null,Object? output = null,}) {
  return _then(_Combo(
keys: null == keys ? _self._keys : keys // ignore: cast_nullable_to_non_nullable
as List<VirtualKeyId>,output: null == output ? _self.output : output // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}


/// @nodoc
mixin _$KeymapLayer {

 String get name; Map<VirtualKeyId, ActionBinding> get bindings;
/// Create a copy of KeymapLayer
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$KeymapLayerCopyWith<KeymapLayer> get copyWith => _$KeymapLayerCopyWithImpl<KeymapLayer>(this as KeymapLayer, _$identity);

  /// Serializes this KeymapLayer to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is KeymapLayer&&(identical(other.name, name) || other.name == name)&&const DeepCollectionEquality().equals(other.bindings, bindings));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,name,const DeepCollectionEquality().hash(bindings));

@override
String toString() {
  return 'KeymapLayer(name: $name, bindings: $bindings)';
}


}

/// @nodoc
abstract mixin class $KeymapLayerCopyWith<$Res>  {
  factory $KeymapLayerCopyWith(KeymapLayer value, $Res Function(KeymapLayer) _then) = _$KeymapLayerCopyWithImpl;
@useResult
$Res call({
 String name, Map<VirtualKeyId, ActionBinding> bindings
});




}
/// @nodoc
class _$KeymapLayerCopyWithImpl<$Res>
    implements $KeymapLayerCopyWith<$Res> {
  _$KeymapLayerCopyWithImpl(this._self, this._then);

  final KeymapLayer _self;
  final $Res Function(KeymapLayer) _then;

/// Create a copy of KeymapLayer
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? name = null,Object? bindings = null,}) {
  return _then(_self.copyWith(
name: null == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as String,bindings: null == bindings ? _self.bindings : bindings // ignore: cast_nullable_to_non_nullable
as Map<VirtualKeyId, ActionBinding>,
  ));
}

}


/// Adds pattern-matching-related methods to [KeymapLayer].
extension KeymapLayerPatterns on KeymapLayer {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _KeymapLayer value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _KeymapLayer() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _KeymapLayer value)  $default,){
final _that = this;
switch (_that) {
case _KeymapLayer():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _KeymapLayer value)?  $default,){
final _that = this;
switch (_that) {
case _KeymapLayer() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String name,  Map<VirtualKeyId, ActionBinding> bindings)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _KeymapLayer() when $default != null:
return $default(_that.name,_that.bindings);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String name,  Map<VirtualKeyId, ActionBinding> bindings)  $default,) {final _that = this;
switch (_that) {
case _KeymapLayer():
return $default(_that.name,_that.bindings);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String name,  Map<VirtualKeyId, ActionBinding> bindings)?  $default,) {final _that = this;
switch (_that) {
case _KeymapLayer() when $default != null:
return $default(_that.name,_that.bindings);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _KeymapLayer extends KeymapLayer {
  const _KeymapLayer({required this.name, final  Map<VirtualKeyId, ActionBinding> bindings = const <VirtualKeyId, ActionBinding>{}}): _bindings = bindings,super._();
  factory _KeymapLayer.fromJson(Map<String, dynamic> json) => _$KeymapLayerFromJson(json);

@override final  String name;
 final  Map<VirtualKeyId, ActionBinding> _bindings;
@override@JsonKey() Map<VirtualKeyId, ActionBinding> get bindings {
  if (_bindings is EqualUnmodifiableMapView) return _bindings;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableMapView(_bindings);
}


/// Create a copy of KeymapLayer
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$KeymapLayerCopyWith<_KeymapLayer> get copyWith => __$KeymapLayerCopyWithImpl<_KeymapLayer>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$KeymapLayerToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _KeymapLayer&&(identical(other.name, name) || other.name == name)&&const DeepCollectionEquality().equals(other._bindings, _bindings));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,name,const DeepCollectionEquality().hash(_bindings));

@override
String toString() {
  return 'KeymapLayer(name: $name, bindings: $bindings)';
}


}

/// @nodoc
abstract mixin class _$KeymapLayerCopyWith<$Res> implements $KeymapLayerCopyWith<$Res> {
  factory _$KeymapLayerCopyWith(_KeymapLayer value, $Res Function(_KeymapLayer) _then) = __$KeymapLayerCopyWithImpl;
@override @useResult
$Res call({
 String name, Map<VirtualKeyId, ActionBinding> bindings
});




}
/// @nodoc
class __$KeymapLayerCopyWithImpl<$Res>
    implements _$KeymapLayerCopyWith<$Res> {
  __$KeymapLayerCopyWithImpl(this._self, this._then);

  final _KeymapLayer _self;
  final $Res Function(_KeymapLayer) _then;

/// Create a copy of KeymapLayer
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? name = null,Object? bindings = null,}) {
  return _then(_KeymapLayer(
name: null == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as String,bindings: null == bindings ? _self._bindings : bindings // ignore: cast_nullable_to_non_nullable
as Map<VirtualKeyId, ActionBinding>,
  ));
}


}


/// @nodoc
mixin _$Keymap {

 KeymapId get id;@JsonKey(name: 'virtual_layout_id') VirtualLayoutId get virtualLayoutId; String get name; List<KeymapLayer> get layers; List<Combo> get combos;
/// Create a copy of Keymap
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$KeymapCopyWith<Keymap> get copyWith => _$KeymapCopyWithImpl<Keymap>(this as Keymap, _$identity);

  /// Serializes this Keymap to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is Keymap&&(identical(other.id, id) || other.id == id)&&(identical(other.virtualLayoutId, virtualLayoutId) || other.virtualLayoutId == virtualLayoutId)&&(identical(other.name, name) || other.name == name)&&const DeepCollectionEquality().equals(other.layers, layers)&&const DeepCollectionEquality().equals(other.combos, combos));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,id,virtualLayoutId,name,const DeepCollectionEquality().hash(layers),const DeepCollectionEquality().hash(combos));

@override
String toString() {
  return 'Keymap(id: $id, virtualLayoutId: $virtualLayoutId, name: $name, layers: $layers, combos: $combos)';
}


}

/// @nodoc
abstract mixin class $KeymapCopyWith<$Res>  {
  factory $KeymapCopyWith(Keymap value, $Res Function(Keymap) _then) = _$KeymapCopyWithImpl;
@useResult
$Res call({
 KeymapId id,@JsonKey(name: 'virtual_layout_id') VirtualLayoutId virtualLayoutId, String name, List<KeymapLayer> layers, List<Combo> combos
});




}
/// @nodoc
class _$KeymapCopyWithImpl<$Res>
    implements $KeymapCopyWith<$Res> {
  _$KeymapCopyWithImpl(this._self, this._then);

  final Keymap _self;
  final $Res Function(Keymap) _then;

/// Create a copy of Keymap
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? id = null,Object? virtualLayoutId = null,Object? name = null,Object? layers = null,Object? combos = null,}) {
  return _then(_self.copyWith(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as KeymapId,virtualLayoutId: null == virtualLayoutId ? _self.virtualLayoutId : virtualLayoutId // ignore: cast_nullable_to_non_nullable
as VirtualLayoutId,name: null == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as String,layers: null == layers ? _self.layers : layers // ignore: cast_nullable_to_non_nullable
as List<KeymapLayer>,combos: null == combos ? _self.combos : combos // ignore: cast_nullable_to_non_nullable
as List<Combo>,
  ));
}

}


/// Adds pattern-matching-related methods to [Keymap].
extension KeymapPatterns on Keymap {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _Keymap value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _Keymap() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _Keymap value)  $default,){
final _that = this;
switch (_that) {
case _Keymap():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _Keymap value)?  $default,){
final _that = this;
switch (_that) {
case _Keymap() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( KeymapId id, @JsonKey(name: 'virtual_layout_id')  VirtualLayoutId virtualLayoutId,  String name,  List<KeymapLayer> layers,  List<Combo> combos)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _Keymap() when $default != null:
return $default(_that.id,_that.virtualLayoutId,_that.name,_that.layers,_that.combos);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( KeymapId id, @JsonKey(name: 'virtual_layout_id')  VirtualLayoutId virtualLayoutId,  String name,  List<KeymapLayer> layers,  List<Combo> combos)  $default,) {final _that = this;
switch (_that) {
case _Keymap():
return $default(_that.id,_that.virtualLayoutId,_that.name,_that.layers,_that.combos);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( KeymapId id, @JsonKey(name: 'virtual_layout_id')  VirtualLayoutId virtualLayoutId,  String name,  List<KeymapLayer> layers,  List<Combo> combos)?  $default,) {final _that = this;
switch (_that) {
case _Keymap() when $default != null:
return $default(_that.id,_that.virtualLayoutId,_that.name,_that.layers,_that.combos);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _Keymap extends Keymap {
  const _Keymap({required this.id, @JsonKey(name: 'virtual_layout_id') required this.virtualLayoutId, this.name = 'Untitled Keymap', final  List<KeymapLayer> layers = const <KeymapLayer>[], final  List<Combo> combos = const <Combo>[]}): _layers = layers,_combos = combos,super._();
  factory _Keymap.fromJson(Map<String, dynamic> json) => _$KeymapFromJson(json);

@override final  KeymapId id;
@override@JsonKey(name: 'virtual_layout_id') final  VirtualLayoutId virtualLayoutId;
@override@JsonKey() final  String name;
 final  List<KeymapLayer> _layers;
@override@JsonKey() List<KeymapLayer> get layers {
  if (_layers is EqualUnmodifiableListView) return _layers;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_layers);
}

 final  List<Combo> _combos;
@override@JsonKey() List<Combo> get combos {
  if (_combos is EqualUnmodifiableListView) return _combos;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_combos);
}


/// Create a copy of Keymap
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$KeymapCopyWith<_Keymap> get copyWith => __$KeymapCopyWithImpl<_Keymap>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$KeymapToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _Keymap&&(identical(other.id, id) || other.id == id)&&(identical(other.virtualLayoutId, virtualLayoutId) || other.virtualLayoutId == virtualLayoutId)&&(identical(other.name, name) || other.name == name)&&const DeepCollectionEquality().equals(other._layers, _layers)&&const DeepCollectionEquality().equals(other._combos, _combos));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,id,virtualLayoutId,name,const DeepCollectionEquality().hash(_layers),const DeepCollectionEquality().hash(_combos));

@override
String toString() {
  return 'Keymap(id: $id, virtualLayoutId: $virtualLayoutId, name: $name, layers: $layers, combos: $combos)';
}


}

/// @nodoc
abstract mixin class _$KeymapCopyWith<$Res> implements $KeymapCopyWith<$Res> {
  factory _$KeymapCopyWith(_Keymap value, $Res Function(_Keymap) _then) = __$KeymapCopyWithImpl;
@override @useResult
$Res call({
 KeymapId id,@JsonKey(name: 'virtual_layout_id') VirtualLayoutId virtualLayoutId, String name, List<KeymapLayer> layers, List<Combo> combos
});




}
/// @nodoc
class __$KeymapCopyWithImpl<$Res>
    implements _$KeymapCopyWith<$Res> {
  __$KeymapCopyWithImpl(this._self, this._then);

  final _Keymap _self;
  final $Res Function(_Keymap) _then;

/// Create a copy of Keymap
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? id = null,Object? virtualLayoutId = null,Object? name = null,Object? layers = null,Object? combos = null,}) {
  return _then(_Keymap(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as KeymapId,virtualLayoutId: null == virtualLayoutId ? _self.virtualLayoutId : virtualLayoutId // ignore: cast_nullable_to_non_nullable
as VirtualLayoutId,name: null == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as String,layers: null == layers ? _self._layers : layers // ignore: cast_nullable_to_non_nullable
as List<KeymapLayer>,combos: null == combos ? _self._combos : combos // ignore: cast_nullable_to_non_nullable
as List<Combo>,
  ));
}


}

// dart format on
