// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'action_binding.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
ActionBinding _$ActionBindingFromJson(
  Map<String, dynamic> json
) {
        switch (json['type']) {
                  case 'standard_key':
          return ActionBindingStandardKey.fromJson(
            json
          );
                case 'macro':
          return ActionBindingMacro.fromJson(
            json
          );
                case 'layer_toggle':
          return ActionBindingLayerToggle.fromJson(
            json
          );
                case 'tap_hold':
          return ActionBindingTapHold.fromJson(
            json
          );
                case 'transparent':
          return ActionBindingTransparent.fromJson(
            json
          );
        
          default:
            throw CheckedFromJsonException(
  json,
  'type',
  'ActionBinding',
  'Invalid union type "${json['type']}"!'
);
        }
      
}

/// @nodoc
mixin _$ActionBinding {



  /// Serializes this ActionBinding to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ActionBinding);
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ActionBinding()';
}


}

/// @nodoc
class $ActionBindingCopyWith<$Res>  {
$ActionBindingCopyWith(ActionBinding _, $Res Function(ActionBinding) __);
}


/// Adds pattern-matching-related methods to [ActionBinding].
extension ActionBindingPatterns on ActionBinding {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( ActionBindingStandardKey value)?  standardKey,TResult Function( ActionBindingMacro value)?  macro,TResult Function( ActionBindingLayerToggle value)?  layerToggle,TResult Function( ActionBindingTapHold value)?  tapHold,TResult Function( ActionBindingTransparent value)?  transparent,required TResult orElse(),}){
final _that = this;
switch (_that) {
case ActionBindingStandardKey() when standardKey != null:
return standardKey(_that);case ActionBindingMacro() when macro != null:
return macro(_that);case ActionBindingLayerToggle() when layerToggle != null:
return layerToggle(_that);case ActionBindingTapHold() when tapHold != null:
return tapHold(_that);case ActionBindingTransparent() when transparent != null:
return transparent(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( ActionBindingStandardKey value)  standardKey,required TResult Function( ActionBindingMacro value)  macro,required TResult Function( ActionBindingLayerToggle value)  layerToggle,required TResult Function( ActionBindingTapHold value)  tapHold,required TResult Function( ActionBindingTransparent value)  transparent,}){
final _that = this;
switch (_that) {
case ActionBindingStandardKey():
return standardKey(_that);case ActionBindingMacro():
return macro(_that);case ActionBindingLayerToggle():
return layerToggle(_that);case ActionBindingTapHold():
return tapHold(_that);case ActionBindingTransparent():
return transparent(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( ActionBindingStandardKey value)?  standardKey,TResult? Function( ActionBindingMacro value)?  macro,TResult? Function( ActionBindingLayerToggle value)?  layerToggle,TResult? Function( ActionBindingTapHold value)?  tapHold,TResult? Function( ActionBindingTransparent value)?  transparent,}){
final _that = this;
switch (_that) {
case ActionBindingStandardKey() when standardKey != null:
return standardKey(_that);case ActionBindingMacro() when macro != null:
return macro(_that);case ActionBindingLayerToggle() when layerToggle != null:
return layerToggle(_that);case ActionBindingTapHold() when tapHold != null:
return tapHold(_that);case ActionBindingTransparent() when transparent != null:
return transparent(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function(@JsonKey(name: 'value')  String value)?  standardKey,TResult Function(@JsonKey(name: 'value')  String value)?  macro,TResult Function(@JsonKey(name: 'value')  String value)?  layerToggle,TResult Function(@JsonKey(name: 'value')  List<String> value)?  tapHold,TResult Function()?  transparent,required TResult orElse(),}) {final _that = this;
switch (_that) {
case ActionBindingStandardKey() when standardKey != null:
return standardKey(_that.value);case ActionBindingMacro() when macro != null:
return macro(_that.value);case ActionBindingLayerToggle() when layerToggle != null:
return layerToggle(_that.value);case ActionBindingTapHold() when tapHold != null:
return tapHold(_that.value);case ActionBindingTransparent() when transparent != null:
return transparent();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function(@JsonKey(name: 'value')  String value)  standardKey,required TResult Function(@JsonKey(name: 'value')  String value)  macro,required TResult Function(@JsonKey(name: 'value')  String value)  layerToggle,required TResult Function(@JsonKey(name: 'value')  List<String> value)  tapHold,required TResult Function()  transparent,}) {final _that = this;
switch (_that) {
case ActionBindingStandardKey():
return standardKey(_that.value);case ActionBindingMacro():
return macro(_that.value);case ActionBindingLayerToggle():
return layerToggle(_that.value);case ActionBindingTapHold():
return tapHold(_that.value);case ActionBindingTransparent():
return transparent();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function(@JsonKey(name: 'value')  String value)?  standardKey,TResult? Function(@JsonKey(name: 'value')  String value)?  macro,TResult? Function(@JsonKey(name: 'value')  String value)?  layerToggle,TResult? Function(@JsonKey(name: 'value')  List<String> value)?  tapHold,TResult? Function()?  transparent,}) {final _that = this;
switch (_that) {
case ActionBindingStandardKey() when standardKey != null:
return standardKey(_that.value);case ActionBindingMacro() when macro != null:
return macro(_that.value);case ActionBindingLayerToggle() when layerToggle != null:
return layerToggle(_that.value);case ActionBindingTapHold() when tapHold != null:
return tapHold(_that.value);case ActionBindingTransparent() when transparent != null:
return transparent();case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class ActionBindingStandardKey implements ActionBinding {
  const ActionBindingStandardKey({@JsonKey(name: 'value') this.value = '', final  String? $type}): $type = $type ?? 'standard_key';
  factory ActionBindingStandardKey.fromJson(Map<String, dynamic> json) => _$ActionBindingStandardKeyFromJson(json);

@JsonKey(name: 'value') final  String value;

@JsonKey(name: 'type')
final String $type;


/// Create a copy of ActionBinding
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ActionBindingStandardKeyCopyWith<ActionBindingStandardKey> get copyWith => _$ActionBindingStandardKeyCopyWithImpl<ActionBindingStandardKey>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$ActionBindingStandardKeyToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ActionBindingStandardKey&&(identical(other.value, value) || other.value == value));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,value);

@override
String toString() {
  return 'ActionBinding.standardKey(value: $value)';
}


}

/// @nodoc
abstract mixin class $ActionBindingStandardKeyCopyWith<$Res> implements $ActionBindingCopyWith<$Res> {
  factory $ActionBindingStandardKeyCopyWith(ActionBindingStandardKey value, $Res Function(ActionBindingStandardKey) _then) = _$ActionBindingStandardKeyCopyWithImpl;
@useResult
$Res call({
@JsonKey(name: 'value') String value
});




}
/// @nodoc
class _$ActionBindingStandardKeyCopyWithImpl<$Res>
    implements $ActionBindingStandardKeyCopyWith<$Res> {
  _$ActionBindingStandardKeyCopyWithImpl(this._self, this._then);

  final ActionBindingStandardKey _self;
  final $Res Function(ActionBindingStandardKey) _then;

/// Create a copy of ActionBinding
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? value = null,}) {
  return _then(ActionBindingStandardKey(
value: null == value ? _self.value : value // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc
@JsonSerializable()

class ActionBindingMacro implements ActionBinding {
  const ActionBindingMacro({@JsonKey(name: 'value') this.value = '', final  String? $type}): $type = $type ?? 'macro';
  factory ActionBindingMacro.fromJson(Map<String, dynamic> json) => _$ActionBindingMacroFromJson(json);

@JsonKey(name: 'value') final  String value;

@JsonKey(name: 'type')
final String $type;


/// Create a copy of ActionBinding
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ActionBindingMacroCopyWith<ActionBindingMacro> get copyWith => _$ActionBindingMacroCopyWithImpl<ActionBindingMacro>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$ActionBindingMacroToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ActionBindingMacro&&(identical(other.value, value) || other.value == value));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,value);

@override
String toString() {
  return 'ActionBinding.macro(value: $value)';
}


}

/// @nodoc
abstract mixin class $ActionBindingMacroCopyWith<$Res> implements $ActionBindingCopyWith<$Res> {
  factory $ActionBindingMacroCopyWith(ActionBindingMacro value, $Res Function(ActionBindingMacro) _then) = _$ActionBindingMacroCopyWithImpl;
@useResult
$Res call({
@JsonKey(name: 'value') String value
});




}
/// @nodoc
class _$ActionBindingMacroCopyWithImpl<$Res>
    implements $ActionBindingMacroCopyWith<$Res> {
  _$ActionBindingMacroCopyWithImpl(this._self, this._then);

  final ActionBindingMacro _self;
  final $Res Function(ActionBindingMacro) _then;

/// Create a copy of ActionBinding
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? value = null,}) {
  return _then(ActionBindingMacro(
value: null == value ? _self.value : value // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc
@JsonSerializable()

class ActionBindingLayerToggle implements ActionBinding {
  const ActionBindingLayerToggle({@JsonKey(name: 'value') this.value = '', final  String? $type}): $type = $type ?? 'layer_toggle';
  factory ActionBindingLayerToggle.fromJson(Map<String, dynamic> json) => _$ActionBindingLayerToggleFromJson(json);

@JsonKey(name: 'value') final  String value;

@JsonKey(name: 'type')
final String $type;


/// Create a copy of ActionBinding
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ActionBindingLayerToggleCopyWith<ActionBindingLayerToggle> get copyWith => _$ActionBindingLayerToggleCopyWithImpl<ActionBindingLayerToggle>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$ActionBindingLayerToggleToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ActionBindingLayerToggle&&(identical(other.value, value) || other.value == value));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,value);

@override
String toString() {
  return 'ActionBinding.layerToggle(value: $value)';
}


}

/// @nodoc
abstract mixin class $ActionBindingLayerToggleCopyWith<$Res> implements $ActionBindingCopyWith<$Res> {
  factory $ActionBindingLayerToggleCopyWith(ActionBindingLayerToggle value, $Res Function(ActionBindingLayerToggle) _then) = _$ActionBindingLayerToggleCopyWithImpl;
@useResult
$Res call({
@JsonKey(name: 'value') String value
});




}
/// @nodoc
class _$ActionBindingLayerToggleCopyWithImpl<$Res>
    implements $ActionBindingLayerToggleCopyWith<$Res> {
  _$ActionBindingLayerToggleCopyWithImpl(this._self, this._then);

  final ActionBindingLayerToggle _self;
  final $Res Function(ActionBindingLayerToggle) _then;

/// Create a copy of ActionBinding
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? value = null,}) {
  return _then(ActionBindingLayerToggle(
value: null == value ? _self.value : value // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc
@JsonSerializable()

class ActionBindingTapHold implements ActionBinding {
  const ActionBindingTapHold({@JsonKey(name: 'value') final  List<String> value = const [], final  String? $type}): _value = value,$type = $type ?? 'tap_hold';
  factory ActionBindingTapHold.fromJson(Map<String, dynamic> json) => _$ActionBindingTapHoldFromJson(json);

 final  List<String> _value;
@JsonKey(name: 'value') List<String> get value {
  if (_value is EqualUnmodifiableListView) return _value;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_value);
}


@JsonKey(name: 'type')
final String $type;


/// Create a copy of ActionBinding
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ActionBindingTapHoldCopyWith<ActionBindingTapHold> get copyWith => _$ActionBindingTapHoldCopyWithImpl<ActionBindingTapHold>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$ActionBindingTapHoldToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ActionBindingTapHold&&const DeepCollectionEquality().equals(other._value, _value));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_value));

@override
String toString() {
  return 'ActionBinding.tapHold(value: $value)';
}


}

/// @nodoc
abstract mixin class $ActionBindingTapHoldCopyWith<$Res> implements $ActionBindingCopyWith<$Res> {
  factory $ActionBindingTapHoldCopyWith(ActionBindingTapHold value, $Res Function(ActionBindingTapHold) _then) = _$ActionBindingTapHoldCopyWithImpl;
@useResult
$Res call({
@JsonKey(name: 'value') List<String> value
});




}
/// @nodoc
class _$ActionBindingTapHoldCopyWithImpl<$Res>
    implements $ActionBindingTapHoldCopyWith<$Res> {
  _$ActionBindingTapHoldCopyWithImpl(this._self, this._then);

  final ActionBindingTapHold _self;
  final $Res Function(ActionBindingTapHold) _then;

/// Create a copy of ActionBinding
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? value = null,}) {
  return _then(ActionBindingTapHold(
value: null == value ? _self._value : value // ignore: cast_nullable_to_non_nullable
as List<String>,
  ));
}


}

/// @nodoc
@JsonSerializable()

class ActionBindingTransparent implements ActionBinding {
  const ActionBindingTransparent({final  String? $type}): $type = $type ?? 'transparent';
  factory ActionBindingTransparent.fromJson(Map<String, dynamic> json) => _$ActionBindingTransparentFromJson(json);



@JsonKey(name: 'type')
final String $type;



@override
Map<String, dynamic> toJson() {
  return _$ActionBindingTransparentToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ActionBindingTransparent);
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ActionBinding.transparent()';
}


}




// dart format on
