// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'flow_graph.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$FlowPosition {

 double get x; double get y;
/// Create a copy of FlowPosition
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$FlowPositionCopyWith<FlowPosition> get copyWith => _$FlowPositionCopyWithImpl<FlowPosition>(this as FlowPosition, _$identity);

  /// Serializes this FlowPosition to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is FlowPosition&&(identical(other.x, x) || other.x == x)&&(identical(other.y, y) || other.y == y));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,x,y);

@override
String toString() {
  return 'FlowPosition(x: $x, y: $y)';
}


}

/// @nodoc
abstract mixin class $FlowPositionCopyWith<$Res>  {
  factory $FlowPositionCopyWith(FlowPosition value, $Res Function(FlowPosition) _then) = _$FlowPositionCopyWithImpl;
@useResult
$Res call({
 double x, double y
});




}
/// @nodoc
class _$FlowPositionCopyWithImpl<$Res>
    implements $FlowPositionCopyWith<$Res> {
  _$FlowPositionCopyWithImpl(this._self, this._then);

  final FlowPosition _self;
  final $Res Function(FlowPosition) _then;

/// Create a copy of FlowPosition
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? x = null,Object? y = null,}) {
  return _then(_self.copyWith(
x: null == x ? _self.x : x // ignore: cast_nullable_to_non_nullable
as double,y: null == y ? _self.y : y // ignore: cast_nullable_to_non_nullable
as double,
  ));
}

}


/// Adds pattern-matching-related methods to [FlowPosition].
extension FlowPositionPatterns on FlowPosition {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _FlowPosition value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _FlowPosition() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _FlowPosition value)  $default,){
final _that = this;
switch (_that) {
case _FlowPosition():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _FlowPosition value)?  $default,){
final _that = this;
switch (_that) {
case _FlowPosition() when $default != null:
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
case _FlowPosition() when $default != null:
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
case _FlowPosition():
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
case _FlowPosition() when $default != null:
return $default(_that.x,_that.y);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _FlowPosition implements FlowPosition {
  const _FlowPosition({required this.x, required this.y});
  factory _FlowPosition.fromJson(Map<String, dynamic> json) => _$FlowPositionFromJson(json);

@override final  double x;
@override final  double y;

/// Create a copy of FlowPosition
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$FlowPositionCopyWith<_FlowPosition> get copyWith => __$FlowPositionCopyWithImpl<_FlowPosition>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$FlowPositionToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _FlowPosition&&(identical(other.x, x) || other.x == x)&&(identical(other.y, y) || other.y == y));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,x,y);

@override
String toString() {
  return 'FlowPosition(x: $x, y: $y)';
}


}

/// @nodoc
abstract mixin class _$FlowPositionCopyWith<$Res> implements $FlowPositionCopyWith<$Res> {
  factory _$FlowPositionCopyWith(_FlowPosition value, $Res Function(_FlowPosition) _then) = __$FlowPositionCopyWithImpl;
@override @useResult
$Res call({
 double x, double y
});




}
/// @nodoc
class __$FlowPositionCopyWithImpl<$Res>
    implements _$FlowPositionCopyWith<$Res> {
  __$FlowPositionCopyWithImpl(this._self, this._then);

  final _FlowPosition _self;
  final $Res Function(_FlowPosition) _then;

/// Create a copy of FlowPosition
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? x = null,Object? y = null,}) {
  return _then(_FlowPosition(
x: null == x ? _self.x : x // ignore: cast_nullable_to_non_nullable
as double,y: null == y ? _self.y : y // ignore: cast_nullable_to_non_nullable
as double,
  ));
}


}


