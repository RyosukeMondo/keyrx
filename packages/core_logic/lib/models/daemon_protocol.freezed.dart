// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'daemon_protocol.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
  'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models',
);

JsonRpcRequest _$JsonRpcRequestFromJson(Map<String, dynamic> json) {
  return _JsonRpcRequest.fromJson(json);
}

/// @nodoc
mixin _$JsonRpcRequest {
  String get jsonrpc => throw _privateConstructorUsedError;
  String get method => throw _privateConstructorUsedError;
  RequestId get id => throw _privateConstructorUsedError;
  Map<String, dynamic>? get params => throw _privateConstructorUsedError;

  /// Serializes this JsonRpcRequest to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of JsonRpcRequest
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $JsonRpcRequestCopyWith<JsonRpcRequest> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $JsonRpcRequestCopyWith<$Res> {
  factory $JsonRpcRequestCopyWith(
    JsonRpcRequest value,
    $Res Function(JsonRpcRequest) then,
  ) = _$JsonRpcRequestCopyWithImpl<$Res, JsonRpcRequest>;
  @useResult
  $Res call({
    String jsonrpc,
    String method,
    RequestId id,
    Map<String, dynamic>? params,
  });
}

/// @nodoc
class _$JsonRpcRequestCopyWithImpl<$Res, $Val extends JsonRpcRequest>
    implements $JsonRpcRequestCopyWith<$Res> {
  _$JsonRpcRequestCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of JsonRpcRequest
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? jsonrpc = null,
    Object? method = null,
    Object? id = null,
    Object? params = freezed,
  }) {
    return _then(
      _value.copyWith(
            jsonrpc: null == jsonrpc
                ? _value.jsonrpc
                : jsonrpc // ignore: cast_nullable_to_non_nullable
                      as String,
            method: null == method
                ? _value.method
                : method // ignore: cast_nullable_to_non_nullable
                      as String,
            id: null == id
                ? _value.id
                : id // ignore: cast_nullable_to_non_nullable
                      as RequestId,
            params: freezed == params
                ? _value.params
                : params // ignore: cast_nullable_to_non_nullable
                      as Map<String, dynamic>?,
          )
          as $Val,
    );
  }
}

/// @nodoc
abstract class _$$JsonRpcRequestImplCopyWith<$Res>
    implements $JsonRpcRequestCopyWith<$Res> {
  factory _$$JsonRpcRequestImplCopyWith(
    _$JsonRpcRequestImpl value,
    $Res Function(_$JsonRpcRequestImpl) then,
  ) = __$$JsonRpcRequestImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({
    String jsonrpc,
    String method,
    RequestId id,
    Map<String, dynamic>? params,
  });
}

/// @nodoc
class __$$JsonRpcRequestImplCopyWithImpl<$Res>
    extends _$JsonRpcRequestCopyWithImpl<$Res, _$JsonRpcRequestImpl>
    implements _$$JsonRpcRequestImplCopyWith<$Res> {
  __$$JsonRpcRequestImplCopyWithImpl(
    _$JsonRpcRequestImpl _value,
    $Res Function(_$JsonRpcRequestImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of JsonRpcRequest
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? jsonrpc = null,
    Object? method = null,
    Object? id = null,
    Object? params = freezed,
  }) {
    return _then(
      _$JsonRpcRequestImpl(
        jsonrpc: null == jsonrpc
            ? _value.jsonrpc
            : jsonrpc // ignore: cast_nullable_to_non_nullable
                  as String,
        method: null == method
            ? _value.method
            : method // ignore: cast_nullable_to_non_nullable
                  as String,
        id: null == id
            ? _value.id
            : id // ignore: cast_nullable_to_non_nullable
                  as RequestId,
        params: freezed == params
            ? _value._params
            : params // ignore: cast_nullable_to_non_nullable
                  as Map<String, dynamic>?,
      ),
    );
  }
}

/// @nodoc
@JsonSerializable()
class _$JsonRpcRequestImpl implements _JsonRpcRequest {
  const _$JsonRpcRequestImpl({
    required this.jsonrpc,
    required this.method,
    required this.id,
    final Map<String, dynamic>? params,
  }) : _params = params;

  factory _$JsonRpcRequestImpl.fromJson(Map<String, dynamic> json) =>
      _$$JsonRpcRequestImplFromJson(json);

  @override
  final String jsonrpc;
  @override
  final String method;
  @override
  final RequestId id;
  final Map<String, dynamic>? _params;
  @override
  Map<String, dynamic>? get params {
    final value = _params;
    if (value == null) return null;
    if (_params is EqualUnmodifiableMapView) return _params;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableMapView(value);
  }

  @override
  String toString() {
    return 'JsonRpcRequest(jsonrpc: $jsonrpc, method: $method, id: $id, params: $params)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$JsonRpcRequestImpl &&
            (identical(other.jsonrpc, jsonrpc) || other.jsonrpc == jsonrpc) &&
            (identical(other.method, method) || other.method == method) &&
            (identical(other.id, id) || other.id == id) &&
            const DeepCollectionEquality().equals(other._params, _params));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(
    runtimeType,
    jsonrpc,
    method,
    id,
    const DeepCollectionEquality().hash(_params),
  );

  /// Create a copy of JsonRpcRequest
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$JsonRpcRequestImplCopyWith<_$JsonRpcRequestImpl> get copyWith =>
      __$$JsonRpcRequestImplCopyWithImpl<_$JsonRpcRequestImpl>(
        this,
        _$identity,
      );

  @override
  Map<String, dynamic> toJson() {
    return _$$JsonRpcRequestImplToJson(this);
  }
}

abstract class _JsonRpcRequest implements JsonRpcRequest {
  const factory _JsonRpcRequest({
    required final String jsonrpc,
    required final String method,
    required final RequestId id,
    final Map<String, dynamic>? params,
  }) = _$JsonRpcRequestImpl;

  factory _JsonRpcRequest.fromJson(Map<String, dynamic> json) =
      _$JsonRpcRequestImpl.fromJson;

  @override
  String get jsonrpc;
  @override
  String get method;
  @override
  RequestId get id;
  @override
  Map<String, dynamic>? get params;

  /// Create a copy of JsonRpcRequest
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$JsonRpcRequestImplCopyWith<_$JsonRpcRequestImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

JsonRpcResponse _$JsonRpcResponseFromJson(Map<String, dynamic> json) {
  return _JsonRpcResponse.fromJson(json);
}

/// @nodoc
mixin _$JsonRpcResponse {
  String get jsonrpc => throw _privateConstructorUsedError;
  dynamic get result => throw _privateConstructorUsedError;
  RequestId get id => throw _privateConstructorUsedError;

  /// Serializes this JsonRpcResponse to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of JsonRpcResponse
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $JsonRpcResponseCopyWith<JsonRpcResponse> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $JsonRpcResponseCopyWith<$Res> {
  factory $JsonRpcResponseCopyWith(
    JsonRpcResponse value,
    $Res Function(JsonRpcResponse) then,
  ) = _$JsonRpcResponseCopyWithImpl<$Res, JsonRpcResponse>;
  @useResult
  $Res call({String jsonrpc, dynamic result, RequestId id});
}

/// @nodoc
class _$JsonRpcResponseCopyWithImpl<$Res, $Val extends JsonRpcResponse>
    implements $JsonRpcResponseCopyWith<$Res> {
  _$JsonRpcResponseCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of JsonRpcResponse
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? jsonrpc = null,
    Object? result = freezed,
    Object? id = null,
  }) {
    return _then(
      _value.copyWith(
            jsonrpc: null == jsonrpc
                ? _value.jsonrpc
                : jsonrpc // ignore: cast_nullable_to_non_nullable
                      as String,
            result: freezed == result
                ? _value.result
                : result // ignore: cast_nullable_to_non_nullable
                      as dynamic,
            id: null == id
                ? _value.id
                : id // ignore: cast_nullable_to_non_nullable
                      as RequestId,
          )
          as $Val,
    );
  }
}

/// @nodoc
abstract class _$$JsonRpcResponseImplCopyWith<$Res>
    implements $JsonRpcResponseCopyWith<$Res> {
  factory _$$JsonRpcResponseImplCopyWith(
    _$JsonRpcResponseImpl value,
    $Res Function(_$JsonRpcResponseImpl) then,
  ) = __$$JsonRpcResponseImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({String jsonrpc, dynamic result, RequestId id});
}

/// @nodoc
class __$$JsonRpcResponseImplCopyWithImpl<$Res>
    extends _$JsonRpcResponseCopyWithImpl<$Res, _$JsonRpcResponseImpl>
    implements _$$JsonRpcResponseImplCopyWith<$Res> {
  __$$JsonRpcResponseImplCopyWithImpl(
    _$JsonRpcResponseImpl _value,
    $Res Function(_$JsonRpcResponseImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of JsonRpcResponse
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? jsonrpc = null,
    Object? result = freezed,
    Object? id = null,
  }) {
    return _then(
      _$JsonRpcResponseImpl(
        jsonrpc: null == jsonrpc
            ? _value.jsonrpc
            : jsonrpc // ignore: cast_nullable_to_non_nullable
                  as String,
        result: freezed == result
            ? _value.result
            : result // ignore: cast_nullable_to_non_nullable
                  as dynamic,
        id: null == id
            ? _value.id
            : id // ignore: cast_nullable_to_non_nullable
                  as RequestId,
      ),
    );
  }
}

/// @nodoc
@JsonSerializable()
class _$JsonRpcResponseImpl implements _JsonRpcResponse {
  const _$JsonRpcResponseImpl({
    required this.jsonrpc,
    required this.result,
    required this.id,
  });

  factory _$JsonRpcResponseImpl.fromJson(Map<String, dynamic> json) =>
      _$$JsonRpcResponseImplFromJson(json);

  @override
  final String jsonrpc;
  @override
  final dynamic result;
  @override
  final RequestId id;

  @override
  String toString() {
    return 'JsonRpcResponse(jsonrpc: $jsonrpc, result: $result, id: $id)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$JsonRpcResponseImpl &&
            (identical(other.jsonrpc, jsonrpc) || other.jsonrpc == jsonrpc) &&
            const DeepCollectionEquality().equals(other.result, result) &&
            (identical(other.id, id) || other.id == id));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(
    runtimeType,
    jsonrpc,
    const DeepCollectionEquality().hash(result),
    id,
  );

  /// Create a copy of JsonRpcResponse
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$JsonRpcResponseImplCopyWith<_$JsonRpcResponseImpl> get copyWith =>
      __$$JsonRpcResponseImplCopyWithImpl<_$JsonRpcResponseImpl>(
        this,
        _$identity,
      );

  @override
  Map<String, dynamic> toJson() {
    return _$$JsonRpcResponseImplToJson(this);
  }
}

abstract class _JsonRpcResponse implements JsonRpcResponse {
  const factory _JsonRpcResponse({
    required final String jsonrpc,
    required final dynamic result,
    required final RequestId id,
  }) = _$JsonRpcResponseImpl;

  factory _JsonRpcResponse.fromJson(Map<String, dynamic> json) =
      _$JsonRpcResponseImpl.fromJson;

  @override
  String get jsonrpc;
  @override
  dynamic get result;
  @override
  RequestId get id;

  /// Create a copy of JsonRpcResponse
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$JsonRpcResponseImplCopyWith<_$JsonRpcResponseImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

ErrorObject _$ErrorObjectFromJson(Map<String, dynamic> json) {
  return _ErrorObject.fromJson(json);
}

/// @nodoc
mixin _$ErrorObject {
  int get code => throw _privateConstructorUsedError;
  String get message => throw _privateConstructorUsedError;
  dynamic get data => throw _privateConstructorUsedError;

  /// Serializes this ErrorObject to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of ErrorObject
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $ErrorObjectCopyWith<ErrorObject> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $ErrorObjectCopyWith<$Res> {
  factory $ErrorObjectCopyWith(
    ErrorObject value,
    $Res Function(ErrorObject) then,
  ) = _$ErrorObjectCopyWithImpl<$Res, ErrorObject>;
  @useResult
  $Res call({int code, String message, dynamic data});
}

/// @nodoc
class _$ErrorObjectCopyWithImpl<$Res, $Val extends ErrorObject>
    implements $ErrorObjectCopyWith<$Res> {
  _$ErrorObjectCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of ErrorObject
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? code = null,
    Object? message = null,
    Object? data = freezed,
  }) {
    return _then(
      _value.copyWith(
            code: null == code
                ? _value.code
                : code // ignore: cast_nullable_to_non_nullable
                      as int,
            message: null == message
                ? _value.message
                : message // ignore: cast_nullable_to_non_nullable
                      as String,
            data: freezed == data
                ? _value.data
                : data // ignore: cast_nullable_to_non_nullable
                      as dynamic,
          )
          as $Val,
    );
  }
}

/// @nodoc
abstract class _$$ErrorObjectImplCopyWith<$Res>
    implements $ErrorObjectCopyWith<$Res> {
  factory _$$ErrorObjectImplCopyWith(
    _$ErrorObjectImpl value,
    $Res Function(_$ErrorObjectImpl) then,
  ) = __$$ErrorObjectImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({int code, String message, dynamic data});
}

/// @nodoc
class __$$ErrorObjectImplCopyWithImpl<$Res>
    extends _$ErrorObjectCopyWithImpl<$Res, _$ErrorObjectImpl>
    implements _$$ErrorObjectImplCopyWith<$Res> {
  __$$ErrorObjectImplCopyWithImpl(
    _$ErrorObjectImpl _value,
    $Res Function(_$ErrorObjectImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of ErrorObject
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? code = null,
    Object? message = null,
    Object? data = freezed,
  }) {
    return _then(
      _$ErrorObjectImpl(
        code: null == code
            ? _value.code
            : code // ignore: cast_nullable_to_non_nullable
                  as int,
        message: null == message
            ? _value.message
            : message // ignore: cast_nullable_to_non_nullable
                  as String,
        data: freezed == data
            ? _value.data
            : data // ignore: cast_nullable_to_non_nullable
                  as dynamic,
      ),
    );
  }
}

