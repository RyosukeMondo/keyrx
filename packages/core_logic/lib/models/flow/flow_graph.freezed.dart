// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'flow_graph.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
  'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models',
);

FlowPosition _$FlowPositionFromJson(Map<String, dynamic> json) {
  return _FlowPosition.fromJson(json);
}

/// @nodoc
mixin _$FlowPosition {
  double get x => throw _privateConstructorUsedError;
  double get y => throw _privateConstructorUsedError;

  /// Serializes this FlowPosition to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of FlowPosition
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $FlowPositionCopyWith<FlowPosition> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $FlowPositionCopyWith<$Res> {
  factory $FlowPositionCopyWith(
    FlowPosition value,
    $Res Function(FlowPosition) then,
  ) = _$FlowPositionCopyWithImpl<$Res, FlowPosition>;
  @useResult
  $Res call({double x, double y});
}

/// @nodoc
class _$FlowPositionCopyWithImpl<$Res, $Val extends FlowPosition>
    implements $FlowPositionCopyWith<$Res> {
  _$FlowPositionCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of FlowPosition
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? x = null, Object? y = null}) {
    return _then(
      _value.copyWith(
            x: null == x
                ? _value.x
                : x // ignore: cast_nullable_to_non_nullable
                      as double,
            y: null == y
                ? _value.y
                : y // ignore: cast_nullable_to_non_nullable
                      as double,
          )
          as $Val,
    );
  }
}

/// @nodoc
abstract class _$$FlowPositionImplCopyWith<$Res>
    implements $FlowPositionCopyWith<$Res> {
  factory _$$FlowPositionImplCopyWith(
    _$FlowPositionImpl value,
    $Res Function(_$FlowPositionImpl) then,
  ) = __$$FlowPositionImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({double x, double y});
}

/// @nodoc
class __$$FlowPositionImplCopyWithImpl<$Res>
    extends _$FlowPositionCopyWithImpl<$Res, _$FlowPositionImpl>
    implements _$$FlowPositionImplCopyWith<$Res> {
  __$$FlowPositionImplCopyWithImpl(
    _$FlowPositionImpl _value,
    $Res Function(_$FlowPositionImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of FlowPosition
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? x = null, Object? y = null}) {
    return _then(
      _$FlowPositionImpl(
        x: null == x
            ? _value.x
            : x // ignore: cast_nullable_to_non_nullable
                  as double,
        y: null == y
            ? _value.y
            : y // ignore: cast_nullable_to_non_nullable
                  as double,
      ),
    );
  }
}

/// @nodoc
@JsonSerializable()
class _$FlowPositionImpl implements _FlowPosition {
  const _$FlowPositionImpl({required this.x, required this.y});

  factory _$FlowPositionImpl.fromJson(Map<String, dynamic> json) =>
      _$$FlowPositionImplFromJson(json);

  @override
  final double x;
  @override
  final double y;

  @override
  String toString() {
    return 'FlowPosition(x: $x, y: $y)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$FlowPositionImpl &&
            (identical(other.x, x) || other.x == x) &&
            (identical(other.y, y) || other.y == y));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, x, y);

  /// Create a copy of FlowPosition
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$FlowPositionImplCopyWith<_$FlowPositionImpl> get copyWith =>
      __$$FlowPositionImplCopyWithImpl<_$FlowPositionImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$FlowPositionImplToJson(this);
  }
}

abstract class _FlowPosition implements FlowPosition {
  const factory _FlowPosition({
    required final double x,
    required final double y,
  }) = _$FlowPositionImpl;

  factory _FlowPosition.fromJson(Map<String, dynamic> json) =
      _$FlowPositionImpl.fromJson;

  @override
  double get x;
  @override
  double get y;

