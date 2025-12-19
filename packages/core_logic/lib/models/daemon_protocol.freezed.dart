// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'daemon_protocol.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$JsonRpcRequest {

 String get jsonrpc; String get method; RequestId get id; Map<String, dynamic>? get params;
/// Create a copy of JsonRpcRequest
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$JsonRpcRequestCopyWith<JsonRpcRequest> get copyWith => _$JsonRpcRequestCopyWithImpl<JsonRpcRequest>(this as JsonRpcRequest, _$identity);

  /// Serializes this JsonRpcRequest to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is JsonRpcRequest&&(identical(other.jsonrpc, jsonrpc) || other.jsonrpc == jsonrpc)&&(identical(other.method, method) || other.method == method)&&(identical(other.id, id) || other.id == id)&&const DeepCollectionEquality().equals(other.params, params));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,jsonrpc,method,id,const DeepCollectionEquality().hash(params));

@override
String toString() {
  return 'JsonRpcRequest(jsonrpc: $jsonrpc, method: $method, id: $id, params: $params)';
}


}

/// @nodoc
abstract mixin class $JsonRpcRequestCopyWith<$Res>  {
  factory $JsonRpcRequestCopyWith(JsonRpcRequest value, $Res Function(JsonRpcRequest) _then) = _$JsonRpcRequestCopyWithImpl;
@useResult
$Res call({
 String jsonrpc, String method, RequestId id, Map<String, dynamic>? params
});




}
/// @nodoc
class _$JsonRpcRequestCopyWithImpl<$Res>
    implements $JsonRpcRequestCopyWith<$Res> {
  _$JsonRpcRequestCopyWithImpl(this._self, this._then);

  final JsonRpcRequest _self;
  final $Res Function(JsonRpcRequest) _then;

/// Create a copy of JsonRpcRequest
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? jsonrpc = null,Object? method = null,Object? id = null,Object? params = freezed,}) {
  return _then(_self.copyWith(
jsonrpc: null == jsonrpc ? _self.jsonrpc : jsonrpc // ignore: cast_nullable_to_non_nullable
as String,method: null == method ? _self.method : method // ignore: cast_nullable_to_non_nullable
as String,id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as RequestId,params: freezed == params ? _self.params : params // ignore: cast_nullable_to_non_nullable
as Map<String, dynamic>?,
  ));
}

}


/// Adds pattern-matching-related methods to [JsonRpcRequest].
extension JsonRpcRequestPatterns on JsonRpcRequest {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _JsonRpcRequest value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _JsonRpcRequest() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _JsonRpcRequest value)  $default,){
final _that = this;
switch (_that) {
case _JsonRpcRequest():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _JsonRpcRequest value)?  $default,){
final _that = this;
switch (_that) {
case _JsonRpcRequest() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String jsonrpc,  String method,  RequestId id,  Map<String, dynamic>? params)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _JsonRpcRequest() when $default != null:
return $default(_that.jsonrpc,_that.method,_that.id,_that.params);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String jsonrpc,  String method,  RequestId id,  Map<String, dynamic>? params)  $default,) {final _that = this;
switch (_that) {
case _JsonRpcRequest():
return $default(_that.jsonrpc,_that.method,_that.id,_that.params);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String jsonrpc,  String method,  RequestId id,  Map<String, dynamic>? params)?  $default,) {final _that = this;
switch (_that) {
case _JsonRpcRequest() when $default != null:
return $default(_that.jsonrpc,_that.method,_that.id,_that.params);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _JsonRpcRequest implements JsonRpcRequest {
  const _JsonRpcRequest({required this.jsonrpc, required this.method, required this.id, final  Map<String, dynamic>? params}): _params = params;
  factory _JsonRpcRequest.fromJson(Map<String, dynamic> json) => _$JsonRpcRequestFromJson(json);

@override final  String jsonrpc;
@override final  String method;
@override final  RequestId id;
 final  Map<String, dynamic>? _params;
@override Map<String, dynamic>? get params {
  final value = _params;
  if (value == null) return null;
  if (_params is EqualUnmodifiableMapView) return _params;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableMapView(value);
}


/// Create a copy of JsonRpcRequest
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$JsonRpcRequestCopyWith<_JsonRpcRequest> get copyWith => __$JsonRpcRequestCopyWithImpl<_JsonRpcRequest>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$JsonRpcRequestToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _JsonRpcRequest&&(identical(other.jsonrpc, jsonrpc) || other.jsonrpc == jsonrpc)&&(identical(other.method, method) || other.method == method)&&(identical(other.id, id) || other.id == id)&&const DeepCollectionEquality().equals(other._params, _params));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,jsonrpc,method,id,const DeepCollectionEquality().hash(_params));

@override
String toString() {
  return 'JsonRpcRequest(jsonrpc: $jsonrpc, method: $method, id: $id, params: $params)';
}


}

/// @nodoc
abstract mixin class _$JsonRpcRequestCopyWith<$Res> implements $JsonRpcRequestCopyWith<$Res> {
  factory _$JsonRpcRequestCopyWith(_JsonRpcRequest value, $Res Function(_JsonRpcRequest) _then) = __$JsonRpcRequestCopyWithImpl;
@override @useResult
$Res call({
 String jsonrpc, String method, RequestId id, Map<String, dynamic>? params
});




}
/// @nodoc
class __$JsonRpcRequestCopyWithImpl<$Res>
    implements _$JsonRpcRequestCopyWith<$Res> {
  __$JsonRpcRequestCopyWithImpl(this._self, this._then);

  final _JsonRpcRequest _self;
  final $Res Function(_JsonRpcRequest) _then;

/// Create a copy of JsonRpcRequest
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? jsonrpc = null,Object? method = null,Object? id = null,Object? params = freezed,}) {
  return _then(_JsonRpcRequest(
jsonrpc: null == jsonrpc ? _self.jsonrpc : jsonrpc // ignore: cast_nullable_to_non_nullable
as String,method: null == method ? _self.method : method // ignore: cast_nullable_to_non_nullable
as String,id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as RequestId,params: freezed == params ? _self._params : params // ignore: cast_nullable_to_non_nullable
as Map<String, dynamic>?,
  ));
}


}


/// @nodoc
mixin _$JsonRpcResponse {

 String get jsonrpc; dynamic get result; RequestId get id;
/// Create a copy of JsonRpcResponse
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$JsonRpcResponseCopyWith<JsonRpcResponse> get copyWith => _$JsonRpcResponseCopyWithImpl<JsonRpcResponse>(this as JsonRpcResponse, _$identity);

  /// Serializes this JsonRpcResponse to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is JsonRpcResponse&&(identical(other.jsonrpc, jsonrpc) || other.jsonrpc == jsonrpc)&&const DeepCollectionEquality().equals(other.result, result)&&(identical(other.id, id) || other.id == id));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,jsonrpc,const DeepCollectionEquality().hash(result),id);

@override
String toString() {
  return 'JsonRpcResponse(jsonrpc: $jsonrpc, result: $result, id: $id)';
}


}

/// @nodoc
abstract mixin class $JsonRpcResponseCopyWith<$Res>  {
  factory $JsonRpcResponseCopyWith(JsonRpcResponse value, $Res Function(JsonRpcResponse) _then) = _$JsonRpcResponseCopyWithImpl;
@useResult
$Res call({
 String jsonrpc, dynamic result, RequestId id
});




}
/// @nodoc
class _$JsonRpcResponseCopyWithImpl<$Res>
    implements $JsonRpcResponseCopyWith<$Res> {
  _$JsonRpcResponseCopyWithImpl(this._self, this._then);

  final JsonRpcResponse _self;
  final $Res Function(JsonRpcResponse) _then;

/// Create a copy of JsonRpcResponse
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? jsonrpc = null,Object? result = freezed,Object? id = null,}) {
  return _then(_self.copyWith(
jsonrpc: null == jsonrpc ? _self.jsonrpc : jsonrpc // ignore: cast_nullable_to_non_nullable
as String,result: freezed == result ? _self.result : result // ignore: cast_nullable_to_non_nullable
as dynamic,id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as RequestId,
  ));
}

}


/// Adds pattern-matching-related methods to [JsonRpcResponse].
extension JsonRpcResponsePatterns on JsonRpcResponse {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _JsonRpcResponse value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _JsonRpcResponse() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _JsonRpcResponse value)  $default,){
final _that = this;
switch (_that) {
case _JsonRpcResponse():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _JsonRpcResponse value)?  $default,){
final _that = this;
switch (_that) {
case _JsonRpcResponse() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String jsonrpc,  dynamic result,  RequestId id)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _JsonRpcResponse() when $default != null:
return $default(_that.jsonrpc,_that.result,_that.id);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String jsonrpc,  dynamic result,  RequestId id)  $default,) {final _that = this;
switch (_that) {
case _JsonRpcResponse():
return $default(_that.jsonrpc,_that.result,_that.id);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String jsonrpc,  dynamic result,  RequestId id)?  $default,) {final _that = this;
switch (_that) {
case _JsonRpcResponse() when $default != null:
return $default(_that.jsonrpc,_that.result,_that.id);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _JsonRpcResponse implements JsonRpcResponse {
  const _JsonRpcResponse({required this.jsonrpc, required this.result, required this.id});
  factory _JsonRpcResponse.fromJson(Map<String, dynamic> json) => _$JsonRpcResponseFromJson(json);

@override final  String jsonrpc;
@override final  dynamic result;
@override final  RequestId id;

/// Create a copy of JsonRpcResponse
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$JsonRpcResponseCopyWith<_JsonRpcResponse> get copyWith => __$JsonRpcResponseCopyWithImpl<_JsonRpcResponse>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$JsonRpcResponseToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _JsonRpcResponse&&(identical(other.jsonrpc, jsonrpc) || other.jsonrpc == jsonrpc)&&const DeepCollectionEquality().equals(other.result, result)&&(identical(other.id, id) || other.id == id));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,jsonrpc,const DeepCollectionEquality().hash(result),id);

@override
String toString() {
  return 'JsonRpcResponse(jsonrpc: $jsonrpc, result: $result, id: $id)';
}


}

/// @nodoc
abstract mixin class _$JsonRpcResponseCopyWith<$Res> implements $JsonRpcResponseCopyWith<$Res> {
  factory _$JsonRpcResponseCopyWith(_JsonRpcResponse value, $Res Function(_JsonRpcResponse) _then) = __$JsonRpcResponseCopyWithImpl;
@override @useResult
$Res call({
 String jsonrpc, dynamic result, RequestId id
});




}
/// @nodoc
class __$JsonRpcResponseCopyWithImpl<$Res>
    implements _$JsonRpcResponseCopyWith<$Res> {
  __$JsonRpcResponseCopyWithImpl(this._self, this._then);

  final _JsonRpcResponse _self;
  final $Res Function(_JsonRpcResponse) _then;

/// Create a copy of JsonRpcResponse
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? jsonrpc = null,Object? result = freezed,Object? id = null,}) {
  return _then(_JsonRpcResponse(
jsonrpc: null == jsonrpc ? _self.jsonrpc : jsonrpc // ignore: cast_nullable_to_non_nullable
as String,result: freezed == result ? _self.result : result // ignore: cast_nullable_to_non_nullable
as dynamic,id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as RequestId,
  ));
}


}


/// @nodoc
mixin _$ErrorObject {

 int get code; String get message; dynamic get data;
/// Create a copy of ErrorObject
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ErrorObjectCopyWith<ErrorObject> get copyWith => _$ErrorObjectCopyWithImpl<ErrorObject>(this as ErrorObject, _$identity);

  /// Serializes this ErrorObject to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ErrorObject&&(identical(other.code, code) || other.code == code)&&(identical(other.message, message) || other.message == message)&&const DeepCollectionEquality().equals(other.data, data));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,code,message,const DeepCollectionEquality().hash(data));

@override
String toString() {
  return 'ErrorObject(code: $code, message: $message, data: $data)';
}


}

/// @nodoc
abstract mixin class $ErrorObjectCopyWith<$Res>  {
  factory $ErrorObjectCopyWith(ErrorObject value, $Res Function(ErrorObject) _then) = _$ErrorObjectCopyWithImpl;
@useResult
$Res call({
 int code, String message, dynamic data
});




}
/// @nodoc
class _$ErrorObjectCopyWithImpl<$Res>
    implements $ErrorObjectCopyWith<$Res> {
  _$ErrorObjectCopyWithImpl(this._self, this._then);

  final ErrorObject _self;
  final $Res Function(ErrorObject) _then;

/// Create a copy of ErrorObject
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? code = null,Object? message = null,Object? data = freezed,}) {
  return _then(_self.copyWith(
code: null == code ? _self.code : code // ignore: cast_nullable_to_non_nullable
as int,message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,data: freezed == data ? _self.data : data // ignore: cast_nullable_to_non_nullable
as dynamic,
  ));
}

}