/// @nodoc
mixin _$FlowEdge {

 String get from; String get to; String get slotId; int get priority; bool get active; EdgeStatus get status;
/// Create a copy of FlowEdge
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$FlowEdgeCopyWith<FlowEdge> get copyWith => _$FlowEdgeCopyWithImpl<FlowEdge>(this as FlowEdge, _$identity);

  /// Serializes this FlowEdge to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is FlowEdge&&(identical(other.from, from) || other.from == from)&&(identical(other.to, to) || other.to == to)&&(identical(other.slotId, slotId) || other.slotId == slotId)&&(identical(other.priority, priority) || other.priority == priority)&&(identical(other.active, active) || other.active == active)&&(identical(other.status, status) || other.status == status));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,from,to,slotId,priority,active,status);

@override
String toString() {
  return 'FlowEdge(from: $from, to: $to, slotId: $slotId, priority: $priority, active: $active, status: $status)';
}


}

/// @nodoc
abstract mixin class $FlowEdgeCopyWith<$Res>  {
  factory $FlowEdgeCopyWith(FlowEdge value, $Res Function(FlowEdge) _then) = _$FlowEdgeCopyWithImpl;
@useResult
$Res call({
 String from, String to, String slotId, int priority, bool active, EdgeStatus status
});




}
/// @nodoc
class _$FlowEdgeCopyWithImpl<$Res>
    implements $FlowEdgeCopyWith<$Res> {
  _$FlowEdgeCopyWithImpl(this._self, this._then);

  final FlowEdge _self;
  final $Res Function(FlowEdge) _then;

/// Create a copy of FlowEdge
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? from = null,Object? to = null,Object? slotId = null,Object? priority = null,Object? active = null,Object? status = null,}) {
  return _then(_self.copyWith(
from: null == from ? _self.from : from // ignore: cast_nullable_to_non_nullable
as String,to: null == to ? _self.to : to // ignore: cast_nullable_to_non_nullable
as String,slotId: null == slotId ? _self.slotId : slotId // ignore: cast_nullable_to_non_nullable
as String,priority: null == priority ? _self.priority : priority // ignore: cast_nullable_to_non_nullable
as int,active: null == active ? _self.active : active // ignore: cast_nullable_to_non_nullable
as bool,status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as EdgeStatus,
  ));
}

}


/// Adds pattern-matching-related methods to [FlowEdge].
extension FlowEdgePatterns on FlowEdge {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _FlowEdge value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _FlowEdge() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _FlowEdge value)  $default,){
final _that = this;
switch (_that) {
case _FlowEdge():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _FlowEdge value)?  $default,){
final _that = this;
switch (_that) {
case _FlowEdge() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String from,  String to,  String slotId,  int priority,  bool active,  EdgeStatus status)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _FlowEdge() when $default != null:
return $default(_that.from,_that.to,_that.slotId,_that.priority,_that.active,_that.status);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String from,  String to,  String slotId,  int priority,  bool active,  EdgeStatus status)  $default,) {final _that = this;
switch (_that) {
case _FlowEdge():
return $default(_that.from,_that.to,_that.slotId,_that.priority,_that.active,_that.status);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String from,  String to,  String slotId,  int priority,  bool active,  EdgeStatus status)?  $default,) {final _that = this;
switch (_that) {
case _FlowEdge() when $default != null:
return $default(_that.from,_that.to,_that.slotId,_that.priority,_that.active,_that.status);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _FlowEdge implements FlowEdge {
  const _FlowEdge({required this.from, required this.to, required this.slotId, required this.priority, this.active = false, this.status = EdgeStatus.valid});
  factory _FlowEdge.fromJson(Map<String, dynamic> json) => _$FlowEdgeFromJson(json);

@override final  String from;
@override final  String to;
@override final  String slotId;
@override final  int priority;
@override@JsonKey() final  bool active;
@override@JsonKey() final  EdgeStatus status;

/// Create a copy of FlowEdge
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$FlowEdgeCopyWith<_FlowEdge> get copyWith => __$FlowEdgeCopyWithImpl<_FlowEdge>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$FlowEdgeToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _FlowEdge&&(identical(other.from, from) || other.from == from)&&(identical(other.to, to) || other.to == to)&&(identical(other.slotId, slotId) || other.slotId == slotId)&&(identical(other.priority, priority) || other.priority == priority)&&(identical(other.active, active) || other.active == active)&&(identical(other.status, status) || other.status == status));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,from,to,slotId,priority,active,status);

@override
String toString() {
  return 'FlowEdge(from: $from, to: $to, slotId: $slotId, priority: $priority, active: $active, status: $status)';
}


}

