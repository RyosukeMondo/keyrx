import 'package:flutter/material.dart';
import 'app_assets.dart';
import 'asset_button.dart';
import 'asset_icon.dart';

/// Example page demonstrating the use of generated SD assets
class AssetDemoPage extends StatelessWidget {
  const AssetDemoPage({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Container(
        // Use generated background
        decoration: const BoxDecoration(
          image: DecorationImage(
            image: AssetImage(AppAssets.mainBackground),
            fit: BoxFit.cover,
          ),
        ),
        child: SafeArea(
          child: Center(
            child: Container(
              // Panel with generated background
              padding: const EdgeInsets.all(24),
              margin: const EdgeInsets.all(32),
              decoration: BoxDecoration(
                image: const DecorationImage(
                  image: AssetImage(AppAssets.panelBackground),
                  fit: BoxFit.cover,
                ),
                borderRadius: BorderRadius.circular(16),
                boxShadow: [
                  BoxShadow(
                    color: Colors.black.withValues(alpha: 0.3),
                    blurRadius: 20,
                    offset: const Offset(0, 10),
                  ),
                ],
              ),
              child: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  // App Icon
                  Image.asset(
                    AppAssets.appIconBase,
                    width: 120,
                    height: 120,
                  ),
                  const SizedBox(height: 24),

                  // Title
                  const Text(
                    'KeyRx Configuration',
                    style: TextStyle(
                      fontSize: 24,
                      fontWeight: FontWeight.bold,
                      color: Colors.white,
                    ),
                  ),
                  const SizedBox(height: 32),

                  // Icons Row
                  Row(
                    mainAxisAlignment: MainAxisAlignment.spaceEvenly,
                    children: [
                      _buildIconCard(
                        icon: const AssetIcon(
                          type: AssetIconType.key,
                          size: 48,
                        ),
                        label: 'Keys',
                      ),
                      _buildIconCard(
                        icon: const AssetIcon(
                          type: AssetIconType.profile,
                          size: 48,
                        ),
                        label: 'Profiles',
                      ),
                      _buildIconCard(
                        icon: const AssetIcon(
                          type: AssetIconType.settings,
                          size: 48,
                        ),
                        label: 'Settings',
                      ),
                    ],
                  ),
                  const SizedBox(height: 32),

                  // Buttons
                  AssetButton(
                    text: 'Apply Configuration',
                    type: AssetButtonType.primary,
                    width: 300,
                    onPressed: () {
                      debugPrint('Primary action pressed');
                    },
                  ),
                  const SizedBox(height: 12),
                  AssetButton(
                    text: 'Load Default',
                    type: AssetButtonType.secondary,
                    width: 300,
                    onPressed: () {
                      debugPrint('Secondary action pressed');
                    },
                  ),
                  const SizedBox(height: 12),
                  AssetButton(
                    text: 'Reset All',
                    type: AssetButtonType.danger,
                    width: 300,
                    onPressed: () {
                      debugPrint('Danger action pressed');
                    },
                  ),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildIconCard({
    required Widget icon,
    required String label,
  }) {
    return Column(
      children: [
        Container(
          padding: const EdgeInsets.all(16),
          decoration: BoxDecoration(
            color: Colors.white.withValues(alpha: 0.1),
            borderRadius: BorderRadius.circular(12),
          ),
          child: icon,
        ),
        const SizedBox(height: 8),
        Text(
          label,
          style: const TextStyle(
            color: Colors.white70,
            fontSize: 14,
          ),
        ),
      ],
    );
  }
}

/// Example: Using assets in existing widgets
class IntegratedExample extends StatelessWidget {
  const IntegratedExample({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Row(
          children: [
            Image.asset(
              AppAssets.appIconBase,
              width: 32,
              height: 32,
            ),
            const SizedBox(width: 12),
            const Text('KeyRx'),
          ],
        ),
        actions: [
          IconButton(
            icon: const AssetIcon(
              type: AssetIconType.settings,
              size: 24,
            ),
            onPressed: () {},
          ),
        ],
      ),
      body: Stack(
        children: [
          // Background
          Image.asset(
            AppAssets.mainBackground,
            width: double.infinity,
            height: double.infinity,
            fit: BoxFit.cover,
          ),
          // Content
          Center(
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                const AssetIcon(
                  type: AssetIconType.key,
                  size: 64,
                ),
                const SizedBox(height: 24),
                AssetButton(
                  text: 'Configure Keys',
                  type: AssetButtonType.primary,
                  onPressed: () {},
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

/// Example: Custom button with asset background
class CustomAssetButton extends StatelessWidget {
  final String text;
  final VoidCallback onPressed;
  final String backgroundAsset;

  const CustomAssetButton({
    super.key,
    required this.text,
    required this.onPressed,
    required this.backgroundAsset,
  });

  @override
  Widget build(BuildContext context) {
    return InkWell(
      onTap: onPressed,
      borderRadius: BorderRadius.circular(8),
      child: Container(
        height: 48,
        padding: const EdgeInsets.symmetric(horizontal: 24),
        decoration: BoxDecoration(
          image: DecorationImage(
            image: AssetImage(backgroundAsset),
            fit: BoxFit.cover,
          ),
          borderRadius: BorderRadius.circular(8),
        ),
        child: Center(
          child: Text(
            text,
            style: const TextStyle(
              color: Colors.white,
              fontSize: 16,
              fontWeight: FontWeight.w600,
            ),
          ),
        ),
      ),
    );
  }
}