/// Adds pattern-matching-related methods to [ErrorObject].
extension ErrorObjectPatterns on ErrorObject {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _ErrorObject value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _ErrorObject() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _ErrorObject value)  $default,){
final _that = this;
switch (_that) {
case _ErrorObject():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _ErrorObject value)?  $default,){
final _that = this;
switch (_that) {
case _ErrorObject() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( int code,  String message,  dynamic data)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _ErrorObject() when $default != null:
return $default(_that.code,_that.message,_that.data);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( int code,  String message,  dynamic data)  $default,) {final _that = this;
switch (_that) {
case _ErrorObject():
return $default(_that.code,_that.message,_that.data);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( int code,  String message,  dynamic data)?  $default,) {final _that = this;
switch (_that) {
case _ErrorObject() when $default != null:
return $default(_that.code,_that.message,_that.data);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _ErrorObject implements ErrorObject {
  const _ErrorObject({required this.code, required this.message, this.data});
  factory _ErrorObject.fromJson(Map<String, dynamic> json) => _$ErrorObjectFromJson(json);

@override final  int code;
@override final  String message;
@override final  dynamic data;

/// Create a copy of ErrorObject
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$ErrorObjectCopyWith<_ErrorObject> get copyWith => __$ErrorObjectCopyWithImpl<_ErrorObject>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$ErrorObjectToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _ErrorObject&&(identical(other.code, code) || other.code == code)&&(identical(other.message, message) || other.message == message)&&const DeepCollectionEquality().equals(other.data, data));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,code,message,const DeepCollectionEquality().hash(data));

@override
String toString() {
  return 'ErrorObject(code: $code, message: $message, data: $data)';
}


}

/// @nodoc
abstract mixin class _$ErrorObjectCopyWith<$Res> implements $ErrorObjectCopyWith<$Res> {
  factory _$ErrorObjectCopyWith(_ErrorObject value, $Res Function(_ErrorObject) _then) = __$ErrorObjectCopyWithImpl;
@override @useResult
$Res call({
 int code, String message, dynamic data
});




}
/// @nodoc
class __$ErrorObjectCopyWithImpl<$Res>
    implements _$ErrorObjectCopyWith<$Res> {
  __$ErrorObjectCopyWithImpl(this._self, this._then);

  final _ErrorObject _self;
  final $Res Function(_ErrorObject) _then;

/// Create a copy of ErrorObject
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? code = null,Object? message = null,Object? data = freezed,}) {
  return _then(_ErrorObject(
code: null == code ? _self.code : code // ignore: cast_nullable_to_non_nullable
as int,message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,data: freezed == data ? _self.data : data // ignore: cast_nullable_to_non_nullable
as dynamic,
  ));
}


}


/// @nodoc
mixin _$JsonRpcError {

 String get jsonrpc; ErrorObject get error; RequestId get id;
/// Create a copy of JsonRpcError
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$JsonRpcErrorCopyWith<JsonRpcError> get copyWith => _$JsonRpcErrorCopyWithImpl<JsonRpcError>(this as JsonRpcError, _$identity);

  /// Serializes this JsonRpcError to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is JsonRpcError&&(identical(other.jsonrpc, jsonrpc) || other.jsonrpc == jsonrpc)&&(identical(other.error, error) || other.error == error)&&(identical(other.id, id) || other.id == id));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,jsonrpc,error,id);

@override
String toString() {
  return 'JsonRpcError(jsonrpc: $jsonrpc, error: $error, id: $id)';
}


}

/// @nodoc
abstract mixin class $JsonRpcErrorCopyWith<$Res>  {
  factory $JsonRpcErrorCopyWith(JsonRpcError value, $Res Function(JsonRpcError) _then) = _$JsonRpcErrorCopyWithImpl;
@useResult
$Res call({
 String jsonrpc, ErrorObject error, RequestId id
});


$ErrorObjectCopyWith<$Res> get error;

}
/// @nodoc
class _$JsonRpcErrorCopyWithImpl<$Res>
    implements $JsonRpcErrorCopyWith<$Res> {
  _$JsonRpcErrorCopyWithImpl(this._self, this._then);

  final JsonRpcError _self;
  final $Res Function(JsonRpcError) _then;

/// Create a copy of JsonRpcError
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? jsonrpc = null,Object? error = null,Object? id = null,}) {
  return _then(_self.copyWith(
jsonrpc: null == jsonrpc ? _self.jsonrpc : jsonrpc // ignore: cast_nullable_to_non_nullable
as String,error: null == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ErrorObject,id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as RequestId,
  ));
}
/// Create a copy of JsonRpcError
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ErrorObjectCopyWith<$Res> get error {
  
  return $ErrorObjectCopyWith<$Res>(_self.error, (value) {
    return _then(_self.copyWith(error: value));
  });
}
}


/// Adds pattern-matching-related methods to [JsonRpcError].
extension JsonRpcErrorPatterns on JsonRpcError {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _JsonRpcError value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _JsonRpcError() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _JsonRpcError value)  $default,){
final _that = this;
switch (_that) {
case _JsonRpcError():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _JsonRpcError value)?  $default,){
final _that = this;
switch (_that) {
case _JsonRpcError() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String jsonrpc,  ErrorObject error,  RequestId id)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _JsonRpcError() when $default != null:
return $default(_that.jsonrpc,_that.error,_that.id);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String jsonrpc,  ErrorObject error,  RequestId id)  $default,) {final _that = this;
switch (_that) {
case _JsonRpcError():
return $default(_that.jsonrpc,_that.error,_that.id);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String jsonrpc,  ErrorObject error,  RequestId id)?  $default,) {final _that = this;
switch (_that) {
case _JsonRpcError() when $default != null:
return $default(_that.jsonrpc,_that.error,_that.id);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _JsonRpcError implements JsonRpcError {
  const _JsonRpcError({required this.jsonrpc, required this.error, required this.id});
  factory _JsonRpcError.fromJson(Map<String, dynamic> json) => _$JsonRpcErrorFromJson(json);

@override final  String jsonrpc;
@override final  ErrorObject error;
@override final  RequestId id;

/// Create a copy of JsonRpcError
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$JsonRpcErrorCopyWith<_JsonRpcError> get copyWith => __$JsonRpcErrorCopyWithImpl<_JsonRpcError>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$JsonRpcErrorToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _JsonRpcError&&(identical(other.jsonrpc, jsonrpc) || other.jsonrpc == jsonrpc)&&(identical(other.error, error) || other.error == error)&&(identical(other.id, id) || other.id == id));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,jsonrpc,error,id);

@override
String toString() {
  return 'JsonRpcError(jsonrpc: $jsonrpc, error: $error, id: $id)';
}


}

/// @nodoc
abstract mixin class _$JsonRpcErrorCopyWith<$Res> implements $JsonRpcErrorCopyWith<$Res> {
  factory _$JsonRpcErrorCopyWith(_JsonRpcError value, $Res Function(_JsonRpcError) _then) = __$JsonRpcErrorCopyWithImpl;
@override @useResult
$Res call({
 String jsonrpc, ErrorObject error, RequestId id
});


@override $ErrorObjectCopyWith<$Res> get error;

}
/// @nodoc
class __$JsonRpcErrorCopyWithImpl<$Res>
    implements _$JsonRpcErrorCopyWith<$Res> {
  __$JsonRpcErrorCopyWithImpl(this._self, this._then);

  final _JsonRpcError _self;
  final $Res Function(_JsonRpcError) _then;

/// Create a copy of JsonRpcError
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? jsonrpc = null,Object? error = null,Object? id = null,}) {
  return _then(_JsonRpcError(
jsonrpc: null == jsonrpc ? _self.jsonrpc : jsonrpc // ignore: cast_nullable_to_non_nullable
as String,error: null == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ErrorObject,id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as RequestId,
  ));
}

/// Create a copy of JsonRpcError
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ErrorObjectCopyWith<$Res> get error {
  
  return $ErrorObjectCopyWith<$Res>(_self.error, (value) {
    return _then(_self.copyWith(error: value));
  });
}
}


/// @nodoc
mixin _$JsonRpcNotification {

 String get jsonrpc; String get method; Map<String, dynamic> get params;
/// Create a copy of JsonRpcNotification
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$JsonRpcNotificationCopyWith<JsonRpcNotification> get copyWith => _$JsonRpcNotificationCopyWithImpl<JsonRpcNotification>(this as JsonRpcNotification, _$identity);

  /// Serializes this JsonRpcNotification to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is JsonRpcNotification&&(identical(other.jsonrpc, jsonrpc) || other.jsonrpc == jsonrpc)&&(identical(other.method, method) || other.method == method)&&const DeepCollectionEquality().equals(other.params, params));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,jsonrpc,method,const DeepCollectionEquality().hash(params));

@override
String toString() {
  return 'JsonRpcNotification(jsonrpc: $jsonrpc, method: $method, params: $params)';
}


}

/// @nodoc
abstract mixin class $JsonRpcNotificationCopyWith<$Res>  {
  factory $JsonRpcNotificationCopyWith(JsonRpcNotification value, $Res Function(JsonRpcNotification) _then) = _$JsonRpcNotificationCopyWithImpl;
@useResult
$Res call({
 String jsonrpc, String method, Map<String, dynamic> params
});




}
/// @nodoc
class _$JsonRpcNotificationCopyWithImpl<$Res>
    implements $JsonRpcNotificationCopyWith<$Res> {
  _$JsonRpcNotificationCopyWithImpl(this._self, this._then);

  final JsonRpcNotification _self;
  final $Res Function(JsonRpcNotification) _then;

/// Create a copy of JsonRpcNotification
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? jsonrpc = null,Object? method = null,Object? params = null,}) {
  return _then(_self.copyWith(
jsonrpc: null == jsonrpc ? _self.jsonrpc : jsonrpc // ignore: cast_nullable_to_non_nullable
as String,method: null == method ? _self.method : method // ignore: cast_nullable_to_non_nullable
as String,params: null == params ? _self.params : params // ignore: cast_nullable_to_non_nullable
as Map<String, dynamic>,
  ));
}

}


/// Adds pattern-matching-related methods to [JsonRpcNotification].
extension JsonRpcNotificationPatterns on JsonRpcNotification {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _JsonRpcNotification value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _JsonRpcNotification() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _JsonRpcNotification value)  $default,){
final _that = this;
switch (_that) {
case _JsonRpcNotification():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _JsonRpcNotification value)?  $default,){
final _that = this;
switch (_that) {
case _JsonRpcNotification() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String jsonrpc,  String method,  Map<String, dynamic> params)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _JsonRpcNotification() when $default != null:
return $default(_that.jsonrpc,_that.method,_that.params);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String jsonrpc,  String method,  Map<String, dynamic> params)  $default,) {final _that = this;
switch (_that) {
case _JsonRpcNotification():
return $default(_that.jsonrpc,_that.method,_that.params);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String jsonrpc,  String method,  Map<String, dynamic> params)?  $default,) {final _that = this;
switch (_that) {
case _JsonRpcNotification() when $default != null:
return $default(_that.jsonrpc,_that.method,_that.params);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _JsonRpcNotification implements JsonRpcNotification {
  const _JsonRpcNotification({required this.jsonrpc, required this.method, required final  Map<String, dynamic> params}): _params = params;
  factory _JsonRpcNotification.fromJson(Map<String, dynamic> json) => _$JsonRpcNotificationFromJson(json);

@override final  String jsonrpc;
@override final  String method;
 final  Map<String, dynamic> _params;
@override Map<String, dynamic> get params {
  if (_params is EqualUnmodifiableMapView) return _params;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableMapView(_params);
}


/// Create a copy of JsonRpcNotification
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$JsonRpcNotificationCopyWith<_JsonRpcNotification> get copyWith => __$JsonRpcNotificationCopyWithImpl<_JsonRpcNotification>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$JsonRpcNotificationToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _JsonRpcNotification&&(identical(other.jsonrpc, jsonrpc) || other.jsonrpc == jsonrpc)&&(identical(other.method, method) || other.method == method)&&const DeepCollectionEquality().equals(other._params, _params));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,jsonrpc,method,const DeepCollectionEquality().hash(_params));

@override
String toString() {
  return 'JsonRpcNotification(jsonrpc: $jsonrpc, method: $method, params: $params)';
}


}