  /// Create a copy of FlowPosition
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$FlowPositionImplCopyWith<_$FlowPositionImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

FlowEdge _$FlowEdgeFromJson(Map<String, dynamic> json) {
  return _FlowEdge.fromJson(json);
}

/// @nodoc
mixin _$FlowEdge {
  String get from => throw _privateConstructorUsedError;
  String get to => throw _privateConstructorUsedError;
  String get slotId => throw _privateConstructorUsedError;
  int get priority => throw _privateConstructorUsedError;
  bool get active => throw _privateConstructorUsedError;
  EdgeStatus get status => throw _privateConstructorUsedError;

  /// Serializes this FlowEdge to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of FlowEdge
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $FlowEdgeCopyWith<FlowEdge> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $FlowEdgeCopyWith<$Res> {
  factory $FlowEdgeCopyWith(FlowEdge value, $Res Function(FlowEdge) then) =
      _$FlowEdgeCopyWithImpl<$Res, FlowEdge>;
  @useResult
  $Res call({
    String from,
    String to,
    String slotId,
    int priority,
    bool active,
    EdgeStatus status,
  });
}

/// @nodoc
class _$FlowEdgeCopyWithImpl<$Res, $Val extends FlowEdge>
    implements $FlowEdgeCopyWith<$Res> {
  _$FlowEdgeCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of FlowEdge
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? from = null,
    Object? to = null,
    Object? slotId = null,
    Object? priority = null,
    Object? active = null,
    Object? status = null,
  }) {
    return _then(
      _value.copyWith(
            from: null == from
                ? _value.from
                : from // ignore: cast_nullable_to_non_nullable
                      as String,
            to: null == to
                ? _value.to
                : to // ignore: cast_nullable_to_non_nullable
                      as String,
            slotId: null == slotId
                ? _value.slotId
                : slotId // ignore: cast_nullable_to_non_nullable
                      as String,
            priority: null == priority
                ? _value.priority
                : priority // ignore: cast_nullable_to_non_nullable
                      as int,
            active: null == active
                ? _value.active
                : active // ignore: cast_nullable_to_non_nullable
                      as bool,
            status: null == status
                ? _value.status
                : status // ignore: cast_nullable_to_non_nullable
                      as EdgeStatus,
          )
          as $Val,
    );
  }
}

/// @nodoc
abstract class _$$FlowEdgeImplCopyWith<$Res>
    implements $FlowEdgeCopyWith<$Res> {
  factory _$$FlowEdgeImplCopyWith(
    _$FlowEdgeImpl value,
    $Res Function(_$FlowEdgeImpl) then,
  ) = __$$FlowEdgeImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({
    String from,
    String to,
    String slotId,
    int priority,
    bool active,
    EdgeStatus status,
  });
}

/// @nodoc
class __$$FlowEdgeImplCopyWithImpl<$Res>
    extends _$FlowEdgeCopyWithImpl<$Res, _$FlowEdgeImpl>
    implements _$$FlowEdgeImplCopyWith<$Res> {
  __$$FlowEdgeImplCopyWithImpl(
    _$FlowEdgeImpl _value,
    $Res Function(_$FlowEdgeImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of FlowEdge
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? from = null,
    Object? to = null,
    Object? slotId = null,
    Object? priority = null,
    Object? active = null,
    Object? status = null,
  }) {
    return _then(
      _$FlowEdgeImpl(
        from: null == from
            ? _value.from
            : from // ignore: cast_nullable_to_non_nullable
                  as String,
        to: null == to
            ? _value.to
            : to // ignore: cast_nullable_to_non_nullable
                  as String,
        slotId: null == slotId
            ? _value.slotId
            : slotId // ignore: cast_nullable_to_non_nullable
                  as String,
        priority: null == priority
            ? _value.priority
            : priority // ignore: cast_nullable_to_non_nullable
                  as int,
        active: null == active
            ? _value.active
            : active // ignore: cast_nullable_to_non_nullable
                  as bool,
        status: null == status
            ? _value.status
            : status // ignore: cast_nullable_to_non_nullable
                  as EdgeStatus,
      ),
    );
  }
}

/// @nodoc
@JsonSerializable()
class _$FlowEdgeImpl implements _FlowEdge {
  const _$FlowEdgeImpl({
    required this.from,
    required this.to,
    required this.slotId,
    required this.priority,
    this.active = false,
    this.status = EdgeStatus.valid,
  });

  factory _$FlowEdgeImpl.fromJson(Map<String, dynamic> json) =>
      _$$FlowEdgeImplFromJson(json);

  @override
  final String from;
  @override
  final String to;
  @override
  final String slotId;
  @override
  final int priority;
  @override
  @JsonKey()
  final bool active;
  @override
  @JsonKey()
  final EdgeStatus status;

