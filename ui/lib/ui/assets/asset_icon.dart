import 'package:flutter/material.dart';
import 'app_assets.dart';

/// Custom icon widget using generated SD assets
class AssetIcon extends StatelessWidget {
  final AssetIconType type;
  final double size;
  final Color? tintColor;

  const AssetIcon({
    super.key,
    required this.type,
    this.size = 24,
    this.tintColor,
  });

  @override
  Widget build(BuildContext context) {
    final String assetPath = switch (type) {
      AssetIconType.key => AppAssets.keyIcon,
      AssetIconType.settings => AppAssets.settingsIcon,
      AssetIconType.profile => AppAssets.profileIcon,
    };

    return Image.asset(
      assetPath,
      width: size,
      height: size,
      color: tintColor,
      fit: BoxFit.contain,
    );
  }
}

enum AssetIconType { key, settings, profile }