/// @nodoc
abstract mixin class _$JsonRpcNotificationCopyWith<$Res> implements $JsonRpcNotificationCopyWith<$Res> {
  factory _$JsonRpcNotificationCopyWith(_JsonRpcNotification value, $Res Function(_JsonRpcNotification) _then) = __$JsonRpcNotificationCopyWithImpl;
@override @useResult
$Res call({
 String jsonrpc, String method, Map<String, dynamic> params
});




}
/// @nodoc
class __$JsonRpcNotificationCopyWithImpl<$Res>
    implements _$JsonRpcNotificationCopyWith<$Res> {
  __$JsonRpcNotificationCopyWithImpl(this._self, this._then);

  final _JsonRpcNotification _self;
  final $Res Function(_JsonRpcNotification) _then;

/// Create a copy of JsonRpcNotification
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? jsonrpc = null,Object? method = null,Object? params = null,}) {
  return _then(_JsonRpcNotification(
jsonrpc: null == jsonrpc ? _self.jsonrpc : jsonrpc // ignore: cast_nullable_to_non_nullable
as String,method: null == method ? _self.method : method // ignore: cast_nullable_to_non_nullable
as String,params: null == params ? _self._params : params // ignore: cast_nullable_to_non_nullable
as Map<String, dynamic>,
  ));
}


}

/// @nodoc
mixin _$IncomingMessage {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is IncomingMessage);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'IncomingMessage()';
}


}

/// @nodoc
class $IncomingMessageCopyWith<$Res>  {
$IncomingMessageCopyWith(IncomingMessage _, $Res Function(IncomingMessage) __);
}


/// Adds pattern-matching-related methods to [IncomingMessage].
extension IncomingMessagePatterns on IncomingMessage {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( _IncomingResponse value)?  response,TResult Function( _IncomingError value)?  error,TResult Function( _IncomingNotification value)?  notification,required TResult orElse(),}){
final _that = this;
switch (_that) {
case _IncomingResponse() when response != null:
return response(_that);case _IncomingError() when error != null:
return error(_that);case _IncomingNotification() when notification != null:
return notification(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( _IncomingResponse value)  response,required TResult Function( _IncomingError value)  error,required TResult Function( _IncomingNotification value)  notification,}){
final _that = this;
switch (_that) {
case _IncomingResponse():
return response(_that);case _IncomingError():
return error(_that);case _IncomingNotification():
return notification(_that);case _:
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( _IncomingResponse value)?  response,TResult? Function( _IncomingError value)?  error,TResult? Function( _IncomingNotification value)?  notification,}){
final _that = this;
switch (_that) {
case _IncomingResponse() when response != null:
return response(_that);case _IncomingError() when error != null:
return error(_that);case _IncomingNotification() when notification != null:
return notification(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( JsonRpcResponse response)?  response,TResult Function( JsonRpcError error)?  error,TResult Function( JsonRpcNotification notification)?  notification,required TResult orElse(),}) {final _that = this;
switch (_that) {
case _IncomingResponse() when response != null:
return response(_that.response);case _IncomingError() when error != null:
return error(_that.error);case _IncomingNotification() when notification != null:
return notification(_that.notification);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( JsonRpcResponse response)  response,required TResult Function( JsonRpcError error)  error,required TResult Function( JsonRpcNotification notification)  notification,}) {final _that = this;
switch (_that) {
case _IncomingResponse():
return response(_that.response);case _IncomingError():
return error(_that.error);case _IncomingNotification():
return notification(_that.notification);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( JsonRpcResponse response)?  response,TResult? Function( JsonRpcError error)?  error,TResult? Function( JsonRpcNotification notification)?  notification,}) {final _that = this;
switch (_that) {
case _IncomingResponse() when response != null:
return response(_that.response);case _IncomingError() when error != null:
return error(_that.error);case _IncomingNotification() when notification != null:
return notification(_that.notification);case _:
  return null;

}
}

}

/// @nodoc


class _IncomingResponse implements IncomingMessage {
  const _IncomingResponse(this.response);
  

 final  JsonRpcResponse response;

/// Create a copy of IncomingMessage
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$IncomingResponseCopyWith<_IncomingResponse> get copyWith => __$IncomingResponseCopyWithImpl<_IncomingResponse>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _IncomingResponse&&(identical(other.response, response) || other.response == response));
}


@override
int get hashCode => Object.hash(runtimeType,response);

@override
String toString() {
  return 'IncomingMessage.response(response: $response)';
}


}

/// @nodoc
abstract mixin class _$IncomingResponseCopyWith<$Res> implements $IncomingMessageCopyWith<$Res> {
  factory _$IncomingResponseCopyWith(_IncomingResponse value, $Res Function(_IncomingResponse) _then) = __$IncomingResponseCopyWithImpl;
@useResult
$Res call({
 JsonRpcResponse response
});


$JsonRpcResponseCopyWith<$Res> get response;

}
/// @nodoc
class __$IncomingResponseCopyWithImpl<$Res>
    implements _$IncomingResponseCopyWith<$Res> {
  __$IncomingResponseCopyWithImpl(this._self, this._then);

  final _IncomingResponse _self;
  final $Res Function(_IncomingResponse) _then;

/// Create a copy of IncomingMessage
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? response = null,}) {
  return _then(_IncomingResponse(
null == response ? _self.response : response // ignore: cast_nullable_to_non_nullable
as JsonRpcResponse,
  ));
}

/// Create a copy of IncomingMessage
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$JsonRpcResponseCopyWith<$Res> get response {
  
  return $JsonRpcResponseCopyWith<$Res>(_self.response, (value) {
    return _then(_self.copyWith(response: value));
  });
}
}

/// @nodoc


class _IncomingError implements IncomingMessage {
  const _IncomingError(this.error);
  

 final  JsonRpcError error;

/// Create a copy of IncomingMessage
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$IncomingErrorCopyWith<_IncomingError> get copyWith => __$IncomingErrorCopyWithImpl<_IncomingError>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _IncomingError&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,error);

@override
String toString() {
  return 'IncomingMessage.error(error: $error)';
}


}

/// @nodoc
abstract mixin class _$IncomingErrorCopyWith<$Res> implements $IncomingMessageCopyWith<$Res> {
  factory _$IncomingErrorCopyWith(_IncomingError value, $Res Function(_IncomingError) _then) = __$IncomingErrorCopyWithImpl;
@useResult
$Res call({
 JsonRpcError error
});


$JsonRpcErrorCopyWith<$Res> get error;

}
/// @nodoc
class __$IncomingErrorCopyWithImpl<$Res>
    implements _$IncomingErrorCopyWith<$Res> {
  __$IncomingErrorCopyWithImpl(this._self, this._then);

  final _IncomingError _self;
  final $Res Function(_IncomingError) _then;

/// Create a copy of IncomingMessage
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? error = null,}) {
  return _then(_IncomingError(
null == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as JsonRpcError,
  ));
}

/// Create a copy of IncomingMessage
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$JsonRpcErrorCopyWith<$Res> get error {
  
  return $JsonRpcErrorCopyWith<$Res>(_self.error, (value) {
    return _then(_self.copyWith(error: value));
  });
}
}

/// @nodoc


class _IncomingNotification implements IncomingMessage {
  const _IncomingNotification(this.notification);
  

 final  JsonRpcNotification notification;

/// Create a copy of IncomingMessage
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$IncomingNotificationCopyWith<_IncomingNotification> get copyWith => __$IncomingNotificationCopyWithImpl<_IncomingNotification>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _IncomingNotification&&(identical(other.notification, notification) || other.notification == notification));
}


@override
int get hashCode => Object.hash(runtimeType,notification);

@override
String toString() {
  return 'IncomingMessage.notification(notification: $notification)';
}


}

/// @nodoc
abstract mixin class _$IncomingNotificationCopyWith<$Res> implements $IncomingMessageCopyWith<$Res> {
  factory _$IncomingNotificationCopyWith(_IncomingNotification value, $Res Function(_IncomingNotification) _then) = __$IncomingNotificationCopyWithImpl;
@useResult
$Res call({
 JsonRpcNotification notification
});


$JsonRpcNotificationCopyWith<$Res> get notification;

}
/// @nodoc
class __$IncomingNotificationCopyWithImpl<$Res>
    implements _$IncomingNotificationCopyWith<$Res> {
  __$IncomingNotificationCopyWithImpl(this._self, this._then);

  final _IncomingNotification _self;
  final $Res Function(_IncomingNotification) _then;

/// Create a copy of IncomingMessage
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? notification = null,}) {
  return _then(_IncomingNotification(
null == notification ? _self.notification : notification // ignore: cast_nullable_to_non_nullable
as JsonRpcNotification,
  ));
}

/// Create a copy of IncomingMessage
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$JsonRpcNotificationCopyWith<$Res> get notification {
  
  return $JsonRpcNotificationCopyWith<$Res>(_self.notification, (value) {
    return _then(_self.copyWith(notification: value));
  });
}
}


/// @nodoc
mixin _$DaemonDeviceInfo {

 String get path; String get name;@JsonKey(name: 'vendor_id') String? get vendorId;@JsonKey(name: 'product_id') String? get productId;
/// Create a copy of DaemonDeviceInfo
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$DaemonDeviceInfoCopyWith<DaemonDeviceInfo> get copyWith => _$DaemonDeviceInfoCopyWithImpl<DaemonDeviceInfo>(this as DaemonDeviceInfo, _$identity);

  /// Serializes this DaemonDeviceInfo to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DaemonDeviceInfo&&(identical(other.path, path) || other.path == path)&&(identical(other.name, name) || other.name == name)&&(identical(other.vendorId, vendorId) || other.vendorId == vendorId)&&(identical(other.productId, productId) || other.productId == productId));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,path,name,vendorId,productId);

@override
String toString() {
  return 'DaemonDeviceInfo(path: $path, name: $name, vendorId: $vendorId, productId: $productId)';
}


}

/// @nodoc
abstract mixin class $DaemonDeviceInfoCopyWith<$Res>  {
  factory $DaemonDeviceInfoCopyWith(DaemonDeviceInfo value, $Res Function(DaemonDeviceInfo) _then) = _$DaemonDeviceInfoCopyWithImpl;
@useResult
$Res call({
 String path, String name,@JsonKey(name: 'vendor_id') String? vendorId,@JsonKey(name: 'product_id') String? productId
});




}
/// @nodoc
class _$DaemonDeviceInfoCopyWithImpl<$Res>
    implements $DaemonDeviceInfoCopyWith<$Res> {
  _$DaemonDeviceInfoCopyWithImpl(this._self, this._then);

  final DaemonDeviceInfo _self;
  final $Res Function(DaemonDeviceInfo) _then;

/// Create a copy of DaemonDeviceInfo
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? path = null,Object? name = null,Object? vendorId = freezed,Object? productId = freezed,}) {
  return _then(_self.copyWith(
path: null == path ? _self.path : path // ignore: cast_nullable_to_non_nullable
as String,name: null == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as String,vendorId: freezed == vendorId ? _self.vendorId : vendorId // ignore: cast_nullable_to_non_nullable
as String?,productId: freezed == productId ? _self.productId : productId // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}

}


/// Adds pattern-matching-related methods to [DaemonDeviceInfo].
extension DaemonDeviceInfoPatterns on DaemonDeviceInfo {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _DaemonDeviceInfo value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _DaemonDeviceInfo() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _DaemonDeviceInfo value)  $default,){
final _that = this;
switch (_that) {
case _DaemonDeviceInfo():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _DaemonDeviceInfo value)?  $default,){
final _that = this;
switch (_that) {
case _DaemonDeviceInfo() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String path,  String name, @JsonKey(name: 'vendor_id')  String? vendorId, @JsonKey(name: 'product_id')  String? productId)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _DaemonDeviceInfo() when $default != null:
return $default(_that.path,_that.name,_that.vendorId,_that.productId);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String path,  String name, @JsonKey(name: 'vendor_id')  String? vendorId, @JsonKey(name: 'product_id')  String? productId)  $default,) {final _that = this;
switch (_that) {
case _DaemonDeviceInfo():
return $default(_that.path,_that.name,_that.vendorId,_that.productId);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String path,  String name, @JsonKey(name: 'vendor_id')  String? vendorId, @JsonKey(name: 'product_id')  String? productId)?  $default,) {final _that = this;
switch (_that) {
case _DaemonDeviceInfo() when $default != null:
return $default(_that.path,_that.name,_that.vendorId,_that.productId);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _DaemonDeviceInfo implements DaemonDeviceInfo {
  const _DaemonDeviceInfo({required this.path, required this.name, @JsonKey(name: 'vendor_id') this.vendorId, @JsonKey(name: 'product_id') this.productId});
  factory _DaemonDeviceInfo.fromJson(Map<String, dynamic> json) => _$DaemonDeviceInfoFromJson(json);

@override final  String path;
@override final  String name;
@override@JsonKey(name: 'vendor_id') final  String? vendorId;
@override@JsonKey(name: 'product_id') final  String? productId;

/// Create a copy of DaemonDeviceInfo
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$DaemonDeviceInfoCopyWith<_DaemonDeviceInfo> get copyWith => __$DaemonDeviceInfoCopyWithImpl<_DaemonDeviceInfo>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$DaemonDeviceInfoToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _DaemonDeviceInfo&&(identical(other.path, path) || other.path == path)&&(identical(other.name, name) || other.name == name)&&(identical(other.vendorId, vendorId) || other.vendorId == vendorId)&&(identical(other.productId, productId) || other.productId == productId));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,path,name,vendorId,productId);

@override
String toString() {
  return 'DaemonDeviceInfo(path: $path, name: $name, vendorId: $vendorId, productId: $productId)';
}


}