/// @nodoc
abstract mixin class _$FlowEdgeCopyWith<$Res> implements $FlowEdgeCopyWith<$Res> {
  factory _$FlowEdgeCopyWith(_FlowEdge value, $Res Function(_FlowEdge) _then) = __$FlowEdgeCopyWithImpl;
@override @useResult
$Res call({
 String from, String to, String slotId, int priority, bool active, EdgeStatus status
});




}
/// @nodoc
class __$FlowEdgeCopyWithImpl<$Res>
    implements _$FlowEdgeCopyWith<$Res> {
  __$FlowEdgeCopyWithImpl(this._self, this._then);

  final _FlowEdge _self;
  final $Res Function(_FlowEdge) _then;

/// Create a copy of FlowEdge
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? from = null,Object? to = null,Object? slotId = null,Object? priority = null,Object? active = null,Object? status = null,}) {
  return _then(_FlowEdge(
from: null == from ? _self.from : from // ignore: cast_nullable_to_non_nullable
as String,to: null == to ? _self.to : to // ignore: cast_nullable_to_non_nullable
as String,slotId: null == slotId ? _self.slotId : slotId // ignore: cast_nullable_to_non_nullable
as String,priority: null == priority ? _self.priority : priority // ignore: cast_nullable_to_non_nullable
as int,active: null == active ? _self.active : active // ignore: cast_nullable_to_non_nullable
as bool,status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as EdgeStatus,
  ));
}


}

FlowNode _$FlowNodeFromJson(
  Map<String, dynamic> json
) {
        switch (json['type']) {
                  case 'device':
          return DeviceFlowNode.fromJson(
            json
          );
                case 'hardwareProfile':
          return HardwareProfileFlowNode.fromJson(
            json
          );
                case 'keymap':
          return KeymapFlowNode.fromJson(
            json
          );
        
          default:
            throw CheckedFromJsonException(
  json,
  'type',
  'FlowNode',
  'Invalid union type "${json['type']}"!'
);
        }
      
}

/// @nodoc
mixin _$FlowNode {

 String get id; String? get label;
/// Create a copy of FlowNode
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$FlowNodeCopyWith<FlowNode> get copyWith => _$FlowNodeCopyWithImpl<FlowNode>(this as FlowNode, _$identity);

  /// Serializes this FlowNode to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is FlowNode&&(identical(other.id, id) || other.id == id)&&(identical(other.label, label) || other.label == label));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,id,label);

@override
String toString() {
  return 'FlowNode(id: $id, label: $label)';
}


}

