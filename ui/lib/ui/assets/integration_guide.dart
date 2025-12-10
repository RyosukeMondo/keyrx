/// Integration Guide: How to use generated SD assets in existing KeyRx UI
///
/// This file shows practical examples of integrating the generated assets
/// into your existing KeyRx application components.
library;

import 'package:flutter/material.dart';
import 'app_assets.dart';
import 'asset_button.dart';
import 'asset_icon.dart';

// ============================================================================
// Example 1: Enhancing Existing AppBar
// ============================================================================

class EnhancedAppBar extends StatelessWidget implements PreferredSizeWidget {
  final String title;

  const EnhancedAppBar({super.key, required this.title});

  @override
  Widget build(BuildContext context) {
    return AppBar(
      // Use app icon in title
      title: Row(
        children: [
          Image.asset(AppAssets.appIconBase, width: 32, height: 32),
          const SizedBox(width: 12),
          Text(title),
        ],
      ),
      // Use custom icons in actions
      actions: [
        IconButton(
          icon: const AssetIcon(type: AssetIconType.profile, size: 24),
          tooltip: 'Profiles',
          onPressed: () {},
        ),
        IconButton(
          icon: const AssetIcon(type: AssetIconType.settings, size: 24),
          tooltip: 'Settings',
          onPressed: () {},
        ),
      ],
    );
  }

  @override
  Size get preferredSize => const Size.fromHeight(kToolbarHeight);
}

// ============================================================================
// Example 2: Configuration Panel with Background
// ============================================================================

class ConfigurationPanel extends StatelessWidget {
  final String title;
  final Widget child;

  const ConfigurationPanel({
    super.key,
    required this.title,
    required this.child,
  });

  @override
  Widget build(BuildContext context) {
    return Container(
      margin: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        // Use generated panel background
        image: const DecorationImage(
          image: AssetImage(AppAssets.panelBackground),
          fit: BoxFit.cover,
        ),
        borderRadius: BorderRadius.circular(12),
        boxShadow: [
          BoxShadow(
            color: Colors.black.withValues(alpha: 0.2),
            blurRadius: 10,
            offset: const Offset(0, 4),
          ),
        ],
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          // Panel header
          Padding(
            padding: const EdgeInsets.all(16),
            child: Row(
              children: [
                const AssetIcon(type: AssetIconType.key, size: 24),
                const SizedBox(width: 12),
                Text(
                  title,
                  style: const TextStyle(
                    fontSize: 18,
                    fontWeight: FontWeight.bold,
                    color: Colors.white,
                  ),
                ),
              ],
            ),
          ),
          const Divider(color: Colors.white24),
          // Panel content
          Expanded(child: child),
        ],
      ),
    );
  }
}

// ============================================================================
// Example 3: Action Buttons for Forms
// ============================================================================

class FormActions extends StatelessWidget {
  final VoidCallback onSave;
  final VoidCallback onCancel;
  final VoidCallback? onDelete;

  const FormActions({
    super.key,
    required this.onSave,
    required this.onCancel,
    this.onDelete,
  });

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(16),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.end,
        children: [
          if (onDelete != null) ...[
            AssetButton(
              text: 'Delete',
              type: AssetButtonType.danger,
              onPressed: onDelete,
            ),
            const Spacer(),
          ],
          AssetButton(
            text: 'Cancel',
            type: AssetButtonType.secondary,
            onPressed: onCancel,
          ),
          const SizedBox(width: 12),
          AssetButton(
            text: 'Save',
            type: AssetButtonType.primary,
            onPressed: onSave,
          ),
        ],
      ),
    );
  }
}

// ============================================================================
// Example 4: Main App Layout with Background
// ============================================================================

class MainAppLayout extends StatelessWidget {
  final Widget child;

  const MainAppLayout({super.key, required this.child});

  @override
  Widget build(BuildContext context) {
    return Stack(
      children: [
        // Background image
        Positioned.fill(
          child: Image.asset(AppAssets.mainBackground, fit: BoxFit.cover),
        ),
        // Semi-transparent overlay for better readability
        Positioned.fill(
          child: Container(color: Colors.black.withValues(alpha: 0.3)),
        ),
        // Content
        child,
      ],
    );
  }
}

// ============================================================================
// Example 5: Navigation Drawer with Icons
// ============================================================================

class AppDrawer extends StatelessWidget {
  const AppDrawer({super.key});

  @override
  Widget build(BuildContext context) {
    return Drawer(
      child: Container(
        decoration: const BoxDecoration(
          image: DecorationImage(
            image: AssetImage(AppAssets.panelBackground),
            fit: BoxFit.cover,
          ),
        ),
        child: ListView(
          padding: EdgeInsets.zero,
          children: [
            DrawerHeader(
              decoration: const BoxDecoration(
                gradient: LinearGradient(colors: [Colors.blue, Colors.purple]),
              ),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Image.asset(AppAssets.appIconBase, width: 64, height: 64),
                  const SizedBox(height: 8),
                  const Text(
                    'KeyRx',
                    style: TextStyle(
                      color: Colors.white,
                      fontSize: 24,
                      fontWeight: FontWeight.bold,
                    ),
                  ),
                ],
              ),
            ),
            _DrawerItem(
              icon: const AssetIcon(type: AssetIconType.key, size: 24),
              title: 'Key Mappings',
              onTap: () {},
            ),
            _DrawerItem(
              icon: const AssetIcon(type: AssetIconType.profile, size: 24),
              title: 'Profiles',
              onTap: () {},
            ),
            _DrawerItem(
              icon: const AssetIcon(type: AssetIconType.settings, size: 24),
              title: 'Settings',
              onTap: () {},
            ),
          ],
        ),
      ),
    );
  }
}