/// @nodoc
abstract mixin class _$DaemonDeviceInfoCopyWith<$Res> implements $DaemonDeviceInfoCopyWith<$Res> {
  factory _$DaemonDeviceInfoCopyWith(_DaemonDeviceInfo value, $Res Function(_DaemonDeviceInfo) _then) = __$DaemonDeviceInfoCopyWithImpl;
@override @useResult
$Res call({
 String path, String name,@JsonKey(name: 'vendor_id') String? vendorId,@JsonKey(name: 'product_id') String? productId
});




}
/// @nodoc
class __$DaemonDeviceInfoCopyWithImpl<$Res>
    implements _$DaemonDeviceInfoCopyWith<$Res> {
  __$DaemonDeviceInfoCopyWithImpl(this._self, this._then);

  final _DaemonDeviceInfo _self;
  final $Res Function(_DaemonDeviceInfo) _then;

/// Create a copy of DaemonDeviceInfo
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? path = null,Object? name = null,Object? vendorId = freezed,Object? productId = freezed,}) {
  return _then(_DaemonDeviceInfo(
path: null == path ? _self.path : path // ignore: cast_nullable_to_non_nullable
as String,name: null == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as String,vendorId: freezed == vendorId ? _self.vendorId : vendorId // ignore: cast_nullable_to_non_nullable
as String?,productId: freezed == productId ? _self.productId : productId // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}


/// @nodoc
mixin _$EngineStatus {

 String get status;// "running", "stopped", "error"
@JsonKey(name: 'config_path') String? get configPath;@JsonKey(name: 'script_path') String? get scriptPath; List<DaemonDeviceInfo> get devices;@JsonKey(name: 'uptime_seconds') int? get uptimeSeconds;@JsonKey(name: 'events_processed') int? get eventsProcessed;
/// Create a copy of EngineStatus
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$EngineStatusCopyWith<EngineStatus> get copyWith => _$EngineStatusCopyWithImpl<EngineStatus>(this as EngineStatus, _$identity);

  /// Serializes this EngineStatus to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is EngineStatus&&(identical(other.status, status) || other.status == status)&&(identical(other.configPath, configPath) || other.configPath == configPath)&&(identical(other.scriptPath, scriptPath) || other.scriptPath == scriptPath)&&const DeepCollectionEquality().equals(other.devices, devices)&&(identical(other.uptimeSeconds, uptimeSeconds) || other.uptimeSeconds == uptimeSeconds)&&(identical(other.eventsProcessed, eventsProcessed) || other.eventsProcessed == eventsProcessed));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,status,configPath,scriptPath,const DeepCollectionEquality().hash(devices),uptimeSeconds,eventsProcessed);

@override
String toString() {
  return 'EngineStatus(status: $status, configPath: $configPath, scriptPath: $scriptPath, devices: $devices, uptimeSeconds: $uptimeSeconds, eventsProcessed: $eventsProcessed)';
}


}

/// @nodoc
abstract mixin class $EngineStatusCopyWith<$Res>  {
  factory $EngineStatusCopyWith(EngineStatus value, $Res Function(EngineStatus) _then) = _$EngineStatusCopyWithImpl;
@useResult
$Res call({
 String status,@JsonKey(name: 'config_path') String? configPath,@JsonKey(name: 'script_path') String? scriptPath, List<DaemonDeviceInfo> devices,@JsonKey(name: 'uptime_seconds') int? uptimeSeconds,@JsonKey(name: 'events_processed') int? eventsProcessed
});




}
/// @nodoc
class _$EngineStatusCopyWithImpl<$Res>
    implements $EngineStatusCopyWith<$Res> {
  _$EngineStatusCopyWithImpl(this._self, this._then);

  final EngineStatus _self;
  final $Res Function(EngineStatus) _then;

/// Create a copy of EngineStatus
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? status = null,Object? configPath = freezed,Object? scriptPath = freezed,Object? devices = null,Object? uptimeSeconds = freezed,Object? eventsProcessed = freezed,}) {
  return _then(_self.copyWith(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as String,configPath: freezed == configPath ? _self.configPath : configPath // ignore: cast_nullable_to_non_nullable
as String?,scriptPath: freezed == scriptPath ? _self.scriptPath : scriptPath // ignore: cast_nullable_to_non_nullable
as String?,devices: null == devices ? _self.devices : devices // ignore: cast_nullable_to_non_nullable
as List<DaemonDeviceInfo>,uptimeSeconds: freezed == uptimeSeconds ? _self.uptimeSeconds : uptimeSeconds // ignore: cast_nullable_to_non_nullable
as int?,eventsProcessed: freezed == eventsProcessed ? _self.eventsProcessed : eventsProcessed // ignore: cast_nullable_to_non_nullable
as int?,
  ));
}

}


/// Adds pattern-matching-related methods to [EngineStatus].
extension EngineStatusPatterns on EngineStatus {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _EngineStatus value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _EngineStatus() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _EngineStatus value)  $default,){
final _that = this;
switch (_that) {
case _EngineStatus():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _EngineStatus value)?  $default,){
final _that = this;
switch (_that) {
case _EngineStatus() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String status, @JsonKey(name: 'config_path')  String? configPath, @JsonKey(name: 'script_path')  String? scriptPath,  List<DaemonDeviceInfo> devices, @JsonKey(name: 'uptime_seconds')  int? uptimeSeconds, @JsonKey(name: 'events_processed')  int? eventsProcessed)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _EngineStatus() when $default != null:
return $default(_that.status,_that.configPath,_that.scriptPath,_that.devices,_that.uptimeSeconds,_that.eventsProcessed);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String status, @JsonKey(name: 'config_path')  String? configPath, @JsonKey(name: 'script_path')  String? scriptPath,  List<DaemonDeviceInfo> devices, @JsonKey(name: 'uptime_seconds')  int? uptimeSeconds, @JsonKey(name: 'events_processed')  int? eventsProcessed)  $default,) {final _that = this;
switch (_that) {
case _EngineStatus():
return $default(_that.status,_that.configPath,_that.scriptPath,_that.devices,_that.uptimeSeconds,_that.eventsProcessed);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String status, @JsonKey(name: 'config_path')  String? configPath, @JsonKey(name: 'script_path')  String? scriptPath,  List<DaemonDeviceInfo> devices, @JsonKey(name: 'uptime_seconds')  int? uptimeSeconds, @JsonKey(name: 'events_processed')  int? eventsProcessed)?  $default,) {final _that = this;
switch (_that) {
case _EngineStatus() when $default != null:
return $default(_that.status,_that.configPath,_that.scriptPath,_that.devices,_that.uptimeSeconds,_that.eventsProcessed);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _EngineStatus implements EngineStatus {
  const _EngineStatus({required this.status, @JsonKey(name: 'config_path') this.configPath, @JsonKey(name: 'script_path') this.scriptPath, final  List<DaemonDeviceInfo> devices = const [], @JsonKey(name: 'uptime_seconds') this.uptimeSeconds, @JsonKey(name: 'events_processed') this.eventsProcessed}): _devices = devices;
  factory _EngineStatus.fromJson(Map<String, dynamic> json) => _$EngineStatusFromJson(json);

@override final  String status;
// "running", "stopped", "error"
@override@JsonKey(name: 'config_path') final  String? configPath;
@override@JsonKey(name: 'script_path') final  String? scriptPath;
 final  List<DaemonDeviceInfo> _devices;
@override@JsonKey() List<DaemonDeviceInfo> get devices {
  if (_devices is EqualUnmodifiableListView) return _devices;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_devices);
}

@override@JsonKey(name: 'uptime_seconds') final  int? uptimeSeconds;
@override@JsonKey(name: 'events_processed') final  int? eventsProcessed;

/// Create a copy of EngineStatus
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$EngineStatusCopyWith<_EngineStatus> get copyWith => __$EngineStatusCopyWithImpl<_EngineStatus>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$EngineStatusToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _EngineStatus&&(identical(other.status, status) || other.status == status)&&(identical(other.configPath, configPath) || other.configPath == configPath)&&(identical(other.scriptPath, scriptPath) || other.scriptPath == scriptPath)&&const DeepCollectionEquality().equals(other._devices, _devices)&&(identical(other.uptimeSeconds, uptimeSeconds) || other.uptimeSeconds == uptimeSeconds)&&(identical(other.eventsProcessed, eventsProcessed) || other.eventsProcessed == eventsProcessed));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,status,configPath,scriptPath,const DeepCollectionEquality().hash(_devices),uptimeSeconds,eventsProcessed);

@override
String toString() {
  return 'EngineStatus(status: $status, configPath: $configPath, scriptPath: $scriptPath, devices: $devices, uptimeSeconds: $uptimeSeconds, eventsProcessed: $eventsProcessed)';
}


}

/// @nodoc
abstract mixin class _$EngineStatusCopyWith<$Res> implements $EngineStatusCopyWith<$Res> {
  factory _$EngineStatusCopyWith(_EngineStatus value, $Res Function(_EngineStatus) _then) = __$EngineStatusCopyWithImpl;
@override @useResult
$Res call({
 String status,@JsonKey(name: 'config_path') String? configPath,@JsonKey(name: 'script_path') String? scriptPath, List<DaemonDeviceInfo> devices,@JsonKey(name: 'uptime_seconds') int? uptimeSeconds,@JsonKey(name: 'events_processed') int? eventsProcessed
});




}
/// @nodoc
class __$EngineStatusCopyWithImpl<$Res>
    implements _$EngineStatusCopyWith<$Res> {
  __$EngineStatusCopyWithImpl(this._self, this._then);

  final _EngineStatus _self;
  final $Res Function(_EngineStatus) _then;

/// Create a copy of EngineStatus
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? status = null,Object? configPath = freezed,Object? scriptPath = freezed,Object? devices = null,Object? uptimeSeconds = freezed,Object? eventsProcessed = freezed,}) {
  return _then(_EngineStatus(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as String,configPath: freezed == configPath ? _self.configPath : configPath // ignore: cast_nullable_to_non_nullable
as String?,scriptPath: freezed == scriptPath ? _self.scriptPath : scriptPath // ignore: cast_nullable_to_non_nullable
as String?,devices: null == devices ? _self._devices : devices // ignore: cast_nullable_to_non_nullable
as List<DaemonDeviceInfo>,uptimeSeconds: freezed == uptimeSeconds ? _self.uptimeSeconds : uptimeSeconds // ignore: cast_nullable_to_non_nullable
as int?,eventsProcessed: freezed == eventsProcessed ? _self.eventsProcessed : eventsProcessed // ignore: cast_nullable_to_non_nullable
as int?,
  ));
}


}


