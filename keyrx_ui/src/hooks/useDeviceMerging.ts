import { useEffect, useState, useRef } from 'react';
import { useDevices } from '@/hooks/useDevices';
import {
  extractDevicePatterns,
  hasGlobalMappings,
  type RhaiAST,
} from '@/utils/rhaiParser';
import type { Device } from '@/components/DeviceSelector';
import { buildScopes } from '@/utils/deviceScopes';
import { remappableDevices } from '@/utils/deviceVisibility';

interface UseDeviceMergingProps {
  syncEngine: {
    state: string;
    getAST: () => RhaiAST | null;
  };
  configStore: {
    setGlobalSelected: (selected: boolean) => void;
    setSelectedDevices: (devices: string[]) => void;
  };
}

/**
 * Hook for merging connected devices with devices defined in Rhai configuration
 */
export function useDeviceMerging({
  syncEngine,
  configStore,
}: UseDeviceMergingProps) {
  const { data: devicesData } = useDevices();
  const [mergedDevices, setMergedDevices] = useState<Device[]>([]);
  // Track last processed AST to prevent infinite loops
  const lastASTRef = useRef<RhaiAST | null>(null);
  // Track if initial auto-selection has been done
  const initialSelectionDoneRef = useRef(false);

  useEffect(() => {
    const ast = syncEngine.getAST();
    if (!ast) {
      // No AST yet, just use connected devices
      setMergedDevices(
        remappableDevices(devicesData).map((d) => ({
          id: d.id,
          name: d.name,
          serial: d.serial || undefined,
          connected: true,
        }))
      );
      return;
    }

    // Extract device patterns from Rhai script
    const devicePatternsInRhai = extractDevicePatterns(ast);

    const merged = buildScopes(
      devicePatternsInRhai,
      remappableDevices(devicesData)
    );

    setMergedDevices(merged);

    // Only auto-populate device selector once per AST change
    // Skip if we've already processed this exact AST instance
    if (lastASTRef.current === ast && initialSelectionDoneRef.current) {
      return;
    }
    lastASTRef.current = ast;

    // Auto-populate device selector based on Rhai content (only on first load)
    if (!initialSelectionDoneRef.current) {
      initialSelectionDoneRef.current = true;

      const hasWildcardDevice = devicePatternsInRhai.includes('*');
      const hasGlobal = hasGlobalMappings(ast) || hasWildcardDevice;
      if (hasGlobal) {
        configStore.setGlobalSelected(true);
      } else if (devicePatternsInRhai.length > 0) {
        // The profile only has device blocks: land on those, not on an
        // empty Global keyboard.
        configStore.setGlobalSelected(false);
      }

      // If Rhai has device blocks, auto-select those devices (excluding "*")
      const nonWildcardPatterns = devicePatternsInRhai.filter((p) => p !== '*');
      if (nonWildcardPatterns.length > 0) {
        const devicesToSelect = merged
          .filter(
            (device) =>
              device.pattern !== undefined &&
              nonWildcardPatterns.includes(device.pattern)
          )
          .map((device) => device.id);

        if (devicesToSelect.length > 0) {
          configStore.setSelectedDevices(devicesToSelect);
        }
      }
    }
    // Note: syncEngine and configStore objects excluded from deps to prevent infinite loops
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [
    syncEngine.state,
    syncEngine.getAST,
    devicesData,
    configStore.setGlobalSelected,
    configStore.setSelectedDevices,
  ]);

  return mergedDevices;
}
