// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'result.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
  'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models',
);

/// @nodoc
mixin _$Result<T> {
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(T value) ok,
    required TResult Function(FacadeError error) err,
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(T value)? ok,
    TResult? Function(FacadeError error)? err,
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(T value)? ok,
    TResult Function(FacadeError error)? err,
    required TResult orElse(),
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(Ok<T> value) ok,
    required TResult Function(Err<T> value) err,
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(Ok<T> value)? ok,
    TResult? Function(Err<T> value)? err,
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(Ok<T> value)? ok,
    TResult Function(Err<T> value)? err,
    required TResult orElse(),
  }) => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $ResultCopyWith<T, $Res> {
  factory $ResultCopyWith(Result<T> value, $Res Function(Result<T>) then) =
      _$ResultCopyWithImpl<T, $Res, Result<T>>;
}

/// @nodoc
class _$ResultCopyWithImpl<T, $Res, $Val extends Result<T>>
    implements $ResultCopyWith<T, $Res> {
  _$ResultCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of Result
  /// with the given fields replaced by the non-null parameter values.
}

/// @nodoc
abstract class _$$OkImplCopyWith<T, $Res> {
  factory _$$OkImplCopyWith(
    _$OkImpl<T> value,
    $Res Function(_$OkImpl<T>) then,
  ) = __$$OkImplCopyWithImpl<T, $Res>;
  @useResult
  $Res call({T value});
}

/// @nodoc
class __$$OkImplCopyWithImpl<T, $Res>
    extends _$ResultCopyWithImpl<T, $Res, _$OkImpl<T>>
    implements _$$OkImplCopyWith<T, $Res> {
  __$$OkImplCopyWithImpl(_$OkImpl<T> _value, $Res Function(_$OkImpl<T>) _then)
    : super(_value, _then);

  /// Create a copy of Result
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? value = freezed}) {
    return _then(
      _$OkImpl<T>(
        freezed == value
            ? _value.value
            : value // ignore: cast_nullable_to_non_nullable
                  as T,
      ),
    );
  }
}

/// @nodoc

class _$OkImpl<T> extends Ok<T> {
  const _$OkImpl(this.value) : super._();

  @override
  final T value;

  @override
  String toString() {
    return 'Result<$T>.ok(value: $value)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$OkImpl<T> &&
            const DeepCollectionEquality().equals(other.value, value));
  }

  @override
  int get hashCode =>
      Object.hash(runtimeType, const DeepCollectionEquality().hash(value));

  /// Create a copy of Result
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$OkImplCopyWith<T, _$OkImpl<T>> get copyWith =>
      __$$OkImplCopyWithImpl<T, _$OkImpl<T>>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(T value) ok,
    required TResult Function(FacadeError error) err,
  }) {
    return ok(value);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(T value)? ok,
    TResult? Function(FacadeError error)? err,
  }) {
    return ok?.call(value);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(T value)? ok,
    TResult Function(FacadeError error)? err,
    required TResult orElse(),
  }) {
    if (ok != null) {
      return ok(value);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(Ok<T> value) ok,
    required TResult Function(Err<T> value) err,
  }) {
    return ok(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(Ok<T> value)? ok,
    TResult? Function(Err<T> value)? err,
  }) {
    return ok?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(Ok<T> value)? ok,
    TResult Function(Err<T> value)? err,
    required TResult orElse(),
  }) {
    if (ok != null) {
      return ok(this);
    }
    return orElse();
  }
}

abstract class Ok<T> extends Result<T> {
  const factory Ok(final T value) = _$OkImpl<T>;
  const Ok._() : super._();

  T get value;

  /// Create a copy of Result
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$OkImplCopyWith<T, _$OkImpl<T>> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$ErrImplCopyWith<T, $Res> {
  factory _$$ErrImplCopyWith(
    _$ErrImpl<T> value,
    $Res Function(_$ErrImpl<T>) then,
  ) = __$$ErrImplCopyWithImpl<T, $Res>;
  @useResult
  $Res call({FacadeError error});

  $FacadeErrorCopyWith<$Res> get error;
}

/// @nodoc
class __$$ErrImplCopyWithImpl<T, $Res>
    extends _$ResultCopyWithImpl<T, $Res, _$ErrImpl<T>>
    implements _$$ErrImplCopyWith<T, $Res> {
  __$$ErrImplCopyWithImpl(
    _$ErrImpl<T> _value,
    $Res Function(_$ErrImpl<T>) _then,
  ) : super(_value, _then);

  /// Create a copy of Result
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? error = null}) {
    return _then(
      _$ErrImpl<T>(
        null == error
            ? _value.error
            : error // ignore: cast_nullable_to_non_nullable
                  as FacadeError,
      ),
    );
  }

  /// Create a copy of Result
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $FacadeErrorCopyWith<$Res> get error {
    return $FacadeErrorCopyWith<$Res>(_value.error, (value) {
      return _then(_value.copyWith(error: value));
    });
  }
}

/// @nodoc

class _$ErrImpl<T> extends Err<T> {
  const _$ErrImpl(this.error) : super._();

  @override
  final FacadeError error;