/// @nodoc
@JsonSerializable()
class _$ErrorObjectImpl implements _ErrorObject {
  const _$ErrorObjectImpl({
    required this.code,
    required this.message,
    this.data,
  });

  factory _$ErrorObjectImpl.fromJson(Map<String, dynamic> json) =>
      _$$ErrorObjectImplFromJson(json);

  @override
  final int code;
  @override
  final String message;
  @override
  final dynamic data;

  @override
  String toString() {
    return 'ErrorObject(code: $code, message: $message, data: $data)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ErrorObjectImpl &&
            (identical(other.code, code) || other.code == code) &&
            (identical(other.message, message) || other.message == message) &&
            const DeepCollectionEquality().equals(other.data, data));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(
    runtimeType,
    code,
    message,
    const DeepCollectionEquality().hash(data),
  );

  /// Create a copy of ErrorObject
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$ErrorObjectImplCopyWith<_$ErrorObjectImpl> get copyWith =>
      __$$ErrorObjectImplCopyWithImpl<_$ErrorObjectImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$ErrorObjectImplToJson(this);
  }
}

abstract class _ErrorObject implements ErrorObject {
  const factory _ErrorObject({
    required final int code,
    required final String message,
    final dynamic data,
  }) = _$ErrorObjectImpl;

  factory _ErrorObject.fromJson(Map<String, dynamic> json) =
      _$ErrorObjectImpl.fromJson;

  @override
  int get code;
  @override
  String get message;
  @override
  dynamic get data;

  /// Create a copy of ErrorObject
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ErrorObjectImplCopyWith<_$ErrorObjectImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

JsonRpcError _$JsonRpcErrorFromJson(Map<String, dynamic> json) {
  return _JsonRpcError.fromJson(json);
}

/// @nodoc
mixin _$JsonRpcError {
  String get jsonrpc => throw _privateConstructorUsedError;
  ErrorObject get error => throw _privateConstructorUsedError;
  RequestId get id => throw _privateConstructorUsedError;

  /// Serializes this JsonRpcError to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of JsonRpcError
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $JsonRpcErrorCopyWith<JsonRpcError> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $JsonRpcErrorCopyWith<$Res> {
  factory $JsonRpcErrorCopyWith(
    JsonRpcError value,
    $Res Function(JsonRpcError) then,
  ) = _$JsonRpcErrorCopyWithImpl<$Res, JsonRpcError>;
  @useResult
  $Res call({String jsonrpc, ErrorObject error, RequestId id});

  $ErrorObjectCopyWith<$Res> get error;
}

/// @nodoc
class _$JsonRpcErrorCopyWithImpl<$Res, $Val extends JsonRpcError>
    implements $JsonRpcErrorCopyWith<$Res> {
  _$JsonRpcErrorCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of JsonRpcError
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? jsonrpc = null, Object? error = null, Object? id = null}) {
    return _then(
      _value.copyWith(
            jsonrpc: null == jsonrpc
                ? _value.jsonrpc
                : jsonrpc // ignore: cast_nullable_to_non_nullable
                      as String,
            error: null == error
                ? _value.error
                : error // ignore: cast_nullable_to_non_nullable
                      as ErrorObject,
            id: null == id
                ? _value.id
                : id // ignore: cast_nullable_to_non_nullable
                      as RequestId,
          )
          as $Val,
    );
  }

  /// Create a copy of JsonRpcError
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $ErrorObjectCopyWith<$Res> get error {
    return $ErrorObjectCopyWith<$Res>(_value.error, (value) {
      return _then(_value.copyWith(error: value) as $Val);
    });
  }
}

/// @nodoc
abstract class _$$JsonRpcErrorImplCopyWith<$Res>
    implements $JsonRpcErrorCopyWith<$Res> {
  factory _$$JsonRpcErrorImplCopyWith(
    _$JsonRpcErrorImpl value,
    $Res Function(_$JsonRpcErrorImpl) then,
  ) = __$$JsonRpcErrorImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({String jsonrpc, ErrorObject error, RequestId id});

  @override
  $ErrorObjectCopyWith<$Res> get error;
}

/// @nodoc
class __$$JsonRpcErrorImplCopyWithImpl<$Res>
    extends _$JsonRpcErrorCopyWithImpl<$Res, _$JsonRpcErrorImpl>
    implements _$$JsonRpcErrorImplCopyWith<$Res> {
  __$$JsonRpcErrorImplCopyWithImpl(
    _$JsonRpcErrorImpl _value,
    $Res Function(_$JsonRpcErrorImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of JsonRpcError
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? jsonrpc = null, Object? error = null, Object? id = null}) {
    return _then(
      _$JsonRpcErrorImpl(
        jsonrpc: null == jsonrpc
            ? _value.jsonrpc
            : jsonrpc // ignore: cast_nullable_to_non_nullable
                  as String,
        error: null == error
            ? _value.error
            : error // ignore: cast_nullable_to_non_nullable
                  as ErrorObject,
        id: null == id
            ? _value.id
            : id // ignore: cast_nullable_to_non_nullable
                  as RequestId,
      ),
    );
  }
}

/// @nodoc
@JsonSerializable()
class _$JsonRpcErrorImpl implements _JsonRpcError {
  const _$JsonRpcErrorImpl({
    required this.jsonrpc,
    required this.error,
    required this.id,
  });

  factory _$JsonRpcErrorImpl.fromJson(Map<String, dynamic> json) =>
      _$$JsonRpcErrorImplFromJson(json);

  @override
  final String jsonrpc;
  @override
  final ErrorObject error;
  @override
  final RequestId id;

  @override
  String toString() {
    return 'JsonRpcError(jsonrpc: $jsonrpc, error: $error, id: $id)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$JsonRpcErrorImpl &&
            (identical(other.jsonrpc, jsonrpc) || other.jsonrpc == jsonrpc) &&
            (identical(other.error, error) || other.error == error) &&
            (identical(other.id, id) || other.id == id));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, jsonrpc, error, id);

  /// Create a copy of JsonRpcError
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$JsonRpcErrorImplCopyWith<_$JsonRpcErrorImpl> get copyWith =>
      __$$JsonRpcErrorImplCopyWithImpl<_$JsonRpcErrorImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$JsonRpcErrorImplToJson(this);
  }
}

abstract class _JsonRpcError implements JsonRpcError {
  const factory _JsonRpcError({
    required final String jsonrpc,
    required final ErrorObject error,
    required final RequestId id,
  }) = _$JsonRpcErrorImpl;

  factory _JsonRpcError.fromJson(Map<String, dynamic> json) =
      _$JsonRpcErrorImpl.fromJson;

  @override
  String get jsonrpc;
  @override
  ErrorObject get error;
  @override
  RequestId get id;

  /// Create a copy of JsonRpcError
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$JsonRpcErrorImplCopyWith<_$JsonRpcErrorImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

JsonRpcNotification _$JsonRpcNotificationFromJson(Map<String, dynamic> json) {
  return _JsonRpcNotification.fromJson(json);
}

/// @nodoc
mixin _$JsonRpcNotification {
  String get jsonrpc => throw _privateConstructorUsedError;
  String get method => throw _privateConstructorUsedError;
  Map<String, dynamic> get params => throw _privateConstructorUsedError;

  /// Serializes this JsonRpcNotification to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of JsonRpcNotification
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $JsonRpcNotificationCopyWith<JsonRpcNotification> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $JsonRpcNotificationCopyWith<$Res> {
  factory $JsonRpcNotificationCopyWith(
    JsonRpcNotification value,
    $Res Function(JsonRpcNotification) then,
  ) = _$JsonRpcNotificationCopyWithImpl<$Res, JsonRpcNotification>;
  @useResult
  $Res call({String jsonrpc, String method, Map<String, dynamic> params});
}

/// @nodoc
class _$JsonRpcNotificationCopyWithImpl<$Res, $Val extends JsonRpcNotification>
    implements $JsonRpcNotificationCopyWith<$Res> {
  _$JsonRpcNotificationCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of JsonRpcNotification
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? jsonrpc = null,
    Object? method = null,
    Object? params = null,
  }) {
    return _then(
      _value.copyWith(
            jsonrpc: null == jsonrpc
                ? _value.jsonrpc
                : jsonrpc // ignore: cast_nullable_to_non_nullable
                      as String,
            method: null == method
                ? _value.method
                : method // ignore: cast_nullable_to_non_nullable
                      as String,
            params: null == params
                ? _value.params
                : params // ignore: cast_nullable_to_non_nullable
                      as Map<String, dynamic>,
          )
          as $Val,
    );
  }
}

/// @nodoc
abstract class _$$JsonRpcNotificationImplCopyWith<$Res>
    implements $JsonRpcNotificationCopyWith<$Res> {
  factory _$$JsonRpcNotificationImplCopyWith(
    _$JsonRpcNotificationImpl value,
    $Res Function(_$JsonRpcNotificationImpl) then,
  ) = __$$JsonRpcNotificationImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({String jsonrpc, String method, Map<String, dynamic> params});
}

/// @nodoc
class __$$JsonRpcNotificationImplCopyWithImpl<$Res>
    extends _$JsonRpcNotificationCopyWithImpl<$Res, _$JsonRpcNotificationImpl>
    implements _$$JsonRpcNotificationImplCopyWith<$Res> {
  __$$JsonRpcNotificationImplCopyWithImpl(
    _$JsonRpcNotificationImpl _value,
    $Res Function(_$JsonRpcNotificationImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of JsonRpcNotification
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? jsonrpc = null,
    Object? method = null,
    Object? params = null,
  }) {
    return _then(
      _$JsonRpcNotificationImpl(
        jsonrpc: null == jsonrpc
            ? _value.jsonrpc
            : jsonrpc // ignore: cast_nullable_to_non_nullable
                  as String,
        method: null == method
            ? _value.method
            : method // ignore: cast_nullable_to_non_nullable
                  as String,
        params: null == params
            ? _value._params
            : params // ignore: cast_nullable_to_non_nullable
                  as Map<String, dynamic>,
      ),
    );
  }
}

/// @nodoc
@JsonSerializable()
class _$JsonRpcNotificationImpl implements _JsonRpcNotification {
  const _$JsonRpcNotificationImpl({
    required this.jsonrpc,
    required this.method,
    required final Map<String, dynamic> params,
  }) : _params = params;

  factory _$JsonRpcNotificationImpl.fromJson(Map<String, dynamic> json) =>
      _$$JsonRpcNotificationImplFromJson(json);

  @override
  final String jsonrpc;
  @override
  final String method;
  final Map<String, dynamic> _params;
  @override
  Map<String, dynamic> get params {
    if (_params is EqualUnmodifiableMapView) return _params;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableMapView(_params);
  }

  @override
  String toString() {
    return 'JsonRpcNotification(jsonrpc: $jsonrpc, method: $method, params: $params)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$JsonRpcNotificationImpl &&
            (identical(other.jsonrpc, jsonrpc) || other.jsonrpc == jsonrpc) &&
            (identical(other.method, method) || other.method == method) &&
            const DeepCollectionEquality().equals(other._params, _params));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(
    runtimeType,
    jsonrpc,
    method,
    const DeepCollectionEquality().hash(_params),
  );

  /// Create a copy of JsonRpcNotification
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$JsonRpcNotificationImplCopyWith<_$JsonRpcNotificationImpl> get copyWith =>
      __$$JsonRpcNotificationImplCopyWithImpl<_$JsonRpcNotificationImpl>(
        this,
        _$identity,
      );

  @override
  Map<String, dynamic> toJson() {
    return _$$JsonRpcNotificationImplToJson(this);
  }
}

abstract class _JsonRpcNotification implements JsonRpcNotification {
  const factory _JsonRpcNotification({
    required final String jsonrpc,
    required final String method,
    required final Map<String, dynamic> params,
  }) = _$JsonRpcNotificationImpl;

  factory _JsonRpcNotification.fromJson(Map<String, dynamic> json) =
      _$JsonRpcNotificationImpl.fromJson;

  @override
  String get jsonrpc;
  @override
  String get method;
  @override
  Map<String, dynamic> get params;

  /// Create a copy of JsonRpcNotification
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$JsonRpcNotificationImplCopyWith<_$JsonRpcNotificationImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
mixin _$IncomingMessage {
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(JsonRpcResponse response) response,
    required TResult Function(JsonRpcError error) error,
    required TResult Function(JsonRpcNotification notification) notification,
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(JsonRpcResponse response)? response,
    TResult? Function(JsonRpcError error)? error,
    TResult? Function(JsonRpcNotification notification)? notification,
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(JsonRpcResponse response)? response,
    TResult Function(JsonRpcError error)? error,
    TResult Function(JsonRpcNotification notification)? notification,
    required TResult orElse(),
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(_IncomingResponse value) response,
    required TResult Function(_IncomingError value) error,
    required TResult Function(_IncomingNotification value) notification,
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(_IncomingResponse value)? response,
    TResult? Function(_IncomingError value)? error,
    TResult? Function(_IncomingNotification value)? notification,
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(_IncomingResponse value)? response,
    TResult Function(_IncomingError value)? error,
    TResult Function(_IncomingNotification value)? notification,
    required TResult orElse(),
  }) => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $IncomingMessageCopyWith<$Res> {
  factory $IncomingMessageCopyWith(
    IncomingMessage value,
    $Res Function(IncomingMessage) then,
  ) = _$IncomingMessageCopyWithImpl<$Res, IncomingMessage>;
}

/// @nodoc
class _$IncomingMessageCopyWithImpl<$Res, $Val extends IncomingMessage>
    implements $IncomingMessageCopyWith<$Res> {
  _$IncomingMessageCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of IncomingMessage
  /// with the given fields replaced by the non-null parameter values.
}

/// @nodoc
abstract class _$$IncomingResponseImplCopyWith<$Res> {
  factory _$$IncomingResponseImplCopyWith(
    _$IncomingResponseImpl value,
    $Res Function(_$IncomingResponseImpl) then,
  ) = __$$IncomingResponseImplCopyWithImpl<$Res>;
  @useResult
  $Res call({JsonRpcResponse response});

  $JsonRpcResponseCopyWith<$Res> get response;
}

