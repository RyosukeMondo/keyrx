// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'virtual_layout.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$KeyPosition {

 double get x; double get y;
/// Create a copy of KeyPosition
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$KeyPositionCopyWith<KeyPosition> get copyWith => _$KeyPositionCopyWithImpl<KeyPosition>(this as KeyPosition, _$identity);

  /// Serializes this KeyPosition to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is KeyPosition&&(identical(other.x, x) || other.x == x)&&(identical(other.y, y) || other.y == y));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,x,y);

@override
String toString() {
  return 'KeyPosition(x: $x, y: $y)';
}


}

/// @nodoc
abstract mixin class $KeyPositionCopyWith<$Res>  {
  factory $KeyPositionCopyWith(KeyPosition value, $Res Function(KeyPosition) _then) = _$KeyPositionCopyWithImpl;
@useResult
$Res call({
 double x, double y
});




}
/// @nodoc
class _$KeyPositionCopyWithImpl<$Res>
    implements $KeyPositionCopyWith<$Res> {
  _$KeyPositionCopyWithImpl(this._self, this._then);

  final KeyPosition _self;
  final $Res Function(KeyPosition) _then;

/// Create a copy of KeyPosition
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? x = null,Object? y = null,}) {
  return _then(_self.copyWith(
x: null == x ? _self.x : x // ignore: cast_nullable_to_non_nullable
as double,y: null == y ? _self.y : y // ignore: cast_nullable_to_non_nullable
as double,
  ));
}

}


/// Adds pattern-matching-related methods to [KeyPosition].
extension KeyPositionPatterns on KeyPosition {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _KeyPosition value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _KeyPosition() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _KeyPosition value)  $default,){
final _that = this;
switch (_that) {
case _KeyPosition():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _KeyPosition value)?  $default,){
final _that = this;
switch (_that) {
case _KeyPosition() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( double x,  double y)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _KeyPosition() when $default != null:
return $default(_that.x,_that.y);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( double x,  double y)  $default,) {final _that = this;
switch (_that) {
case _KeyPosition():
return $default(_that.x,_that.y);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( double x,  double y)?  $default,) {final _that = this;
switch (_that) {
case _KeyPosition() when $default != null:
return $default(_that.x,_that.y);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _KeyPosition extends KeyPosition {
  const _KeyPosition({required this.x, required this.y}): super._();
  factory _KeyPosition.fromJson(Map<String, dynamic> json) => _$KeyPositionFromJson(json);

@override final  double x;
@override final  double y;

/// Create a copy of KeyPosition
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$KeyPositionCopyWith<_KeyPosition> get copyWith => __$KeyPositionCopyWithImpl<_KeyPosition>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$KeyPositionToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _KeyPosition&&(identical(other.x, x) || other.x == x)&&(identical(other.y, y) || other.y == y));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,x,y);

@override
String toString() {
  return 'KeyPosition(x: $x, y: $y)';
}


}

/// @nodoc
abstract mixin class _$KeyPositionCopyWith<$Res> implements $KeyPositionCopyWith<$Res> {
  factory _$KeyPositionCopyWith(_KeyPosition value, $Res Function(_KeyPosition) _then) = __$KeyPositionCopyWithImpl;
@override @useResult
$Res call({
 double x, double y
});




}
/// @nodoc
class __$KeyPositionCopyWithImpl<$Res>
    implements _$KeyPositionCopyWith<$Res> {
  __$KeyPositionCopyWithImpl(this._self, this._then);

  final _KeyPosition _self;
  final $Res Function(_KeyPosition) _then;

/// Create a copy of KeyPosition
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? x = null,Object? y = null,}) {
  return _then(_KeyPosition(
x: null == x ? _self.x : x // ignore: cast_nullable_to_non_nullable
as double,y: null == y ? _self.y : y // ignore: cast_nullable_to_non_nullable
as double,
  ));
}


}


/// @nodoc
mixin _$KeySize {

 double get width; double get height;
/// Create a copy of KeySize
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$KeySizeCopyWith<KeySize> get copyWith => _$KeySizeCopyWithImpl<KeySize>(this as KeySize, _$identity);

  /// Serializes this KeySize to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is KeySize&&(identical(other.width, width) || other.width == width)&&(identical(other.height, height) || other.height == height));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,width,height);

@override
String toString() {
  return 'KeySize(width: $width, height: $height)';
}


}