/// @nodoc
mixin _$InputEvent {

@JsonKey(name: 'timestamp_us') int get timestampUs;@JsonKey(name: 'device_path') String get devicePath;@JsonKey(name: 'device_name') String get deviceName;@JsonKey(name: 'event_type') String get eventType;// "key_press", "key_release"
@JsonKey(name: 'key_code') int get keyCode;@JsonKey(name: 'key_name') String get keyName; List<String> get modifiers;
/// Create a copy of InputEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$InputEventCopyWith<InputEvent> get copyWith => _$InputEventCopyWithImpl<InputEvent>(this as InputEvent, _$identity);

  /// Serializes this InputEvent to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is InputEvent&&(identical(other.timestampUs, timestampUs) || other.timestampUs == timestampUs)&&(identical(other.devicePath, devicePath) || other.devicePath == devicePath)&&(identical(other.deviceName, deviceName) || other.deviceName == deviceName)&&(identical(other.eventType, eventType) || other.eventType == eventType)&&(identical(other.keyCode, keyCode) || other.keyCode == keyCode)&&(identical(other.keyName, keyName) || other.keyName == keyName)&&const DeepCollectionEquality().equals(other.modifiers, modifiers));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,timestampUs,devicePath,deviceName,eventType,keyCode,keyName,const DeepCollectionEquality().hash(modifiers));

@override
String toString() {
  return 'InputEvent(timestampUs: $timestampUs, devicePath: $devicePath, deviceName: $deviceName, eventType: $eventType, keyCode: $keyCode, keyName: $keyName, modifiers: $modifiers)';
}


}

/// @nodoc
abstract mixin class $InputEventCopyWith<$Res>  {
  factory $InputEventCopyWith(InputEvent value, $Res Function(InputEvent) _then) = _$InputEventCopyWithImpl;
@useResult
$Res call({
@JsonKey(name: 'timestamp_us') int timestampUs,@JsonKey(name: 'device_path') String devicePath,@JsonKey(name: 'device_name') String deviceName,@JsonKey(name: 'event_type') String eventType,@JsonKey(name: 'key_code') int keyCode,@JsonKey(name: 'key_name') String keyName, List<String> modifiers
});




}
/// @nodoc
class _$InputEventCopyWithImpl<$Res>
    implements $InputEventCopyWith<$Res> {
  _$InputEventCopyWithImpl(this._self, this._then);

  final InputEvent _self;
  final $Res Function(InputEvent) _then;

/// Create a copy of InputEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? timestampUs = null,Object? devicePath = null,Object? deviceName = null,Object? eventType = null,Object? keyCode = null,Object? keyName = null,Object? modifiers = null,}) {
  return _then(_self.copyWith(
timestampUs: null == timestampUs ? _self.timestampUs : timestampUs // ignore: cast_nullable_to_non_nullable
as int,devicePath: null == devicePath ? _self.devicePath : devicePath // ignore: cast_nullable_to_non_nullable
as String,deviceName: null == deviceName ? _self.deviceName : deviceName // ignore: cast_nullable_to_non_nullable
as String,eventType: null == eventType ? _self.eventType : eventType // ignore: cast_nullable_to_non_nullable
as String,keyCode: null == keyCode ? _self.keyCode : keyCode // ignore: cast_nullable_to_non_nullable
as int,keyName: null == keyName ? _self.keyName : keyName // ignore: cast_nullable_to_non_nullable
as String,modifiers: null == modifiers ? _self.modifiers : modifiers // ignore: cast_nullable_to_non_nullable
as List<String>,
  ));
}

}


/// Adds pattern-matching-related methods to [InputEvent].
extension InputEventPatterns on InputEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _InputEvent value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _InputEvent() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _InputEvent value)  $default,){
final _that = this;
switch (_that) {
case _InputEvent():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _InputEvent value)?  $default,){
final _that = this;
switch (_that) {
case _InputEvent() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function(@JsonKey(name: 'timestamp_us')  int timestampUs, @JsonKey(name: 'device_path')  String devicePath, @JsonKey(name: 'device_name')  String deviceName, @JsonKey(name: 'event_type')  String eventType, @JsonKey(name: 'key_code')  int keyCode, @JsonKey(name: 'key_name')  String keyName,  List<String> modifiers)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _InputEvent() when $default != null:
return $default(_that.timestampUs,_that.devicePath,_that.deviceName,_that.eventType,_that.keyCode,_that.keyName,_that.modifiers);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function(@JsonKey(name: 'timestamp_us')  int timestampUs, @JsonKey(name: 'device_path')  String devicePath, @JsonKey(name: 'device_name')  String deviceName, @JsonKey(name: 'event_type')  String eventType, @JsonKey(name: 'key_code')  int keyCode, @JsonKey(name: 'key_name')  String keyName,  List<String> modifiers)  $default,) {final _that = this;
switch (_that) {
case _InputEvent():
return $default(_that.timestampUs,_that.devicePath,_that.deviceName,_that.eventType,_that.keyCode,_that.keyName,_that.modifiers);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function(@JsonKey(name: 'timestamp_us')  int timestampUs, @JsonKey(name: 'device_path')  String devicePath, @JsonKey(name: 'device_name')  String deviceName, @JsonKey(name: 'event_type')  String eventType, @JsonKey(name: 'key_code')  int keyCode, @JsonKey(name: 'key_name')  String keyName,  List<String> modifiers)?  $default,) {final _that = this;
switch (_that) {
case _InputEvent() when $default != null:
return $default(_that.timestampUs,_that.devicePath,_that.deviceName,_that.eventType,_that.keyCode,_that.keyName,_that.modifiers);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _InputEvent implements InputEvent {
  const _InputEvent({@JsonKey(name: 'timestamp_us') required this.timestampUs, @JsonKey(name: 'device_path') required this.devicePath, @JsonKey(name: 'device_name') required this.deviceName, @JsonKey(name: 'event_type') required this.eventType, @JsonKey(name: 'key_code') required this.keyCode, @JsonKey(name: 'key_name') required this.keyName, final  List<String> modifiers = const []}): _modifiers = modifiers;
  factory _InputEvent.fromJson(Map<String, dynamic> json) => _$InputEventFromJson(json);

@override@JsonKey(name: 'timestamp_us') final  int timestampUs;
@override@JsonKey(name: 'device_path') final  String devicePath;
@override@JsonKey(name: 'device_name') final  String deviceName;
@override@JsonKey(name: 'event_type') final  String eventType;
// "key_press", "key_release"
@override@JsonKey(name: 'key_code') final  int keyCode;
@override@JsonKey(name: 'key_name') final  String keyName;
 final  List<String> _modifiers;
@override@JsonKey() List<String> get modifiers {
  if (_modifiers is EqualUnmodifiableListView) return _modifiers;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_modifiers);
}


/// Create a copy of InputEvent
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$InputEventCopyWith<_InputEvent> get copyWith => __$InputEventCopyWithImpl<_InputEvent>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$InputEventToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _InputEvent&&(identical(other.timestampUs, timestampUs) || other.timestampUs == timestampUs)&&(identical(other.devicePath, devicePath) || other.devicePath == devicePath)&&(identical(other.deviceName, deviceName) || other.deviceName == deviceName)&&(identical(other.eventType, eventType) || other.eventType == eventType)&&(identical(other.keyCode, keyCode) || other.keyCode == keyCode)&&(identical(other.keyName, keyName) || other.keyName == keyName)&&const DeepCollectionEquality().equals(other._modifiers, _modifiers));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,timestampUs,devicePath,deviceName,eventType,keyCode,keyName,const DeepCollectionEquality().hash(_modifiers));

@override
String toString() {
  return 'InputEvent(timestampUs: $timestampUs, devicePath: $devicePath, deviceName: $deviceName, eventType: $eventType, keyCode: $keyCode, keyName: $keyName, modifiers: $modifiers)';
}


}

/// @nodoc
abstract mixin class _$InputEventCopyWith<$Res> implements $InputEventCopyWith<$Res> {
  factory _$InputEventCopyWith(_InputEvent value, $Res Function(_InputEvent) _then) = __$InputEventCopyWithImpl;
@override @useResult
$Res call({
@JsonKey(name: 'timestamp_us') int timestampUs,@JsonKey(name: 'device_path') String devicePath,@JsonKey(name: 'device_name') String deviceName,@JsonKey(name: 'event_type') String eventType,@JsonKey(name: 'key_code') int keyCode,@JsonKey(name: 'key_name') String keyName, List<String> modifiers
});




}
/// @nodoc
class __$InputEventCopyWithImpl<$Res>
    implements _$InputEventCopyWith<$Res> {
  __$InputEventCopyWithImpl(this._self, this._then);

  final _InputEvent _self;
  final $Res Function(_InputEvent) _then;

/// Create a copy of InputEvent
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? timestampUs = null,Object? devicePath = null,Object? deviceName = null,Object? eventType = null,Object? keyCode = null,Object? keyName = null,Object? modifiers = null,}) {
  return _then(_InputEvent(
timestampUs: null == timestampUs ? _self.timestampUs : timestampUs // ignore: cast_nullable_to_non_nullable
as int,devicePath: null == devicePath ? _self.devicePath : devicePath // ignore: cast_nullable_to_non_nullable
as String,deviceName: null == deviceName ? _self.deviceName : deviceName // ignore: cast_nullable_to_non_nullable
as String,eventType: null == eventType ? _self.eventType : eventType // ignore: cast_nullable_to_non_nullable
as String,keyCode: null == keyCode ? _self.keyCode : keyCode // ignore: cast_nullable_to_non_nullable
as int,keyName: null == keyName ? _self.keyName : keyName // ignore: cast_nullable_to_non_nullable
as String,modifiers: null == modifiers ? _self._modifiers : modifiers // ignore: cast_nullable_to_non_nullable
as List<String>,
  ));
}


}


/// @nodoc
mixin _$OutputEvent {

@JsonKey(name: 'timestamp_us') int get timestampUs;@JsonKey(name: 'action_type') String get actionType;// "key_press", "key_release", "tap"
@JsonKey(name: 'key_code') int get keyCode;@JsonKey(name: 'key_name') String get keyName;
/// Create a copy of OutputEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$OutputEventCopyWith<OutputEvent> get copyWith => _$OutputEventCopyWithImpl<OutputEvent>(this as OutputEvent, _$identity);

  /// Serializes this OutputEvent to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is OutputEvent&&(identical(other.timestampUs, timestampUs) || other.timestampUs == timestampUs)&&(identical(other.actionType, actionType) || other.actionType == actionType)&&(identical(other.keyCode, keyCode) || other.keyCode == keyCode)&&(identical(other.keyName, keyName) || other.keyName == keyName));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,timestampUs,actionType,keyCode,keyName);

@override
String toString() {
  return 'OutputEvent(timestampUs: $timestampUs, actionType: $actionType, keyCode: $keyCode, keyName: $keyName)';
}


}

/// @nodoc
abstract mixin class $OutputEventCopyWith<$Res>  {
  factory $OutputEventCopyWith(OutputEvent value, $Res Function(OutputEvent) _then) = _$OutputEventCopyWithImpl;
@useResult
$Res call({
@JsonKey(name: 'timestamp_us') int timestampUs,@JsonKey(name: 'action_type') String actionType,@JsonKey(name: 'key_code') int keyCode,@JsonKey(name: 'key_name') String keyName
});




}
/// @nodoc
class _$OutputEventCopyWithImpl<$Res>
    implements $OutputEventCopyWith<$Res> {
  _$OutputEventCopyWithImpl(this._self, this._then);

  final OutputEvent _self;
  final $Res Function(OutputEvent) _then;

/// Create a copy of OutputEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? timestampUs = null,Object? actionType = null,Object? keyCode = null,Object? keyName = null,}) {
  return _then(_self.copyWith(
timestampUs: null == timestampUs ? _self.timestampUs : timestampUs // ignore: cast_nullable_to_non_nullable
as int,actionType: null == actionType ? _self.actionType : actionType // ignore: cast_nullable_to_non_nullable
as String,keyCode: null == keyCode ? _self.keyCode : keyCode // ignore: cast_nullable_to_non_nullable
as int,keyName: null == keyName ? _self.keyName : keyName // ignore: cast_nullable_to_non_nullable
as String,
  ));
}

}


