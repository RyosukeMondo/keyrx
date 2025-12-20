/// Optional feature tour for KeyFlow with step-by-step highlights.
///
/// Provides an interactive walkthrough of KeyFlow features with spotlight effects
/// and contextual help overlays.
library;

import 'package:flutter/material.dart';
import 'package:shared_preferences/shared_preferences.dart';

/// Feature tour for KeyFlow showing key features with spotlight highlights.
///
/// The tour walks users through:
/// 1. Device nodes
/// 2. Edge connections
/// 3. Add slot button
/// 4. Keyboard shortcuts
/// 5. Canvas navigation
class FeatureTour extends StatefulWidget {
  const FeatureTour({required this.child, this.onComplete, super.key});

  final Widget child;
  final VoidCallback? onComplete;

  static const _tourCompletedKey = 'keyflow_feature_tour_completed';

  /// Check if the tour has been completed before.
  static Future<bool> isCompleted() async {
    final prefs = await SharedPreferences.getInstance();
    return prefs.getBool(_tourCompletedKey) ?? false;
  }

  /// Mark the tour as completed.
  static Future<void> markCompleted() async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.setBool(_tourCompletedKey, true);
  }

  /// Reset tour completion state (for testing).
  static Future<void> reset() async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.remove(_tourCompletedKey);
  }

  @override
  State<FeatureTour> createState() => _FeatureTourState();
}

class _FeatureTourState extends State<FeatureTour> {
  int _currentStep = -1; // -1 means tour is not active

  final _steps = const [
    _TourStep(
      title: 'Device Nodes',
      description:
          'These represent your physical keyboards. Green means connected and active. Click to see details.',
      targetType: _TargetType.deviceNode,
      position: _TooltipPosition.right,
    ),
    _TourStep(
      title: 'Edge Connections',
      description:
          'Solid lines show active slots, dashed lines show fallbacks. Numbers indicate priority order.',
      targetType: _TargetType.edge,
      position: _TooltipPosition.top,
    ),
    _TourStep(
      title: 'Add Slot Button',
      description:
          'Click the + button on device nodes to add new slot connections.',
      targetType: _TargetType.addButton,
      position: _TooltipPosition.right,
    ),
    _TourStep(
      title: 'Pan & Zoom',
      description:
          'Use mouse wheel or trackpad to zoom. Click and drag empty space to pan the canvas.',
      targetType: _TargetType.canvas,
      position: _TooltipPosition.center,
    ),
    _TourStep(
      title: 'Keyboard Shortcuts',
      description:
          'Press Ctrl+? anytime to see all keyboard shortcuts. Tab navigates nodes, arrow keys move them.',
      targetType: _TargetType.canvas,
      position: _TooltipPosition.center,
    ),
  ];

  /// Start the feature tour.
  void startTour() {
    setState(() => _currentStep = 0);
  }

  /// Move to the next step.
  void _nextStep() {
    if (_currentStep < _steps.length - 1) {
      setState(() => _currentStep++);
    } else {
      _completeTour();
    }
  }

  /// Move to the previous step.
  void _previousStep() {
    if (_currentStep > 0) {
      setState(() => _currentStep--);
    }
  }

  /// Skip the tour.
  void _skipTour() {
    _completeTour();
  }

  /// Complete the tour and mark as done.
  Future<void> _completeTour() async {
    await FeatureTour.markCompleted();
    setState(() => _currentStep = -1);
    widget.onComplete?.call();
  }

  @override
  Widget build(BuildContext context) {
    if (_currentStep < 0) {
      // Tour not active, just show child
      return widget.child;
    }

    final step = _steps[_currentStep];

    return Stack(
      children: [
        // Original content
        widget.child,

        // Dark overlay with spotlight cutout
        _buildOverlay(),

        // Tooltip with step information
        _buildTooltip(step),
      ],
    );
  }

  Widget _buildOverlay() => Positioned.fill(
    child: Container(
      color: Colors.black.withValues(alpha: 0.7),
      child: GestureDetector(
        onTap: () {}, // Prevent taps from reaching content below
      ),
    ),
  );