/// @nodoc
abstract mixin class $KeySizeCopyWith<$Res>  {
  factory $KeySizeCopyWith(KeySize value, $Res Function(KeySize) _then) = _$KeySizeCopyWithImpl;
@useResult
$Res call({
 double width, double height
});




}
/// @nodoc
class _$KeySizeCopyWithImpl<$Res>
    implements $KeySizeCopyWith<$Res> {
  _$KeySizeCopyWithImpl(this._self, this._then);

  final KeySize _self;
  final $Res Function(KeySize) _then;

/// Create a copy of KeySize
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? width = null,Object? height = null,}) {
  return _then(_self.copyWith(
width: null == width ? _self.width : width // ignore: cast_nullable_to_non_nullable
as double,height: null == height ? _self.height : height // ignore: cast_nullable_to_non_nullable
as double,
  ));
}

}


/// Adds pattern-matching-related methods to [KeySize].
extension KeySizePatterns on KeySize {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _KeySize value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _KeySize() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _KeySize value)  $default,){
final _that = this;
switch (_that) {
case _KeySize():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _KeySize value)?  $default,){
final _that = this;
switch (_that) {
case _KeySize() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( double width,  double height)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _KeySize() when $default != null:
return $default(_that.width,_that.height);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( double width,  double height)  $default,) {final _that = this;
switch (_that) {
case _KeySize():
return $default(_that.width,_that.height);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( double width,  double height)?  $default,) {final _that = this;
switch (_that) {
case _KeySize() when $default != null:
return $default(_that.width,_that.height);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _KeySize extends KeySize {
  const _KeySize({required this.width, required this.height}): super._();
  factory _KeySize.fromJson(Map<String, dynamic> json) => _$KeySizeFromJson(json);

@override final  double width;
@override final  double height;

/// Create a copy of KeySize
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$KeySizeCopyWith<_KeySize> get copyWith => __$KeySizeCopyWithImpl<_KeySize>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$KeySizeToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _KeySize&&(identical(other.width, width) || other.width == width)&&(identical(other.height, height) || other.height == height));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,width,height);

@override
String toString() {
  return 'KeySize(width: $width, height: $height)';
}


}

/// @nodoc
abstract mixin class _$KeySizeCopyWith<$Res> implements $KeySizeCopyWith<$Res> {
  factory _$KeySizeCopyWith(_KeySize value, $Res Function(_KeySize) _then) = __$KeySizeCopyWithImpl;
@override @useResult
$Res call({
 double width, double height
});




}
/// @nodoc
class __$KeySizeCopyWithImpl<$Res>
    implements _$KeySizeCopyWith<$Res> {
  __$KeySizeCopyWithImpl(this._self, this._then);

  final _KeySize _self;
  final $Res Function(_KeySize) _then;

/// Create a copy of KeySize
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? width = null,Object? height = null,}) {
  return _then(_KeySize(
width: null == width ? _self.width : width // ignore: cast_nullable_to_non_nullable
as double,height: null == height ? _self.height : height // ignore: cast_nullable_to_non_nullable
as double,
  ));
}


}


/// @nodoc
mixin _$VirtualKeyDef {

 VirtualKeyId get id; String get label; KeyPosition? get position; KeySize? get size; int? get row; int? get column;
/// Create a copy of VirtualKeyDef
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$VirtualKeyDefCopyWith<VirtualKeyDef> get copyWith => _$VirtualKeyDefCopyWithImpl<VirtualKeyDef>(this as VirtualKeyDef, _$identity);

  /// Serializes this VirtualKeyDef to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is VirtualKeyDef&&(identical(other.id, id) || other.id == id)&&(identical(other.label, label) || other.label == label)&&(identical(other.position, position) || other.position == position)&&(identical(other.size, size) || other.size == size)&&(identical(other.row, row) || other.row == row)&&(identical(other.column, column) || other.column == column));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,id,label,position,size,row,column);

@override
String toString() {
  return 'VirtualKeyDef(id: $id, label: $label, position: $position, size: $size, row: $row, column: $column)';
}


}

