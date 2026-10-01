import { useCallback, useEffect, useMemo } from 'react';
import {
  useDevices,
  useGlobalLayoutSetting,
  useSetGlobalLayout,
} from './useDevices';
import { useUpdateDevice } from './useUpdateDevice';
import { useKeyboardLayout } from './useKeyboardLayout';
import type { LayoutType } from '@/components/KeyboardVisualizer';
import type { RhaiAST } from '@/utils/rhaiParser';
import { buildScopes } from '@/utils/deviceScopes';
import {
  detectLayout,
  devicesForScopes,
  resolveStoredLayout,
} from '@/utils/layoutPreference';

interface UseLayoutPreferenceArgs {
  source: string | undefined;
  ast: RhaiAST | null;
  globalSelected: boolean;
  selectedScopeIds: string[];
}

/**
 * Keyboard layout for the config page, remembered per device.
 *
 * Reads the layout stored on the selected device(s) (or the global default),
 * falls back to detection, and persists every user choice back to the daemon
 * so it is still there after a reload or a profile switch.
 */
export function useLayoutPreference({
  source,
  ast,
  globalSelected,
  selectedScopeIds,
}: UseLayoutPreferenceArgs) {
  const { data: devices = [] } = useDevices();
  const { data: globalLayout } = useGlobalLayoutSetting();
  const { mutate: updateDevice } = useUpdateDevice();
  const { mutate: setGlobalLayout } = useSetGlobalLayout();

  const selectedDevices = useMemo(() => {
    const scopes = buildScopes(
      (ast?.deviceBlocks ?? []).map((b) => b.pattern),
      devices
    ).filter((s) => selectedScopeIds.includes(s.id));
    return devicesForScopes(scopes, devices);
  }, [ast, devices, selectedScopeIds]);

  const stored = resolveStoredLayout(selectedDevices, globalLayout);
  const detected = detectLayout(
    source,
    selectedDevices.length > 0 || globalSelected
      ? selectedDevices.length > 0
        ? selectedDevices
        : devices.filter((d) => !d.isVirtual)
      : []
  );
  const effective: LayoutType = stored ?? detected;

  const { layout, setLayout, layoutKeys } = useKeyboardLayout(effective);
  useEffect(() => {
    setLayout(effective);
  }, [effective, setLayout]);

  /** User picked a layout: show it now and remember it. */
  const chooseLayout = useCallback(
    (next: LayoutType) => {
      setLayout(next);
      if (selectedDevices.length > 0) {
        selectedDevices.forEach((d) =>
          updateDevice({ id: d.id, layout: next })
        );
      } else {
        setGlobalLayout(next);
      }
    },
    [selectedDevices, setLayout, updateDevice, setGlobalLayout]
  );

  return { layout, layoutKeys, chooseLayout };
}
