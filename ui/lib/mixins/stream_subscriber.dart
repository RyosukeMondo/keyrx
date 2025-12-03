import 'dart:async';

import 'package:flutter/widgets.dart';

/// Mixin that handles stream subscription lifecycle management.
///
/// Automatically cancels all subscriptions on dispose and provides
/// a consistent pattern for subscribing to streams with mounted checks.
///
/// Usage:
/// ```dart
/// class _MyPageState extends State<MyPage> with StreamSubscriber {
///   @override
///   void initState() {
///     super.initState();
///     subscribe(myStream, onData: (data) {
///       setState(() => _data = data);
///     });
///   }
/// }
/// ```
mixin StreamSubscriber<T extends StatefulWidget> on State<T> {
  final List<StreamSubscription<dynamic>> _subscriptions = [];

  /// Subscribe to a stream with automatic lifecycle management.
  ///
  /// The subscription is tracked and will be canceled on dispose.
  /// Callbacks are guarded with mounted checks to prevent setState errors.
  void subscribe<S>(
    Stream<S> stream, {
    required void Function(S data) onData,
    void Function(Object error, StackTrace stackTrace)? onError,
  }) {
    final subscription = stream.listen(
      (data) {
        if (!mounted) return;
        onData(data);
      },
      onError: (Object error, StackTrace stackTrace) {
        if (!mounted) return;
        onError?.call(error, stackTrace);
      },
    );
    _subscriptions.add(subscription);
  }

  /// Cancel all tracked subscriptions.
  void cancelAllSubscriptions() {
    for (final sub in _subscriptions) {
      sub.cancel();
    }
    _subscriptions.clear();
  }

  @override
  void dispose() {
    cancelAllSubscriptions();
    super.dispose();
  }
}