/// @nodoc
abstract mixin class $VirtualKeyDefCopyWith<$Res>  {
  factory $VirtualKeyDefCopyWith(VirtualKeyDef value, $Res Function(VirtualKeyDef) _then) = _$VirtualKeyDefCopyWithImpl;
@useResult
$Res call({
 VirtualKeyId id, String label, KeyPosition? position, KeySize? size, int? row, int? column
});


$KeyPositionCopyWith<$Res>? get position;$KeySizeCopyWith<$Res>? get size;

}
/// @nodoc
class _$VirtualKeyDefCopyWithImpl<$Res>
    implements $VirtualKeyDefCopyWith<$Res> {
  _$VirtualKeyDefCopyWithImpl(this._self, this._then);

  final VirtualKeyDef _self;
  final $Res Function(VirtualKeyDef) _then;

/// Create a copy of VirtualKeyDef
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? id = null,Object? label = null,Object? position = freezed,Object? size = freezed,Object? row = freezed,Object? column = freezed,}) {
  return _then(_self.copyWith(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as VirtualKeyId,label: null == label ? _self.label : label // ignore: cast_nullable_to_non_nullable
as String,position: freezed == position ? _self.position : position // ignore: cast_nullable_to_non_nullable
as KeyPosition?,size: freezed == size ? _self.size : size // ignore: cast_nullable_to_non_nullable
as KeySize?,row: freezed == row ? _self.row : row // ignore: cast_nullable_to_non_nullable
as int?,column: freezed == column ? _self.column : column // ignore: cast_nullable_to_non_nullable
as int?,
  ));
}
/// Create a copy of VirtualKeyDef
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$KeyPositionCopyWith<$Res>? get position {
    if (_self.position == null) {
    return null;
  }

  return $KeyPositionCopyWith<$Res>(_self.position!, (value) {
    return _then(_self.copyWith(position: value));
  });
}/// Create a copy of VirtualKeyDef
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$KeySizeCopyWith<$Res>? get size {
    if (_self.size == null) {
    return null;
  }

  return $KeySizeCopyWith<$Res>(_self.size!, (value) {
    return _then(_self.copyWith(size: value));
  });
}
}


/// Adds pattern-matching-related methods to [VirtualKeyDef].
extension VirtualKeyDefPatterns on VirtualKeyDef {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _VirtualKeyDef value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _VirtualKeyDef() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _VirtualKeyDef value)  $default,){
final _that = this;
switch (_that) {
case _VirtualKeyDef():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _VirtualKeyDef value)?  $default,){
final _that = this;
switch (_that) {
case _VirtualKeyDef() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( VirtualKeyId id,  String label,  KeyPosition? position,  KeySize? size,  int? row,  int? column)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _VirtualKeyDef() when $default != null:
return $default(_that.id,_that.label,_that.position,_that.size,_that.row,_that.column);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( VirtualKeyId id,  String label,  KeyPosition? position,  KeySize? size,  int? row,  int? column)  $default,) {final _that = this;
switch (_that) {
case _VirtualKeyDef():
return $default(_that.id,_that.label,_that.position,_that.size,_that.row,_that.column);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( VirtualKeyId id,  String label,  KeyPosition? position,  KeySize? size,  int? row,  int? column)?  $default,) {final _that = this;
switch (_that) {
case _VirtualKeyDef() when $default != null:
return $default(_that.id,_that.label,_that.position,_that.size,_that.row,_that.column);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _VirtualKeyDef extends VirtualKeyDef {
  const _VirtualKeyDef({required this.id, required this.label, this.position, this.size, this.row, this.column}): super._();
  factory _VirtualKeyDef.fromJson(Map<String, dynamic> json) => _$VirtualKeyDefFromJson(json);

@override final  VirtualKeyId id;
@override final  String label;
@override final  KeyPosition? position;
@override final  KeySize? size;
@override final  int? row;
@override final  int? column;

/// Create a copy of VirtualKeyDef
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$VirtualKeyDefCopyWith<_VirtualKeyDef> get copyWith => __$VirtualKeyDefCopyWithImpl<_VirtualKeyDef>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$VirtualKeyDefToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _VirtualKeyDef&&(identical(other.id, id) || other.id == id)&&(identical(other.label, label) || other.label == label)&&(identical(other.position, position) || other.position == position)&&(identical(other.size, size) || other.size == size)&&(identical(other.row, row) || other.row == row)&&(identical(other.column, column) || other.column == column));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,id,label,position,size,row,column);

@override
String toString() {
  return 'VirtualKeyDef(id: $id, label: $label, position: $position, size: $size, row: $row, column: $column)';
}


}

/// @nodoc
abstract mixin class _$VirtualKeyDefCopyWith<$Res> implements $VirtualKeyDefCopyWith<$Res> {
  factory _$VirtualKeyDefCopyWith(_VirtualKeyDef value, $Res Function(_VirtualKeyDef) _then) = __$VirtualKeyDefCopyWithImpl;
@override @useResult
$Res call({
 VirtualKeyId id, String label, KeyPosition? position, KeySize? size, int? row, int? column
});


@override $KeyPositionCopyWith<$Res>? get position;@override $KeySizeCopyWith<$Res>? get size;

}
/// @nodoc
class __$VirtualKeyDefCopyWithImpl<$Res>
    implements _$VirtualKeyDefCopyWith<$Res> {
  __$VirtualKeyDefCopyWithImpl(this._self, this._then);

  final _VirtualKeyDef _self;
  final $Res Function(_VirtualKeyDef) _then;

/// Create a copy of VirtualKeyDef
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? id = null,Object? label = null,Object? position = freezed,Object? size = freezed,Object? row = freezed,Object? column = freezed,}) {
  return _then(_VirtualKeyDef(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as VirtualKeyId,label: null == label ? _self.label : label // ignore: cast_nullable_to_non_nullable
as String,position: freezed == position ? _self.position : position // ignore: cast_nullable_to_non_nullable
as KeyPosition?,size: freezed == size ? _self.size : size // ignore: cast_nullable_to_non_nullable
as KeySize?,row: freezed == row ? _self.row : row // ignore: cast_nullable_to_non_nullable
as int?,column: freezed == column ? _self.column : column // ignore: cast_nullable_to_non_nullable
as int?,
  ));
}

/// Create a copy of VirtualKeyDef
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$KeyPositionCopyWith<$Res>? get position {
    if (_self.position == null) {
    return null;
  }

  return $KeyPositionCopyWith<$Res>(_self.position!, (value) {
    return _then(_self.copyWith(position: value));
  });
}/// Create a copy of VirtualKeyDef
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$KeySizeCopyWith<$Res>? get size {
    if (_self.size == null) {
    return null;
  }

  return $KeySizeCopyWith<$Res>(_self.size!, (value) {
    return _then(_self.copyWith(size: value));
  });
}
}


