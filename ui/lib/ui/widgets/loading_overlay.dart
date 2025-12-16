import 'package:flutter/material.dart';

/// Full-screen loading overlay that blocks interaction while active.
class LoadingOverlay extends StatelessWidget {
  const LoadingOverlay({
    required this.child,
    required this.isLoading,
    super.key,
    this.message,
    this.barrierColor,
  });
  final Widget child;
  final bool isLoading;
  final String? message;
  final Color? barrierColor;

  @override
  Widget build(BuildContext context) => Stack(
    children: [
      IgnorePointer(ignoring: isLoading, child: child),
      AnimatedSwitcher(
        duration: const Duration(milliseconds: 200),
        switchInCurve: Curves.easeOut,
        switchOutCurve: Curves.easeIn,
        child: isLoading ? _buildOverlay(context) : const SizedBox.shrink(),
      ),
    ],
  );

  Widget _buildOverlay(BuildContext context) {
    final color = barrierColor ?? Colors.black54;

    return SizedBox.expand(
      child: DecoratedBox(
        decoration: BoxDecoration(color: color),
        child: Center(
          child: ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 280),
            child: Card(
              elevation: 8,
              child: Padding(
                padding: const EdgeInsets.symmetric(
                  horizontal: 20,
                  vertical: 16,
                ),
                child: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    const SizedBox(
                      width: 32,
                      height: 32,
                      child: CircularProgressIndicator(strokeWidth: 3),
                    ),
                    const SizedBox(width: 16),
                    Expanded(
                      child: Text(
                        message ?? 'Working...',
                        style: Theme.of(context).textTheme.bodyMedium,
                      ),
                    ),
                  ],
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }
}