  @override
  String toString() {
    return 'FlowEdge(from: $from, to: $to, slotId: $slotId, priority: $priority, active: $active, status: $status)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$FlowEdgeImpl &&
            (identical(other.from, from) || other.from == from) &&
            (identical(other.to, to) || other.to == to) &&
            (identical(other.slotId, slotId) || other.slotId == slotId) &&
            (identical(other.priority, priority) ||
                other.priority == priority) &&
            (identical(other.active, active) || other.active == active) &&
            (identical(other.status, status) || other.status == status));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode =>
      Object.hash(runtimeType, from, to, slotId, priority, active, status);

  /// Create a copy of FlowEdge
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$FlowEdgeImplCopyWith<_$FlowEdgeImpl> get copyWith =>
      __$$FlowEdgeImplCopyWithImpl<_$FlowEdgeImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$FlowEdgeImplToJson(this);
  }
}

abstract class _FlowEdge implements FlowEdge {
  const factory _FlowEdge({
    required final String from,
    required final String to,
    required final String slotId,
    required final int priority,
    final bool active,
    final EdgeStatus status,
  }) = _$FlowEdgeImpl;

  factory _FlowEdge.fromJson(Map<String, dynamic> json) =
      _$FlowEdgeImpl.fromJson;

  @override
  String get from;
  @override
  String get to;
  @override
  String get slotId;
  @override
  int get priority;
  @override
  bool get active;
  @override
  EdgeStatus get status;

  /// Create a copy of FlowEdge
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$FlowEdgeImplCopyWith<_$FlowEdgeImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

FlowNode _$FlowNodeFromJson(Map<String, dynamic> json) {
  switch (json['type']) {
    case 'device':
      return DeviceFlowNode.fromJson(json);
    case 'hardwareProfile':
      return HardwareProfileFlowNode.fromJson(json);
    case 'keymap':
      return KeymapFlowNode.fromJson(json);

    default:
      throw CheckedFromJsonException(
        json,
        'type',
        'FlowNode',
        'Invalid union type "${json['type']}"!',
      );
  }
}

/// @nodoc
mixin _$FlowNode {
  String get id => throw _privateConstructorUsedError;
  String? get label => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(
      String id,
      DeviceInstanceId deviceId,
      bool isConnected,
      String? label,
    )
    device,
    required TResult Function(String id, String? label) hardwareProfile,
    required TResult Function(String id, String? label, int? layers) keymap,
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(
      String id,
      DeviceInstanceId deviceId,
      bool isConnected,
      String? label,
    )?
    device,
    TResult? Function(String id, String? label)? hardwareProfile,
    TResult? Function(String id, String? label, int? layers)? keymap,
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(
      String id,
      DeviceInstanceId deviceId,
      bool isConnected,
      String? label,
    )?
    device,
    TResult Function(String id, String? label)? hardwareProfile,
    TResult Function(String id, String? label, int? layers)? keymap,
    required TResult orElse(),
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(DeviceFlowNode value) device,
    required TResult Function(HardwareProfileFlowNode value) hardwareProfile,
    required TResult Function(KeymapFlowNode value) keymap,
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(DeviceFlowNode value)? device,
    TResult? Function(HardwareProfileFlowNode value)? hardwareProfile,
    TResult? Function(KeymapFlowNode value)? keymap,
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(DeviceFlowNode value)? device,
    TResult Function(HardwareProfileFlowNode value)? hardwareProfile,
    TResult Function(KeymapFlowNode value)? keymap,
    required TResult orElse(),
  }) => throw _privateConstructorUsedError;

  /// Serializes this FlowNode to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of FlowNode
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $FlowNodeCopyWith<FlowNode> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $FlowNodeCopyWith<$Res> {
  factory $FlowNodeCopyWith(FlowNode value, $Res Function(FlowNode) then) =
      _$FlowNodeCopyWithImpl<$Res, FlowNode>;
  @useResult
  $Res call({String id, String? label});
}

/// @nodoc
class _$FlowNodeCopyWithImpl<$Res, $Val extends FlowNode>
    implements $FlowNodeCopyWith<$Res> {
  _$FlowNodeCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of FlowNode
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? id = null, Object? label = freezed}) {
    return _then(
      _value.copyWith(
            id: null == id
                ? _value.id
                : id // ignore: cast_nullable_to_non_nullable
                      as String,
            label: freezed == label
                ? _value.label
                : label // ignore: cast_nullable_to_non_nullable
                      as String?,
          )
          as $Val,
    );
  }
}