/// @nodoc
abstract mixin class $FlowNodeCopyWith<$Res>  {
  factory $FlowNodeCopyWith(FlowNode value, $Res Function(FlowNode) _then) = _$FlowNodeCopyWithImpl;
@useResult
$Res call({
 String id, String? label
});




}
/// @nodoc
class _$FlowNodeCopyWithImpl<$Res>
    implements $FlowNodeCopyWith<$Res> {
  _$FlowNodeCopyWithImpl(this._self, this._then);

  final FlowNode _self;
  final $Res Function(FlowNode) _then;

/// Create a copy of FlowNode
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? id = null,Object? label = freezed,}) {
  return _then(_self.copyWith(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,label: freezed == label ? _self.label : label // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}

}


/// Adds pattern-matching-related methods to [FlowNode].
extension FlowNodePatterns on FlowNode {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( DeviceFlowNode value)?  device,TResult Function( HardwareProfileFlowNode value)?  hardwareProfile,TResult Function( KeymapFlowNode value)?  keymap,required TResult orElse(),}){
final _that = this;
switch (_that) {
case DeviceFlowNode() when device != null:
return device(_that);case HardwareProfileFlowNode() when hardwareProfile != null:
return hardwareProfile(_that);case KeymapFlowNode() when keymap != null:
return keymap(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( DeviceFlowNode value)  device,required TResult Function( HardwareProfileFlowNode value)  hardwareProfile,required TResult Function( KeymapFlowNode value)  keymap,}){
final _that = this;
switch (_that) {
case DeviceFlowNode():
return device(_that);case HardwareProfileFlowNode():
return hardwareProfile(_that);case KeymapFlowNode():
return keymap(_that);case _:
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( DeviceFlowNode value)?  device,TResult? Function( HardwareProfileFlowNode value)?  hardwareProfile,TResult? Function( KeymapFlowNode value)?  keymap,}){
final _that = this;
switch (_that) {
case DeviceFlowNode() when device != null:
return device(_that);case HardwareProfileFlowNode() when hardwareProfile != null:
return hardwareProfile(_that);case KeymapFlowNode() when keymap != null:
return keymap(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String id,  DeviceInstanceId deviceId,  bool isConnected,  String? label)?  device,TResult Function( HardwareProfileId id,  String? label)?  hardwareProfile,TResult Function( KeymapId id,  String? label,  int? layers)?  keymap,required TResult orElse(),}) {final _that = this;
switch (_that) {
case DeviceFlowNode() when device != null:
return device(_that.id,_that.deviceId,_that.isConnected,_that.label);case HardwareProfileFlowNode() when hardwareProfile != null:
return hardwareProfile(_that.id,_that.label);case KeymapFlowNode() when keymap != null:
return keymap(_that.id,_that.label,_that.layers);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String id,  DeviceInstanceId deviceId,  bool isConnected,  String? label)  device,required TResult Function( HardwareProfileId id,  String? label)  hardwareProfile,required TResult Function( KeymapId id,  String? label,  int? layers)  keymap,}) {final _that = this;
switch (_that) {
case DeviceFlowNode():
return device(_that.id,_that.deviceId,_that.isConnected,_that.label);case HardwareProfileFlowNode():
return hardwareProfile(_that.id,_that.label);case KeymapFlowNode():
return keymap(_that.id,_that.label,_that.layers);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String id,  DeviceInstanceId deviceId,  bool isConnected,  String? label)?  device,TResult? Function( HardwareProfileId id,  String? label)?  hardwareProfile,TResult? Function( KeymapId id,  String? label,  int? layers)?  keymap,}) {final _that = this;
switch (_that) {
case DeviceFlowNode() when device != null:
return device(_that.id,_that.deviceId,_that.isConnected,_that.label);case HardwareProfileFlowNode() when hardwareProfile != null:
return hardwareProfile(_that.id,_that.label);case KeymapFlowNode() when keymap != null:
return keymap(_that.id,_that.label,_that.layers);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class DeviceFlowNode implements FlowNode {
  const DeviceFlowNode({required this.id, required this.deviceId, this.isConnected = true, this.label, final  String? $type}): $type = $type ?? 'device';
  factory DeviceFlowNode.fromJson(Map<String, dynamic> json) => _$DeviceFlowNodeFromJson(json);

@override final  String id;
 final  DeviceInstanceId deviceId;
@JsonKey() final  bool isConnected;
@override final  String? label;

@JsonKey(name: 'type')
final String $type;


/// Create a copy of FlowNode
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$DeviceFlowNodeCopyWith<DeviceFlowNode> get copyWith => _$DeviceFlowNodeCopyWithImpl<DeviceFlowNode>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$DeviceFlowNodeToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DeviceFlowNode&&(identical(other.id, id) || other.id == id)&&(identical(other.deviceId, deviceId) || other.deviceId == deviceId)&&(identical(other.isConnected, isConnected) || other.isConnected == isConnected)&&(identical(other.label, label) || other.label == label));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,id,deviceId,isConnected,label);

@override
String toString() {
  return 'FlowNode.device(id: $id, deviceId: $deviceId, isConnected: $isConnected, label: $label)';
}


}