/// @nodoc
class __$$IncomingResponseImplCopyWithImpl<$Res>
    extends _$IncomingMessageCopyWithImpl<$Res, _$IncomingResponseImpl>
    implements _$$IncomingResponseImplCopyWith<$Res> {
  __$$IncomingResponseImplCopyWithImpl(
    _$IncomingResponseImpl _value,
    $Res Function(_$IncomingResponseImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of IncomingMessage
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? response = null}) {
    return _then(
      _$IncomingResponseImpl(
        null == response
            ? _value.response
            : response // ignore: cast_nullable_to_non_nullable
                  as JsonRpcResponse,
      ),
    );
  }

  /// Create a copy of IncomingMessage
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $JsonRpcResponseCopyWith<$Res> get response {
    return $JsonRpcResponseCopyWith<$Res>(_value.response, (value) {
      return _then(_value.copyWith(response: value));
    });
  }
}

/// @nodoc

class _$IncomingResponseImpl implements _IncomingResponse {
  const _$IncomingResponseImpl(this.response);

  @override
  final JsonRpcResponse response;

  @override
  String toString() {
    return 'IncomingMessage.response(response: $response)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$IncomingResponseImpl &&
            (identical(other.response, response) ||
                other.response == response));
  }

  @override
  int get hashCode => Object.hash(runtimeType, response);

  /// Create a copy of IncomingMessage
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$IncomingResponseImplCopyWith<_$IncomingResponseImpl> get copyWith =>
      __$$IncomingResponseImplCopyWithImpl<_$IncomingResponseImpl>(
        this,
        _$identity,
      );

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(JsonRpcResponse response) response,
    required TResult Function(JsonRpcError error) error,
    required TResult Function(JsonRpcNotification notification) notification,
  }) {
    return response(this.response);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(JsonRpcResponse response)? response,
    TResult? Function(JsonRpcError error)? error,
    TResult? Function(JsonRpcNotification notification)? notification,
  }) {
    return response?.call(this.response);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(JsonRpcResponse response)? response,
    TResult Function(JsonRpcError error)? error,
    TResult Function(JsonRpcNotification notification)? notification,
    required TResult orElse(),
  }) {
    if (response != null) {
      return response(this.response);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(_IncomingResponse value) response,
    required TResult Function(_IncomingError value) error,
    required TResult Function(_IncomingNotification value) notification,
  }) {
    return response(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(_IncomingResponse value)? response,
    TResult? Function(_IncomingError value)? error,
    TResult? Function(_IncomingNotification value)? notification,
  }) {
    return response?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(_IncomingResponse value)? response,
    TResult Function(_IncomingError value)? error,
    TResult Function(_IncomingNotification value)? notification,
    required TResult orElse(),
  }) {
    if (response != null) {
      return response(this);
    }
    return orElse();
  }
}

abstract class _IncomingResponse implements IncomingMessage {
  const factory _IncomingResponse(final JsonRpcResponse response) =
      _$IncomingResponseImpl;

  JsonRpcResponse get response;

  /// Create a copy of IncomingMessage
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$IncomingResponseImplCopyWith<_$IncomingResponseImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$IncomingErrorImplCopyWith<$Res> {
  factory _$$IncomingErrorImplCopyWith(
    _$IncomingErrorImpl value,
    $Res Function(_$IncomingErrorImpl) then,
  ) = __$$IncomingErrorImplCopyWithImpl<$Res>;
  @useResult
  $Res call({JsonRpcError error});

  $JsonRpcErrorCopyWith<$Res> get error;
}

/// @nodoc
class __$$IncomingErrorImplCopyWithImpl<$Res>
    extends _$IncomingMessageCopyWithImpl<$Res, _$IncomingErrorImpl>
    implements _$$IncomingErrorImplCopyWith<$Res> {
  __$$IncomingErrorImplCopyWithImpl(
    _$IncomingErrorImpl _value,
    $Res Function(_$IncomingErrorImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of IncomingMessage
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? error = null}) {
    return _then(
      _$IncomingErrorImpl(
        null == error
            ? _value.error
            : error // ignore: cast_nullable_to_non_nullable
                  as JsonRpcError,
      ),
    );
  }

  /// Create a copy of IncomingMessage
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $JsonRpcErrorCopyWith<$Res> get error {
    return $JsonRpcErrorCopyWith<$Res>(_value.error, (value) {
      return _then(_value.copyWith(error: value));
    });
  }
}

/// @nodoc

class _$IncomingErrorImpl implements _IncomingError {
  const _$IncomingErrorImpl(this.error);

  @override
  final JsonRpcError error;

  @override
  String toString() {
    return 'IncomingMessage.error(error: $error)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$IncomingErrorImpl &&
            (identical(other.error, error) || other.error == error));
  }

  @override
  int get hashCode => Object.hash(runtimeType, error);

  /// Create a copy of IncomingMessage
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$IncomingErrorImplCopyWith<_$IncomingErrorImpl> get copyWith =>
      __$$IncomingErrorImplCopyWithImpl<_$IncomingErrorImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(JsonRpcResponse response) response,
    required TResult Function(JsonRpcError error) error,
    required TResult Function(JsonRpcNotification notification) notification,
  }) {
    return error(this.error);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(JsonRpcResponse response)? response,
    TResult? Function(JsonRpcError error)? error,
    TResult? Function(JsonRpcNotification notification)? notification,
  }) {
    return error?.call(this.error);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(JsonRpcResponse response)? response,
    TResult Function(JsonRpcError error)? error,
    TResult Function(JsonRpcNotification notification)? notification,
    required TResult orElse(),
  }) {
    if (error != null) {
      return error(this.error);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(_IncomingResponse value) response,
    required TResult Function(_IncomingError value) error,
    required TResult Function(_IncomingNotification value) notification,
  }) {
    return error(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(_IncomingResponse value)? response,
    TResult? Function(_IncomingError value)? error,
    TResult? Function(_IncomingNotification value)? notification,
  }) {
    return error?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(_IncomingResponse value)? response,
    TResult Function(_IncomingError value)? error,
    TResult Function(_IncomingNotification value)? notification,
    required TResult orElse(),
  }) {
    if (error != null) {
      return error(this);
    }
    return orElse();
  }
}

abstract class _IncomingError implements IncomingMessage {
  const factory _IncomingError(final JsonRpcError error) = _$IncomingErrorImpl;

  JsonRpcError get error;

  /// Create a copy of IncomingMessage
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$IncomingErrorImplCopyWith<_$IncomingErrorImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$IncomingNotificationImplCopyWith<$Res> {
  factory _$$IncomingNotificationImplCopyWith(
    _$IncomingNotificationImpl value,
    $Res Function(_$IncomingNotificationImpl) then,
  ) = __$$IncomingNotificationImplCopyWithImpl<$Res>;
  @useResult
  $Res call({JsonRpcNotification notification});

  $JsonRpcNotificationCopyWith<$Res> get notification;
}

/// @nodoc
class __$$IncomingNotificationImplCopyWithImpl<$Res>
    extends _$IncomingMessageCopyWithImpl<$Res, _$IncomingNotificationImpl>
    implements _$$IncomingNotificationImplCopyWith<$Res> {
  __$$IncomingNotificationImplCopyWithImpl(
    _$IncomingNotificationImpl _value,
    $Res Function(_$IncomingNotificationImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of IncomingMessage
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? notification = null}) {
    return _then(
      _$IncomingNotificationImpl(
        null == notification
            ? _value.notification
            : notification // ignore: cast_nullable_to_non_nullable
                  as JsonRpcNotification,
      ),
    );
  }

  /// Create a copy of IncomingMessage
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $JsonRpcNotificationCopyWith<$Res> get notification {
    return $JsonRpcNotificationCopyWith<$Res>(_value.notification, (value) {
      return _then(_value.copyWith(notification: value));
    });
  }
}

/// @nodoc

class _$IncomingNotificationImpl implements _IncomingNotification {
  const _$IncomingNotificationImpl(this.notification);

  @override
  final JsonRpcNotification notification;

  @override
  String toString() {
    return 'IncomingMessage.notification(notification: $notification)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$IncomingNotificationImpl &&
            (identical(other.notification, notification) ||
                other.notification == notification));
  }

  @override
  int get hashCode => Object.hash(runtimeType, notification);

  /// Create a copy of IncomingMessage
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$IncomingNotificationImplCopyWith<_$IncomingNotificationImpl>
  get copyWith =>
      __$$IncomingNotificationImplCopyWithImpl<_$IncomingNotificationImpl>(
        this,
        _$identity,
      );

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(JsonRpcResponse response) response,
    required TResult Function(JsonRpcError error) error,
    required TResult Function(JsonRpcNotification notification) notification,
  }) {
    return notification(this.notification);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(JsonRpcResponse response)? response,
    TResult? Function(JsonRpcError error)? error,
    TResult? Function(JsonRpcNotification notification)? notification,
  }) {
    return notification?.call(this.notification);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(JsonRpcResponse response)? response,
    TResult Function(JsonRpcError error)? error,
    TResult Function(JsonRpcNotification notification)? notification,
    required TResult orElse(),
  }) {
    if (notification != null) {
      return notification(this.notification);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(_IncomingResponse value) response,
    required TResult Function(_IncomingError value) error,
    required TResult Function(_IncomingNotification value) notification,
  }) {
    return notification(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(_IncomingResponse value)? response,
    TResult? Function(_IncomingError value)? error,
    TResult? Function(_IncomingNotification value)? notification,
  }) {
    return notification?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(_IncomingResponse value)? response,
    TResult Function(_IncomingError value)? error,
    TResult Function(_IncomingNotification value)? notification,
    required TResult orElse(),
  }) {
    if (notification != null) {
      return notification(this);
    }
    return orElse();
  }
}

abstract class _IncomingNotification implements IncomingMessage {
  const factory _IncomingNotification(final JsonRpcNotification notification) =
      _$IncomingNotificationImpl;

  JsonRpcNotification get notification;

  /// Create a copy of IncomingMessage
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$IncomingNotificationImplCopyWith<_$IncomingNotificationImpl>
  get copyWith => throw _privateConstructorUsedError;
}

DaemonDeviceInfo _$DaemonDeviceInfoFromJson(Map<String, dynamic> json) {
  return _DaemonDeviceInfo.fromJson(json);
}

/// @nodoc
mixin _$DaemonDeviceInfo {
  String get path => throw _privateConstructorUsedError;
  String get name => throw _privateConstructorUsedError;
  @JsonKey(name: 'vendor_id')
  String? get vendorId => throw _privateConstructorUsedError;
  @JsonKey(name: 'product_id')
  String? get productId => throw _privateConstructorUsedError;

  /// Serializes this DaemonDeviceInfo to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of DaemonDeviceInfo
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $DaemonDeviceInfoCopyWith<DaemonDeviceInfo> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $DaemonDeviceInfoCopyWith<$Res> {
  factory $DaemonDeviceInfoCopyWith(
    DaemonDeviceInfo value,
    $Res Function(DaemonDeviceInfo) then,
  ) = _$DaemonDeviceInfoCopyWithImpl<$Res, DaemonDeviceInfo>;
  @useResult
  $Res call({
    String path,
    String name,
    @JsonKey(name: 'vendor_id') String? vendorId,
    @JsonKey(name: 'product_id') String? productId,
  });
}

/// @nodoc
class _$DaemonDeviceInfoCopyWithImpl<$Res, $Val extends DaemonDeviceInfo>
    implements $DaemonDeviceInfoCopyWith<$Res> {
  _$DaemonDeviceInfoCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of DaemonDeviceInfo
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? path = null,
    Object? name = null,
    Object? vendorId = freezed,
    Object? productId = freezed,
  }) {
    return _then(
      _value.copyWith(
            path: null == path
                ? _value.path
                : path // ignore: cast_nullable_to_non_nullable
                      as String,
            name: null == name
                ? _value.name
                : name // ignore: cast_nullable_to_non_nullable
                      as String,
            vendorId: freezed == vendorId
                ? _value.vendorId
                : vendorId // ignore: cast_nullable_to_non_nullable
                      as String?,
            productId: freezed == productId
                ? _value.productId
                : productId // ignore: cast_nullable_to_non_nullable
                      as String?,
          )
          as $Val,
    );
  }
}

/// @nodoc
abstract class _$$DaemonDeviceInfoImplCopyWith<$Res>
    implements $DaemonDeviceInfoCopyWith<$Res> {
  factory _$$DaemonDeviceInfoImplCopyWith(
    _$DaemonDeviceInfoImpl value,
    $Res Function(_$DaemonDeviceInfoImpl) then,
  ) = __$$DaemonDeviceInfoImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({
    String path,
    String name,
    @JsonKey(name: 'vendor_id') String? vendorId,
    @JsonKey(name: 'product_id') String? productId,
  });
}

/// @nodoc
class __$$DaemonDeviceInfoImplCopyWithImpl<$Res>
    extends _$DaemonDeviceInfoCopyWithImpl<$Res, _$DaemonDeviceInfoImpl>
    implements _$$DaemonDeviceInfoImplCopyWith<$Res> {
  __$$DaemonDeviceInfoImplCopyWithImpl(
    _$DaemonDeviceInfoImpl _value,
    $Res Function(_$DaemonDeviceInfoImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of DaemonDeviceInfo
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? path = null,
    Object? name = null,
    Object? vendorId = freezed,
    Object? productId = freezed,
  }) {
    return _then(
      _$DaemonDeviceInfoImpl(
        path: null == path
            ? _value.path
            : path // ignore: cast_nullable_to_non_nullable
                  as String,
        name: null == name
            ? _value.name
            : name // ignore: cast_nullable_to_non_nullable
                  as String,
        vendorId: freezed == vendorId
            ? _value.vendorId
            : vendorId // ignore: cast_nullable_to_non_nullable
                  as String?,
        productId: freezed == productId
            ? _value.productId
            : productId // ignore: cast_nullable_to_non_nullable
                  as String?,
      ),
    );
  }
}

/// @nodoc
@JsonSerializable()
class _$DaemonDeviceInfoImpl implements _DaemonDeviceInfo {
  const _$DaemonDeviceInfoImpl({
    required this.path,
    required this.name,
    @JsonKey(name: 'vendor_id') this.vendorId,
    @JsonKey(name: 'product_id') this.productId,
  });