/// @nodoc
abstract class _$$DeviceFlowNodeImplCopyWith<$Res>
    implements $FlowNodeCopyWith<$Res> {
  factory _$$DeviceFlowNodeImplCopyWith(
    _$DeviceFlowNodeImpl value,
    $Res Function(_$DeviceFlowNodeImpl) then,
  ) = __$$DeviceFlowNodeImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({
    String id,
    DeviceInstanceId deviceId,
    bool isConnected,
    String? label,
  });

  $DeviceInstanceIdCopyWith<$Res> get deviceId;
}

/// @nodoc
class __$$DeviceFlowNodeImplCopyWithImpl<$Res>
    extends _$FlowNodeCopyWithImpl<$Res, _$DeviceFlowNodeImpl>
    implements _$$DeviceFlowNodeImplCopyWith<$Res> {
  __$$DeviceFlowNodeImplCopyWithImpl(
    _$DeviceFlowNodeImpl _value,
    $Res Function(_$DeviceFlowNodeImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of FlowNode
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? deviceId = null,
    Object? isConnected = null,
    Object? label = freezed,
  }) {
    return _then(
      _$DeviceFlowNodeImpl(
        id: null == id
            ? _value.id
            : id // ignore: cast_nullable_to_non_nullable
                  as String,
        deviceId: null == deviceId
            ? _value.deviceId
            : deviceId // ignore: cast_nullable_to_non_nullable
                  as DeviceInstanceId,
        isConnected: null == isConnected
            ? _value.isConnected
            : isConnected // ignore: cast_nullable_to_non_nullable
                  as bool,
        label: freezed == label
            ? _value.label
            : label // ignore: cast_nullable_to_non_nullable
                  as String?,
      ),
    );
  }

  /// Create a copy of FlowNode
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $DeviceInstanceIdCopyWith<$Res> get deviceId {
    return $DeviceInstanceIdCopyWith<$Res>(_value.deviceId, (value) {
      return _then(_value.copyWith(deviceId: value));
    });
  }
}

/// @nodoc
@JsonSerializable()
class _$DeviceFlowNodeImpl implements DeviceFlowNode {
  const _$DeviceFlowNodeImpl({
    required this.id,
    required this.deviceId,
    this.isConnected = true,
    this.label,
    final String? $type,
  }) : $type = $type ?? 'device';

  factory _$DeviceFlowNodeImpl.fromJson(Map<String, dynamic> json) =>
      _$$DeviceFlowNodeImplFromJson(json);

  @override
  final String id;
  @override
  final DeviceInstanceId deviceId;
  @override
  @JsonKey()
  final bool isConnected;
  @override
  final String? label;

  @JsonKey(name: 'type')
  final String $type;

  @override
  String toString() {
    return 'FlowNode.device(id: $id, deviceId: $deviceId, isConnected: $isConnected, label: $label)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$DeviceFlowNodeImpl &&
            (identical(other.id, id) || other.id == id) &&
            (identical(other.deviceId, deviceId) ||
                other.deviceId == deviceId) &&
            (identical(other.isConnected, isConnected) ||
                other.isConnected == isConnected) &&
            (identical(other.label, label) || other.label == label));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode =>
      Object.hash(runtimeType, id, deviceId, isConnected, label);

  /// Create a copy of FlowNode
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$DeviceFlowNodeImplCopyWith<_$DeviceFlowNodeImpl> get copyWith =>
      __$$DeviceFlowNodeImplCopyWithImpl<_$DeviceFlowNodeImpl>(
        this,
        _$identity,
      );

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(
      String id,
      DeviceInstanceId deviceId,
      bool isConnected,
      String? label,
    )
    device,
    required TResult Function(String id, String? label) hardwareProfile,
    required TResult Function(String id, String? label, int? layers) keymap,
  }) {
    return device(id, deviceId, isConnected, label);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(
      String id,
      DeviceInstanceId deviceId,
      bool isConnected,
      String? label,
    )?
    device,
    TResult? Function(String id, String? label)? hardwareProfile,
    TResult? Function(String id, String? label, int? layers)? keymap,
  }) {
    return device?.call(id, deviceId, isConnected, label);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(
      String id,
      DeviceInstanceId deviceId,
      bool isConnected,
      String? label,
    )?
    device,
    TResult Function(String id, String? label)? hardwareProfile,
    TResult Function(String id, String? label, int? layers)? keymap,
    required TResult orElse(),
  }) {
    if (device != null) {
      return device(id, deviceId, isConnected, label);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(DeviceFlowNode value) device,
    required TResult Function(HardwareProfileFlowNode value) hardwareProfile,
    required TResult Function(KeymapFlowNode value) keymap,
  }) {
    return device(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(DeviceFlowNode value)? device,
    TResult? Function(HardwareProfileFlowNode value)? hardwareProfile,
    TResult? Function(KeymapFlowNode value)? keymap,
  }) {
    return device?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(DeviceFlowNode value)? device,
    TResult Function(HardwareProfileFlowNode value)? hardwareProfile,
    TResult Function(KeymapFlowNode value)? keymap,
    required TResult orElse(),
  }) {
    if (device != null) {
      return device(this);
    }
    return orElse();
  }

  @override
  Map<String, dynamic> toJson() {
    return _$$DeviceFlowNodeImplToJson(this);
  }
}

