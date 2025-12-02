// Emergency bypass mode indicator widget.
//
// Displays a prominent banner when bypass mode is active (remapping disabled)
// with an option to re-enable remapping.

import 'dart:async';

import 'package:flutter/material.dart';

import '../ffi/bridge.dart';

/// Widget that displays bypass mode status and provides re-enable control.
///
/// When bypass mode is active (remapping disabled), shows a red banner with
/// a "Re-enable" button. Polls the bypass status periodically.
class BypassIndicator extends StatefulWidget {
  /// The bridge instance to query bypass status.
  final KeyrxBridge bridge;

  /// Polling interval for checking bypass status.
  final Duration pollInterval;

  const BypassIndicator({
    super.key,
    required this.bridge,
    this.pollInterval = const Duration(milliseconds: 500),
  });

  @override
  State<BypassIndicator> createState() => _BypassIndicatorState();
}

class _BypassIndicatorState extends State<BypassIndicator> {
  bool _bypassActive = false;
  Timer? _pollTimer;

  @override
  void initState() {
    super.initState();
    _checkBypassStatus();
    _startPolling();
  }

  @override
  void dispose() {
    _pollTimer?.cancel();
    super.dispose();
  }

  void _startPolling() {
    _pollTimer = Timer.periodic(widget.pollInterval, (_) {
      _checkBypassStatus();
    });
  }

  void _checkBypassStatus() {
    final active = widget.bridge.isBypassActive();
    if (active != _bypassActive) {
      setState(() {
        _bypassActive = active;
      });
    }
  }

  void _handleReEnable() {
    widget.bridge.setBypass(false);
    setState(() {
      _bypassActive = false;
    });
  }

  @override
  Widget build(BuildContext context) {
    if (!_bypassActive) {
      return const SizedBox.shrink();
    }

    return Material(
      color: Colors.red.shade700,
      child: SafeArea(
        bottom: false,
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
          child: Row(
            children: [
              const Icon(
                Icons.warning_amber_rounded,
                color: Colors.white,
                size: 24,
              ),
              const SizedBox(width: 12),
              Expanded(
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      'REMAPPING DISABLED',
                      style: Theme.of(context).textTheme.titleSmall?.copyWith(
                            color: Colors.white,
                            fontWeight: FontWeight.bold,
                          ),
                    ),
                    Text(
                      'Emergency bypass mode is active',
                      style: Theme.of(context).textTheme.bodySmall?.copyWith(
                            color: Colors.white70,
                          ),
                    ),
                  ],
                ),
              ),
              ElevatedButton(
                onPressed: _handleReEnable,
                style: ElevatedButton.styleFrom(
                  backgroundColor: Colors.white,
                  foregroundColor: Colors.red.shade700,
                ),
                child: const Text('Re-enable'),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