  factory _$DaemonDeviceInfoImpl.fromJson(Map<String, dynamic> json) =>
      _$$DaemonDeviceInfoImplFromJson(json);

  @override
  final String path;
  @override
  final String name;
  @override
  @JsonKey(name: 'vendor_id')
  final String? vendorId;
  @override
  @JsonKey(name: 'product_id')
  final String? productId;

  @override
  String toString() {
    return 'DaemonDeviceInfo(path: $path, name: $name, vendorId: $vendorId, productId: $productId)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$DaemonDeviceInfoImpl &&
            (identical(other.path, path) || other.path == path) &&
            (identical(other.name, name) || other.name == name) &&
            (identical(other.vendorId, vendorId) ||
                other.vendorId == vendorId) &&
            (identical(other.productId, productId) ||
                other.productId == productId));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, path, name, vendorId, productId);

  /// Create a copy of DaemonDeviceInfo
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$DaemonDeviceInfoImplCopyWith<_$DaemonDeviceInfoImpl> get copyWith =>
      __$$DaemonDeviceInfoImplCopyWithImpl<_$DaemonDeviceInfoImpl>(
        this,
        _$identity,
      );

  @override
  Map<String, dynamic> toJson() {
    return _$$DaemonDeviceInfoImplToJson(this);
  }
}

abstract class _DaemonDeviceInfo implements DaemonDeviceInfo {
  const factory _DaemonDeviceInfo({
    required final String path,
    required final String name,
    @JsonKey(name: 'vendor_id') final String? vendorId,
    @JsonKey(name: 'product_id') final String? productId,
  }) = _$DaemonDeviceInfoImpl;

  factory _DaemonDeviceInfo.fromJson(Map<String, dynamic> json) =
      _$DaemonDeviceInfoImpl.fromJson;

  @override
  String get path;
  @override
  String get name;
  @override
  @JsonKey(name: 'vendor_id')
  String? get vendorId;
  @override
  @JsonKey(name: 'product_id')
  String? get productId;

  /// Create a copy of DaemonDeviceInfo
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$DaemonDeviceInfoImplCopyWith<_$DaemonDeviceInfoImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

EngineStatus _$EngineStatusFromJson(Map<String, dynamic> json) {
  return _EngineStatus.fromJson(json);
}

/// @nodoc
mixin _$EngineStatus {
  String get status =>
      throw _privateConstructorUsedError; // "running", "stopped", "error"
  @JsonKey(name: 'config_path')
  String? get configPath => throw _privateConstructorUsedError;
  @JsonKey(name: 'script_path')
  String? get scriptPath => throw _privateConstructorUsedError;
  List<DaemonDeviceInfo> get devices => throw _privateConstructorUsedError;
  @JsonKey(name: 'uptime_seconds')
  int? get uptimeSeconds => throw _privateConstructorUsedError;
  @JsonKey(name: 'events_processed')
  int? get eventsProcessed => throw _privateConstructorUsedError;

  /// Serializes this EngineStatus to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of EngineStatus
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $EngineStatusCopyWith<EngineStatus> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $EngineStatusCopyWith<$Res> {
  factory $EngineStatusCopyWith(
    EngineStatus value,
    $Res Function(EngineStatus) then,
  ) = _$EngineStatusCopyWithImpl<$Res, EngineStatus>;
  @useResult
  $Res call({
    String status,
    @JsonKey(name: 'config_path') String? configPath,
    @JsonKey(name: 'script_path') String? scriptPath,
    List<DaemonDeviceInfo> devices,
    @JsonKey(name: 'uptime_seconds') int? uptimeSeconds,
    @JsonKey(name: 'events_processed') int? eventsProcessed,
  });
}

/// @nodoc
class _$EngineStatusCopyWithImpl<$Res, $Val extends EngineStatus>
    implements $EngineStatusCopyWith<$Res> {
  _$EngineStatusCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of EngineStatus
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? status = null,
    Object? configPath = freezed,
    Object? scriptPath = freezed,
    Object? devices = null,
    Object? uptimeSeconds = freezed,
    Object? eventsProcessed = freezed,
  }) {
    return _then(
      _value.copyWith(
            status: null == status
                ? _value.status
                : status // ignore: cast_nullable_to_non_nullable
                      as String,
            configPath: freezed == configPath
                ? _value.configPath
                : configPath // ignore: cast_nullable_to_non_nullable
                      as String?,
            scriptPath: freezed == scriptPath
                ? _value.scriptPath
                : scriptPath // ignore: cast_nullable_to_non_nullable
                      as String?,
            devices: null == devices
                ? _value.devices
                : devices // ignore: cast_nullable_to_non_nullable
                      as List<DaemonDeviceInfo>,
            uptimeSeconds: freezed == uptimeSeconds
                ? _value.uptimeSeconds
                : uptimeSeconds // ignore: cast_nullable_to_non_nullable
                      as int?,
            eventsProcessed: freezed == eventsProcessed
                ? _value.eventsProcessed
                : eventsProcessed // ignore: cast_nullable_to_non_nullable
                      as int?,
          )
          as $Val,
    );
  }
}

/// @nodoc
abstract class _$$EngineStatusImplCopyWith<$Res>
    implements $EngineStatusCopyWith<$Res> {
  factory _$$EngineStatusImplCopyWith(
    _$EngineStatusImpl value,
    $Res Function(_$EngineStatusImpl) then,
  ) = __$$EngineStatusImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({
    String status,
    @JsonKey(name: 'config_path') String? configPath,
    @JsonKey(name: 'script_path') String? scriptPath,
    List<DaemonDeviceInfo> devices,
    @JsonKey(name: 'uptime_seconds') int? uptimeSeconds,
    @JsonKey(name: 'events_processed') int? eventsProcessed,
  });
}

/// @nodoc
class __$$EngineStatusImplCopyWithImpl<$Res>
    extends _$EngineStatusCopyWithImpl<$Res, _$EngineStatusImpl>
    implements _$$EngineStatusImplCopyWith<$Res> {
  __$$EngineStatusImplCopyWithImpl(
    _$EngineStatusImpl _value,
    $Res Function(_$EngineStatusImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of EngineStatus
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? status = null,
    Object? configPath = freezed,
    Object? scriptPath = freezed,
    Object? devices = null,
    Object? uptimeSeconds = freezed,
    Object? eventsProcessed = freezed,
  }) {
    return _then(
      _$EngineStatusImpl(
        status: null == status
            ? _value.status
            : status // ignore: cast_nullable_to_non_nullable
                  as String,
        configPath: freezed == configPath
            ? _value.configPath
            : configPath // ignore: cast_nullable_to_non_nullable
                  as String?,
        scriptPath: freezed == scriptPath
            ? _value.scriptPath
            : scriptPath // ignore: cast_nullable_to_non_nullable
                  as String?,
        devices: null == devices
            ? _value._devices
            : devices // ignore: cast_nullable_to_non_nullable
                  as List<DaemonDeviceInfo>,
        uptimeSeconds: freezed == uptimeSeconds
            ? _value.uptimeSeconds
            : uptimeSeconds // ignore: cast_nullable_to_non_nullable
                  as int?,
        eventsProcessed: freezed == eventsProcessed
            ? _value.eventsProcessed
            : eventsProcessed // ignore: cast_nullable_to_non_nullable
                  as int?,
      ),
    );
  }
}

/// @nodoc
@JsonSerializable()
class _$EngineStatusImpl implements _EngineStatus {
  const _$EngineStatusImpl({
    required this.status,
    @JsonKey(name: 'config_path') this.configPath,
    @JsonKey(name: 'script_path') this.scriptPath,
    final List<DaemonDeviceInfo> devices = const [],
    @JsonKey(name: 'uptime_seconds') this.uptimeSeconds,
    @JsonKey(name: 'events_processed') this.eventsProcessed,
  }) : _devices = devices;

  factory _$EngineStatusImpl.fromJson(Map<String, dynamic> json) =>
      _$$EngineStatusImplFromJson(json);

  @override
  final String status;
  // "running", "stopped", "error"
  @override
  @JsonKey(name: 'config_path')
  final String? configPath;
  @override
  @JsonKey(name: 'script_path')
  final String? scriptPath;
  final List<DaemonDeviceInfo> _devices;
  @override
  @JsonKey()
  List<DaemonDeviceInfo> get devices {
    if (_devices is EqualUnmodifiableListView) return _devices;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_devices);
  }

  @override
  @JsonKey(name: 'uptime_seconds')
  final int? uptimeSeconds;
  @override
  @JsonKey(name: 'events_processed')
  final int? eventsProcessed;

  @override
  String toString() {
    return 'EngineStatus(status: $status, configPath: $configPath, scriptPath: $scriptPath, devices: $devices, uptimeSeconds: $uptimeSeconds, eventsProcessed: $eventsProcessed)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$EngineStatusImpl &&
            (identical(other.status, status) || other.status == status) &&
            (identical(other.configPath, configPath) ||
                other.configPath == configPath) &&
            (identical(other.scriptPath, scriptPath) ||
                other.scriptPath == scriptPath) &&
            const DeepCollectionEquality().equals(other._devices, _devices) &&
            (identical(other.uptimeSeconds, uptimeSeconds) ||
                other.uptimeSeconds == uptimeSeconds) &&
            (identical(other.eventsProcessed, eventsProcessed) ||
                other.eventsProcessed == eventsProcessed));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(
    runtimeType,
    status,
    configPath,
    scriptPath,
    const DeepCollectionEquality().hash(_devices),
    uptimeSeconds,
    eventsProcessed,
  );

  /// Create a copy of EngineStatus
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$EngineStatusImplCopyWith<_$EngineStatusImpl> get copyWith =>
      __$$EngineStatusImplCopyWithImpl<_$EngineStatusImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$EngineStatusImplToJson(this);
  }
}

abstract class _EngineStatus implements EngineStatus {
  const factory _EngineStatus({
    required final String status,
    @JsonKey(name: 'config_path') final String? configPath,
    @JsonKey(name: 'script_path') final String? scriptPath,
    final List<DaemonDeviceInfo> devices,
    @JsonKey(name: 'uptime_seconds') final int? uptimeSeconds,
    @JsonKey(name: 'events_processed') final int? eventsProcessed,
  }) = _$EngineStatusImpl;

  factory _EngineStatus.fromJson(Map<String, dynamic> json) =
      _$EngineStatusImpl.fromJson;

  @override
  String get status; // "running", "stopped", "error"
  @override
  @JsonKey(name: 'config_path')
  String? get configPath;
  @override
  @JsonKey(name: 'script_path')
  String? get scriptPath;
  @override
  List<DaemonDeviceInfo> get devices;
  @override
  @JsonKey(name: 'uptime_seconds')
  int? get uptimeSeconds;
  @override
  @JsonKey(name: 'events_processed')
  int? get eventsProcessed;

  /// Create a copy of EngineStatus
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$EngineStatusImplCopyWith<_$EngineStatusImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

InputEvent _$InputEventFromJson(Map<String, dynamic> json) {
  return _InputEvent.fromJson(json);
}

/// @nodoc
mixin _$InputEvent {
  @JsonKey(name: 'timestamp_us')
  int get timestampUs => throw _privateConstructorUsedError;
  @JsonKey(name: 'device_path')
  String get devicePath => throw _privateConstructorUsedError;
  @JsonKey(name: 'device_name')
  String get deviceName => throw _privateConstructorUsedError;
  @JsonKey(name: 'event_type')
  String get eventType => throw _privateConstructorUsedError; // "key_press", "key_release"
  @JsonKey(name: 'key_code')
  int get keyCode => throw _privateConstructorUsedError;
  @JsonKey(name: 'key_name')
  String get keyName => throw _privateConstructorUsedError;
  List<String> get modifiers => throw _privateConstructorUsedError;

  /// Serializes this InputEvent to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of InputEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $InputEventCopyWith<InputEvent> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $InputEventCopyWith<$Res> {
  factory $InputEventCopyWith(
    InputEvent value,
    $Res Function(InputEvent) then,
  ) = _$InputEventCopyWithImpl<$Res, InputEvent>;
  @useResult
  $Res call({
    @JsonKey(name: 'timestamp_us') int timestampUs,
    @JsonKey(name: 'device_path') String devicePath,
    @JsonKey(name: 'device_name') String deviceName,
    @JsonKey(name: 'event_type') String eventType,
    @JsonKey(name: 'key_code') int keyCode,
    @JsonKey(name: 'key_name') String keyName,
    List<String> modifiers,
  });
}

/// @nodoc
class _$InputEventCopyWithImpl<$Res, $Val extends InputEvent>
    implements $InputEventCopyWith<$Res> {
  _$InputEventCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of InputEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? timestampUs = null,
    Object? devicePath = null,
    Object? deviceName = null,
    Object? eventType = null,
    Object? keyCode = null,
    Object? keyName = null,
    Object? modifiers = null,
  }) {
    return _then(
      _value.copyWith(
            timestampUs: null == timestampUs
                ? _value.timestampUs
                : timestampUs // ignore: cast_nullable_to_non_nullable
                      as int,
            devicePath: null == devicePath
                ? _value.devicePath
                : devicePath // ignore: cast_nullable_to_non_nullable
                      as String,
            deviceName: null == deviceName
                ? _value.deviceName
                : deviceName // ignore: cast_nullable_to_non_nullable
                      as String,
            eventType: null == eventType
                ? _value.eventType
                : eventType // ignore: cast_nullable_to_non_nullable
                      as String,
            keyCode: null == keyCode
                ? _value.keyCode
                : keyCode // ignore: cast_nullable_to_non_nullable
                      as int,
            keyName: null == keyName
                ? _value.keyName
                : keyName // ignore: cast_nullable_to_non_nullable
                      as String,
            modifiers: null == modifiers
                ? _value.modifiers
                : modifiers // ignore: cast_nullable_to_non_nullable
                      as List<String>,
          )
          as $Val,
    );
  }
}