/// @nodoc
abstract mixin class $DeviceFlowNodeCopyWith<$Res> implements $FlowNodeCopyWith<$Res> {
  factory $DeviceFlowNodeCopyWith(DeviceFlowNode value, $Res Function(DeviceFlowNode) _then) = _$DeviceFlowNodeCopyWithImpl;
@override @useResult
$Res call({
 String id, DeviceInstanceId deviceId, bool isConnected, String? label
});


$DeviceInstanceIdCopyWith<$Res> get deviceId;

}
/// @nodoc
class _$DeviceFlowNodeCopyWithImpl<$Res>
    implements $DeviceFlowNodeCopyWith<$Res> {
  _$DeviceFlowNodeCopyWithImpl(this._self, this._then);

  final DeviceFlowNode _self;
  final $Res Function(DeviceFlowNode) _then;

/// Create a copy of FlowNode
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? id = null,Object? deviceId = null,Object? isConnected = null,Object? label = freezed,}) {
  return _then(DeviceFlowNode(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,deviceId: null == deviceId ? _self.deviceId : deviceId // ignore: cast_nullable_to_non_nullable
as DeviceInstanceId,isConnected: null == isConnected ? _self.isConnected : isConnected // ignore: cast_nullable_to_non_nullable
as bool,label: freezed == label ? _self.label : label // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}

/// Create a copy of FlowNode
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$DeviceInstanceIdCopyWith<$Res> get deviceId {
  
  return $DeviceInstanceIdCopyWith<$Res>(_self.deviceId, (value) {
    return _then(_self.copyWith(deviceId: value));
  });
}
}

/// @nodoc
@JsonSerializable()

class HardwareProfileFlowNode implements FlowNode {
  const HardwareProfileFlowNode({required this.id, this.label, final  String? $type}): $type = $type ?? 'hardwareProfile';
  factory HardwareProfileFlowNode.fromJson(Map<String, dynamic> json) => _$HardwareProfileFlowNodeFromJson(json);

@override final  HardwareProfileId id;
@override final  String? label;

@JsonKey(name: 'type')
final String $type;


/// Create a copy of FlowNode
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$HardwareProfileFlowNodeCopyWith<HardwareProfileFlowNode> get copyWith => _$HardwareProfileFlowNodeCopyWithImpl<HardwareProfileFlowNode>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$HardwareProfileFlowNodeToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is HardwareProfileFlowNode&&(identical(other.id, id) || other.id == id)&&(identical(other.label, label) || other.label == label));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,id,label);

@override
String toString() {
  return 'FlowNode.hardwareProfile(id: $id, label: $label)';
}


}