/// @nodoc
mixin _$VirtualLayout {

 VirtualLayoutId get id; String get name;@JsonKey(name: 'layout_type') VirtualLayoutType get layoutType; List<VirtualKeyDef> get keys;
/// Create a copy of VirtualLayout
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$VirtualLayoutCopyWith<VirtualLayout> get copyWith => _$VirtualLayoutCopyWithImpl<VirtualLayout>(this as VirtualLayout, _$identity);

  /// Serializes this VirtualLayout to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is VirtualLayout&&(identical(other.id, id) || other.id == id)&&(identical(other.name, name) || other.name == name)&&(identical(other.layoutType, layoutType) || other.layoutType == layoutType)&&const DeepCollectionEquality().equals(other.keys, keys));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,id,name,layoutType,const DeepCollectionEquality().hash(keys));

@override
String toString() {
  return 'VirtualLayout(id: $id, name: $name, layoutType: $layoutType, keys: $keys)';
}


}

/// @nodoc
abstract mixin class $VirtualLayoutCopyWith<$Res>  {
  factory $VirtualLayoutCopyWith(VirtualLayout value, $Res Function(VirtualLayout) _then) = _$VirtualLayoutCopyWithImpl;
@useResult
$Res call({
 VirtualLayoutId id, String name,@JsonKey(name: 'layout_type') VirtualLayoutType layoutType, List<VirtualKeyDef> keys
});




}
/// @nodoc
class _$VirtualLayoutCopyWithImpl<$Res>
    implements $VirtualLayoutCopyWith<$Res> {
  _$VirtualLayoutCopyWithImpl(this._self, this._then);

  final VirtualLayout _self;
  final $Res Function(VirtualLayout) _then;

/// Create a copy of VirtualLayout
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? id = null,Object? name = null,Object? layoutType = null,Object? keys = null,}) {
  return _then(_self.copyWith(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as VirtualLayoutId,name: null == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as String,layoutType: null == layoutType ? _self.layoutType : layoutType // ignore: cast_nullable_to_non_nullable
as VirtualLayoutType,keys: null == keys ? _self.keys : keys // ignore: cast_nullable_to_non_nullable
as List<VirtualKeyDef>,
  ));
}

}