/// @nodoc
abstract class _$$InputEventImplCopyWith<$Res>
    implements $InputEventCopyWith<$Res> {
  factory _$$InputEventImplCopyWith(
    _$InputEventImpl value,
    $Res Function(_$InputEventImpl) then,
  ) = __$$InputEventImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({
    @JsonKey(name: 'timestamp_us') int timestampUs,
    @JsonKey(name: 'device_path') String devicePath,
    @JsonKey(name: 'device_name') String deviceName,
    @JsonKey(name: 'event_type') String eventType,
    @JsonKey(name: 'key_code') int keyCode,
    @JsonKey(name: 'key_name') String keyName,
    List<String> modifiers,
  });
}

/// @nodoc
class __$$InputEventImplCopyWithImpl<$Res>
    extends _$InputEventCopyWithImpl<$Res, _$InputEventImpl>
    implements _$$InputEventImplCopyWith<$Res> {
  __$$InputEventImplCopyWithImpl(
    _$InputEventImpl _value,
    $Res Function(_$InputEventImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of InputEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? timestampUs = null,
    Object? devicePath = null,
    Object? deviceName = null,
    Object? eventType = null,
    Object? keyCode = null,
    Object? keyName = null,
    Object? modifiers = null,
  }) {
    return _then(
      _$InputEventImpl(
        timestampUs: null == timestampUs
            ? _value.timestampUs
            : timestampUs // ignore: cast_nullable_to_non_nullable
                  as int,
        devicePath: null == devicePath
            ? _value.devicePath
            : devicePath // ignore: cast_nullable_to_non_nullable
                  as String,
        deviceName: null == deviceName
            ? _value.deviceName
            : deviceName // ignore: cast_nullable_to_non_nullable
                  as String,
        eventType: null == eventType
            ? _value.eventType
            : eventType // ignore: cast_nullable_to_non_nullable
                  as String,
        keyCode: null == keyCode
            ? _value.keyCode
            : keyCode // ignore: cast_nullable_to_non_nullable
                  as int,
        keyName: null == keyName
            ? _value.keyName
            : keyName // ignore: cast_nullable_to_non_nullable
                  as String,
        modifiers: null == modifiers
            ? _value._modifiers
            : modifiers // ignore: cast_nullable_to_non_nullable
                  as List<String>,
      ),
    );
  }
}

/// @nodoc
@JsonSerializable()
class _$InputEventImpl implements _InputEvent {
  const _$InputEventImpl({
    @JsonKey(name: 'timestamp_us') required this.timestampUs,
    @JsonKey(name: 'device_path') required this.devicePath,
    @JsonKey(name: 'device_name') required this.deviceName,
    @JsonKey(name: 'event_type') required this.eventType,
    @JsonKey(name: 'key_code') required this.keyCode,
    @JsonKey(name: 'key_name') required this.keyName,
    final List<String> modifiers = const [],
  }) : _modifiers = modifiers;

  factory _$InputEventImpl.fromJson(Map<String, dynamic> json) =>
      _$$InputEventImplFromJson(json);

  @override
  @JsonKey(name: 'timestamp_us')
  final int timestampUs;
  @override
  @JsonKey(name: 'device_path')
  final String devicePath;
  @override
  @JsonKey(name: 'device_name')
  final String deviceName;
  @override
  @JsonKey(name: 'event_type')
  final String eventType;
  // "key_press", "key_release"
  @override
  @JsonKey(name: 'key_code')
  final int keyCode;
  @override
  @JsonKey(name: 'key_name')
  final String keyName;
  final List<String> _modifiers;
  @override
  @JsonKey()
  List<String> get modifiers {
    if (_modifiers is EqualUnmodifiableListView) return _modifiers;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_modifiers);
  }

  @override
  String toString() {
    return 'InputEvent(timestampUs: $timestampUs, devicePath: $devicePath, deviceName: $deviceName, eventType: $eventType, keyCode: $keyCode, keyName: $keyName, modifiers: $modifiers)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$InputEventImpl &&
            (identical(other.timestampUs, timestampUs) ||
                other.timestampUs == timestampUs) &&
            (identical(other.devicePath, devicePath) ||
                other.devicePath == devicePath) &&
            (identical(other.deviceName, deviceName) ||
                other.deviceName == deviceName) &&
            (identical(other.eventType, eventType) ||
                other.eventType == eventType) &&
            (identical(other.keyCode, keyCode) || other.keyCode == keyCode) &&
            (identical(other.keyName, keyName) || other.keyName == keyName) &&
            const DeepCollectionEquality().equals(
              other._modifiers,
              _modifiers,
            ));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(
    runtimeType,
    timestampUs,
    devicePath,
    deviceName,
    eventType,
    keyCode,
    keyName,
    const DeepCollectionEquality().hash(_modifiers),
  );

  /// Create a copy of InputEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$InputEventImplCopyWith<_$InputEventImpl> get copyWith =>
      __$$InputEventImplCopyWithImpl<_$InputEventImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$InputEventImplToJson(this);
  }
}

abstract class _InputEvent implements InputEvent {
  const factory _InputEvent({
    @JsonKey(name: 'timestamp_us') required final int timestampUs,
    @JsonKey(name: 'device_path') required final String devicePath,
    @JsonKey(name: 'device_name') required final String deviceName,
    @JsonKey(name: 'event_type') required final String eventType,
    @JsonKey(name: 'key_code') required final int keyCode,
    @JsonKey(name: 'key_name') required final String keyName,
    final List<String> modifiers,
  }) = _$InputEventImpl;

  factory _InputEvent.fromJson(Map<String, dynamic> json) =
      _$InputEventImpl.fromJson;

  @override
  @JsonKey(name: 'timestamp_us')
  int get timestampUs;
  @override
  @JsonKey(name: 'device_path')
  String get devicePath;
  @override
  @JsonKey(name: 'device_name')
  String get deviceName;
  @override
  @JsonKey(name: 'event_type')
  String get eventType; // "key_press", "key_release"
  @override
  @JsonKey(name: 'key_code')
  int get keyCode;
  @override
  @JsonKey(name: 'key_name')
  String get keyName;
  @override
  List<String> get modifiers;

  /// Create a copy of InputEvent
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$InputEventImplCopyWith<_$InputEventImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

OutputEvent _$OutputEventFromJson(Map<String, dynamic> json) {
  return _OutputEvent.fromJson(json);
}

/// @nodoc
mixin _$OutputEvent {
  @JsonKey(name: 'timestamp_us')
  int get timestampUs => throw _privateConstructorUsedError;
  @JsonKey(name: 'action_type')
  String get actionType => throw _privateConstructorUsedError; // "key_press", "key_release", "tap"
  @JsonKey(name: 'key_code')
  int get keyCode => throw _privateConstructorUsedError;
  @JsonKey(name: 'key_name')
  String get keyName => throw _privateConstructorUsedError;

  /// Serializes this OutputEvent to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of OutputEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $OutputEventCopyWith<OutputEvent> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $OutputEventCopyWith<$Res> {
  factory $OutputEventCopyWith(
    OutputEvent value,
    $Res Function(OutputEvent) then,
  ) = _$OutputEventCopyWithImpl<$Res, OutputEvent>;
  @useResult
  $Res call({
    @JsonKey(name: 'timestamp_us') int timestampUs,
    @JsonKey(name: 'action_type') String actionType,
    @JsonKey(name: 'key_code') int keyCode,
    @JsonKey(name: 'key_name') String keyName,
  });
}

/// @nodoc
class _$OutputEventCopyWithImpl<$Res, $Val extends OutputEvent>
    implements $OutputEventCopyWith<$Res> {
  _$OutputEventCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of OutputEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? timestampUs = null,
    Object? actionType = null,
    Object? keyCode = null,
    Object? keyName = null,
  }) {
    return _then(
      _value.copyWith(
            timestampUs: null == timestampUs
                ? _value.timestampUs
                : timestampUs // ignore: cast_nullable_to_non_nullable
                      as int,
            actionType: null == actionType
                ? _value.actionType
                : actionType // ignore: cast_nullable_to_non_nullable
                      as String,
            keyCode: null == keyCode
                ? _value.keyCode
                : keyCode // ignore: cast_nullable_to_non_nullable
                      as int,
            keyName: null == keyName
                ? _value.keyName
                : keyName // ignore: cast_nullable_to_non_nullable
                      as String,
          )
          as $Val,
    );
  }
}

/// @nodoc
abstract class _$$OutputEventImplCopyWith<$Res>
    implements $OutputEventCopyWith<$Res> {
  factory _$$OutputEventImplCopyWith(
    _$OutputEventImpl value,
    $Res Function(_$OutputEventImpl) then,
  ) = __$$OutputEventImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({
    @JsonKey(name: 'timestamp_us') int timestampUs,
    @JsonKey(name: 'action_type') String actionType,
    @JsonKey(name: 'key_code') int keyCode,
    @JsonKey(name: 'key_name') String keyName,
  });
}

/// @nodoc
class __$$OutputEventImplCopyWithImpl<$Res>
    extends _$OutputEventCopyWithImpl<$Res, _$OutputEventImpl>
    implements _$$OutputEventImplCopyWith<$Res> {
  __$$OutputEventImplCopyWithImpl(
    _$OutputEventImpl _value,
    $Res Function(_$OutputEventImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of OutputEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? timestampUs = null,
    Object? actionType = null,
    Object? keyCode = null,
    Object? keyName = null,
  }) {
    return _then(
      _$OutputEventImpl(
        timestampUs: null == timestampUs
            ? _value.timestampUs
            : timestampUs // ignore: cast_nullable_to_non_nullable
                  as int,
        actionType: null == actionType
            ? _value.actionType
            : actionType // ignore: cast_nullable_to_non_nullable
                  as String,
        keyCode: null == keyCode
            ? _value.keyCode
            : keyCode // ignore: cast_nullable_to_non_nullable
                  as int,
        keyName: null == keyName
            ? _value.keyName
            : keyName // ignore: cast_nullable_to_non_nullable
                  as String,
      ),
    );
  }
}

/// @nodoc
@JsonSerializable()
class _$OutputEventImpl implements _OutputEvent {
  const _$OutputEventImpl({
    @JsonKey(name: 'timestamp_us') required this.timestampUs,
    @JsonKey(name: 'action_type') required this.actionType,
    @JsonKey(name: 'key_code') required this.keyCode,
    @JsonKey(name: 'key_name') required this.keyName,
  });

  factory _$OutputEventImpl.fromJson(Map<String, dynamic> json) =>
      _$$OutputEventImplFromJson(json);

  @override
  @JsonKey(name: 'timestamp_us')
  final int timestampUs;
  @override
  @JsonKey(name: 'action_type')
  final String actionType;
  // "key_press", "key_release", "tap"
  @override
  @JsonKey(name: 'key_code')
  final int keyCode;
  @override
  @JsonKey(name: 'key_name')
  final String keyName;

  @override
  String toString() {
    return 'OutputEvent(timestampUs: $timestampUs, actionType: $actionType, keyCode: $keyCode, keyName: $keyName)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$OutputEventImpl &&
            (identical(other.timestampUs, timestampUs) ||
                other.timestampUs == timestampUs) &&
            (identical(other.actionType, actionType) ||
                other.actionType == actionType) &&
            (identical(other.keyCode, keyCode) || other.keyCode == keyCode) &&
            (identical(other.keyName, keyName) || other.keyName == keyName));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode =>
      Object.hash(runtimeType, timestampUs, actionType, keyCode, keyName);

  /// Create a copy of OutputEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$OutputEventImplCopyWith<_$OutputEventImpl> get copyWith =>
      __$$OutputEventImplCopyWithImpl<_$OutputEventImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$OutputEventImplToJson(this);
  }
}

abstract class _OutputEvent implements OutputEvent {
  const factory _OutputEvent({
    @JsonKey(name: 'timestamp_us') required final int timestampUs,
    @JsonKey(name: 'action_type') required final String actionType,
    @JsonKey(name: 'key_code') required final int keyCode,
    @JsonKey(name: 'key_name') required final String keyName,
  }) = _$OutputEventImpl;

  factory _OutputEvent.fromJson(Map<String, dynamic> json) =
      _$OutputEventImpl.fromJson;

  @override
  @JsonKey(name: 'timestamp_us')
  int get timestampUs;
  @override
  @JsonKey(name: 'action_type')
  String get actionType; // "key_press", "key_release", "tap"
  @override
  @JsonKey(name: 'key_code')
  int get keyCode;
  @override
  @JsonKey(name: 'key_name')
  String get keyName;

  /// Create a copy of OutputEvent
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$OutputEventImplCopyWith<_$OutputEventImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

DecisionEvent _$DecisionEventFromJson(Map<String, dynamic> json) {
  return _DecisionEvent.fromJson(json);
}

/// @nodoc
mixin _$DecisionEvent {
  @JsonKey(name: 'timestamp_us')
  int get timestampUs => throw _privateConstructorUsedError;
  @JsonKey(name: 'key_code')
  int get keyCode => throw _privateConstructorUsedError;
  @JsonKey(name: 'key_name')
  String get keyName => throw _privateConstructorUsedError;
  @JsonKey(name: 'decision_type')
  String get decisionType => throw _privateConstructorUsedError;
  List<String> get outputs => throw _privateConstructorUsedError;

  /// Serializes this DecisionEvent to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of DecisionEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $DecisionEventCopyWith<DecisionEvent> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $DecisionEventCopyWith<$Res> {
  factory $DecisionEventCopyWith(
    DecisionEvent value,
    $Res Function(DecisionEvent) then,
  ) = _$DecisionEventCopyWithImpl<$Res, DecisionEvent>;
  @useResult
  $Res call({
    @JsonKey(name: 'timestamp_us') int timestampUs,
    @JsonKey(name: 'key_code') int keyCode,
    @JsonKey(name: 'key_name') String keyName,
    @JsonKey(name: 'decision_type') String decisionType,
    List<String> outputs,
  });
}

/// @nodoc
class _$DecisionEventCopyWithImpl<$Res, $Val extends DecisionEvent>
    implements $DecisionEventCopyWith<$Res> {
  _$DecisionEventCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of DecisionEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? timestampUs = null,
    Object? keyCode = null,
    Object? keyName = null,
    Object? decisionType = null,
    Object? outputs = null,
  }) {
    return _then(
      _value.copyWith(
            timestampUs: null == timestampUs
                ? _value.timestampUs
                : timestampUs // ignore: cast_nullable_to_non_nullable
                      as int,
            keyCode: null == keyCode
                ? _value.keyCode
                : keyCode // ignore: cast_nullable_to_non_nullable
                      as int,
            keyName: null == keyName
                ? _value.keyName
                : keyName // ignore: cast_nullable_to_non_nullable
                      as String,
            decisionType: null == decisionType
                ? _value.decisionType
                : decisionType // ignore: cast_nullable_to_non_nullable
                      as String,
            outputs: null == outputs
                ? _value.outputs
                : outputs // ignore: cast_nullable_to_non_nullable
                      as List<String>,
          )
          as $Val,
    );
  }
}