/// Adds pattern-matching-related methods to [OutputEvent].
extension OutputEventPatterns on OutputEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _OutputEvent value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _OutputEvent() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _OutputEvent value)  $default,){
final _that = this;
switch (_that) {
case _OutputEvent():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _OutputEvent value)?  $default,){
final _that = this;
switch (_that) {
case _OutputEvent() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function(@JsonKey(name: 'timestamp_us')  int timestampUs, @JsonKey(name: 'action_type')  String actionType, @JsonKey(name: 'key_code')  int keyCode, @JsonKey(name: 'key_name')  String keyName)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _OutputEvent() when $default != null:
return $default(_that.timestampUs,_that.actionType,_that.keyCode,_that.keyName);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function(@JsonKey(name: 'timestamp_us')  int timestampUs, @JsonKey(name: 'action_type')  String actionType, @JsonKey(name: 'key_code')  int keyCode, @JsonKey(name: 'key_name')  String keyName)  $default,) {final _that = this;
switch (_that) {
case _OutputEvent():
return $default(_that.timestampUs,_that.actionType,_that.keyCode,_that.keyName);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function(@JsonKey(name: 'timestamp_us')  int timestampUs, @JsonKey(name: 'action_type')  String actionType, @JsonKey(name: 'key_code')  int keyCode, @JsonKey(name: 'key_name')  String keyName)?  $default,) {final _that = this;
switch (_that) {
case _OutputEvent() when $default != null:
return $default(_that.timestampUs,_that.actionType,_that.keyCode,_that.keyName);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _OutputEvent implements OutputEvent {
  const _OutputEvent({@JsonKey(name: 'timestamp_us') required this.timestampUs, @JsonKey(name: 'action_type') required this.actionType, @JsonKey(name: 'key_code') required this.keyCode, @JsonKey(name: 'key_name') required this.keyName});
  factory _OutputEvent.fromJson(Map<String, dynamic> json) => _$OutputEventFromJson(json);

@override@JsonKey(name: 'timestamp_us') final  int timestampUs;
@override@JsonKey(name: 'action_type') final  String actionType;
// "key_press", "key_release", "tap"
@override@JsonKey(name: 'key_code') final  int keyCode;
@override@JsonKey(name: 'key_name') final  String keyName;

/// Create a copy of OutputEvent
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$OutputEventCopyWith<_OutputEvent> get copyWith => __$OutputEventCopyWithImpl<_OutputEvent>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$OutputEventToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _OutputEvent&&(identical(other.timestampUs, timestampUs) || other.timestampUs == timestampUs)&&(identical(other.actionType, actionType) || other.actionType == actionType)&&(identical(other.keyCode, keyCode) || other.keyCode == keyCode)&&(identical(other.keyName, keyName) || other.keyName == keyName));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,timestampUs,actionType,keyCode,keyName);

@override
String toString() {
  return 'OutputEvent(timestampUs: $timestampUs, actionType: $actionType, keyCode: $keyCode, keyName: $keyName)';
}


}

/// @nodoc
abstract mixin class _$OutputEventCopyWith<$Res> implements $OutputEventCopyWith<$Res> {
  factory _$OutputEventCopyWith(_OutputEvent value, $Res Function(_OutputEvent) _then) = __$OutputEventCopyWithImpl;
@override @useResult
$Res call({
@JsonKey(name: 'timestamp_us') int timestampUs,@JsonKey(name: 'action_type') String actionType,@JsonKey(name: 'key_code') int keyCode,@JsonKey(name: 'key_name') String keyName
});




}
/// @nodoc
class __$OutputEventCopyWithImpl<$Res>
    implements _$OutputEventCopyWith<$Res> {
  __$OutputEventCopyWithImpl(this._self, this._then);

  final _OutputEvent _self;
  final $Res Function(_OutputEvent) _then;

/// Create a copy of OutputEvent
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? timestampUs = null,Object? actionType = null,Object? keyCode = null,Object? keyName = null,}) {
  return _then(_OutputEvent(
timestampUs: null == timestampUs ? _self.timestampUs : timestampUs // ignore: cast_nullable_to_non_nullable
as int,actionType: null == actionType ? _self.actionType : actionType // ignore: cast_nullable_to_non_nullable
as String,keyCode: null == keyCode ? _self.keyCode : keyCode // ignore: cast_nullable_to_non_nullable
as int,keyName: null == keyName ? _self.keyName : keyName // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}


/// @nodoc
mixin _$DecisionEvent {

@JsonKey(name: 'timestamp_us') int get timestampUs;@JsonKey(name: 'key_code') int get keyCode;@JsonKey(name: 'key_name') String get keyName;@JsonKey(name: 'decision_type') String get decisionType; List<String> get outputs;
/// Create a copy of DecisionEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$DecisionEventCopyWith<DecisionEvent> get copyWith => _$DecisionEventCopyWithImpl<DecisionEvent>(this as DecisionEvent, _$identity);

  /// Serializes this DecisionEvent to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DecisionEvent&&(identical(other.timestampUs, timestampUs) || other.timestampUs == timestampUs)&&(identical(other.keyCode, keyCode) || other.keyCode == keyCode)&&(identical(other.keyName, keyName) || other.keyName == keyName)&&(identical(other.decisionType, decisionType) || other.decisionType == decisionType)&&const DeepCollectionEquality().equals(other.outputs, outputs));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,timestampUs,keyCode,keyName,decisionType,const DeepCollectionEquality().hash(outputs));

@override
String toString() {
  return 'DecisionEvent(timestampUs: $timestampUs, keyCode: $keyCode, keyName: $keyName, decisionType: $decisionType, outputs: $outputs)';
}


}

/// @nodoc
abstract mixin class $DecisionEventCopyWith<$Res>  {
  factory $DecisionEventCopyWith(DecisionEvent value, $Res Function(DecisionEvent) _then) = _$DecisionEventCopyWithImpl;
@useResult
$Res call({
@JsonKey(name: 'timestamp_us') int timestampUs,@JsonKey(name: 'key_code') int keyCode,@JsonKey(name: 'key_name') String keyName,@JsonKey(name: 'decision_type') String decisionType, List<String> outputs
});




}
/// @nodoc
class _$DecisionEventCopyWithImpl<$Res>
    implements $DecisionEventCopyWith<$Res> {
  _$DecisionEventCopyWithImpl(this._self, this._then);

  final DecisionEvent _self;
  final $Res Function(DecisionEvent) _then;

/// Create a copy of DecisionEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? timestampUs = null,Object? keyCode = null,Object? keyName = null,Object? decisionType = null,Object? outputs = null,}) {
  return _then(_self.copyWith(
timestampUs: null == timestampUs ? _self.timestampUs : timestampUs // ignore: cast_nullable_to_non_nullable
as int,keyCode: null == keyCode ? _self.keyCode : keyCode // ignore: cast_nullable_to_non_nullable
as int,keyName: null == keyName ? _self.keyName : keyName // ignore: cast_nullable_to_non_nullable
as String,decisionType: null == decisionType ? _self.decisionType : decisionType // ignore: cast_nullable_to_non_nullable
as String,outputs: null == outputs ? _self.outputs : outputs // ignore: cast_nullable_to_non_nullable
as List<String>,
  ));
}

}


/// Adds pattern-matching-related methods to [DecisionEvent].
extension DecisionEventPatterns on DecisionEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _DecisionEvent value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _DecisionEvent() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _DecisionEvent value)  $default,){
final _that = this;
switch (_that) {
case _DecisionEvent():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _DecisionEvent value)?  $default,){
final _that = this;
switch (_that) {
case _DecisionEvent() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function(@JsonKey(name: 'timestamp_us')  int timestampUs, @JsonKey(name: 'key_code')  int keyCode, @JsonKey(name: 'key_name')  String keyName, @JsonKey(name: 'decision_type')  String decisionType,  List<String> outputs)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _DecisionEvent() when $default != null:
return $default(_that.timestampUs,_that.keyCode,_that.keyName,_that.decisionType,_that.outputs);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function(@JsonKey(name: 'timestamp_us')  int timestampUs, @JsonKey(name: 'key_code')  int keyCode, @JsonKey(name: 'key_name')  String keyName, @JsonKey(name: 'decision_type')  String decisionType,  List<String> outputs)  $default,) {final _that = this;
switch (_that) {
case _DecisionEvent():
return $default(_that.timestampUs,_that.keyCode,_that.keyName,_that.decisionType,_that.outputs);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function(@JsonKey(name: 'timestamp_us')  int timestampUs, @JsonKey(name: 'key_code')  int keyCode, @JsonKey(name: 'key_name')  String keyName, @JsonKey(name: 'decision_type')  String decisionType,  List<String> outputs)?  $default,) {final _that = this;
switch (_that) {
case _DecisionEvent() when $default != null:
return $default(_that.timestampUs,_that.keyCode,_that.keyName,_that.decisionType,_that.outputs);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _DecisionEvent implements DecisionEvent {
  const _DecisionEvent({@JsonKey(name: 'timestamp_us') required this.timestampUs, @JsonKey(name: 'key_code') required this.keyCode, @JsonKey(name: 'key_name') required this.keyName, @JsonKey(name: 'decision_type') required this.decisionType, final  List<String> outputs = const []}): _outputs = outputs;
  factory _DecisionEvent.fromJson(Map<String, dynamic> json) => _$DecisionEventFromJson(json);

@override@JsonKey(name: 'timestamp_us') final  int timestampUs;
@override@JsonKey(name: 'key_code') final  int keyCode;
@override@JsonKey(name: 'key_name') final  String keyName;
@override@JsonKey(name: 'decision_type') final  String decisionType;
 final  List<String> _outputs;
@override@JsonKey() List<String> get outputs {
  if (_outputs is EqualUnmodifiableListView) return _outputs;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_outputs);
}


/// Create a copy of DecisionEvent
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$DecisionEventCopyWith<_DecisionEvent> get copyWith => __$DecisionEventCopyWithImpl<_DecisionEvent>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$DecisionEventToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _DecisionEvent&&(identical(other.timestampUs, timestampUs) || other.timestampUs == timestampUs)&&(identical(other.keyCode, keyCode) || other.keyCode == keyCode)&&(identical(other.keyName, keyName) || other.keyName == keyName)&&(identical(other.decisionType, decisionType) || other.decisionType == decisionType)&&const DeepCollectionEquality().equals(other._outputs, _outputs));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,timestampUs,keyCode,keyName,decisionType,const DeepCollectionEquality().hash(_outputs));

@override
String toString() {
  return 'DecisionEvent(timestampUs: $timestampUs, keyCode: $keyCode, keyName: $keyName, decisionType: $decisionType, outputs: $outputs)';
}


}

/// @nodoc
abstract mixin class _$DecisionEventCopyWith<$Res> implements $DecisionEventCopyWith<$Res> {
  factory _$DecisionEventCopyWith(_DecisionEvent value, $Res Function(_DecisionEvent) _then) = __$DecisionEventCopyWithImpl;
@override @useResult
$Res call({
@JsonKey(name: 'timestamp_us') int timestampUs,@JsonKey(name: 'key_code') int keyCode,@JsonKey(name: 'key_name') String keyName,@JsonKey(name: 'decision_type') String decisionType, List<String> outputs
});




}
/// @nodoc
class __$DecisionEventCopyWithImpl<$Res>
    implements _$DecisionEventCopyWith<$Res> {
  __$DecisionEventCopyWithImpl(this._self, this._then);

  final _DecisionEvent _self;
  final $Res Function(_DecisionEvent) _then;

/// Create a copy of DecisionEvent
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? timestampUs = null,Object? keyCode = null,Object? keyName = null,Object? decisionType = null,Object? outputs = null,}) {
  return _then(_DecisionEvent(
timestampUs: null == timestampUs ? _self.timestampUs : timestampUs // ignore: cast_nullable_to_non_nullable
as int,keyCode: null == keyCode ? _self.keyCode : keyCode // ignore: cast_nullable_to_non_nullable
as int,keyName: null == keyName ? _self.keyName : keyName // ignore: cast_nullable_to_non_nullable
as String,decisionType: null == decisionType ? _self.decisionType : decisionType // ignore: cast_nullable_to_non_nullable
as String,outputs: null == outputs ? _self._outputs : outputs // ignore: cast_nullable_to_non_nullable
as List<String>,
  ));
}


}


/// @nodoc
mixin _$StateChangeEvent {

 String get type;// "config_reloaded", "script_reloaded", etc.
@JsonKey(name: 'timestamp_us') int get timestampUs; String get message; Map<String, dynamic>? get details;
/// Create a copy of StateChangeEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$StateChangeEventCopyWith<StateChangeEvent> get copyWith => _$StateChangeEventCopyWithImpl<StateChangeEvent>(this as StateChangeEvent, _$identity);

  /// Serializes this StateChangeEvent to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is StateChangeEvent&&(identical(other.type, type) || other.type == type)&&(identical(other.timestampUs, timestampUs) || other.timestampUs == timestampUs)&&(identical(other.message, message) || other.message == message)&&const DeepCollectionEquality().equals(other.details, details));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,type,timestampUs,message,const DeepCollectionEquality().hash(details));

@override
String toString() {
  return 'StateChangeEvent(type: $type, timestampUs: $timestampUs, message: $message, details: $details)';
}


}