  Widget _buildTooltip(_TourStep step) {
    final theme = Theme.of(context);
    final isFirstStep = _currentStep == 0;
    final isLastStep = _currentStep == _steps.length - 1;

    final Widget tooltip = Container(
      width: 350,
      padding: const EdgeInsets.all(20),
      decoration: BoxDecoration(
        color: theme.colorScheme.surface,
        borderRadius: BorderRadius.circular(12),
        boxShadow: [
          BoxShadow(
            color: Colors.black.withValues(alpha: 0.3),
            blurRadius: 20,
            offset: const Offset(0, 4),
          ),
        ],
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Header
          Row(
            children: [
              Icon(
                Icons.lightbulb_outline,
                color: theme.colorScheme.primary,
                size: 28,
              ),
              const SizedBox(width: 12),
              Expanded(
                child: Text(
                  step.title,
                  style: theme.textTheme.titleLarge?.copyWith(
                    fontWeight: FontWeight.w600,
                  ),
                ),
              ),
              Text(
                '${_currentStep + 1}/${_steps.length}',
                style: theme.textTheme.bodySmall?.copyWith(
                  color: theme.colorScheme.onSurface.withValues(alpha: 0.6),
                ),
              ),
            ],
          ),
          const SizedBox(height: 12),

          // Description
          Text(step.description, style: theme.textTheme.bodyMedium),
          const SizedBox(height: 20),

          // Navigation buttons
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              // Previous button
              if (!isFirstStep)
                TextButton(
                  onPressed: _previousStep,
                  child: const Text('Previous'),
                )
              else
                const SizedBox.shrink(),

              // Skip button
              if (!isLastStep)
                TextButton(
                  onPressed: _skipTour,
                  child: const Text('Skip Tour'),
                ),

              // Next / Done button
              FilledButton(
                onPressed: _nextStep,
                child: Text(isLastStep ? 'Done' : 'Next'),
              ),
            ],
          ),
        ],
      ),
    );

    // Position tooltip based on step position preference
    return Positioned(
      left: step.position == _TooltipPosition.center ? null : 50,
      right: step.position == _TooltipPosition.right ? 50 : null,
      top: step.position == _TooltipPosition.center
          ? null
          : step.position == _TooltipPosition.top
          ? 100
          : null,
      bottom: step.position == _TooltipPosition.center
          ? null
          : step.position == _TooltipPosition.bottom
          ? 100
          : null,
      child: step.position == _TooltipPosition.center
          ? Center(child: tooltip)
          : tooltip,
    );
  }
}

/// Individual step in the feature tour.
class _TourStep {
  const _TourStep({
    required this.title,
    required this.description,
    required this.targetType,
    required this.position,
  });

  final String title;
  final String description;
  final _TargetType targetType;
  final _TooltipPosition position;
}

/// Type of element being highlighted.
enum _TargetType { deviceNode, edge, addButton, canvas }

/// Position of the tooltip relative to the spotlight.
enum _TooltipPosition { top, bottom, right, center }

/// Widget to trigger the feature tour.
///
/// Shows a button or hint to start the tour. Can be placed in the app bar
/// or as a floating action button.
class FeatureTourButton extends StatelessWidget {
  const FeatureTourButton({required this.onStart, super.key});

  final VoidCallback onStart;

  @override
  Widget build(BuildContext context) => IconButton(
    icon: const Icon(Icons.help_outline),
    tooltip: 'Start feature tour',
    onPressed: onStart,
  );
}

/// Hint widget to prompt users to take the tour.
///
/// Can be shown on first visit after dismissing the onboarding dialog.
class FeatureTourHint extends StatefulWidget {
  const FeatureTourHint({required this.onStart, super.key});

  final VoidCallback onStart;

  @override
  State<FeatureTourHint> createState() => _FeatureTourHintState();
}

class _FeatureTourHintState extends State<FeatureTourHint> {
  bool _dismissed = false;

  @override
  Widget build(BuildContext context) {
    if (_dismissed) return const SizedBox.shrink();

    final theme = Theme.of(context);

    return Container(
      margin: const EdgeInsets.all(16),
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: theme.colorScheme.primaryContainer,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(
          color: theme.colorScheme.primary.withValues(alpha: 0.5),
        ),
      ),
      child: Row(
        children: [
          Icon(
            Icons.tour_outlined,
            color: theme.colorScheme.onPrimaryContainer,
          ),
          const SizedBox(width: 12),
          Expanded(
            child: Text(
              'New to KeyFlow? Take a quick tour to learn the basics.',
              style: theme.textTheme.bodyMedium?.copyWith(
                color: theme.colorScheme.onPrimaryContainer,
              ),
            ),
          ),
          TextButton(
            onPressed: () {
              widget.onStart();
              setState(() => _dismissed = true);
            },
            child: const Text('Start Tour'),
          ),
          IconButton(
            icon: const Icon(Icons.close),
            iconSize: 18,
            onPressed: () => setState(() => _dismissed = true),
            tooltip: 'Dismiss',
          ),
        ],
      ),
    );
  }
}