/// @nodoc
abstract class _$$DecisionEventImplCopyWith<$Res>
    implements $DecisionEventCopyWith<$Res> {
  factory _$$DecisionEventImplCopyWith(
    _$DecisionEventImpl value,
    $Res Function(_$DecisionEventImpl) then,
  ) = __$$DecisionEventImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({
    @JsonKey(name: 'timestamp_us') int timestampUs,
    @JsonKey(name: 'key_code') int keyCode,
    @JsonKey(name: 'key_name') String keyName,
    @JsonKey(name: 'decision_type') String decisionType,
    List<String> outputs,
  });
}

/// @nodoc
class __$$DecisionEventImplCopyWithImpl<$Res>
    extends _$DecisionEventCopyWithImpl<$Res, _$DecisionEventImpl>
    implements _$$DecisionEventImplCopyWith<$Res> {
  __$$DecisionEventImplCopyWithImpl(
    _$DecisionEventImpl _value,
    $Res Function(_$DecisionEventImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of DecisionEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? timestampUs = null,
    Object? keyCode = null,
    Object? keyName = null,
    Object? decisionType = null,
    Object? outputs = null,
  }) {
    return _then(
      _$DecisionEventImpl(
        timestampUs: null == timestampUs
            ? _value.timestampUs
            : timestampUs // ignore: cast_nullable_to_non_nullable
                  as int,
        keyCode: null == keyCode
            ? _value.keyCode
            : keyCode // ignore: cast_nullable_to_non_nullable
                  as int,
        keyName: null == keyName
            ? _value.keyName
            : keyName // ignore: cast_nullable_to_non_nullable
                  as String,
        decisionType: null == decisionType
            ? _value.decisionType
            : decisionType // ignore: cast_nullable_to_non_nullable
                  as String,
        outputs: null == outputs
            ? _value._outputs
            : outputs // ignore: cast_nullable_to_non_nullable
                  as List<String>,
      ),
    );
  }
}

/// @nodoc
@JsonSerializable()
class _$DecisionEventImpl implements _DecisionEvent {
  const _$DecisionEventImpl({
    @JsonKey(name: 'timestamp_us') required this.timestampUs,
    @JsonKey(name: 'key_code') required this.keyCode,
    @JsonKey(name: 'key_name') required this.keyName,
    @JsonKey(name: 'decision_type') required this.decisionType,
    final List<String> outputs = const [],
  }) : _outputs = outputs;

  factory _$DecisionEventImpl.fromJson(Map<String, dynamic> json) =>
      _$$DecisionEventImplFromJson(json);

  @override
  @JsonKey(name: 'timestamp_us')
  final int timestampUs;
  @override
  @JsonKey(name: 'key_code')
  final int keyCode;
  @override
  @JsonKey(name: 'key_name')
  final String keyName;
  @override
  @JsonKey(name: 'decision_type')
  final String decisionType;
  final List<String> _outputs;
  @override
  @JsonKey()
  List<String> get outputs {
    if (_outputs is EqualUnmodifiableListView) return _outputs;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_outputs);
  }

  @override
  String toString() {
    return 'DecisionEvent(timestampUs: $timestampUs, keyCode: $keyCode, keyName: $keyName, decisionType: $decisionType, outputs: $outputs)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$DecisionEventImpl &&
            (identical(other.timestampUs, timestampUs) ||
                other.timestampUs == timestampUs) &&
            (identical(other.keyCode, keyCode) || other.keyCode == keyCode) &&
            (identical(other.keyName, keyName) || other.keyName == keyName) &&
            (identical(other.decisionType, decisionType) ||
                other.decisionType == decisionType) &&
            const DeepCollectionEquality().equals(other._outputs, _outputs));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(
    runtimeType,
    timestampUs,
    keyCode,
    keyName,
    decisionType,
    const DeepCollectionEquality().hash(_outputs),
  );

  /// Create a copy of DecisionEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$DecisionEventImplCopyWith<_$DecisionEventImpl> get copyWith =>
      __$$DecisionEventImplCopyWithImpl<_$DecisionEventImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$DecisionEventImplToJson(this);
  }
}

abstract class _DecisionEvent implements DecisionEvent {
  const factory _DecisionEvent({
    @JsonKey(name: 'timestamp_us') required final int timestampUs,
    @JsonKey(name: 'key_code') required final int keyCode,
    @JsonKey(name: 'key_name') required final String keyName,
    @JsonKey(name: 'decision_type') required final String decisionType,
    final List<String> outputs,
  }) = _$DecisionEventImpl;

  factory _DecisionEvent.fromJson(Map<String, dynamic> json) =
      _$DecisionEventImpl.fromJson;

  @override
  @JsonKey(name: 'timestamp_us')
  int get timestampUs;
  @override
  @JsonKey(name: 'key_code')
  int get keyCode;
  @override
  @JsonKey(name: 'key_name')
  String get keyName;
  @override
  @JsonKey(name: 'decision_type')
  String get decisionType;
  @override
  List<String> get outputs;

  /// Create a copy of DecisionEvent
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$DecisionEventImplCopyWith<_$DecisionEventImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

StateChangeEvent _$StateChangeEventFromJson(Map<String, dynamic> json) {
  return _StateChangeEvent.fromJson(json);
}

/// @nodoc
mixin _$StateChangeEvent {
  String get type =>
      throw _privateConstructorUsedError; // "config_reloaded", "script_reloaded", etc.
  @JsonKey(name: 'timestamp_us')
  int get timestampUs => throw _privateConstructorUsedError;
  String get message => throw _privateConstructorUsedError;
  Map<String, dynamic>? get details => throw _privateConstructorUsedError;

  /// Serializes this StateChangeEvent to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of StateChangeEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $StateChangeEventCopyWith<StateChangeEvent> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $StateChangeEventCopyWith<$Res> {
  factory $StateChangeEventCopyWith(
    StateChangeEvent value,
    $Res Function(StateChangeEvent) then,
  ) = _$StateChangeEventCopyWithImpl<$Res, StateChangeEvent>;
  @useResult
  $Res call({
    String type,
    @JsonKey(name: 'timestamp_us') int timestampUs,
    String message,
    Map<String, dynamic>? details,
  });
}

/// @nodoc
class _$StateChangeEventCopyWithImpl<$Res, $Val extends StateChangeEvent>
    implements $StateChangeEventCopyWith<$Res> {
  _$StateChangeEventCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of StateChangeEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? type = null,
    Object? timestampUs = null,
    Object? message = null,
    Object? details = freezed,
  }) {
    return _then(
      _value.copyWith(
            type: null == type
                ? _value.type
                : type // ignore: cast_nullable_to_non_nullable
                      as String,
            timestampUs: null == timestampUs
                ? _value.timestampUs
                : timestampUs // ignore: cast_nullable_to_non_nullable
                      as int,
            message: null == message
                ? _value.message
                : message // ignore: cast_nullable_to_non_nullable
                      as String,
            details: freezed == details
                ? _value.details
                : details // ignore: cast_nullable_to_non_nullable
                      as Map<String, dynamic>?,
          )
          as $Val,
    );
  }
}

/// @nodoc
abstract class _$$StateChangeEventImplCopyWith<$Res>
    implements $StateChangeEventCopyWith<$Res> {
  factory _$$StateChangeEventImplCopyWith(
    _$StateChangeEventImpl value,
    $Res Function(_$StateChangeEventImpl) then,
  ) = __$$StateChangeEventImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({
    String type,
    @JsonKey(name: 'timestamp_us') int timestampUs,
    String message,
    Map<String, dynamic>? details,
  });
}

/// @nodoc
class __$$StateChangeEventImplCopyWithImpl<$Res>
    extends _$StateChangeEventCopyWithImpl<$Res, _$StateChangeEventImpl>
    implements _$$StateChangeEventImplCopyWith<$Res> {
  __$$StateChangeEventImplCopyWithImpl(
    _$StateChangeEventImpl _value,
    $Res Function(_$StateChangeEventImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of StateChangeEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? type = null,
    Object? timestampUs = null,
    Object? message = null,
    Object? details = freezed,
  }) {
    return _then(
      _$StateChangeEventImpl(
        type: null == type
            ? _value.type
            : type // ignore: cast_nullable_to_non_nullable
                  as String,
        timestampUs: null == timestampUs
            ? _value.timestampUs
            : timestampUs // ignore: cast_nullable_to_non_nullable
                  as int,
        message: null == message
            ? _value.message
            : message // ignore: cast_nullable_to_non_nullable
                  as String,
        details: freezed == details
            ? _value._details
            : details // ignore: cast_nullable_to_non_nullable
                  as Map<String, dynamic>?,
      ),
    );
  }
}

/// @nodoc
@JsonSerializable()
class _$StateChangeEventImpl implements _StateChangeEvent {
  const _$StateChangeEventImpl({
    required this.type,
    @JsonKey(name: 'timestamp_us') required this.timestampUs,
    required this.message,
    final Map<String, dynamic>? details,
  }) : _details = details;

  factory _$StateChangeEventImpl.fromJson(Map<String, dynamic> json) =>
      _$$StateChangeEventImplFromJson(json);

  @override
  final String type;
  // "config_reloaded", "script_reloaded", etc.
  @override
  @JsonKey(name: 'timestamp_us')
  final int timestampUs;
  @override
  final String message;
  final Map<String, dynamic>? _details;
  @override
  Map<String, dynamic>? get details {
    final value = _details;
    if (value == null) return null;
    if (_details is EqualUnmodifiableMapView) return _details;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableMapView(value);
  }

  @override
  String toString() {
    return 'StateChangeEvent(type: $type, timestampUs: $timestampUs, message: $message, details: $details)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$StateChangeEventImpl &&
            (identical(other.type, type) || other.type == type) &&
            (identical(other.timestampUs, timestampUs) ||
                other.timestampUs == timestampUs) &&
            (identical(other.message, message) || other.message == message) &&
            const DeepCollectionEquality().equals(other._details, _details));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(
    runtimeType,
    type,
    timestampUs,
    message,
    const DeepCollectionEquality().hash(_details),
  );

  /// Create a copy of StateChangeEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$StateChangeEventImplCopyWith<_$StateChangeEventImpl> get copyWith =>
      __$$StateChangeEventImplCopyWithImpl<_$StateChangeEventImpl>(
        this,
        _$identity,
      );

  @override
  Map<String, dynamic> toJson() {
    return _$$StateChangeEventImplToJson(this);
  }
}

abstract class _StateChangeEvent implements StateChangeEvent {
  const factory _StateChangeEvent({
    required final String type,
    @JsonKey(name: 'timestamp_us') required final int timestampUs,
    required final String message,
    final Map<String, dynamic>? details,
  }) = _$StateChangeEventImpl;

  factory _StateChangeEvent.fromJson(Map<String, dynamic> json) =
      _$StateChangeEventImpl.fromJson;

  @override
  String get type; // "config_reloaded", "script_reloaded", etc.
  @override
  @JsonKey(name: 'timestamp_us')
  int get timestampUs;
  @override
  String get message;
  @override
  Map<String, dynamic>? get details;

  /// Create a copy of StateChangeEvent
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$StateChangeEventImplCopyWith<_$StateChangeEventImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
mixin _$DaemonEvent {
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(InputEvent event) input,
    required TResult Function(OutputEvent event) output,
    required TResult Function(DecisionEvent event) decision,
    required TResult Function(StateChangeEvent event) stateChange,
    required TResult Function(DiscoveryCandidate candidate) discovery,
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(InputEvent event)? input,
    TResult? Function(OutputEvent event)? output,
    TResult? Function(DecisionEvent event)? decision,
    TResult? Function(StateChangeEvent event)? stateChange,
    TResult? Function(DiscoveryCandidate candidate)? discovery,
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(InputEvent event)? input,
    TResult Function(OutputEvent event)? output,
    TResult Function(DecisionEvent event)? decision,
    TResult Function(StateChangeEvent event)? stateChange,
    TResult Function(DiscoveryCandidate candidate)? discovery,
    required TResult orElse(),
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(_DaemonInputEvent value) input,
    required TResult Function(_DaemonOutputEvent value) output,
    required TResult Function(_DaemonDecisionEvent value) decision,
    required TResult Function(_DaemonStateChangeEvent value) stateChange,
    required TResult Function(_DaemonDiscoveryEvent value) discovery,
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(_DaemonInputEvent value)? input,
    TResult? Function(_DaemonOutputEvent value)? output,
    TResult? Function(_DaemonDecisionEvent value)? decision,
    TResult? Function(_DaemonStateChangeEvent value)? stateChange,
    TResult? Function(_DaemonDiscoveryEvent value)? discovery,
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(_DaemonInputEvent value)? input,
    TResult Function(_DaemonOutputEvent value)? output,
    TResult Function(_DaemonDecisionEvent value)? decision,
    TResult Function(_DaemonStateChangeEvent value)? stateChange,
    TResult Function(_DaemonDiscoveryEvent value)? discovery,
    required TResult orElse(),
  }) => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $DaemonEventCopyWith<$Res> {
  factory $DaemonEventCopyWith(
    DaemonEvent value,
    $Res Function(DaemonEvent) then,
  ) = _$DaemonEventCopyWithImpl<$Res, DaemonEvent>;
}