abstract class DeviceFlowNode implements FlowNode {
  const factory DeviceFlowNode({
    required final String id,
    required final DeviceInstanceId deviceId,
    final bool isConnected,
    final String? label,
  }) = _$DeviceFlowNodeImpl;

  factory DeviceFlowNode.fromJson(Map<String, dynamic> json) =
      _$DeviceFlowNodeImpl.fromJson;

  @override
  String get id;
  DeviceInstanceId get deviceId;
  bool get isConnected;
  @override
  String? get label;

  /// Create a copy of FlowNode
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$DeviceFlowNodeImplCopyWith<_$DeviceFlowNodeImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$HardwareProfileFlowNodeImplCopyWith<$Res>
    implements $FlowNodeCopyWith<$Res> {
  factory _$$HardwareProfileFlowNodeImplCopyWith(
    _$HardwareProfileFlowNodeImpl value,
    $Res Function(_$HardwareProfileFlowNodeImpl) then,
  ) = __$$HardwareProfileFlowNodeImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({String id, String? label});
}

/// @nodoc
class __$$HardwareProfileFlowNodeImplCopyWithImpl<$Res>
    extends _$FlowNodeCopyWithImpl<$Res, _$HardwareProfileFlowNodeImpl>
    implements _$$HardwareProfileFlowNodeImplCopyWith<$Res> {
  __$$HardwareProfileFlowNodeImplCopyWithImpl(
    _$HardwareProfileFlowNodeImpl _value,
    $Res Function(_$HardwareProfileFlowNodeImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of FlowNode
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? id = null, Object? label = freezed}) {
    return _then(
      _$HardwareProfileFlowNodeImpl(
        id: null == id
            ? _value.id
            : id // ignore: cast_nullable_to_non_nullable
                  as String,
        label: freezed == label
            ? _value.label
            : label // ignore: cast_nullable_to_non_nullable
                  as String?,
      ),
    );
  }
}

/// @nodoc
@JsonSerializable()
class _$HardwareProfileFlowNodeImpl implements HardwareProfileFlowNode {
  const _$HardwareProfileFlowNodeImpl({
    required this.id,
    this.label,
    final String? $type,
  }) : $type = $type ?? 'hardwareProfile';

  factory _$HardwareProfileFlowNodeImpl.fromJson(Map<String, dynamic> json) =>
      _$$HardwareProfileFlowNodeImplFromJson(json);

  @override
  final String id;
  @override
  final String? label;

  @JsonKey(name: 'type')
  final String $type;

  @override
  String toString() {
    return 'FlowNode.hardwareProfile(id: $id, label: $label)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$HardwareProfileFlowNodeImpl &&
            (identical(other.id, id) || other.id == id) &&
            (identical(other.label, label) || other.label == label));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, id, label);