/// @nodoc
abstract mixin class $HardwareProfileFlowNodeCopyWith<$Res> implements $FlowNodeCopyWith<$Res> {
  factory $HardwareProfileFlowNodeCopyWith(HardwareProfileFlowNode value, $Res Function(HardwareProfileFlowNode) _then) = _$HardwareProfileFlowNodeCopyWithImpl;
@override @useResult
$Res call({
 HardwareProfileId id, String? label
});




}
/// @nodoc
class _$HardwareProfileFlowNodeCopyWithImpl<$Res>
    implements $HardwareProfileFlowNodeCopyWith<$Res> {
  _$HardwareProfileFlowNodeCopyWithImpl(this._self, this._then);

  final HardwareProfileFlowNode _self;
  final $Res Function(HardwareProfileFlowNode) _then;

/// Create a copy of FlowNode
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? id = null,Object? label = freezed,}) {
  return _then(HardwareProfileFlowNode(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as HardwareProfileId,label: freezed == label ? _self.label : label // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

/// @nodoc
@JsonSerializable()

class KeymapFlowNode implements FlowNode {
  const KeymapFlowNode({required this.id, this.label, this.layers, final  String? $type}): $type = $type ?? 'keymap';
  factory KeymapFlowNode.fromJson(Map<String, dynamic> json) => _$KeymapFlowNodeFromJson(json);

@override final  KeymapId id;
@override final  String? label;
 final  int? layers;

@JsonKey(name: 'type')
final String $type;


/// Create a copy of FlowNode
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$KeymapFlowNodeCopyWith<KeymapFlowNode> get copyWith => _$KeymapFlowNodeCopyWithImpl<KeymapFlowNode>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$KeymapFlowNodeToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is KeymapFlowNode&&(identical(other.id, id) || other.id == id)&&(identical(other.label, label) || other.label == label)&&(identical(other.layers, layers) || other.layers == layers));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,id,label,layers);

@override
String toString() {
  return 'FlowNode.keymap(id: $id, label: $label, layers: $layers)';
}


}

/// @nodoc
abstract mixin class $KeymapFlowNodeCopyWith<$Res> implements $FlowNodeCopyWith<$Res> {
  factory $KeymapFlowNodeCopyWith(KeymapFlowNode value, $Res Function(KeymapFlowNode) _then) = _$KeymapFlowNodeCopyWithImpl;
@override @useResult
$Res call({
 KeymapId id, String? label, int? layers
});




}
/// @nodoc
class _$KeymapFlowNodeCopyWithImpl<$Res>
    implements $KeymapFlowNodeCopyWith<$Res> {
  _$KeymapFlowNodeCopyWithImpl(this._self, this._then);

  final KeymapFlowNode _self;
  final $Res Function(KeymapFlowNode) _then;

/// Create a copy of FlowNode
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? id = null,Object? label = freezed,Object? layers = freezed,}) {
  return _then(KeymapFlowNode(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as KeymapId,label: freezed == label ? _self.label : label // ignore: cast_nullable_to_non_nullable
as String?,layers: freezed == layers ? _self.layers : layers // ignore: cast_nullable_to_non_nullable
as int?,
  ));
}


}


/// @nodoc
mixin _$FlowGraph {

 List<FlowNode> get nodes; List<FlowEdge> get edges; Map<String, FlowPosition> get positions;
/// Create a copy of FlowGraph
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$FlowGraphCopyWith<FlowGraph> get copyWith => _$FlowGraphCopyWithImpl<FlowGraph>(this as FlowGraph, _$identity);

  /// Serializes this FlowGraph to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is FlowGraph&&const DeepCollectionEquality().equals(other.nodes, nodes)&&const DeepCollectionEquality().equals(other.edges, edges)&&const DeepCollectionEquality().equals(other.positions, positions));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(nodes),const DeepCollectionEquality().hash(edges),const DeepCollectionEquality().hash(positions));

@override
String toString() {
  return 'FlowGraph(nodes: $nodes, edges: $edges, positions: $positions)';
}


}

/// @nodoc
abstract mixin class $FlowGraphCopyWith<$Res>  {
  factory $FlowGraphCopyWith(FlowGraph value, $Res Function(FlowGraph) _then) = _$FlowGraphCopyWithImpl;
@useResult
$Res call({
 List<FlowNode> nodes, List<FlowEdge> edges, Map<String, FlowPosition> positions
});




}
/// @nodoc
class _$FlowGraphCopyWithImpl<$Res>
    implements $FlowGraphCopyWith<$Res> {
  _$FlowGraphCopyWithImpl(this._self, this._then);

  final FlowGraph _self;
  final $Res Function(FlowGraph) _then;

/// Create a copy of FlowGraph
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? nodes = null,Object? edges = null,Object? positions = null,}) {
  return _then(_self.copyWith(
nodes: null == nodes ? _self.nodes : nodes // ignore: cast_nullable_to_non_nullable
as List<FlowNode>,edges: null == edges ? _self.edges : edges // ignore: cast_nullable_to_non_nullable
as List<FlowEdge>,positions: null == positions ? _self.positions : positions // ignore: cast_nullable_to_non_nullable
as Map<String, FlowPosition>,
  ));
}

}