  @override
  String toString() {
    return 'Result<$T>.err(error: $error)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ErrImpl<T> &&
            (identical(other.error, error) || other.error == error));
  }

  @override
  int get hashCode => Object.hash(runtimeType, error);

  /// Create a copy of Result
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$ErrImplCopyWith<T, _$ErrImpl<T>> get copyWith =>
      __$$ErrImplCopyWithImpl<T, _$ErrImpl<T>>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(T value) ok,
    required TResult Function(FacadeError error) err,
  }) {
    return err(error);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(T value)? ok,
    TResult? Function(FacadeError error)? err,
  }) {
    return err?.call(error);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(T value)? ok,
    TResult Function(FacadeError error)? err,
    required TResult orElse(),
  }) {
    if (err != null) {
      return err(error);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(Ok<T> value) ok,
    required TResult Function(Err<T> value) err,
  }) {
    return err(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(Ok<T> value)? ok,
    TResult? Function(Err<T> value)? err,
  }) {
    return err?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(Ok<T> value)? ok,
    TResult Function(Err<T> value)? err,
    required TResult orElse(),
  }) {
    if (err != null) {
      return err(this);
    }
    return orElse();
  }
}

abstract class Err<T> extends Result<T> {
  const factory Err(final FacadeError error) = _$ErrImpl<T>;
  const Err._() : super._();

  FacadeError get error;

  /// Create a copy of Result
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ErrImplCopyWith<T, _$ErrImpl<T>> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
mixin _$FacadeError {
  String get code => throw _privateConstructorUsedError;
  String get message => throw _privateConstructorUsedError;
  String get userMessage => throw _privateConstructorUsedError;
  Object? get originalError => throw _privateConstructorUsedError;

  /// Create a copy of FacadeError
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $FacadeErrorCopyWith<FacadeError> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $FacadeErrorCopyWith<$Res> {
  factory $FacadeErrorCopyWith(
    FacadeError value,
    $Res Function(FacadeError) then,
  ) = _$FacadeErrorCopyWithImpl<$Res, FacadeError>;
  @useResult
  $Res call({
    String code,
    String message,
    String userMessage,
    Object? originalError,
  });
}

/// @nodoc
class _$FacadeErrorCopyWithImpl<$Res, $Val extends FacadeError>
    implements $FacadeErrorCopyWith<$Res> {
  _$FacadeErrorCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of FacadeError
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? code = null,
    Object? message = null,
    Object? userMessage = null,
    Object? originalError = freezed,
  }) {
    return _then(
      _value.copyWith(
            code: null == code
                ? _value.code
                : code // ignore: cast_nullable_to_non_nullable
                      as String,
            message: null == message
                ? _value.message
                : message // ignore: cast_nullable_to_non_nullable
                      as String,
            userMessage: null == userMessage
                ? _value.userMessage
                : userMessage // ignore: cast_nullable_to_non_nullable
                      as String,
            originalError: freezed == originalError
                ? _value.originalError
                : originalError,
          )
          as $Val,
    );
  }
}

/// @nodoc
abstract class _$$FacadeErrorImplCopyWith<$Res>
    implements $FacadeErrorCopyWith<$Res> {
  factory _$$FacadeErrorImplCopyWith(
    _$FacadeErrorImpl value,
    $Res Function(_$FacadeErrorImpl) then,
  ) = __$$FacadeErrorImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({
    String code,
    String message,
    String userMessage,
    Object? originalError,
  });
}

/// @nodoc
class __$$FacadeErrorImplCopyWithImpl<$Res>
    extends _$FacadeErrorCopyWithImpl<$Res, _$FacadeErrorImpl>
    implements _$$FacadeErrorImplCopyWith<$Res> {
  __$$FacadeErrorImplCopyWithImpl(
    _$FacadeErrorImpl _value,
    $Res Function(_$FacadeErrorImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of FacadeError
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? code = null,
    Object? message = null,
    Object? userMessage = null,
    Object? originalError = freezed,
  }) {
    return _then(
      _$FacadeErrorImpl(
        code: null == code
            ? _value.code
            : code // ignore: cast_nullable_to_non_nullable
                  as String,
        message: null == message
            ? _value.message
            : message // ignore: cast_nullable_to_non_nullable
                  as String,
        userMessage: null == userMessage
            ? _value.userMessage
            : userMessage // ignore: cast_nullable_to_non_nullable
                  as String,
        originalError: freezed == originalError
            ? _value.originalError
            : originalError,
      ),
    );
  }
}

/// @nodoc

class _$FacadeErrorImpl extends _FacadeError {
  const _$FacadeErrorImpl({
    required this.code,
    required this.message,
    required this.userMessage,
    this.originalError,
  }) : super._();

  @override
  final String code;
  @override
  final String message;
  @override
  final String userMessage;
  @override
  final Object? originalError;

  @override
  String toString() {
    return 'FacadeError(code: $code, message: $message, userMessage: $userMessage, originalError: $originalError)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$FacadeErrorImpl &&
            (identical(other.code, code) || other.code == code) &&
            (identical(other.message, message) || other.message == message) &&
            (identical(other.userMessage, userMessage) ||
                other.userMessage == userMessage) &&
            const DeepCollectionEquality().equals(
              other.originalError,
              originalError,
            ));
  }

  @override
  int get hashCode => Object.hash(
    runtimeType,
    code,
    message,
    userMessage,
    const DeepCollectionEquality().hash(originalError),
  );

  /// Create a copy of FacadeError
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$FacadeErrorImplCopyWith<_$FacadeErrorImpl> get copyWith =>
      __$$FacadeErrorImplCopyWithImpl<_$FacadeErrorImpl>(this, _$identity);
}

abstract class _FacadeError extends FacadeError {
  const factory _FacadeError({
    required final String code,
    required final String message,
    required final String userMessage,
    final Object? originalError,
  }) = _$FacadeErrorImpl;
  const _FacadeError._() : super._();

  @override
  String get code;
  @override
  String get message;
  @override
  String get userMessage;
  @override
  Object? get originalError;

  /// Create a copy of FacadeError
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$FacadeErrorImplCopyWith<_$FacadeErrorImpl> get copyWith =>
      throw _privateConstructorUsedError;
}