  /// Create a copy of FlowNode
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$HardwareProfileFlowNodeImplCopyWith<_$HardwareProfileFlowNodeImpl>
  get copyWith =>
      __$$HardwareProfileFlowNodeImplCopyWithImpl<
        _$HardwareProfileFlowNodeImpl
      >(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(
      String id,
      DeviceInstanceId deviceId,
      bool isConnected,
      String? label,
    )
    device,
    required TResult Function(String id, String? label) hardwareProfile,
    required TResult Function(String id, String? label, int? layers) keymap,
  }) {
    return hardwareProfile(id, label);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(
      String id,
      DeviceInstanceId deviceId,
      bool isConnected,
      String? label,
    )?
    device,
    TResult? Function(String id, String? label)? hardwareProfile,
    TResult? Function(String id, String? label, int? layers)? keymap,
  }) {
    return hardwareProfile?.call(id, label);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(
      String id,
      DeviceInstanceId deviceId,
      bool isConnected,
      String? label,
    )?
    device,
    TResult Function(String id, String? label)? hardwareProfile,
    TResult Function(String id, String? label, int? layers)? keymap,
    required TResult orElse(),
  }) {
    if (hardwareProfile != null) {
      return hardwareProfile(id, label);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(DeviceFlowNode value) device,
    required TResult Function(HardwareProfileFlowNode value) hardwareProfile,
    required TResult Function(KeymapFlowNode value) keymap,
  }) {
    return hardwareProfile(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(DeviceFlowNode value)? device,
    TResult? Function(HardwareProfileFlowNode value)? hardwareProfile,
    TResult? Function(KeymapFlowNode value)? keymap,
  }) {
    return hardwareProfile?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(DeviceFlowNode value)? device,
    TResult Function(HardwareProfileFlowNode value)? hardwareProfile,
    TResult Function(KeymapFlowNode value)? keymap,
    required TResult orElse(),
  }) {
    if (hardwareProfile != null) {
      return hardwareProfile(this);
    }
    return orElse();
  }

  @override
  Map<String, dynamic> toJson() {
    return _$$HardwareProfileFlowNodeImplToJson(this);
  }
}

abstract class HardwareProfileFlowNode implements FlowNode {
  const factory HardwareProfileFlowNode({
    required final String id,
    final String? label,
  }) = _$HardwareProfileFlowNodeImpl;

  factory HardwareProfileFlowNode.fromJson(Map<String, dynamic> json) =
      _$HardwareProfileFlowNodeImpl.fromJson;

  @override
  String get id;
  @override
  String? get label;

  /// Create a copy of FlowNode
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$HardwareProfileFlowNodeImplCopyWith<_$HardwareProfileFlowNodeImpl>
  get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$KeymapFlowNodeImplCopyWith<$Res>
    implements $FlowNodeCopyWith<$Res> {
  factory _$$KeymapFlowNodeImplCopyWith(
    _$KeymapFlowNodeImpl value,
    $Res Function(_$KeymapFlowNodeImpl) then,
  ) = __$$KeymapFlowNodeImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({String id, String? label, int? layers});
}

/// @nodoc
class __$$KeymapFlowNodeImplCopyWithImpl<$Res>
    extends _$FlowNodeCopyWithImpl<$Res, _$KeymapFlowNodeImpl>
    implements _$$KeymapFlowNodeImplCopyWith<$Res> {
  __$$KeymapFlowNodeImplCopyWithImpl(
    _$KeymapFlowNodeImpl _value,
    $Res Function(_$KeymapFlowNodeImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of FlowNode
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? label = freezed,
    Object? layers = freezed,
  }) {
    return _then(
      _$KeymapFlowNodeImpl(
        id: null == id
            ? _value.id
            : id // ignore: cast_nullable_to_non_nullable
                  as String,
        label: freezed == label
            ? _value.label
            : label // ignore: cast_nullable_to_non_nullable
                  as String?,
        layers: freezed == layers
            ? _value.layers
            : layers // ignore: cast_nullable_to_non_nullable
                  as int?,
      ),
    );
  }
}

/// @nodoc
@JsonSerializable()
class _$KeymapFlowNodeImpl implements KeymapFlowNode {
  const _$KeymapFlowNodeImpl({
    required this.id,
    this.label,
    this.layers,
    final String? $type,
  }) : $type = $type ?? 'keymap';

  factory _$KeymapFlowNodeImpl.fromJson(Map<String, dynamic> json) =>
      _$$KeymapFlowNodeImplFromJson(json);

  @override
  final String id;
  @override
  final String? label;
  @override
  final int? layers;

  @JsonKey(name: 'type')
  final String $type;

  @override
  String toString() {
    return 'FlowNode.keymap(id: $id, label: $label, layers: $layers)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$KeymapFlowNodeImpl &&
            (identical(other.id, id) || other.id == id) &&
            (identical(other.label, label) || other.label == label) &&
            (identical(other.layers, layers) || other.layers == layers));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, id, label, layers);