/// @nodoc
class _$DaemonEventCopyWithImpl<$Res, $Val extends DaemonEvent>
    implements $DaemonEventCopyWith<$Res> {
  _$DaemonEventCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of DaemonEvent
  /// with the given fields replaced by the non-null parameter values.
}

/// @nodoc
abstract class _$$DaemonInputEventImplCopyWith<$Res> {
  factory _$$DaemonInputEventImplCopyWith(
    _$DaemonInputEventImpl value,
    $Res Function(_$DaemonInputEventImpl) then,
  ) = __$$DaemonInputEventImplCopyWithImpl<$Res>;
  @useResult
  $Res call({InputEvent event});

  $InputEventCopyWith<$Res> get event;
}

/// @nodoc
class __$$DaemonInputEventImplCopyWithImpl<$Res>
    extends _$DaemonEventCopyWithImpl<$Res, _$DaemonInputEventImpl>
    implements _$$DaemonInputEventImplCopyWith<$Res> {
  __$$DaemonInputEventImplCopyWithImpl(
    _$DaemonInputEventImpl _value,
    $Res Function(_$DaemonInputEventImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of DaemonEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? event = null}) {
    return _then(
      _$DaemonInputEventImpl(
        null == event
            ? _value.event
            : event // ignore: cast_nullable_to_non_nullable
                  as InputEvent,
      ),
    );
  }

  /// Create a copy of DaemonEvent
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $InputEventCopyWith<$Res> get event {
    return $InputEventCopyWith<$Res>(_value.event, (value) {
      return _then(_value.copyWith(event: value));
    });
  }
}

/// @nodoc

class _$DaemonInputEventImpl implements _DaemonInputEvent {
  const _$DaemonInputEventImpl(this.event);

  @override
  final InputEvent event;

  @override
  String toString() {
    return 'DaemonEvent.input(event: $event)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$DaemonInputEventImpl &&
            (identical(other.event, event) || other.event == event));
  }

  @override
  int get hashCode => Object.hash(runtimeType, event);

  /// Create a copy of DaemonEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$DaemonInputEventImplCopyWith<_$DaemonInputEventImpl> get copyWith =>
      __$$DaemonInputEventImplCopyWithImpl<_$DaemonInputEventImpl>(
        this,
        _$identity,
      );

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(InputEvent event) input,
    required TResult Function(OutputEvent event) output,
    required TResult Function(DecisionEvent event) decision,
    required TResult Function(StateChangeEvent event) stateChange,
    required TResult Function(DiscoveryCandidate candidate) discovery,
  }) {
    return input(event);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(InputEvent event)? input,
    TResult? Function(OutputEvent event)? output,
    TResult? Function(DecisionEvent event)? decision,
    TResult? Function(StateChangeEvent event)? stateChange,
    TResult? Function(DiscoveryCandidate candidate)? discovery,
  }) {
    return input?.call(event);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(InputEvent event)? input,
    TResult Function(OutputEvent event)? output,
    TResult Function(DecisionEvent event)? decision,
    TResult Function(StateChangeEvent event)? stateChange,
    TResult Function(DiscoveryCandidate candidate)? discovery,
    required TResult orElse(),
  }) {
    if (input != null) {
      return input(event);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(_DaemonInputEvent value) input,
    required TResult Function(_DaemonOutputEvent value) output,
    required TResult Function(_DaemonDecisionEvent value) decision,
    required TResult Function(_DaemonStateChangeEvent value) stateChange,
    required TResult Function(_DaemonDiscoveryEvent value) discovery,
  }) {
    return input(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(_DaemonInputEvent value)? input,
    TResult? Function(_DaemonOutputEvent value)? output,
    TResult? Function(_DaemonDecisionEvent value)? decision,
    TResult? Function(_DaemonStateChangeEvent value)? stateChange,
    TResult? Function(_DaemonDiscoveryEvent value)? discovery,
  }) {
    return input?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(_DaemonInputEvent value)? input,
    TResult Function(_DaemonOutputEvent value)? output,
    TResult Function(_DaemonDecisionEvent value)? decision,
    TResult Function(_DaemonStateChangeEvent value)? stateChange,
    TResult Function(_DaemonDiscoveryEvent value)? discovery,
    required TResult orElse(),
  }) {
    if (input != null) {
      return input(this);
    }
    return orElse();
  }
}

abstract class _DaemonInputEvent implements DaemonEvent {
  const factory _DaemonInputEvent(final InputEvent event) =
      _$DaemonInputEventImpl;

  InputEvent get event;

  /// Create a copy of DaemonEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$DaemonInputEventImplCopyWith<_$DaemonInputEventImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$DaemonOutputEventImplCopyWith<$Res> {
  factory _$$DaemonOutputEventImplCopyWith(
    _$DaemonOutputEventImpl value,
    $Res Function(_$DaemonOutputEventImpl) then,
  ) = __$$DaemonOutputEventImplCopyWithImpl<$Res>;
  @useResult
  $Res call({OutputEvent event});

  $OutputEventCopyWith<$Res> get event;
}

/// @nodoc
class __$$DaemonOutputEventImplCopyWithImpl<$Res>
    extends _$DaemonEventCopyWithImpl<$Res, _$DaemonOutputEventImpl>
    implements _$$DaemonOutputEventImplCopyWith<$Res> {
  __$$DaemonOutputEventImplCopyWithImpl(
    _$DaemonOutputEventImpl _value,
    $Res Function(_$DaemonOutputEventImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of DaemonEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? event = null}) {
    return _then(
      _$DaemonOutputEventImpl(
        null == event
            ? _value.event
            : event // ignore: cast_nullable_to_non_nullable
                  as OutputEvent,
      ),
    );
  }

  /// Create a copy of DaemonEvent
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $OutputEventCopyWith<$Res> get event {
    return $OutputEventCopyWith<$Res>(_value.event, (value) {
      return _then(_value.copyWith(event: value));
    });
  }
}

/// @nodoc

class _$DaemonOutputEventImpl implements _DaemonOutputEvent {
  const _$DaemonOutputEventImpl(this.event);

  @override
  final OutputEvent event;

  @override
  String toString() {
    return 'DaemonEvent.output(event: $event)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$DaemonOutputEventImpl &&
            (identical(other.event, event) || other.event == event));
  }

  @override
  int get hashCode => Object.hash(runtimeType, event);

  /// Create a copy of DaemonEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$DaemonOutputEventImplCopyWith<_$DaemonOutputEventImpl> get copyWith =>
      __$$DaemonOutputEventImplCopyWithImpl<_$DaemonOutputEventImpl>(
        this,
        _$identity,
      );

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(InputEvent event) input,
    required TResult Function(OutputEvent event) output,
    required TResult Function(DecisionEvent event) decision,
    required TResult Function(StateChangeEvent event) stateChange,
    required TResult Function(DiscoveryCandidate candidate) discovery,
  }) {
    return output(event);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(InputEvent event)? input,
    TResult? Function(OutputEvent event)? output,
    TResult? Function(DecisionEvent event)? decision,
    TResult? Function(StateChangeEvent event)? stateChange,
    TResult? Function(DiscoveryCandidate candidate)? discovery,
  }) {
    return output?.call(event);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(InputEvent event)? input,
    TResult Function(OutputEvent event)? output,
    TResult Function(DecisionEvent event)? decision,
    TResult Function(StateChangeEvent event)? stateChange,
    TResult Function(DiscoveryCandidate candidate)? discovery,
    required TResult orElse(),
  }) {
    if (output != null) {
      return output(event);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(_DaemonInputEvent value) input,
    required TResult Function(_DaemonOutputEvent value) output,
    required TResult Function(_DaemonDecisionEvent value) decision,
    required TResult Function(_DaemonStateChangeEvent value) stateChange,
    required TResult Function(_DaemonDiscoveryEvent value) discovery,
  }) {
    return output(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(_DaemonInputEvent value)? input,
    TResult? Function(_DaemonOutputEvent value)? output,
    TResult? Function(_DaemonDecisionEvent value)? decision,
    TResult? Function(_DaemonStateChangeEvent value)? stateChange,
    TResult? Function(_DaemonDiscoveryEvent value)? discovery,
  }) {
    return output?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(_DaemonInputEvent value)? input,
    TResult Function(_DaemonOutputEvent value)? output,
    TResult Function(_DaemonDecisionEvent value)? decision,
    TResult Function(_DaemonStateChangeEvent value)? stateChange,
    TResult Function(_DaemonDiscoveryEvent value)? discovery,
    required TResult orElse(),
  }) {
    if (output != null) {
      return output(this);
    }
    return orElse();
  }
}

abstract class _DaemonOutputEvent implements DaemonEvent {
  const factory _DaemonOutputEvent(final OutputEvent event) =
      _$DaemonOutputEventImpl;

  OutputEvent get event;

  /// Create a copy of DaemonEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$DaemonOutputEventImplCopyWith<_$DaemonOutputEventImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$DaemonDecisionEventImplCopyWith<$Res> {
  factory _$$DaemonDecisionEventImplCopyWith(
    _$DaemonDecisionEventImpl value,
    $Res Function(_$DaemonDecisionEventImpl) then,
  ) = __$$DaemonDecisionEventImplCopyWithImpl<$Res>;
  @useResult
  $Res call({DecisionEvent event});

  $DecisionEventCopyWith<$Res> get event;
}

/// @nodoc
class __$$DaemonDecisionEventImplCopyWithImpl<$Res>
    extends _$DaemonEventCopyWithImpl<$Res, _$DaemonDecisionEventImpl>
    implements _$$DaemonDecisionEventImplCopyWith<$Res> {
  __$$DaemonDecisionEventImplCopyWithImpl(
    _$DaemonDecisionEventImpl _value,
    $Res Function(_$DaemonDecisionEventImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of DaemonEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? event = null}) {
    return _then(
      _$DaemonDecisionEventImpl(
        null == event
            ? _value.event
            : event // ignore: cast_nullable_to_non_nullable
                  as DecisionEvent,
      ),
    );
  }

  /// Create a copy of DaemonEvent
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $DecisionEventCopyWith<$Res> get event {
    return $DecisionEventCopyWith<$Res>(_value.event, (value) {
      return _then(_value.copyWith(event: value));
    });
  }
}

/// @nodoc

class _$DaemonDecisionEventImpl implements _DaemonDecisionEvent {
  const _$DaemonDecisionEventImpl(this.event);

  @override
  final DecisionEvent event;

  @override
  String toString() {
    return 'DaemonEvent.decision(event: $event)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$DaemonDecisionEventImpl &&
            (identical(other.event, event) || other.event == event));
  }

  @override
  int get hashCode => Object.hash(runtimeType, event);

  /// Create a copy of DaemonEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$DaemonDecisionEventImplCopyWith<_$DaemonDecisionEventImpl> get copyWith =>
      __$$DaemonDecisionEventImplCopyWithImpl<_$DaemonDecisionEventImpl>(
        this,
        _$identity,
      );

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(InputEvent event) input,
    required TResult Function(OutputEvent event) output,
    required TResult Function(DecisionEvent event) decision,
    required TResult Function(StateChangeEvent event) stateChange,
    required TResult Function(DiscoveryCandidate candidate) discovery,
  }) {
    return decision(event);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(InputEvent event)? input,
    TResult? Function(OutputEvent event)? output,
    TResult? Function(DecisionEvent event)? decision,
    TResult? Function(StateChangeEvent event)? stateChange,
    TResult? Function(DiscoveryCandidate candidate)? discovery,
  }) {
    return decision?.call(event);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(InputEvent event)? input,
    TResult Function(OutputEvent event)? output,
    TResult Function(DecisionEvent event)? decision,
    TResult Function(StateChangeEvent event)? stateChange,
    TResult Function(DiscoveryCandidate candidate)? discovery,
    required TResult orElse(),
  }) {
    if (decision != null) {
      return decision(event);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(_DaemonInputEvent value) input,
    required TResult Function(_DaemonOutputEvent value) output,
    required TResult Function(_DaemonDecisionEvent value) decision,
    required TResult Function(_DaemonStateChangeEvent value) stateChange,
    required TResult Function(_DaemonDiscoveryEvent value) discovery,
  }) {
    return decision(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(_DaemonInputEvent value)? input,
    TResult? Function(_DaemonOutputEvent value)? output,
    TResult? Function(_DaemonDecisionEvent value)? decision,
    TResult? Function(_DaemonStateChangeEvent value)? stateChange,
    TResult? Function(_DaemonDiscoveryEvent value)? discovery,
  }) {
    return decision?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(_DaemonInputEvent value)? input,
    TResult Function(_DaemonOutputEvent value)? output,
    TResult Function(_DaemonDecisionEvent value)? decision,
    TResult Function(_DaemonStateChangeEvent value)? stateChange,
    TResult Function(_DaemonDiscoveryEvent value)? discovery,
    required TResult orElse(),
  }) {
    if (decision != null) {
      return decision(this);
    }
    return orElse();
  }
}

abstract class _DaemonDecisionEvent implements DaemonEvent {
  const factory _DaemonDecisionEvent(final DecisionEvent event) =
      _$DaemonDecisionEventImpl;

  DecisionEvent get event;

  /// Create a copy of DaemonEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$DaemonDecisionEventImplCopyWith<_$DaemonDecisionEventImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$DaemonStateChangeEventImplCopyWith<$Res> {
  factory _$$DaemonStateChangeEventImplCopyWith(
    _$DaemonStateChangeEventImpl value,
    $Res Function(_$DaemonStateChangeEventImpl) then,
  ) = __$$DaemonStateChangeEventImplCopyWithImpl<$Res>;
  @useResult
  $Res call({StateChangeEvent event});

  $StateChangeEventCopyWith<$Res> get event;
}

/// @nodoc
class __$$DaemonStateChangeEventImplCopyWithImpl<$Res>
    extends _$DaemonEventCopyWithImpl<$Res, _$DaemonStateChangeEventImpl>
    implements _$$DaemonStateChangeEventImplCopyWith<$Res> {
  __$$DaemonStateChangeEventImplCopyWithImpl(
    _$DaemonStateChangeEventImpl _value,
    $Res Function(_$DaemonStateChangeEventImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of DaemonEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? event = null}) {
    return _then(
      _$DaemonStateChangeEventImpl(
        null == event
            ? _value.event
            : event // ignore: cast_nullable_to_non_nullable
                  as StateChangeEvent,
      ),
    );
  }

  /// Create a copy of DaemonEvent
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $StateChangeEventCopyWith<$Res> get event {
    return $StateChangeEventCopyWith<$Res>(_value.event, (value) {
      return _then(_value.copyWith(event: value));
    });
  }
}

