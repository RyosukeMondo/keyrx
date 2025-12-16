/// Loading skeleton widgets with shimmer effect for KeyFlow.
library;

import 'package:flutter/material.dart';

/// Full-page loading skeleton that mimics the flow layout.
///
/// Displays grey boxes for nodes and lines for edges with an animated
/// shimmer overlay to provide visual feedback during loading.
class FlowLoadingSkeleton extends StatefulWidget {
  const FlowLoadingSkeleton({super.key});

  @override
  State<FlowLoadingSkeleton> createState() => _FlowLoadingSkeletonState();
}

class _FlowLoadingSkeletonState extends State<FlowLoadingSkeleton>
    with SingleTickerProviderStateMixin {
  late final AnimationController _shimmerController;
  late final Animation<double> _shimmerAnimation;

  @override
  void initState() {
    super.initState();
    _shimmerController = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 1200),
    )..repeat();

    _shimmerAnimation = Tween<double>(begin: -1, end: 2).animate(
      CurvedAnimation(parent: _shimmerController, curve: Curves.easeInOut),
    );
  }

  @override
  void dispose() {
    _shimmerController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => Scaffold(
    backgroundColor: const Color(0xFF121212),
    body: Semantics(
      label: 'Loading flow graph, please wait',
      child: AnimatedBuilder(
        animation: _shimmerAnimation,
        builder: (context, child) => Container(
          color: const Color(0xFF121212),
          padding: const EdgeInsets.all(32),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              // Header skeleton
              _ShimmerBar(animation: _shimmerAnimation, width: 220),
              const SizedBox(height: 20),

              // Flow nodes skeleton
              Expanded(
                child: Row(
                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    // Left column - Device nodes
                    Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        _ShimmerCard(
                          animation: _shimmerAnimation,
                          width: 260,
                          height: 220,
                        ),
                        const SizedBox(height: 24),
                        _ShimmerCard(
                          animation: _shimmerAnimation,
                          width: 260,
                          height: 220,
                        ),
                      ],
                    ),

                    // Middle column - Profile nodes
                    Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        const SizedBox(height: 40),
                        _ShimmerCard(
                          animation: _shimmerAnimation,
                          width: 220,
                          height: 160,
                        ),
                        const SizedBox(height: 32),
                        _ShimmerCard(
                          animation: _shimmerAnimation,
                          width: 220,
                          height: 160,
                        ),
                      ],
                    ),

                    // Right column - Keymap nodes
                    Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        const SizedBox(height: 60),
                        _ShimmerCard(
                          animation: _shimmerAnimation,
                          width: 200,
                          height: 140,
                        ),
                        const SizedBox(height: 40),
                        _ShimmerCard(
                          animation: _shimmerAnimation,
                          width: 200,
                          height: 140,
                        ),
                      ],
                    ),
                  ],
                ),
              ),

              // Connection lines skeleton
              const SizedBox(height: 24),
              Row(
                children: [
                  Expanded(
                    child: _ShimmerBar(
                      animation: _shimmerAnimation,
                      width: double.infinity,
                      height: 2,
                    ),
                  ),
                ],
              ),
            ],
          ),
        ),
      ),
    ),
  );
}

/// Shimmer bar widget for small loading placeholders.
class _ShimmerBar extends StatelessWidget {
  const _ShimmerBar({
    required this.animation,
    required this.width,
    this.height = 16,
  });

  final Animation<double> animation;
  final double width;
  final double height;

  @override
  Widget build(BuildContext context) => _ShimmerBox(
    animation: animation,
    width: width,
    height: height,
    radius: 6,
  );
}

/// Shimmer card widget for node placeholders.
class _ShimmerCard extends StatelessWidget {
  const _ShimmerCard({
    required this.animation,
    required this.width,
    required this.height,
  });

  final Animation<double> animation;
  final double width;
  final double height;

  @override
  Widget build(BuildContext context) => _ShimmerBox(
    animation: animation,
    width: width,
    height: height,
    radius: 12,
  );
}

/// Base shimmer box with animated gradient.
class _ShimmerBox extends StatelessWidget {
  const _ShimmerBox({
    required this.animation,
    required this.width,
    required this.height,
    required this.radius,
  });

  final Animation<double> animation;
  final double width;
  final double height;
  final double radius;

  @override
  Widget build(BuildContext context) {
    final gradient = LinearGradient(
      colors: const [Color(0xFF1E1E1E), Color(0xFF2C2C2C), Color(0xFF1E1E1E)],
      stops: const [0.1, 0.5, 0.9],
      transform: _SlidingGradientTransform(slidePercent: animation.value),
    );

    return Container(
      width: width,
      height: height,
      decoration: BoxDecoration(
        color: const Color(0xFF1E1E1E),
        borderRadius: BorderRadius.circular(radius),
        gradient: gradient,
      ),
    );
  }
}

/// Gradient transform that slides the shimmer effect.
class _SlidingGradientTransform extends GradientTransform {
  const _SlidingGradientTransform({required this.slidePercent});