  /// Create a copy of FlowNode
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$KeymapFlowNodeImplCopyWith<_$KeymapFlowNodeImpl> get copyWith =>
      __$$KeymapFlowNodeImplCopyWithImpl<_$KeymapFlowNodeImpl>(
        this,
        _$identity,
      );

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(
      String id,
      DeviceInstanceId deviceId,
      bool isConnected,
      String? label,
    )
    device,
    required TResult Function(String id, String? label) hardwareProfile,
    required TResult Function(String id, String? label, int? layers) keymap,
  }) {
    return keymap(id, label, layers);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(
      String id,
      DeviceInstanceId deviceId,
      bool isConnected,
      String? label,
    )?
    device,
    TResult? Function(String id, String? label)? hardwareProfile,
    TResult? Function(String id, String? label, int? layers)? keymap,
  }) {
    return keymap?.call(id, label, layers);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(
      String id,
      DeviceInstanceId deviceId,
      bool isConnected,
      String? label,
    )?
    device,
    TResult Function(String id, String? label)? hardwareProfile,
    TResult Function(String id, String? label, int? layers)? keymap,
    required TResult orElse(),
  }) {
    if (keymap != null) {
      return keymap(id, label, layers);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(DeviceFlowNode value) device,
    required TResult Function(HardwareProfileFlowNode value) hardwareProfile,
    required TResult Function(KeymapFlowNode value) keymap,
  }) {
    return keymap(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(DeviceFlowNode value)? device,
    TResult? Function(HardwareProfileFlowNode value)? hardwareProfile,
    TResult? Function(KeymapFlowNode value)? keymap,
  }) {
    return keymap?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(DeviceFlowNode value)? device,
    TResult Function(HardwareProfileFlowNode value)? hardwareProfile,
    TResult Function(KeymapFlowNode value)? keymap,
    required TResult orElse(),
  }) {
    if (keymap != null) {
      return keymap(this);
    }
    return orElse();
  }

  @override
  Map<String, dynamic> toJson() {
    return _$$KeymapFlowNodeImplToJson(this);
  }
}

abstract class KeymapFlowNode implements FlowNode {
  const factory KeymapFlowNode({
    required final String id,
    final String? label,
    final int? layers,
  }) = _$KeymapFlowNodeImpl;

  factory KeymapFlowNode.fromJson(Map<String, dynamic> json) =
      _$KeymapFlowNodeImpl.fromJson;

  @override
  String get id;
  @override
  String? get label;
  int? get layers;

  /// Create a copy of FlowNode
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$KeymapFlowNodeImplCopyWith<_$KeymapFlowNodeImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

FlowGraph _$FlowGraphFromJson(Map<String, dynamic> json) {
  return _FlowGraph.fromJson(json);
}

/// @nodoc
mixin _$FlowGraph {
  List<FlowNode> get nodes => throw _privateConstructorUsedError;
  List<FlowEdge> get edges => throw _privateConstructorUsedError;
  Map<String, FlowPosition> get positions => throw _privateConstructorUsedError;

  /// Serializes this FlowGraph to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of FlowGraph
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $FlowGraphCopyWith<FlowGraph> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $FlowGraphCopyWith<$Res> {
  factory $FlowGraphCopyWith(FlowGraph value, $Res Function(FlowGraph) then) =
      _$FlowGraphCopyWithImpl<$Res, FlowGraph>;
  @useResult
  $Res call({
    List<FlowNode> nodes,
    List<FlowEdge> edges,
    Map<String, FlowPosition> positions,
  });
}

/// @nodoc
class _$FlowGraphCopyWithImpl<$Res, $Val extends FlowGraph>
    implements $FlowGraphCopyWith<$Res> {
  _$FlowGraphCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of FlowGraph
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? nodes = null,
    Object? edges = null,
    Object? positions = null,
  }) {
    return _then(
      _value.copyWith(
            nodes: null == nodes
                ? _value.nodes
                : nodes // ignore: cast_nullable_to_non_nullable
                      as List<FlowNode>,
            edges: null == edges
                ? _value.edges
                : edges // ignore: cast_nullable_to_non_nullable
                      as List<FlowEdge>,
            positions: null == positions
                ? _value.positions
                : positions // ignore: cast_nullable_to_non_nullable
                      as Map<String, FlowPosition>,
          )
          as $Val,
    );
  }
}

