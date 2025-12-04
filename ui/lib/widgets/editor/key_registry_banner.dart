/// Widget for displaying the key registry status banner.
library;

import 'package:flutter/material.dart';

import '../common/styled_icon_button.dart';

class KeyRegistryBanner extends StatelessWidget {
  const KeyRegistryBanner({
    super.key, required this.isFetchingKeys, required this.usingFallbackKeys,
    required this.canonicalKeysCount, required this.registryError,
    required this.onRefresh,
  });

  final bool isFetchingKeys;
  final bool usingFallbackKeys;
  final int canonicalKeysCount;
  final String? registryError;
  final VoidCallback onRefresh;

  @override
  Widget build(BuildContext context) {
    final statusText = isFetchingKeys ? 'Fetching canonical keys...'
        : usingFallbackKeys ? 'Using fallback key list'
        : 'Loaded $canonicalKeysCount canonical keys';
    final color = usingFallbackKeys ? Colors.orange : Colors.green;
    return Card(
      margin: EdgeInsets.zero, elevation: 0,
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
        child: Row(children: [
          Icon(Icons.key, color: color),
          const SizedBox(width: 8),
          Expanded(child: Column(crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(statusText,
                  style: TextStyle(color: color, fontWeight: FontWeight.w600)),
              if (registryError != null)
                Text(registryError!, style: TextStyle(color: Colors.orange.shade700),
                    overflow: TextOverflow.ellipsis),
            ],
          )),
          if (isFetchingKeys)
            const SizedBox(width: 16, height: 16,
                child: CircularProgressIndicator(strokeWidth: 2))
          else
            StyledIconButton(
              icon: Icons.refresh,
              onPressed: onRefresh,
              tooltip: 'Refresh key registry',
            ),
        ]),
      ),
    );
  }
}
