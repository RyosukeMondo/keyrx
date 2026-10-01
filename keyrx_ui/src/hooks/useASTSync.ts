import { useEffect, useRef } from 'react';
import { useDevices } from '@/hooks/useDevices';
import type { KeyMapping } from '@/types';
import { buildScopes, scopeMatchesBlock } from '@/utils/deviceScopes';
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

// Map Japanese VK names to English equivalents for layout matching
const japaneseKeyMap: Record<string, string> = {
  VK_無変換: 'VK_Muhenkan',
  VK_変換: 'VK_Henkan',
  VK_ひらがな: 'VK_Hiragana',
  VK_カタカナ: 'VK_Katakana',
  VK_全角: 'VK_Zenkaku',
};

// Normalize key codes to VK_ format for consistent lookup
const normalizeKeyCode = (key: string): string => {
  if (!key) return key;
  if (japaneseKeyMap[key]) return japaneseKeyMap[key];
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
  if (knownKeys.includes(key.toUpperCase())) return `VK_${key.toUpperCase()}`;
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
  // Track the last processed (AST, selection, devices) triple to prevent
  // infinite loops. The AST alone is not enough: the selection and the
  // connected-device list arrive after the AST and must re-project it,
  // otherwise the editor stays empty until the AST object changes.
  const lastInputRef = useRef<{ ast: object; signature: string } | null>(null);

  useEffect(() => {
    // Only sync when state is idle (parsing complete)
    if (syncEngine.state !== 'idle') return;

    const ast = syncEngine.getAST();
    if (!ast) return;

    const signature = JSON.stringify([
      globalSelected,
      selectedDevices,
      (devicesData ?? []).map((d) => [d.id, d.name, d.serial, d.path]),
    ]);
    const last = lastInputRef.current;
    if (last && last.ast === ast && last.signature === signature) return;
    lastInputRef.current = { ast, signature };

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
      const connected = devicesData ?? [];
      const scopes = buildScopes(
        ast.deviceBlocks.map((block) => block.pattern),
        connected
      );
      ast.deviceBlocks.forEach((block) => {
        // Wildcard pattern "*" applies to all devices; any other pattern is
        // resolved with the daemon's glob rule (see utils/devicePattern).
        const matchesSelectedDevice =
          block.pattern === '*' ||
          scopes.some(
            (scope) =>
              selectedDevices.includes(scope.id) &&
              scopeMatchesBlock(scope, connected, block.pattern)
          );

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
    // Note: syncEngine and configStore objects are excluded from deps to prevent infinite loops
    // We depend on syncEngine.state for timing, and getAST is a stable memoized function
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [
    syncEngine.state,
    syncEngine.getAST,
    globalSelected,
    selectedDevices,
    devicesData,
    configStore.loadLayerMappings,
  ]);
}