/// @nodoc
abstract mixin class $StateChangeEventCopyWith<$Res>  {
  factory $StateChangeEventCopyWith(StateChangeEvent value, $Res Function(StateChangeEvent) _then) = _$StateChangeEventCopyWithImpl;
@useResult
$Res call({
 String type,@JsonKey(name: 'timestamp_us') int timestampUs, String message, Map<String, dynamic>? details
});




}
/// @nodoc
class _$StateChangeEventCopyWithImpl<$Res>
    implements $StateChangeEventCopyWith<$Res> {
  _$StateChangeEventCopyWithImpl(this._self, this._then);

  final StateChangeEvent _self;
  final $Res Function(StateChangeEvent) _then;

/// Create a copy of StateChangeEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? type = null,Object? timestampUs = null,Object? message = null,Object? details = freezed,}) {
  return _then(_self.copyWith(
type: null == type ? _self.type : type // ignore: cast_nullable_to_non_nullable
as String,timestampUs: null == timestampUs ? _self.timestampUs : timestampUs // ignore: cast_nullable_to_non_nullable
as int,message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,details: freezed == details ? _self.details : details // ignore: cast_nullable_to_non_nullable
as Map<String, dynamic>?,
  ));
}

}


/// Adds pattern-matching-related methods to [StateChangeEvent].
extension StateChangeEventPatterns on StateChangeEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _StateChangeEvent value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _StateChangeEvent() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _StateChangeEvent value)  $default,){
final _that = this;
switch (_that) {
case _StateChangeEvent():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _StateChangeEvent value)?  $default,){
final _that = this;
switch (_that) {
case _StateChangeEvent() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String type, @JsonKey(name: 'timestamp_us')  int timestampUs,  String message,  Map<String, dynamic>? details)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _StateChangeEvent() when $default != null:
return $default(_that.type,_that.timestampUs,_that.message,_that.details);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String type, @JsonKey(name: 'timestamp_us')  int timestampUs,  String message,  Map<String, dynamic>? details)  $default,) {final _that = this;
switch (_that) {
case _StateChangeEvent():
return $default(_that.type,_that.timestampUs,_that.message,_that.details);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String type, @JsonKey(name: 'timestamp_us')  int timestampUs,  String message,  Map<String, dynamic>? details)?  $default,) {final _that = this;
switch (_that) {
case _StateChangeEvent() when $default != null:
return $default(_that.type,_that.timestampUs,_that.message,_that.details);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _StateChangeEvent implements StateChangeEvent {
  const _StateChangeEvent({required this.type, @JsonKey(name: 'timestamp_us') required this.timestampUs, required this.message, final  Map<String, dynamic>? details}): _details = details;
  factory _StateChangeEvent.fromJson(Map<String, dynamic> json) => _$StateChangeEventFromJson(json);

@override final  String type;
// "config_reloaded", "script_reloaded", etc.
@override@JsonKey(name: 'timestamp_us') final  int timestampUs;
@override final  String message;
 final  Map<String, dynamic>? _details;
@override Map<String, dynamic>? get details {
  final value = _details;
  if (value == null) return null;
  if (_details is EqualUnmodifiableMapView) return _details;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableMapView(value);
}


/// Create a copy of StateChangeEvent
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$StateChangeEventCopyWith<_StateChangeEvent> get copyWith => __$StateChangeEventCopyWithImpl<_StateChangeEvent>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$StateChangeEventToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _StateChangeEvent&&(identical(other.type, type) || other.type == type)&&(identical(other.timestampUs, timestampUs) || other.timestampUs == timestampUs)&&(identical(other.message, message) || other.message == message)&&const DeepCollectionEquality().equals(other._details, _details));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,type,timestampUs,message,const DeepCollectionEquality().hash(_details));

@override
String toString() {
  return 'StateChangeEvent(type: $type, timestampUs: $timestampUs, message: $message, details: $details)';
}


}

/// @nodoc
abstract mixin class _$StateChangeEventCopyWith<$Res> implements $StateChangeEventCopyWith<$Res> {
  factory _$StateChangeEventCopyWith(_StateChangeEvent value, $Res Function(_StateChangeEvent) _then) = __$StateChangeEventCopyWithImpl;
@override @useResult
$Res call({
 String type,@JsonKey(name: 'timestamp_us') int timestampUs, String message, Map<String, dynamic>? details
});




}
/// @nodoc
class __$StateChangeEventCopyWithImpl<$Res>
    implements _$StateChangeEventCopyWith<$Res> {
  __$StateChangeEventCopyWithImpl(this._self, this._then);

  final _StateChangeEvent _self;
  final $Res Function(_StateChangeEvent) _then;

/// Create a copy of StateChangeEvent
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? type = null,Object? timestampUs = null,Object? message = null,Object? details = freezed,}) {
  return _then(_StateChangeEvent(
type: null == type ? _self.type : type // ignore: cast_nullable_to_non_nullable
as String,timestampUs: null == timestampUs ? _self.timestampUs : timestampUs // ignore: cast_nullable_to_non_nullable
as int,message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,details: freezed == details ? _self._details : details // ignore: cast_nullable_to_non_nullable
as Map<String, dynamic>?,
  ));
}


}

/// @nodoc
mixin _$DaemonEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DaemonEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'DaemonEvent()';
}


}

/// @nodoc
class $DaemonEventCopyWith<$Res>  {
$DaemonEventCopyWith(DaemonEvent _, $Res Function(DaemonEvent) __);
}


/// Adds pattern-matching-related methods to [DaemonEvent].
extension DaemonEventPatterns on DaemonEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( _DaemonInputEvent value)?  input,TResult Function( _DaemonOutputEvent value)?  output,TResult Function( _DaemonDecisionEvent value)?  decision,TResult Function( _DaemonStateChangeEvent value)?  stateChange,TResult Function( _DaemonDiscoveryEvent value)?  discovery,required TResult orElse(),}){
final _that = this;
switch (_that) {
case _DaemonInputEvent() when input != null:
return input(_that);case _DaemonOutputEvent() when output != null:
return output(_that);case _DaemonDecisionEvent() when decision != null:
return decision(_that);case _DaemonStateChangeEvent() when stateChange != null:
return stateChange(_that);case _DaemonDiscoveryEvent() when discovery != null:
return discovery(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( _DaemonInputEvent value)  input,required TResult Function( _DaemonOutputEvent value)  output,required TResult Function( _DaemonDecisionEvent value)  decision,required TResult Function( _DaemonStateChangeEvent value)  stateChange,required TResult Function( _DaemonDiscoveryEvent value)  discovery,}){
final _that = this;
switch (_that) {
case _DaemonInputEvent():
return input(_that);case _DaemonOutputEvent():
return output(_that);case _DaemonDecisionEvent():
return decision(_that);case _DaemonStateChangeEvent():
return stateChange(_that);case _DaemonDiscoveryEvent():
return discovery(_that);case _:
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( _DaemonInputEvent value)?  input,TResult? Function( _DaemonOutputEvent value)?  output,TResult? Function( _DaemonDecisionEvent value)?  decision,TResult? Function( _DaemonStateChangeEvent value)?  stateChange,TResult? Function( _DaemonDiscoveryEvent value)?  discovery,}){
final _that = this;
switch (_that) {
case _DaemonInputEvent() when input != null:
return input(_that);case _DaemonOutputEvent() when output != null:
return output(_that);case _DaemonDecisionEvent() when decision != null:
return decision(_that);case _DaemonStateChangeEvent() when stateChange != null:
return stateChange(_that);case _DaemonDiscoveryEvent() when discovery != null:
return discovery(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( InputEvent event)?  input,TResult Function( OutputEvent event)?  output,TResult Function( DecisionEvent event)?  decision,TResult Function( StateChangeEvent event)?  stateChange,TResult Function( DiscoveryCandidate candidate)?  discovery,required TResult orElse(),}) {final _that = this;
switch (_that) {
case _DaemonInputEvent() when input != null:
return input(_that.event);case _DaemonOutputEvent() when output != null:
return output(_that.event);case _DaemonDecisionEvent() when decision != null:
return decision(_that.event);case _DaemonStateChangeEvent() when stateChange != null:
return stateChange(_that.event);case _DaemonDiscoveryEvent() when discovery != null:
return discovery(_that.candidate);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( InputEvent event)  input,required TResult Function( OutputEvent event)  output,required TResult Function( DecisionEvent event)  decision,required TResult Function( StateChangeEvent event)  stateChange,required TResult Function( DiscoveryCandidate candidate)  discovery,}) {final _that = this;
switch (_that) {
case _DaemonInputEvent():
return input(_that.event);case _DaemonOutputEvent():
return output(_that.event);case _DaemonDecisionEvent():
return decision(_that.event);case _DaemonStateChangeEvent():
return stateChange(_that.event);case _DaemonDiscoveryEvent():
return discovery(_that.candidate);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( InputEvent event)?  input,TResult? Function( OutputEvent event)?  output,TResult? Function( DecisionEvent event)?  decision,TResult? Function( StateChangeEvent event)?  stateChange,TResult? Function( DiscoveryCandidate candidate)?  discovery,}) {final _that = this;
switch (_that) {
case _DaemonInputEvent() when input != null:
return input(_that.event);case _DaemonOutputEvent() when output != null:
return output(_that.event);case _DaemonDecisionEvent() when decision != null:
return decision(_that.event);case _DaemonStateChangeEvent() when stateChange != null:
return stateChange(_that.event);case _DaemonDiscoveryEvent() when discovery != null:
return discovery(_that.candidate);case _:
  return null;

}
}

}

/// @nodoc


class _DaemonInputEvent implements DaemonEvent {
  const _DaemonInputEvent(this.event);
  

 final  InputEvent event;

/// Create a copy of DaemonEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$DaemonInputEventCopyWith<_DaemonInputEvent> get copyWith => __$DaemonInputEventCopyWithImpl<_DaemonInputEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _DaemonInputEvent&&(identical(other.event, event) || other.event == event));
}


@override
int get hashCode => Object.hash(runtimeType,event);

@override
String toString() {
  return 'DaemonEvent.input(event: $event)';
}


}

/// @nodoc
abstract mixin class _$DaemonInputEventCopyWith<$Res> implements $DaemonEventCopyWith<$Res> {
  factory _$DaemonInputEventCopyWith(_DaemonInputEvent value, $Res Function(_DaemonInputEvent) _then) = __$DaemonInputEventCopyWithImpl;
@useResult
$Res call({
 InputEvent event
});


$InputEventCopyWith<$Res> get event;

}
/// @nodoc
class __$DaemonInputEventCopyWithImpl<$Res>
    implements _$DaemonInputEventCopyWith<$Res> {
  __$DaemonInputEventCopyWithImpl(this._self, this._then);

  final _DaemonInputEvent _self;
  final $Res Function(_DaemonInputEvent) _then;

/// Create a copy of DaemonEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? event = null,}) {
  return _then(_DaemonInputEvent(
null == event ? _self.event : event // ignore: cast_nullable_to_non_nullable
as InputEvent,
  ));
}

/// Create a copy of DaemonEvent
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$InputEventCopyWith<$Res> get event {
  
  return $InputEventCopyWith<$Res>(_self.event, (value) {
    return _then(_self.copyWith(event: value));
  });
}
}

/// @nodoc


class _DaemonOutputEvent implements DaemonEvent {
  const _DaemonOutputEvent(this.event);
  

 final  OutputEvent event;

/// Create a copy of DaemonEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$DaemonOutputEventCopyWith<_DaemonOutputEvent> get copyWith => __$DaemonOutputEventCopyWithImpl<_DaemonOutputEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _DaemonOutputEvent&&(identical(other.event, event) || other.event == event));
}


@override
int get hashCode => Object.hash(runtimeType,event);

@override
String toString() {
  return 'DaemonEvent.output(event: $event)';
}


}

