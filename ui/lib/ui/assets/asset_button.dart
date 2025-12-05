import 'package:flutter/material.dart';
import 'app_assets.dart';

/// Custom button widget using generated SD assets
class AssetButton extends StatelessWidget {
  final String text;
  final VoidCallback? onPressed;
  final AssetButtonType type;
  final double? width;
  final double? height;

  const AssetButton({
    super.key,
    required this.text,
    required this.onPressed,
    this.type = AssetButtonType.primary,
    this.width,
    this.height,
  });

  @override
  Widget build(BuildContext context) {
    final String assetPath = switch (type) {
      AssetButtonType.primary => AppAssets.primaryButton,
      AssetButtonType.secondary => AppAssets.secondaryButton,
      AssetButtonType.danger => AppAssets.dangerButton,
    };

    final Color textColor = switch (type) {
      AssetButtonType.primary => Colors.white,
      AssetButtonType.secondary => Colors.white70,
      AssetButtonType.danger => Colors.white,
    };

    return SizedBox(
      width: width,
      height: height ?? 48,
      child: Material(
        color: Colors.transparent,
        child: InkWell(
          onTap: onPressed,
          borderRadius: BorderRadius.circular(8),
          child: Ink(
            decoration: BoxDecoration(
              image: DecorationImage(
                image: AssetImage(assetPath),
                fit: BoxFit.cover,
              ),
              borderRadius: BorderRadius.circular(8),
            ),
            child: Center(
              child: Text(
                text,
                style: TextStyle(
                  color: textColor,
                  fontSize: 16,
                  fontWeight: FontWeight.w600,
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }
}

enum AssetButtonType {
  primary,
  secondary,
  danger,
}
