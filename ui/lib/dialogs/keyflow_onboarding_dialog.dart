/// Onboarding dialog for first-time KeyFlow users.
///
/// Explains core concepts: devices, slots, priorities, and how to use the flow editor.
library;

import 'package:flutter/material.dart';
import 'package:shared_preferences/shared_preferences.dart';

/// Shows the KeyFlow onboarding dialog on first visit.
///
/// Users can dismiss with "Don't show again" checkbox to skip future displays.
class KeyFlowOnboardingDialog extends StatefulWidget {
  const KeyFlowOnboardingDialog({super.key});

  static const _onboardingSeenKey = 'keyflow_onboarding_seen';

  /// Check if onboarding has been seen before.
  static Future<bool> shouldShow() async {
    final prefs = await SharedPreferences.getInstance();
    return !(prefs.getBool(_onboardingSeenKey) ?? false);
  }

  /// Mark onboarding as seen.
  static Future<void> markAsSeen() async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.setBool(_onboardingSeenKey, true);
  }

  /// Show the onboarding dialog if not seen before.
  static Future<void> showIfNeeded(BuildContext context) async {
    if (await shouldShow()) {
      if (!context.mounted) return;
      await show(context);
    }
  }

  /// Show the onboarding dialog.
  static Future<void> show(BuildContext context) => showDialog<void>(
    context: context,
    barrierDismissible: false,
    builder: (context) => const KeyFlowOnboardingDialog(),
  );

  @override
  State<KeyFlowOnboardingDialog> createState() =>
      _KeyFlowOnboardingDialogState();
}

class _KeyFlowOnboardingDialogState extends State<KeyFlowOnboardingDialog> {
  bool _dontShowAgain = false;
  int _currentPage = 0;

  final _pages = const [
    _OnboardingPage(
      icon: Icons.account_tree_outlined,
      title: 'Welcome to KeyFlow',
      description:
          'KeyFlow visualizes how your keyboard devices connect to profiles and keymaps.',
      details: [
        'See all your devices and their configurations at a glance',
        'Understand the relationship between devices, profiles, and keymaps',
        'Manage slot connections with drag-and-drop simplicity',
      ],
    ),
    _OnboardingPage(
      icon: Icons.devices,
      title: 'Devices, Profiles & Keymaps',
      description: 'KeyFlow shows three types of nodes connected by edges.',
      details: [
        'Devices: Your physical keyboards (left column)',
        'Profiles: Hardware layout definitions (middle column)',
        'Keymaps: Your key bindings and layers (right column)',
        'Edges: Connections showing slot relationships',
      ],
    ),
    _OnboardingPage(
      icon: Icons.link,
      title: 'Slots & Priority',
      description:
          'Slots connect devices to profiles and keymaps with a priority order.',
      details: [
        'Active slot (solid line): Currently in use',
        'Fallback slot (dashed line): Backup configuration',
        'Priority 1: Tried first (highest priority)',
        'Priority 2+: Fallback if priority 1 fails',
        'Drag priority badges to reorder slots',
      ],
    ),
    _OnboardingPage(
      icon: Icons.touch_app,
      title: 'Interact with the Flow',
      description:
          'KeyFlow is fully interactive. Move nodes, manage slots, and more.',
      details: [
        'Pan & Zoom: Navigate the canvas freely',
        'Drag nodes: Rearrange layout to your preference',
        'Click edges: Select and manage slot connections',
        'Add slots: Click + on device nodes',
        'Delete slots: Right-click edge or press Delete',
      ],
    ),
    _OnboardingPage(
      icon: Icons.keyboard_outlined,
      title: 'Keyboard Shortcuts',
      description: 'KeyFlow is fully keyboard-accessible.',
      details: [
        'Tab: Navigate between nodes',
        'Arrow keys: Move focused node',
        'Enter: Open node details',
        'Delete: Remove selected edge',
        'Ctrl+0: Reset zoom',
        'Ctrl+?: Show all shortcuts',
      ],
    ),
  ];

  void _nextPage() {
    if (_currentPage < _pages.length - 1) {
      setState(() => _currentPage++);
    } else {
      _finish();
    }
  }

  void _previousPage() {
    if (_currentPage > 0) {
      setState(() => _currentPage--);
    }
  }

