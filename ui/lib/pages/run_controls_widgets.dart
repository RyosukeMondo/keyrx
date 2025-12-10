/// Reusable widgets for the run controls page.
///
/// Provides [StatusIndicator] for displaying engine status,
/// and various card builders for device selection and script info.
library;

import 'package:flutter/material.dart';

/// Displays a status indicator with icon, label, and value.
///
/// Used to show engine state, device selection, script status,
/// and recording state in the run controls page.
class StatusIndicator extends StatelessWidget {
  const StatusIndicator({
    super.key,
    required this.icon,
    required this.label,
    required this.value,
    required this.isActive,
    this.activeColor,
  });

  /// The icon to display.
  final IconData icon;

  /// The label text (e.g., "Engine", "Device").
  final String label;

  /// The current value text.
  final String value;

  /// Whether this status is active/enabled.
  final bool isActive;

  /// The color when active. Defaults to green.
  final Color? activeColor;

  @override
  Widget build(BuildContext context) {
    final color = isActive ? (activeColor ?? Colors.green) : Colors.grey;

    return Row(
      children: [
        Icon(icon, size: 16, color: color),
        const SizedBox(width: 8),
        SizedBox(
          width: 80,
          child: Text(
            label,
            style: Theme.of(
              context,
            ).textTheme.bodyMedium?.copyWith(color: Colors.grey),
          ),
        ),
        Expanded(
          child: Text(
            value,
            style: Theme.of(context).textTheme.bodyMedium?.copyWith(
              color: color,
              fontWeight: isActive ? FontWeight.w500 : FontWeight.normal,
            ),
            overflow: TextOverflow.ellipsis,
          ),
        ),
      ],
    );
  }
}

/// A loading card showing a progress indicator with message.
class LoadingCard extends StatelessWidget {
  const LoadingCard({super.key, this.message = 'Loading...'});

  /// The message to display while loading.
  final String message;

  @override
  Widget build(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Row(
          children: [
            const SizedBox(
              width: 20,
              height: 20,
              child: CircularProgressIndicator(strokeWidth: 2),
            ),
            const SizedBox(width: 12),
            Text(message),
          ],
        ),
      ),
    );
  }
}

/// An error card with retry button.
class ErrorCard extends StatelessWidget {
  const ErrorCard({super.key, required this.error, required this.onRetry});

  /// The error message to display.
  final String error;

  /// Callback when retry is pressed.
  final VoidCallback onRetry;

  @override
  Widget build(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Row(
          children: [
            const Icon(Icons.error_outline, color: Colors.red),
            const SizedBox(width: 12),
            Expanded(child: Text(error)),
            TextButton(onPressed: onRetry, child: const Text('Retry')),
          ],
        ),
      ),
    );
  }
}

/// A large start/stop button for engine control.
class StartStopButton extends StatelessWidget {
  const StartStopButton({
    super.key,
    required this.isRunning,
    required this.isBusy,
    required this.onPressed,
    this.startingLabel = 'Starting...',
    this.stoppingLabel = 'Stopping...',
    this.startLabel = 'Start Engine',
    this.stopLabel = 'Stop Engine',
  });

  /// Whether the engine is currently running.
  final bool isRunning;

  /// Whether the engine is in a transitional state (starting/stopping).
  final bool isBusy;

  /// Callback when the button is pressed.
  final VoidCallback onPressed;

  /// Label shown while starting.
  final String startingLabel;

  /// Label shown while stopping.
  final String stoppingLabel;

  /// Label shown when stopped.
  final String startLabel;

  /// Label shown when running.
  final String stopLabel;

  @override
  Widget build(BuildContext context) {
    final isStarting = isBusy && !isRunning;
    final label = isBusy
        ? (isStarting ? startingLabel : stoppingLabel)
        : (isRunning ? stopLabel : startLabel);

    return SizedBox(
      height: 80,
      child: FilledButton.icon(
        onPressed: isBusy ? null : onPressed,
        style: FilledButton.styleFrom(
          backgroundColor: isRunning ? Colors.red : Colors.green,
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(16),
          ),
        ),
        icon: isBusy
            ? const SizedBox(
                width: 24,
                height: 24,
                child: CircularProgressIndicator(
                  strokeWidth: 2,
                  color: Colors.white,
                ),
              )
            : Icon(isRunning ? Icons.stop : Icons.play_arrow, size: 32),
        label: Text(
          label,
          style: const TextStyle(fontSize: 20, fontWeight: FontWeight.bold),
        ),
      ),
    );
  }
}
