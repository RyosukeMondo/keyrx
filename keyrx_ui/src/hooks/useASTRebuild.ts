import { useCallback } from 'react';
import { useDevices } from '@/hooks/useDevices';
import type { KeyMapping } from '@/types';
import type {
  KeyMapping as RhaiKeyMapping,
  RhaiAST,
  DeviceBlock,
} from '@/utils/rhaiParser';
import type { Device } from '@/components/DeviceSelector';

interface UseASTRebuildProps {
  configStore: {
    getAllLayers: () => string[];
    getLayerMappings: (layerId: string) => Map<string, KeyMapping>;
  };
  syncEngine: {
    onVisualChange: (ast: RhaiAST) => void;
  };
  globalSelected: boolean;
  selectedDevices: string[];
  devices: Device[];
}

/**
 * Hook for rebuilding AST from visual editor state and syncing to code editor
 */
export function useASTRebuild({
  configStore,
  syncEngine,
  globalSelected,
  selectedDevices,
  devices,
}: UseASTRebuildProps) {
  useDevices();

  const rebuildAndSyncAST = useCallback(() => {
    // Convert a KeyMapping to RhaiKeyMapping
    const convertToRhaiMapping = (
      key: string,
      m: KeyMapping
    ): RhaiKeyMapping => {
      // Map internal types to Rhai-compatible types
      // modifier, lock, layer_active are treated as 'simple' for Rhai output
      const rhaiType: RhaiKeyMapping['type'] =
        m.type === 'modifier' || m.type === 'lock' || m.type === 'layer_active'
          ? 'simple'
          : m.type;

      const baseMapping: RhaiKeyMapping = {
        type: rhaiType,
        sourceKey: key,
        line: 0,
      };

      if (m.type === 'simple' && m.tapAction) {
        baseMapping.targetKey = m.tapAction;
      } else if (m.type === 'tap_hold' && m.tapAction && m.holdAction) {
        baseMapping.tapHold = {
          tapAction: m.tapAction,
          holdAction: m.holdAction,
          thresholdMs: m.threshold || 200,
        };
      } else if (m.type === 'macro' && m.macroSteps) {
        baseMapping.macro = {
          keys: m.macroSteps.filter((s) => s.key).map((s) => s.key!),
          delayMs: m.macroSteps.find((s) => s.delayMs)?.delayMs,
        };
      } else if (m.type === 'layer_switch' && m.targetLayer) {
        baseMapping.layerSwitch = {
          layerId: m.targetLayer,
        };
      }

      return baseMapping;
    };

    // Get all layers from store
    const allLayers = configStore.getAllLayers();

    // Helper: build layer structures from store (non-base layers)
    const buildLayers = () =>
      allLayers
        .filter((layerId) => layerId !== 'base')
        .map((layerId) => {
          const layerMappings = configStore.getLayerMappings(layerId);
          const rhaiMappings: RhaiKeyMapping[] = [];
          layerMappings.forEach((mapping, key) => {
            rhaiMappings.push(convertToRhaiMapping(key, mapping));
          });
          // Convert layer ID to modifier format (md-00 -> MD_00)
          const modifierName = layerId.toUpperCase().replace('-', '_');
          return {
            modifiers: [modifierName],
            mappings: rhaiMappings,
            startLine: 0,
            endLine: 0,
          };
        })
        .filter((layer) => layer.mappings.length > 0);

    // Helper: build base mappings from store
    const buildBaseMappings = (): RhaiKeyMapping[] => {
      const baseMappings = configStore.getLayerMappings('base');
      const result: RhaiKeyMapping[] = [];
      baseMappings.forEach((mapping, key) => {
        result.push(convertToRhaiMapping(key, mapping));
      });
      return result;
    };

    const globalMappings: RhaiKeyMapping[] = [];
    const deviceBlocks: DeviceBlock[] = [];

    if (globalSelected) {
      // When global is selected, wrap mappings in device_start("*") block
      // to preserve the wildcard device pattern and layer structures
      deviceBlocks.push({
        pattern: '*',
        mappings: buildBaseMappings(),
        layers: buildLayers(),
        startLine: 0,
        endLine: 0,
      });
    }

    // Build device-specific blocks for selected devices
    selectedDevices.forEach((deviceId) => {
      const device = devices.find((d) => d.id === deviceId);
      if (!device) return;

      deviceBlocks.push({
        pattern: device.serial || device.name,
        mappings: buildBaseMappings(),
        layers: buildLayers(),
        startLine: 0,
        endLine: 0,
      });
    });

    // Update sync engine with new AST
    syncEngine.onVisualChange({
      imports: [],
      globalMappings,
      deviceBlocks,
      comments: [],
    });
  }, [configStore, syncEngine, globalSelected, selectedDevices, devices]);

  return rebuildAndSyncAST;
}