  void _skip() {
    _finish();
  }

  Future<void> _finish() async {
    if (_dontShowAgain) {
      await KeyFlowOnboardingDialog.markAsSeen();
    }
    if (!mounted) return;
    Navigator.of(context).pop();
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final page = _pages[_currentPage];
    final isLastPage = _currentPage == _pages.length - 1;

    return Dialog(
      child: Container(
        width: 600,
        constraints: const BoxConstraints(maxHeight: 700),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            // Header
            Padding(
              padding: const EdgeInsets.all(24),
              child: Row(
                children: [
                  Icon(page.icon, size: 32, color: theme.colorScheme.primary),
                  const SizedBox(width: 12),
                  Expanded(
                    child: Text(
                      page.title,
                      style: theme.textTheme.headlineSmall?.copyWith(
                        fontWeight: FontWeight.w600,
                      ),
                    ),
                  ),
                  IconButton(
                    icon: const Icon(Icons.close),
                    onPressed: _skip,
                    tooltip: 'Skip onboarding',
                  ),
                ],
              ),
            ),
            const Divider(height: 1),

            // Content
            Flexible(
              child: SingleChildScrollView(
                padding: const EdgeInsets.all(24),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    // Description
                    Text(
                      page.description,
                      style: theme.textTheme.bodyLarge?.copyWith(
                        color: theme.colorScheme.onSurface.withOpacity(0.8),
                      ),
                    ),
                    const SizedBox(height: 24),

                    // Details
                    ...page.details.map(
                      (detail) => Padding(
                        padding: const EdgeInsets.only(bottom: 12),
                        child: Row(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Icon(
                              Icons.check_circle,
                              size: 20,
                              color: theme.colorScheme.primary,
                            ),
                            const SizedBox(width: 12),
                            Expanded(
                              child: Text(
                                detail,
                                style: theme.textTheme.bodyMedium,
                              ),
                            ),
                          ],
                        ),
                      ),
                    ),
                  ],
                ),
              ),
            ),

            // Page indicator
            Padding(
              padding: const EdgeInsets.symmetric(vertical: 16),
              child: Row(
                mainAxisAlignment: MainAxisAlignment.center,
                children: List.generate(
                  _pages.length,
                  (index) => Container(
                    margin: const EdgeInsets.symmetric(horizontal: 4),
                    width: 8,
                    height: 8,
                    decoration: BoxDecoration(
                      shape: BoxShape.circle,
                      color: index == _currentPage
                          ? theme.colorScheme.primary
                          : theme.colorScheme.outline.withOpacity(0.3),
                    ),
                  ),
                ),
              ),
            ),

            const Divider(height: 1),

            // Footer
            Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                children: [
                  // Don't show again checkbox
                  Row(
                    children: [
                      Checkbox(
                        value: _dontShowAgain,
                        onChanged: (value) {
                          setState(() => _dontShowAgain = value ?? false);
                        },
                      ),
                      Expanded(
                        child: GestureDetector(
                          onTap: () {
                            setState(() => _dontShowAgain = !_dontShowAgain);
                          },
                          child: Text(
                            "Don't show this again",
                            style: theme.textTheme.bodyMedium,
                          ),
                        ),
                      ),
                    ],
                  ),
                  const SizedBox(height: 12),

                  // Navigation buttons
                  Row(
                    mainAxisAlignment: MainAxisAlignment.spaceBetween,
                    children: [
                      // Previous button
                      if (_currentPage > 0)
                        TextButton(
                          onPressed: _previousPage,
                          child: const Text('Previous'),
                        )
                      else
                        const SizedBox.shrink(),

                      // Skip button (only on first pages)
                      if (!isLastPage)
                        TextButton(onPressed: _skip, child: const Text('Skip')),

                      // Next / Get Started button
                      FilledButton(
                        onPressed: _nextPage,
                        child: Text(isLastPage ? 'Get Started' : 'Next'),
                      ),
                    ],
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}

/// Individual page in the onboarding flow.
class _OnboardingPage {
  const _OnboardingPage({
    required this.icon,
    required this.title,
    required this.description,
    required this.details,
  });

  final IconData icon;
  final String title;
  final String description;
  final List<String> details;
}
