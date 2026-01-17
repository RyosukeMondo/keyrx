import { useEffect } from 'react';
import { useDevices } from '@/hooks/useDevices';
import type { KeyMapping } from '@/types';
import type { KeyMapping as RhaiKeyMapping } from '@/utils/rhaiParser';

interface UseASTSyncProps {
  syncEngine: {
    state: string;
    getAST: () => {
      globalMappings: RhaiKeyMapping[];
      deviceBlocks: {
        pattern: string;
        mappings: RhaiKeyMapping[];
        layers: {
          modifiers: string | string[];
          mappings: RhaiKeyMapping[];
        }[];
      }[];
    } | null;
  };
  configStore: {
    loadLayerMappings: (mappings: Map<string, Map<string, KeyMapping>>) => void;
  };
  globalSelected: boolean;
  selectedDevices: string[];
}

// Normalize key codes to VK_ format for consistent lookup
const normalizeKeyCode = (key: string): string => {
  if (!key) return key;
  if (key.startsWith('VK_')) return key;
  if (key.startsWith('KC_')) return key.replace(/^KC_/, 'VK_');
  if (/^[A-Z0-9]$/i.test(key)) return `VK_${key.toUpperCase()}`;

  const knownKeys = [
    'ESCAPE',
    'ENTER',
    'SPACE',
    'TAB',
    'BACKSPACE',
    'DELETE',
    'INSERT',
    'HOME',
    'END',
    'PAGEUP',
    'PAGEDOWN',
    'UP',
    'DOWN',
    'LEFT',
    'RIGHT',
    'CAPSLOCK',
    'NUMLOCK',
    'SCROLLLOCK',
    'LEFTSHIFT',
    'RIGHTSHIFT',
    'LEFTCONTROL',
    'RIGHTCONTROL',
    'LEFTALT',
    'RIGHTALT',
    'LEFTMETA',
    'RIGHTMETA',
  ];
  if (knownKeys.includes(key.toUpperCase()))
    return `VK_${key.toUpperCase()}`;
  return key;
};

// Convert RhaiKeyMapping to visual KeyMapping
const convertToVisualMapping = (m: RhaiKeyMapping): KeyMapping => {
  const visualMapping: KeyMapping = {
    type: m.type,
  };

  if (m.type === 'simple' && m.targetKey) {
    visualMapping.tapAction = m.targetKey;
  } else if (m.type === 'tap_hold' && m.tapHold) {
    visualMapping.tapAction = m.tapHold.tapAction;
    visualMapping.holdAction = m.tapHold.holdAction;
    visualMapping.threshold = m.tapHold.thresholdMs;
  } else if (m.type === 'macro' && m.macro) {
    visualMapping.macroSteps = m.macro.keys.map((key) => ({
      type: 'press' as const,
      key,
    }));
  } else if (m.type === 'layer_switch' && m.layerSwitch) {
    visualMapping.targetLayer = m.layerSwitch.layerId;
  }

  return visualMapping;
};

/**
 * Hook for syncing visual editor state from parsed AST (layer-aware)
 */
export function useASTSync({
  syncEngine,
  configStore,
  globalSelected,
  selectedDevices,
}: UseASTSyncProps) {
  const { data: devicesData } = useDevices();

  useEffect(() => {
    // Only sync when state is idle (parsing complete)
    if (syncEngine.state !== 'idle') return;

    const ast = syncEngine.getAST();
    if (!ast) return;

    // Build layer-aware mappings: Map<layerId, Map<keyCode, KeyMapping>>
    const layerMappings = new Map<string, Map<string, KeyMapping>>();

    // Initialize base layer
    layerMappings.set('base', new Map());

    // Process global mappings (including device_start("*") which is treated as global)
    if (globalSelected) {
      const baseMap = layerMappings.get('base')!;

      // Process top-level global mappings
      ast.globalMappings.forEach((m) => {
        baseMap.set(normalizeKeyCode(m.sourceKey), convertToVisualMapping(m));
      });

      // Also process device_start("*") block as global - "*" means all devices
      const wildcardBlock = ast.deviceBlocks.find(
        (block) => block.pattern === '*'
      );
      if (wildcardBlock) {
        wildcardBlock.mappings.forEach((m) => {
          baseMap.set(normalizeKeyCode(m.sourceKey), convertToVisualMapping(m));
        });

        // Also process layers from wildcard block
        wildcardBlock.layers.forEach((layer) => {
          const layerModifiers = Array.isArray(layer.modifiers)
            ? layer.modifiers
            : [layer.modifiers];
          layerModifiers.forEach((mod: string) => {
            const layerId = mod.toLowerCase().replace('_', '-');
            if (!layerMappings.has(layerId)) {
              layerMappings.set(layerId, new Map());
            }
            const layerMap = layerMappings.get(layerId)!;
            layer.mappings.forEach((m: RhaiKeyMapping) => {
              layerMap.set(
                normalizeKeyCode(m.sourceKey),
                convertToVisualMapping(m)
              );
            });
          });
        });
      }
    }

    // Process device-specific mappings for selected devices
    if (selectedDevices.length > 0) {
      ast.deviceBlocks.forEach((block) => {
        // Special handling for wildcard pattern "*" - applies to all devices
        const isWildcard = block.pattern === '*';

        // Check if this device block matches any selected device
        const matchesSelectedDevice = isWildcard
          ? selectedDevices.includes('disconnected-*') ||
            selectedDevices.length > 0
          : devicesData?.some((device) => {
              const isSelected = selectedDevices.includes(device.id);
              const matchesPattern =
                block.pattern === device.serial ||
                block.pattern === device.name ||
                block.pattern === device.id;
              return isSelected && matchesPattern;
            }) ?? false;

        if (matchesSelectedDevice) {
          // Add base mappings
          const baseMap = layerMappings.get('base')!;
          block.mappings.forEach((m) => {
            baseMap.set(
              normalizeKeyCode(m.sourceKey),
              convertToVisualMapping(m)
            );
          });

          // Add layer-specific mappings
          block.layers.forEach((layer) => {
            const layerModifiers = Array.isArray(layer.modifiers)
              ? layer.modifiers
              : [layer.modifiers];

            // Convert each modifier to layer ID format (MD_00 -> md-00)
            layerModifiers.forEach((mod: string) => {
              const layerId = mod.toLowerCase().replace('_', '-');

              if (!layerMappings.has(layerId)) {
                layerMappings.set(layerId, new Map());
              }

              const layerMap = layerMappings.get(layerId)!;
              layer.mappings.forEach((m: RhaiKeyMapping) => {
                layerMap.set(
                  normalizeKeyCode(m.sourceKey),
                  convertToVisualMapping(m)
                );
              });
            });
          });
        }
      });
    }

    // Load into store
    configStore.loadLayerMappings(layerMappings);
  }, [syncEngine.state, globalSelected, selectedDevices, devicesData, syncEngine, configStore]);
}