/// @nodoc
abstract class _$$FlowGraphImplCopyWith<$Res>
    implements $FlowGraphCopyWith<$Res> {
  factory _$$FlowGraphImplCopyWith(
    _$FlowGraphImpl value,
    $Res Function(_$FlowGraphImpl) then,
  ) = __$$FlowGraphImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({
    List<FlowNode> nodes,
    List<FlowEdge> edges,
    Map<String, FlowPosition> positions,
  });
}

/// @nodoc
class __$$FlowGraphImplCopyWithImpl<$Res>
    extends _$FlowGraphCopyWithImpl<$Res, _$FlowGraphImpl>
    implements _$$FlowGraphImplCopyWith<$Res> {
  __$$FlowGraphImplCopyWithImpl(
    _$FlowGraphImpl _value,
    $Res Function(_$FlowGraphImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of FlowGraph
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? nodes = null,
    Object? edges = null,
    Object? positions = null,
  }) {
    return _then(
      _$FlowGraphImpl(
        nodes: null == nodes
            ? _value._nodes
            : nodes // ignore: cast_nullable_to_non_nullable
                  as List<FlowNode>,
        edges: null == edges
            ? _value._edges
            : edges // ignore: cast_nullable_to_non_nullable
                  as List<FlowEdge>,
        positions: null == positions
            ? _value._positions
            : positions // ignore: cast_nullable_to_non_nullable
                  as Map<String, FlowPosition>,
      ),
    );
  }
}

/// @nodoc
@JsonSerializable()
class _$FlowGraphImpl extends _FlowGraph {
  const _$FlowGraphImpl({
    final List<FlowNode> nodes = const <FlowNode>[],
    final List<FlowEdge> edges = const <FlowEdge>[],
    final Map<String, FlowPosition> positions = const <String, FlowPosition>{},
  }) : _nodes = nodes,
       _edges = edges,
       _positions = positions,
       super._();

  factory _$FlowGraphImpl.fromJson(Map<String, dynamic> json) =>
      _$$FlowGraphImplFromJson(json);

  final List<FlowNode> _nodes;
  @override
  @JsonKey()
  List<FlowNode> get nodes {
    if (_nodes is EqualUnmodifiableListView) return _nodes;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_nodes);
  }

  final List<FlowEdge> _edges;
  @override
  @JsonKey()
  List<FlowEdge> get edges {
    if (_edges is EqualUnmodifiableListView) return _edges;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_edges);
  }

  final Map<String, FlowPosition> _positions;
  @override
  @JsonKey()
  Map<String, FlowPosition> get positions {
    if (_positions is EqualUnmodifiableMapView) return _positions;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableMapView(_positions);
  }

  @override
  String toString() {
    return 'FlowGraph(nodes: $nodes, edges: $edges, positions: $positions)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$FlowGraphImpl &&
            const DeepCollectionEquality().equals(other._nodes, _nodes) &&
            const DeepCollectionEquality().equals(other._edges, _edges) &&
            const DeepCollectionEquality().equals(
              other._positions,
              _positions,
            ));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(
    runtimeType,
    const DeepCollectionEquality().hash(_nodes),
    const DeepCollectionEquality().hash(_edges),
    const DeepCollectionEquality().hash(_positions),
  );

  /// Create a copy of FlowGraph
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$FlowGraphImplCopyWith<_$FlowGraphImpl> get copyWith =>
      __$$FlowGraphImplCopyWithImpl<_$FlowGraphImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$FlowGraphImplToJson(this);
  }
}

abstract class _FlowGraph extends FlowGraph {
  const factory _FlowGraph({
    final List<FlowNode> nodes,
    final List<FlowEdge> edges,
    final Map<String, FlowPosition> positions,
  }) = _$FlowGraphImpl;
  const _FlowGraph._() : super._();

  factory _FlowGraph.fromJson(Map<String, dynamic> json) =
      _$FlowGraphImpl.fromJson;

  @override
  List<FlowNode> get nodes;
  @override
  List<FlowEdge> get edges;
  @override
  Map<String, FlowPosition> get positions;

  /// Create a copy of FlowGraph
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$FlowGraphImplCopyWith<_$FlowGraphImpl> get copyWith =>
      throw _privateConstructorUsedError;
}