  final double slidePercent;

  @override
  Matrix4? transform(Rect bounds, {TextDirection? textDirection}) =>
      Matrix4.translationValues(bounds.width * slidePercent, 0, 0);
}

/// Inline loading spinner for specific operations.
///
/// Used to show loading state for individual operations like
/// adding/deleting slots without blocking the entire UI.
class InlineLoadingSpinner extends StatelessWidget {
  const InlineLoadingSpinner({
    required this.message,
    super.key,
    this.size = 16,
  });

  /// Loading message to display
  final String message;

  /// Size of the spinner
  final double size;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Semantics(
      label: message,
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
        decoration: BoxDecoration(
          color: theme.colorScheme.primaryContainer.withValues(alpha: 0.8),
          borderRadius: BorderRadius.circular(8),
          boxShadow: [
            BoxShadow(
              color: Colors.black.withValues(alpha: 0.1),
              blurRadius: 4,
              offset: const Offset(0, 2),
            ),
          ],
        ),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            SizedBox(
              width: size,
              height: size,
              child: CircularProgressIndicator(
                strokeWidth: 2,
                valueColor: AlwaysStoppedAnimation<Color>(
                  theme.colorScheme.primary,
                ),
              ),
            ),
            const SizedBox(width: 12),
            Text(
              message,
              style: theme.textTheme.bodySmall?.copyWith(
                color: theme.colorScheme.onPrimaryContainer,
                fontWeight: FontWeight.w500,
              ),
            ),
          ],
        ),
      ),
    );
  }
}

/// Compact inline spinner for toolbar or status bar.
class CompactLoadingSpinner extends StatelessWidget {
  const CompactLoadingSpinner({super.key, this.size = 14, this.color});

  final double size;
  final Color? color;

  @override
  Widget build(BuildContext context) => Semantics(
    label: 'Loading',
    child: SizedBox(
      width: size,
      height: size,
      child: CircularProgressIndicator(
        strokeWidth: 2,
        valueColor: AlwaysStoppedAnimation<Color>(
          color ?? Theme.of(context).colorScheme.primary,
        ),
      ),
    ),
  );
}

/// Empty state widget for when no devices are connected.
class FlowEmptyState extends StatelessWidget {
  const FlowEmptyState({super.key});

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Scaffold(
      backgroundColor: const Color(0xFF121212),
      body: Center(
        child: Semantics(
          label:
              'No devices connected. Connect a device to populate your flow.',
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              // Empty state icon
              Container(
                width: 96,
                height: 96,
                decoration: BoxDecoration(
                  color: theme.colorScheme.surfaceContainerHighest,
                  shape: BoxShape.circle,
                ),
                child: const Icon(
                  Icons.usb_off_rounded,
                  size: 48,
                  color: Colors.grey,
                ),
              ),
              const SizedBox(height: 24),

              // Main message
              Text(
                'No devices connected',
                style: theme.textTheme.headlineSmall?.copyWith(
                  color: Colors.white,
                  fontWeight: FontWeight.w600,
                ),
              ),
              const SizedBox(height: 8),

              // Hint message
              Text(
                'Connect a device to populate your flow.',
                style: theme.textTheme.bodyMedium?.copyWith(
                  color: Colors.grey[400],
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

/// Empty state for disconnected devices.
class DisconnectedDevicesState extends StatelessWidget {
  const DisconnectedDevicesState({super.key});

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Scaffold(
      backgroundColor: const Color(0xFF121212),
      body: Center(
        child: Semantics(
          label: 'No connected devices. Waiting for device connection.',
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              // Waiting icon with subtle pulse
              const _PulsingIcon(
                icon: Icons.cable,
                size: 48,
                color: Colors.grey,
              ),
              const SizedBox(height: 24),

              // Main message
              Text(
                'Waiting for devices...',
                style: theme.textTheme.headlineSmall?.copyWith(
                  color: Colors.white,
                  fontWeight: FontWeight.w600,
                ),
              ),
              const SizedBox(height: 8),

              // Hint message
              Text(
                'No connected devices. Waiting for connection.',
                style: theme.textTheme.bodyMedium?.copyWith(
                  color: Colors.grey[400],
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

/// Icon with pulsing animation.
class _PulsingIcon extends StatefulWidget {
  const _PulsingIcon({
    required this.icon,
    required this.size,
    required this.color,
  });

  final IconData icon;
  final double size;
  final Color color;

  @override
  State<_PulsingIcon> createState() => _PulsingIconState();
}

class _PulsingIconState extends State<_PulsingIcon>
    with SingleTickerProviderStateMixin {
  late final AnimationController _controller;
  late final Animation<double> _animation;

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 1500),
    )..repeat(reverse: true);

    _animation = Tween<double>(
      begin: 0.6,
      end: 1,
    ).animate(CurvedAnimation(parent: _controller, curve: Curves.easeInOut));
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => FadeTransition(
    opacity: _animation,
    child: Icon(widget.icon, size: widget.size, color: widget.color),
  );
}