/// Adds pattern-matching-related methods to [VirtualLayout].
extension VirtualLayoutPatterns on VirtualLayout {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _VirtualLayout value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _VirtualLayout() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _VirtualLayout value)  $default,){
final _that = this;
switch (_that) {
case _VirtualLayout():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _VirtualLayout value)?  $default,){
final _that = this;
switch (_that) {
case _VirtualLayout() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( VirtualLayoutId id,  String name, @JsonKey(name: 'layout_type')  VirtualLayoutType layoutType,  List<VirtualKeyDef> keys)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _VirtualLayout() when $default != null:
return $default(_that.id,_that.name,_that.layoutType,_that.keys);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( VirtualLayoutId id,  String name, @JsonKey(name: 'layout_type')  VirtualLayoutType layoutType,  List<VirtualKeyDef> keys)  $default,) {final _that = this;
switch (_that) {
case _VirtualLayout():
return $default(_that.id,_that.name,_that.layoutType,_that.keys);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( VirtualLayoutId id,  String name, @JsonKey(name: 'layout_type')  VirtualLayoutType layoutType,  List<VirtualKeyDef> keys)?  $default,) {final _that = this;
switch (_that) {
case _VirtualLayout() when $default != null:
return $default(_that.id,_that.name,_that.layoutType,_that.keys);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _VirtualLayout extends VirtualLayout {
  const _VirtualLayout({required this.id, required this.name, @JsonKey(name: 'layout_type') required this.layoutType, final  List<VirtualKeyDef> keys = const <VirtualKeyDef>[]}): _keys = keys,super._();
  factory _VirtualLayout.fromJson(Map<String, dynamic> json) => _$VirtualLayoutFromJson(json);

@override final  VirtualLayoutId id;
@override final  String name;
@override@JsonKey(name: 'layout_type') final  VirtualLayoutType layoutType;
 final  List<VirtualKeyDef> _keys;
@override@JsonKey() List<VirtualKeyDef> get keys {
  if (_keys is EqualUnmodifiableListView) return _keys;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_keys);
}


/// Create a copy of VirtualLayout
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$VirtualLayoutCopyWith<_VirtualLayout> get copyWith => __$VirtualLayoutCopyWithImpl<_VirtualLayout>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$VirtualLayoutToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _VirtualLayout&&(identical(other.id, id) || other.id == id)&&(identical(other.name, name) || other.name == name)&&(identical(other.layoutType, layoutType) || other.layoutType == layoutType)&&const DeepCollectionEquality().equals(other._keys, _keys));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,id,name,layoutType,const DeepCollectionEquality().hash(_keys));

@override
String toString() {
  return 'VirtualLayout(id: $id, name: $name, layoutType: $layoutType, keys: $keys)';
}


}

/// @nodoc
abstract mixin class _$VirtualLayoutCopyWith<$Res> implements $VirtualLayoutCopyWith<$Res> {
  factory _$VirtualLayoutCopyWith(_VirtualLayout value, $Res Function(_VirtualLayout) _then) = __$VirtualLayoutCopyWithImpl;
@override @useResult
$Res call({
 VirtualLayoutId id, String name,@JsonKey(name: 'layout_type') VirtualLayoutType layoutType, List<VirtualKeyDef> keys
});




}
/// @nodoc
class __$VirtualLayoutCopyWithImpl<$Res>
    implements _$VirtualLayoutCopyWith<$Res> {
  __$VirtualLayoutCopyWithImpl(this._self, this._then);

  final _VirtualLayout _self;
  final $Res Function(_VirtualLayout) _then;

/// Create a copy of VirtualLayout
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? id = null,Object? name = null,Object? layoutType = null,Object? keys = null,}) {
  return _then(_VirtualLayout(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as VirtualLayoutId,name: null == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as String,layoutType: null == layoutType ? _self.layoutType : layoutType // ignore: cast_nullable_to_non_nullable
as VirtualLayoutType,keys: null == keys ? _self._keys : keys // ignore: cast_nullable_to_non_nullable
as List<VirtualKeyDef>,
  ));
}


}

// dart format on