/// @nodoc
abstract mixin class _$DaemonOutputEventCopyWith<$Res> implements $DaemonEventCopyWith<$Res> {
  factory _$DaemonOutputEventCopyWith(_DaemonOutputEvent value, $Res Function(_DaemonOutputEvent) _then) = __$DaemonOutputEventCopyWithImpl;
@useResult
$Res call({
 OutputEvent event
});


$OutputEventCopyWith<$Res> get event;

}
/// @nodoc
class __$DaemonOutputEventCopyWithImpl<$Res>
    implements _$DaemonOutputEventCopyWith<$Res> {
  __$DaemonOutputEventCopyWithImpl(this._self, this._then);

  final _DaemonOutputEvent _self;
  final $Res Function(_DaemonOutputEvent) _then;

/// Create a copy of DaemonEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? event = null,}) {
  return _then(_DaemonOutputEvent(
null == event ? _self.event : event // ignore: cast_nullable_to_non_nullable
as OutputEvent,
  ));
}

/// Create a copy of DaemonEvent
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$OutputEventCopyWith<$Res> get event {
  
  return $OutputEventCopyWith<$Res>(_self.event, (value) {
    return _then(_self.copyWith(event: value));
  });
}
}

/// @nodoc


class _DaemonDecisionEvent implements DaemonEvent {
  const _DaemonDecisionEvent(this.event);
  

 final  DecisionEvent event;

/// Create a copy of DaemonEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$DaemonDecisionEventCopyWith<_DaemonDecisionEvent> get copyWith => __$DaemonDecisionEventCopyWithImpl<_DaemonDecisionEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _DaemonDecisionEvent&&(identical(other.event, event) || other.event == event));
}


@override
int get hashCode => Object.hash(runtimeType,event);

@override
String toString() {
  return 'DaemonEvent.decision(event: $event)';
}


}

/// @nodoc
abstract mixin class _$DaemonDecisionEventCopyWith<$Res> implements $DaemonEventCopyWith<$Res> {
  factory _$DaemonDecisionEventCopyWith(_DaemonDecisionEvent value, $Res Function(_DaemonDecisionEvent) _then) = __$DaemonDecisionEventCopyWithImpl;
@useResult
$Res call({
 DecisionEvent event
});


$DecisionEventCopyWith<$Res> get event;

}
/// @nodoc
class __$DaemonDecisionEventCopyWithImpl<$Res>
    implements _$DaemonDecisionEventCopyWith<$Res> {
  __$DaemonDecisionEventCopyWithImpl(this._self, this._then);

  final _DaemonDecisionEvent _self;
  final $Res Function(_DaemonDecisionEvent) _then;

/// Create a copy of DaemonEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? event = null,}) {
  return _then(_DaemonDecisionEvent(
null == event ? _self.event : event // ignore: cast_nullable_to_non_nullable
as DecisionEvent,
  ));
}

/// Create a copy of DaemonEvent
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$DecisionEventCopyWith<$Res> get event {
  
  return $DecisionEventCopyWith<$Res>(_self.event, (value) {
    return _then(_self.copyWith(event: value));
  });
}
}

/// @nodoc


class _DaemonStateChangeEvent implements DaemonEvent {
  const _DaemonStateChangeEvent(this.event);
  

 final  StateChangeEvent event;

/// Create a copy of DaemonEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$DaemonStateChangeEventCopyWith<_DaemonStateChangeEvent> get copyWith => __$DaemonStateChangeEventCopyWithImpl<_DaemonStateChangeEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _DaemonStateChangeEvent&&(identical(other.event, event) || other.event == event));
}


@override
int get hashCode => Object.hash(runtimeType,event);

@override
String toString() {
  return 'DaemonEvent.stateChange(event: $event)';
}


}

/// @nodoc
abstract mixin class _$DaemonStateChangeEventCopyWith<$Res> implements $DaemonEventCopyWith<$Res> {
  factory _$DaemonStateChangeEventCopyWith(_DaemonStateChangeEvent value, $Res Function(_DaemonStateChangeEvent) _then) = __$DaemonStateChangeEventCopyWithImpl;
@useResult
$Res call({
 StateChangeEvent event
});


$StateChangeEventCopyWith<$Res> get event;

}
/// @nodoc
class __$DaemonStateChangeEventCopyWithImpl<$Res>
    implements _$DaemonStateChangeEventCopyWith<$Res> {
  __$DaemonStateChangeEventCopyWithImpl(this._self, this._then);

  final _DaemonStateChangeEvent _self;
  final $Res Function(_DaemonStateChangeEvent) _then;

/// Create a copy of DaemonEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? event = null,}) {
  return _then(_DaemonStateChangeEvent(
null == event ? _self.event : event // ignore: cast_nullable_to_non_nullable
as StateChangeEvent,
  ));
}

/// Create a copy of DaemonEvent
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$StateChangeEventCopyWith<$Res> get event {
  
  return $StateChangeEventCopyWith<$Res>(_self.event, (value) {
    return _then(_self.copyWith(event: value));
  });
}
}

/// @nodoc


class _DaemonDiscoveryEvent implements DaemonEvent {
  const _DaemonDiscoveryEvent(this.candidate);
  

 final  DiscoveryCandidate candidate;

/// Create a copy of DaemonEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$DaemonDiscoveryEventCopyWith<_DaemonDiscoveryEvent> get copyWith => __$DaemonDiscoveryEventCopyWithImpl<_DaemonDiscoveryEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _DaemonDiscoveryEvent&&(identical(other.candidate, candidate) || other.candidate == candidate));
}


@override
int get hashCode => Object.hash(runtimeType,candidate);

@override
String toString() {
  return 'DaemonEvent.discovery(candidate: $candidate)';
}


}

/// @nodoc
abstract mixin class _$DaemonDiscoveryEventCopyWith<$Res> implements $DaemonEventCopyWith<$Res> {
  factory _$DaemonDiscoveryEventCopyWith(_DaemonDiscoveryEvent value, $Res Function(_DaemonDiscoveryEvent) _then) = __$DaemonDiscoveryEventCopyWithImpl;
@useResult
$Res call({
 DiscoveryCandidate candidate
});


$DiscoveryCandidateCopyWith<$Res> get candidate;

}
/// @nodoc
class __$DaemonDiscoveryEventCopyWithImpl<$Res>
    implements _$DaemonDiscoveryEventCopyWith<$Res> {
  __$DaemonDiscoveryEventCopyWithImpl(this._self, this._then);

  final _DaemonDiscoveryEvent _self;
  final $Res Function(_DaemonDiscoveryEvent) _then;

/// Create a copy of DaemonEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? candidate = null,}) {
  return _then(_DaemonDiscoveryEvent(
null == candidate ? _self.candidate : candidate // ignore: cast_nullable_to_non_nullable
as DiscoveryCandidate,
  ));
}

/// Create a copy of DaemonEvent
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$DiscoveryCandidateCopyWith<$Res> get candidate {
  
  return $DiscoveryCandidateCopyWith<$Res>(_self.candidate, (value) {
    return _then(_self.copyWith(candidate: value));
  });
}
}


/// @nodoc
mixin _$DiscoveryCandidate {

 String get deviceId; int get vendorId; int get productId; String get name;
/// Create a copy of DiscoveryCandidate
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$DiscoveryCandidateCopyWith<DiscoveryCandidate> get copyWith => _$DiscoveryCandidateCopyWithImpl<DiscoveryCandidate>(this as DiscoveryCandidate, _$identity);

  /// Serializes this DiscoveryCandidate to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DiscoveryCandidate&&(identical(other.deviceId, deviceId) || other.deviceId == deviceId)&&(identical(other.vendorId, vendorId) || other.vendorId == vendorId)&&(identical(other.productId, productId) || other.productId == productId)&&(identical(other.name, name) || other.name == name));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,deviceId,vendorId,productId,name);

@override
String toString() {
  return 'DiscoveryCandidate(deviceId: $deviceId, vendorId: $vendorId, productId: $productId, name: $name)';
}


}

/// @nodoc
abstract mixin class $DiscoveryCandidateCopyWith<$Res>  {
  factory $DiscoveryCandidateCopyWith(DiscoveryCandidate value, $Res Function(DiscoveryCandidate) _then) = _$DiscoveryCandidateCopyWithImpl;
@useResult
$Res call({
 String deviceId, int vendorId, int productId, String name
});




}
/// @nodoc
class _$DiscoveryCandidateCopyWithImpl<$Res>
    implements $DiscoveryCandidateCopyWith<$Res> {
  _$DiscoveryCandidateCopyWithImpl(this._self, this._then);

  final DiscoveryCandidate _self;
  final $Res Function(DiscoveryCandidate) _then;

/// Create a copy of DiscoveryCandidate
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? deviceId = null,Object? vendorId = null,Object? productId = null,Object? name = null,}) {
  return _then(_self.copyWith(
deviceId: null == deviceId ? _self.deviceId : deviceId // ignore: cast_nullable_to_non_nullable
as String,vendorId: null == vendorId ? _self.vendorId : vendorId // ignore: cast_nullable_to_non_nullable
as int,productId: null == productId ? _self.productId : productId // ignore: cast_nullable_to_non_nullable
as int,name: null == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as String,
  ));
}

}


/// Adds pattern-matching-related methods to [DiscoveryCandidate].
extension DiscoveryCandidatePatterns on DiscoveryCandidate {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _DiscoveryCandidate value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _DiscoveryCandidate() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _DiscoveryCandidate value)  $default,){
final _that = this;
switch (_that) {
case _DiscoveryCandidate():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _DiscoveryCandidate value)?  $default,){
final _that = this;
switch (_that) {
case _DiscoveryCandidate() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String deviceId,  int vendorId,  int productId,  String name)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _DiscoveryCandidate() when $default != null:
return $default(_that.deviceId,_that.vendorId,_that.productId,_that.name);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String deviceId,  int vendorId,  int productId,  String name)  $default,) {final _that = this;
switch (_that) {
case _DiscoveryCandidate():
return $default(_that.deviceId,_that.vendorId,_that.productId,_that.name);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String deviceId,  int vendorId,  int productId,  String name)?  $default,) {final _that = this;
switch (_that) {
case _DiscoveryCandidate() when $default != null:
return $default(_that.deviceId,_that.vendorId,_that.productId,_that.name);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _DiscoveryCandidate implements DiscoveryCandidate {
  const _DiscoveryCandidate({required this.deviceId, required this.vendorId, required this.productId, required this.name});
  factory _DiscoveryCandidate.fromJson(Map<String, dynamic> json) => _$DiscoveryCandidateFromJson(json);

@override final  String deviceId;
@override final  int vendorId;
@override final  int productId;
@override final  String name;

/// Create a copy of DiscoveryCandidate
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$DiscoveryCandidateCopyWith<_DiscoveryCandidate> get copyWith => __$DiscoveryCandidateCopyWithImpl<_DiscoveryCandidate>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$DiscoveryCandidateToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _DiscoveryCandidate&&(identical(other.deviceId, deviceId) || other.deviceId == deviceId)&&(identical(other.vendorId, vendorId) || other.vendorId == vendorId)&&(identical(other.productId, productId) || other.productId == productId)&&(identical(other.name, name) || other.name == name));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,deviceId,vendorId,productId,name);

@override
String toString() {
  return 'DiscoveryCandidate(deviceId: $deviceId, vendorId: $vendorId, productId: $productId, name: $name)';
}


}

/// @nodoc
abstract mixin class _$DiscoveryCandidateCopyWith<$Res> implements $DiscoveryCandidateCopyWith<$Res> {
  factory _$DiscoveryCandidateCopyWith(_DiscoveryCandidate value, $Res Function(_DiscoveryCandidate) _then) = __$DiscoveryCandidateCopyWithImpl;
@override @useResult
$Res call({
 String deviceId, int vendorId, int productId, String name
});




}
/// @nodoc
class __$DiscoveryCandidateCopyWithImpl<$Res>
    implements _$DiscoveryCandidateCopyWith<$Res> {
  __$DiscoveryCandidateCopyWithImpl(this._self, this._then);

  final _DiscoveryCandidate _self;
  final $Res Function(_DiscoveryCandidate) _then;

/// Create a copy of DiscoveryCandidate
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? deviceId = null,Object? vendorId = null,Object? productId = null,Object? name = null,}) {
  return _then(_DiscoveryCandidate(
deviceId: null == deviceId ? _self.deviceId : deviceId // ignore: cast_nullable_to_non_nullable
as String,vendorId: null == vendorId ? _self.vendorId : vendorId // ignore: cast_nullable_to_non_nullable
as int,productId: null == productId ? _self.productId : productId // ignore: cast_nullable_to_non_nullable
as int,name: null == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

// dart format on