/// Adds pattern-matching-related methods to [FlowGraph].
extension FlowGraphPatterns on FlowGraph {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _FlowGraph value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _FlowGraph() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _FlowGraph value)  $default,){
final _that = this;
switch (_that) {
case _FlowGraph():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _FlowGraph value)?  $default,){
final _that = this;
switch (_that) {
case _FlowGraph() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( List<FlowNode> nodes,  List<FlowEdge> edges,  Map<String, FlowPosition> positions)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _FlowGraph() when $default != null:
return $default(_that.nodes,_that.edges,_that.positions);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( List<FlowNode> nodes,  List<FlowEdge> edges,  Map<String, FlowPosition> positions)  $default,) {final _that = this;
switch (_that) {
case _FlowGraph():
return $default(_that.nodes,_that.edges,_that.positions);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( List<FlowNode> nodes,  List<FlowEdge> edges,  Map<String, FlowPosition> positions)?  $default,) {final _that = this;
switch (_that) {
case _FlowGraph() when $default != null:
return $default(_that.nodes,_that.edges,_that.positions);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _FlowGraph extends FlowGraph {
  const _FlowGraph({final  List<FlowNode> nodes = const <FlowNode>[], final  List<FlowEdge> edges = const <FlowEdge>[], final  Map<String, FlowPosition> positions = const <String, FlowPosition>{}}): _nodes = nodes,_edges = edges,_positions = positions,super._();
  factory _FlowGraph.fromJson(Map<String, dynamic> json) => _$FlowGraphFromJson(json);

 final  List<FlowNode> _nodes;
@override@JsonKey() List<FlowNode> get nodes {
  if (_nodes is EqualUnmodifiableListView) return _nodes;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_nodes);
}

 final  List<FlowEdge> _edges;
@override@JsonKey() List<FlowEdge> get edges {
  if (_edges is EqualUnmodifiableListView) return _edges;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_edges);
}

 final  Map<String, FlowPosition> _positions;
@override@JsonKey() Map<String, FlowPosition> get positions {
  if (_positions is EqualUnmodifiableMapView) return _positions;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableMapView(_positions);
}


/// Create a copy of FlowGraph
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$FlowGraphCopyWith<_FlowGraph> get copyWith => __$FlowGraphCopyWithImpl<_FlowGraph>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$FlowGraphToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _FlowGraph&&const DeepCollectionEquality().equals(other._nodes, _nodes)&&const DeepCollectionEquality().equals(other._edges, _edges)&&const DeepCollectionEquality().equals(other._positions, _positions));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_nodes),const DeepCollectionEquality().hash(_edges),const DeepCollectionEquality().hash(_positions));

@override
String toString() {
  return 'FlowGraph(nodes: $nodes, edges: $edges, positions: $positions)';
}


}

/// @nodoc
abstract mixin class _$FlowGraphCopyWith<$Res> implements $FlowGraphCopyWith<$Res> {
  factory _$FlowGraphCopyWith(_FlowGraph value, $Res Function(_FlowGraph) _then) = __$FlowGraphCopyWithImpl;
@override @useResult
$Res call({
 List<FlowNode> nodes, List<FlowEdge> edges, Map<String, FlowPosition> positions
});




}
/// @nodoc
class __$FlowGraphCopyWithImpl<$Res>
    implements _$FlowGraphCopyWith<$Res> {
  __$FlowGraphCopyWithImpl(this._self, this._then);

  final _FlowGraph _self;
  final $Res Function(_FlowGraph) _then;

/// Create a copy of FlowGraph
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? nodes = null,Object? edges = null,Object? positions = null,}) {
  return _then(_FlowGraph(
nodes: null == nodes ? _self._nodes : nodes // ignore: cast_nullable_to_non_nullable
as List<FlowNode>,edges: null == edges ? _self._edges : edges // ignore: cast_nullable_to_non_nullable
as List<FlowEdge>,positions: null == positions ? _self._positions : positions // ignore: cast_nullable_to_non_nullable
as Map<String, FlowPosition>,
  ));
}


}

// dart format on
