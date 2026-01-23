import { useEffect, useState, useRef } from 'react';
import { useDevices } from '@/hooks/useDevices';
import { extractDevicePatterns, hasGlobalMappings, type RhaiAST } from '@/utils/rhaiParser';
import type { Device } from '@/components/DeviceSelector';

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
      // No AST yet, just use connected devices (filter out disabled devices)
      setMergedDevices(
        devicesData
          ?.filter((d) => d.enabled !== false)
          .map((d) => ({
            id: d.id,
            name: d.name,
            serial: d.serial || undefined,
            connected: true,
          })) || []
      );
      return;
    }

    // Extract device patterns from Rhai script
    const devicePatternsInRhai = extractDevicePatterns(ast);

    // Create a map of connected devices by serial/name/id (filter out disabled devices)
    const connectedDeviceMap = new Map<
      string,
      NonNullable<typeof devicesData>[number]
    >();
    devicesData
      ?.filter((device) => device.enabled !== false)
      .forEach((device) => {
        if (device.serial) connectedDeviceMap.set(device.serial, device);
        connectedDeviceMap.set(device.name, device);
        connectedDeviceMap.set(device.id, device);
      });

    // Build merged device list
    const merged: Device[] = [];
    const addedPatterns = new Set<string>();

    // Add devices from Rhai (may be disconnected)
    // Skip "*" pattern - it represents "all devices" and is handled by Global checkbox
    devicePatternsInRhai
      .filter((pattern) => pattern !== '*')
      .forEach((pattern) => {
        if (addedPatterns.has(pattern)) return;
        addedPatterns.add(pattern);

        // Try to find matching connected device
        const connectedDevice = connectedDeviceMap.get(pattern);
        if (connectedDevice) {
          // Device is both in Rhai and connected
          merged.push({
            id: connectedDevice.id,
            name: connectedDevice.name,
            serial: connectedDevice.serial || undefined,
            connected: true,
          });
        } else {
          // Device in Rhai but not connected (disconnected device)
          merged.push({
            id: `disconnected-${pattern}`,
            name: pattern,
            serial: pattern,
            connected: false,
          });
        }
      });

    // Add connected devices not in Rhai (filter out disabled devices)
    devicesData
      ?.filter((device) => device.enabled !== false)
      .forEach((device) => {
        const isInRhai =
          devicePatternsInRhai.includes(device.serial || '') ||
          devicePatternsInRhai.includes(device.name) ||
          devicePatternsInRhai.includes(device.id);

        if (!isInRhai) {
          merged.push({
            id: device.id,
            name: device.name,
            serial: device.serial || undefined,
            connected: true,
          });
        }
      });

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
      if (hasGlobalMappings(ast) || hasWildcardDevice) {
        configStore.setGlobalSelected(true);
      }

      // If Rhai has device blocks, auto-select those devices (excluding "*")
      const nonWildcardPatterns = devicePatternsInRhai.filter((p) => p !== '*');
      if (nonWildcardPatterns.length > 0) {
        const devicesToSelect = merged
          .filter((device) => {
            const pattern = device.serial || device.name;
            return nonWildcardPatterns.includes(pattern);
          })
          .map((device) => device.id);

        if (devicesToSelect.length > 0) {
          configStore.setSelectedDevices(devicesToSelect);
        }
      }
    }
    // Note: syncEngine and configStore objects excluded from deps to prevent infinite loops
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [syncEngine.state, syncEngine.getAST, devicesData, configStore.setGlobalSelected, configStore.setSelectedDevices]);

  return mergedDevices;
}