class _DrawerItem extends StatelessWidget {
  final Widget icon;
  final String title;
  final VoidCallback onTap;

  const _DrawerItem({
    required this.icon,
    required this.title,
    required this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    return ListTile(
      leading: icon,
      title: Text(title, style: const TextStyle(color: Colors.white)),
      onTap: onTap,
    );
  }
}

// ============================================================================
// Example 6: Status Indicator with Icons
// ============================================================================

class StatusIndicator extends StatelessWidget {
  final bool isActive;
  final String label;

  const StatusIndicator({
    super.key,
    required this.isActive,
    required this.label,
  });

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
      decoration: BoxDecoration(
        image: DecorationImage(
          image: AssetImage(
            isActive ? AppAssets.primaryButton : AppAssets.secondaryButton,
          ),
          fit: BoxFit.cover,
        ),
        borderRadius: BorderRadius.circular(20),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          const AssetIcon(type: AssetIconType.key, size: 16),
          const SizedBox(width: 8),
          Text(
            label,
            style: const TextStyle(
              color: Colors.white,
              fontSize: 14,
              fontWeight: FontWeight.w600,
            ),
          ),
        ],
      ),
    );
  }
}

// ============================================================================
// Example 7: Full Page Example
// ============================================================================

class KeyMappingPage extends StatelessWidget {
  const KeyMappingPage({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: const EnhancedAppBar(title: 'Key Mappings'),
      drawer: const AppDrawer(),
      body: MainAppLayout(
        child: SafeArea(
          child: Column(
            children: [
              // Status bar
              Container(
                padding: const EdgeInsets.all(16),
                child: const Row(
                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                  children: [
                    StatusIndicator(isActive: true, label: 'Active'),
                    StatusIndicator(isActive: false, label: 'Inactive'),
                  ],
                ),
              ),
              // Configuration panel
              Expanded(
                child: ConfigurationPanel(
                  title: 'Current Profile',
                  child: ListView(
                    padding: const EdgeInsets.all(16),
                    children: [
                      const Text(
                        'Configure your key mappings here...',
                        style: TextStyle(color: Colors.white70),
                      ),
                      const SizedBox(height: 16),
                      // Your configuration UI here
                    ],
                  ),
                ),
              ),
              // Action buttons
              FormActions(onSave: () {}, onCancel: () {}, onDelete: () {}),
            ],
          ),
        ),
      ),
    );
  }
}

// ============================================================================
// Example 8: Dialog with Custom Background
// ============================================================================

class AssetDialog extends StatelessWidget {
  final String title;
  final String message;
  final VoidCallback onConfirm;
  final VoidCallback onCancel;

  const AssetDialog({
    super.key,
    required this.title,
    required this.message,
    required this.onConfirm,
    required this.onCancel,
  });

  @override
  Widget build(BuildContext context) {
    return Dialog(
      backgroundColor: Colors.transparent,
      child: Container(
        decoration: BoxDecoration(
          image: const DecorationImage(
            image: AssetImage(AppAssets.panelBackground),
            fit: BoxFit.cover,
          ),
          borderRadius: BorderRadius.circular(16),
        ),
        padding: const EdgeInsets.all(24),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const AssetIcon(type: AssetIconType.settings, size: 48),
            const SizedBox(height: 16),
            Text(
              title,
              style: const TextStyle(
                fontSize: 20,
                fontWeight: FontWeight.bold,
                color: Colors.white,
              ),
            ),
            const SizedBox(height: 12),
            Text(
              message,
              style: const TextStyle(color: Colors.white70),
              textAlign: TextAlign.center,
            ),
            const SizedBox(height: 24),
            Row(
              mainAxisAlignment: MainAxisAlignment.end,
              children: [
                AssetButton(
                  text: 'Cancel',
                  type: AssetButtonType.secondary,
                  onPressed: onCancel,
                ),
                const SizedBox(width: 12),
                AssetButton(
                  text: 'Confirm',
                  type: AssetButtonType.primary,
                  onPressed: onConfirm,
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}

// ============================================================================
// Usage Tips:
// ============================================================================
//
// 1. Replace Material Icons with AssetIcon where appropriate
// 2. Use AssetButton for primary actions to match the theme
// 3. Apply backgrounds to panels and cards for visual consistency
// 4. Use app icon in splash screens and about dialogs
// 5. Maintain the blue-purple gradient theme throughout the app
// 6. Consider tinting AssetIcon with your theme colors
// 7. Use the main background sparingly to avoid overwhelming the UI
//
// ============================================================================
