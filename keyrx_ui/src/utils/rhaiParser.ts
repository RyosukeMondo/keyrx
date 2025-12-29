/**
 * Rhai configuration parser
 *
 * Parses Rhai configuration syntax back into ConfigState.
 * Supports basic mappings, modifiers, locks, and layers.
 * Warns on unsupported features.
 */

import {
  ConfigState,
  Layer,
  Modifier,
  Lock,
  Mapping,
  RhaiParseResult,
} from '../types/configBuilder';

/**
 * Parse Rhai configuration code into ConfigState
 */
export function parseRhaiCode(rhaiCode: string): RhaiParseResult {
  const warnings: string[] = [];
  const errors: string[] = [];

  try {
    const lines = rhaiCode.split('\n').map(l => l.trim());
    const modifiers: Modifier[] = [];
    const locks: Lock[] = [];
    const layers: Layer[] = [];

    // Track modifier and lock IDs to index mapping
    const modifierIdMap = new Map<string, number>();
    const lockIdMap = new Map<string, number>();

    // Initialize base layer
    const baseLayer: Layer = {
      id: 'base-layer',
      name: 'Base',
      mappings: [],
      isBase: true,
    };

    let currentLayer: Layer | null = null;
    let insideDeviceBlock = false;
    let insideWhenBlock = false;
    let whenCondition = '';

    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];

      // Skip comments and empty lines
      if (!line || line.startsWith('//')) continue;

      // Device block
      if (line.startsWith('device_start(')) {
        insideDeviceBlock = true;
        continue;
      }

      if (line.startsWith('device_end()')) {
        insideDeviceBlock = false;
        continue;
      }

      // When block (conditional layer)
      const whenMatch = line.match(/when\("([^"]+)",\s*\[/);
      if (whenMatch) {
        insideWhenBlock = true;
        whenCondition = whenMatch[1];

        // Create a new layer for this when block
        const layerName = `Layer ${layers.length + 1}`;
        currentLayer = {
          id: `layer-${Date.now()}-${Math.random()}`,
          name: layerName,
          mappings: [],
          isBase: false,
        };
        continue;
      }

      // End of when block
      if (line.startsWith(']);') && insideWhenBlock) {
        insideWhenBlock = false;
        if (currentLayer) {
          layers.push(currentLayer);
          currentLayer = null;
        }
        whenCondition = '';
        continue;
      }

      // Parse map() statements
      const mapMatch = line.match(/map\("([^"]+)",\s*"([^"]+)"\)[,;]?/);
      if (mapMatch) {
        const [, sourceKey, targetKey] = mapMatch;

        // Check if this is a modifier definition
        if (targetKey.startsWith('MD_')) {
          const modifierIndex = parseInt(targetKey.substring(3), 16);
          const modifier: Modifier = {
            id: `modifier-${Date.now()}-${Math.random()}`,
            name: `Modifier ${modifierIndex}`,
            triggerKey: addKeyPrefix(sourceKey),
          };
          modifiers.push(modifier);
          modifierIdMap.set(targetKey, modifiers.length - 1);
          continue;
        }

        // Check if this is a lock definition
        if (targetKey.startsWith('LK_')) {
          const lockIndex = parseInt(targetKey.substring(3), 16);
          const lock: Lock = {
            id: `lock-${Date.now()}-${Math.random()}`,
            name: `Lock ${lockIndex}`,
            triggerKey: addKeyPrefix(sourceKey),
          };
          locks.push(lock);
          lockIdMap.set(targetKey, locks.length - 1);
          continue;
        }

        // Regular mapping
        const mapping: Mapping = {
          id: `mapping-${Date.now()}-${Math.random()}`,
          sourceKey: addKeyPrefix(sourceKey),
          targetKey: normalizeTargetKey(targetKey),
          type: 'simple',
        };

        // If we're inside a when block, add to current layer
        if (insideWhenBlock && currentLayer) {
          currentLayer.mappings.push(mapping);
        } else {
          // Base layer mapping
          baseLayer.mappings.push(mapping);
        }
        continue;
      }

      // Warn about unsupported syntax
      if (insideDeviceBlock && line.length > 0) {
        // Check for unsupported features
        if (
          line.includes('tap_hold') ||
          line.includes('macro') ||
          line.includes('sequence')
        ) {
          warnings.push(
            `Line ${i + 1}: Unsupported feature "${line}" - basic mappings only`
          );
        }
      }
    }

    // Always add base layer first
    layers.unshift(baseLayer);

    const config: ConfigState = {
      layers,
      modifiers,
      locks,
    };

    return {
      config,
      warnings,
      errors,
    };
  } catch (error) {
    errors.push(`Parse error: ${error instanceof Error ? error.message : String(error)}`);
    return {
      config: {
        layers: [
          {
            id: 'base-layer',
            name: 'Base',
            mappings: [],
            isBase: true,
          },
        ],
        modifiers: [],
        locks: [],
      },
      warnings,
      errors,
    };
  }
}

/**
 * Add KEY_ prefix to key code if not present
 */
function addKeyPrefix(keyCode: string): string {
  if (
    keyCode.startsWith('KEY_') ||
    keyCode.startsWith('VK_') ||
    keyCode.startsWith('MD_') ||
    keyCode.startsWith('LK_')
  ) {
    return keyCode;
  }
  return `KEY_${keyCode}`;
}

/**
 * Normalize target key by converting VK_ to KEY_
 */
function normalizeTargetKey(keyCode: string): string {
  if (keyCode.startsWith('VK_')) {
    return `KEY_${keyCode.substring(3)}`;
  }
  if (keyCode.startsWith('KEY_')) {
    return keyCode;
  }
  if (keyCode.startsWith('MD_') || keyCode.startsWith('LK_')) {
    return keyCode; // Keep modifier/lock IDs as-is
  }
  return `KEY_${keyCode}`;
}