/// @nodoc

class _$DaemonStateChangeEventImpl implements _DaemonStateChangeEvent {
  const _$DaemonStateChangeEventImpl(this.event);

  @override
  final StateChangeEvent event;

  @override
  String toString() {
    return 'DaemonEvent.stateChange(event: $event)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$DaemonStateChangeEventImpl &&
            (identical(other.event, event) || other.event == event));
  }

  @override
  int get hashCode => Object.hash(runtimeType, event);

  /// Create a copy of DaemonEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$DaemonStateChangeEventImplCopyWith<_$DaemonStateChangeEventImpl>
  get copyWith =>
      __$$DaemonStateChangeEventImplCopyWithImpl<_$DaemonStateChangeEventImpl>(
        this,
        _$identity,
      );

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(InputEvent event) input,
    required TResult Function(OutputEvent event) output,
    required TResult Function(DecisionEvent event) decision,
    required TResult Function(StateChangeEvent event) stateChange,
    required TResult Function(DiscoveryCandidate candidate) discovery,
  }) {
    return stateChange(event);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(InputEvent event)? input,
    TResult? Function(OutputEvent event)? output,
    TResult? Function(DecisionEvent event)? decision,
    TResult? Function(StateChangeEvent event)? stateChange,
    TResult? Function(DiscoveryCandidate candidate)? discovery,
  }) {
    return stateChange?.call(event);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(InputEvent event)? input,
    TResult Function(OutputEvent event)? output,
    TResult Function(DecisionEvent event)? decision,
    TResult Function(StateChangeEvent event)? stateChange,
    TResult Function(DiscoveryCandidate candidate)? discovery,
    required TResult orElse(),
  }) {
    if (stateChange != null) {
      return stateChange(event);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(_DaemonInputEvent value) input,
    required TResult Function(_DaemonOutputEvent value) output,
    required TResult Function(_DaemonDecisionEvent value) decision,
    required TResult Function(_DaemonStateChangeEvent value) stateChange,
    required TResult Function(_DaemonDiscoveryEvent value) discovery,
  }) {
    return stateChange(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(_DaemonInputEvent value)? input,
    TResult? Function(_DaemonOutputEvent value)? output,
    TResult? Function(_DaemonDecisionEvent value)? decision,
    TResult? Function(_DaemonStateChangeEvent value)? stateChange,
    TResult? Function(_DaemonDiscoveryEvent value)? discovery,
  }) {
    return stateChange?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(_DaemonInputEvent value)? input,
    TResult Function(_DaemonOutputEvent value)? output,
    TResult Function(_DaemonDecisionEvent value)? decision,
    TResult Function(_DaemonStateChangeEvent value)? stateChange,
    TResult Function(_DaemonDiscoveryEvent value)? discovery,
    required TResult orElse(),
  }) {
    if (stateChange != null) {
      return stateChange(this);
    }
    return orElse();
  }
}

abstract class _DaemonStateChangeEvent implements DaemonEvent {
  const factory _DaemonStateChangeEvent(final StateChangeEvent event) =
      _$DaemonStateChangeEventImpl;

  StateChangeEvent get event;

  /// Create a copy of DaemonEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$DaemonStateChangeEventImplCopyWith<_$DaemonStateChangeEventImpl>
  get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$DaemonDiscoveryEventImplCopyWith<$Res> {
  factory _$$DaemonDiscoveryEventImplCopyWith(
    _$DaemonDiscoveryEventImpl value,
    $Res Function(_$DaemonDiscoveryEventImpl) then,
  ) = __$$DaemonDiscoveryEventImplCopyWithImpl<$Res>;
  @useResult
  $Res call({DiscoveryCandidate candidate});

  $DiscoveryCandidateCopyWith<$Res> get candidate;
}

/// @nodoc
class __$$DaemonDiscoveryEventImplCopyWithImpl<$Res>
    extends _$DaemonEventCopyWithImpl<$Res, _$DaemonDiscoveryEventImpl>
    implements _$$DaemonDiscoveryEventImplCopyWith<$Res> {
  __$$DaemonDiscoveryEventImplCopyWithImpl(
    _$DaemonDiscoveryEventImpl _value,
    $Res Function(_$DaemonDiscoveryEventImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of DaemonEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? candidate = null}) {
    return _then(
      _$DaemonDiscoveryEventImpl(
        null == candidate
            ? _value.candidate
            : candidate // ignore: cast_nullable_to_non_nullable
                  as DiscoveryCandidate,
      ),
    );
  }

  /// Create a copy of DaemonEvent
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $DiscoveryCandidateCopyWith<$Res> get candidate {
    return $DiscoveryCandidateCopyWith<$Res>(_value.candidate, (value) {
      return _then(_value.copyWith(candidate: value));
    });
  }
}

/// @nodoc

class _$DaemonDiscoveryEventImpl implements _DaemonDiscoveryEvent {
  const _$DaemonDiscoveryEventImpl(this.candidate);

  @override
  final DiscoveryCandidate candidate;

  @override
  String toString() {
    return 'DaemonEvent.discovery(candidate: $candidate)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$DaemonDiscoveryEventImpl &&
            (identical(other.candidate, candidate) ||
                other.candidate == candidate));
  }

  @override
  int get hashCode => Object.hash(runtimeType, candidate);

  /// Create a copy of DaemonEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$DaemonDiscoveryEventImplCopyWith<_$DaemonDiscoveryEventImpl>
  get copyWith =>
      __$$DaemonDiscoveryEventImplCopyWithImpl<_$DaemonDiscoveryEventImpl>(
        this,
        _$identity,
      );

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(InputEvent event) input,
    required TResult Function(OutputEvent event) output,
    required TResult Function(DecisionEvent event) decision,
    required TResult Function(StateChangeEvent event) stateChange,
    required TResult Function(DiscoveryCandidate candidate) discovery,
  }) {
    return discovery(candidate);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(InputEvent event)? input,
    TResult? Function(OutputEvent event)? output,
    TResult? Function(DecisionEvent event)? decision,
    TResult? Function(StateChangeEvent event)? stateChange,
    TResult? Function(DiscoveryCandidate candidate)? discovery,
  }) {
    return discovery?.call(candidate);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(InputEvent event)? input,
    TResult Function(OutputEvent event)? output,
    TResult Function(DecisionEvent event)? decision,
    TResult Function(StateChangeEvent event)? stateChange,
    TResult Function(DiscoveryCandidate candidate)? discovery,
    required TResult orElse(),
  }) {
    if (discovery != null) {
      return discovery(candidate);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(_DaemonInputEvent value) input,
    required TResult Function(_DaemonOutputEvent value) output,
    required TResult Function(_DaemonDecisionEvent value) decision,
    required TResult Function(_DaemonStateChangeEvent value) stateChange,
    required TResult Function(_DaemonDiscoveryEvent value) discovery,
  }) {
    return discovery(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(_DaemonInputEvent value)? input,
    TResult? Function(_DaemonOutputEvent value)? output,
    TResult? Function(_DaemonDecisionEvent value)? decision,
    TResult? Function(_DaemonStateChangeEvent value)? stateChange,
    TResult? Function(_DaemonDiscoveryEvent value)? discovery,
  }) {
    return discovery?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(_DaemonInputEvent value)? input,
    TResult Function(_DaemonOutputEvent value)? output,
    TResult Function(_DaemonDecisionEvent value)? decision,
    TResult Function(_DaemonStateChangeEvent value)? stateChange,
    TResult Function(_DaemonDiscoveryEvent value)? discovery,
    required TResult orElse(),
  }) {
    if (discovery != null) {
      return discovery(this);
    }
    return orElse();
  }
}

abstract class _DaemonDiscoveryEvent implements DaemonEvent {
  const factory _DaemonDiscoveryEvent(final DiscoveryCandidate candidate) =
      _$DaemonDiscoveryEventImpl;

  DiscoveryCandidate get candidate;

  /// Create a copy of DaemonEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$DaemonDiscoveryEventImplCopyWith<_$DaemonDiscoveryEventImpl>
  get copyWith => throw _privateConstructorUsedError;
}

DiscoveryCandidate _$DiscoveryCandidateFromJson(Map<String, dynamic> json) {
  return _DiscoveryCandidate.fromJson(json);
}

/// @nodoc
mixin _$DiscoveryCandidate {
  String get deviceId => throw _privateConstructorUsedError;
  int get vendorId => throw _privateConstructorUsedError;
  int get productId => throw _privateConstructorUsedError;
  String get name => throw _privateConstructorUsedError;

  /// Serializes this DiscoveryCandidate to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of DiscoveryCandidate
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $DiscoveryCandidateCopyWith<DiscoveryCandidate> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $DiscoveryCandidateCopyWith<$Res> {
  factory $DiscoveryCandidateCopyWith(
    DiscoveryCandidate value,
    $Res Function(DiscoveryCandidate) then,
  ) = _$DiscoveryCandidateCopyWithImpl<$Res, DiscoveryCandidate>;
  @useResult
  $Res call({String deviceId, int vendorId, int productId, String name});
}

/// @nodoc
class _$DiscoveryCandidateCopyWithImpl<$Res, $Val extends DiscoveryCandidate>
    implements $DiscoveryCandidateCopyWith<$Res> {
  _$DiscoveryCandidateCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of DiscoveryCandidate
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? deviceId = null,
    Object? vendorId = null,
    Object? productId = null,
    Object? name = null,
  }) {
    return _then(
      _value.copyWith(
            deviceId: null == deviceId
                ? _value.deviceId
                : deviceId // ignore: cast_nullable_to_non_nullable
                      as String,
            vendorId: null == vendorId
                ? _value.vendorId
                : vendorId // ignore: cast_nullable_to_non_nullable
                      as int,
            productId: null == productId
                ? _value.productId
                : productId // ignore: cast_nullable_to_non_nullable
                      as int,
            name: null == name
                ? _value.name
                : name // ignore: cast_nullable_to_non_nullable
                      as String,
          )
          as $Val,
    );
  }
}

/// @nodoc
abstract class _$$DiscoveryCandidateImplCopyWith<$Res>
    implements $DiscoveryCandidateCopyWith<$Res> {
  factory _$$DiscoveryCandidateImplCopyWith(
    _$DiscoveryCandidateImpl value,
    $Res Function(_$DiscoveryCandidateImpl) then,
  ) = __$$DiscoveryCandidateImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({String deviceId, int vendorId, int productId, String name});
}

/// @nodoc
class __$$DiscoveryCandidateImplCopyWithImpl<$Res>
    extends _$DiscoveryCandidateCopyWithImpl<$Res, _$DiscoveryCandidateImpl>
    implements _$$DiscoveryCandidateImplCopyWith<$Res> {
  __$$DiscoveryCandidateImplCopyWithImpl(
    _$DiscoveryCandidateImpl _value,
    $Res Function(_$DiscoveryCandidateImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of DiscoveryCandidate
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? deviceId = null,
    Object? vendorId = null,
    Object? productId = null,
    Object? name = null,
  }) {
    return _then(
      _$DiscoveryCandidateImpl(
        deviceId: null == deviceId
            ? _value.deviceId
            : deviceId // ignore: cast_nullable_to_non_nullable
                  as String,
        vendorId: null == vendorId
            ? _value.vendorId
            : vendorId // ignore: cast_nullable_to_non_nullable
                  as int,
        productId: null == productId
            ? _value.productId
            : productId // ignore: cast_nullable_to_non_nullable
                  as int,
        name: null == name
            ? _value.name
            : name // ignore: cast_nullable_to_non_nullable
                  as String,
      ),
    );
  }
}

/// @nodoc
@JsonSerializable()
class _$DiscoveryCandidateImpl implements _DiscoveryCandidate {
  const _$DiscoveryCandidateImpl({
    required this.deviceId,
    required this.vendorId,
    required this.productId,
    required this.name,
  });

  factory _$DiscoveryCandidateImpl.fromJson(Map<String, dynamic> json) =>
      _$$DiscoveryCandidateImplFromJson(json);

  @override
  final String deviceId;
  @override
  final int vendorId;
  @override
  final int productId;
  @override
  final String name;

  @override
  String toString() {
    return 'DiscoveryCandidate(deviceId: $deviceId, vendorId: $vendorId, productId: $productId, name: $name)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$DiscoveryCandidateImpl &&
            (identical(other.deviceId, deviceId) ||
                other.deviceId == deviceId) &&
            (identical(other.vendorId, vendorId) ||
                other.vendorId == vendorId) &&
            (identical(other.productId, productId) ||
                other.productId == productId) &&
            (identical(other.name, name) || other.name == name));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode =>
      Object.hash(runtimeType, deviceId, vendorId, productId, name);

  /// Create a copy of DiscoveryCandidate
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$DiscoveryCandidateImplCopyWith<_$DiscoveryCandidateImpl> get copyWith =>
      __$$DiscoveryCandidateImplCopyWithImpl<_$DiscoveryCandidateImpl>(
        this,
        _$identity,
      );

  @override
  Map<String, dynamic> toJson() {
    return _$$DiscoveryCandidateImplToJson(this);
  }
}

abstract class _DiscoveryCandidate implements DiscoveryCandidate {
  const factory _DiscoveryCandidate({
    required final String deviceId,
    required final int vendorId,
    required final int productId,
    required final String name,
  }) = _$DiscoveryCandidateImpl;

  factory _DiscoveryCandidate.fromJson(Map<String, dynamic> json) =
      _$DiscoveryCandidateImpl.fromJson;

  @override
  String get deviceId;
  @override
  int get vendorId;
  @override
  int get productId;
  @override
  String get name;

  /// Create a copy of DiscoveryCandidate
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$DiscoveryCandidateImplCopyWith<_$DiscoveryCandidateImpl> get copyWith =>
      throw _privateConstructorUsedError;
}
