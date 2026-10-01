import { useCallback, useEffect, useMemo } from 'react';
import {
  useDevices,
  useGlobalLayoutSetting,
  useSetGlobalLayout,
} from './useDevices';
import { useUpdateDevice } from './useUpdateDevice';
import { useToast } from './useToast';
import { t } from '@/i18n';
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
  const toast = useToast();

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

  /** Store `next` on the selected devices, or as the global default. */
  const persistLayout = useCallback(
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

  /**
   * User picked a layout. With devices selected it is saved per device and
   * announced with an Undo; with none selected it would silently change the
   * default for every device, so that case asks first.
   */
  const chooseLayout = useCallback(
    (next: LayoutType) => {
      const previous = layout;
      if (next === previous) return;
      const isGlobal = selectedDevices.length === 0;
      if (isGlobal && !window.confirm(t('layout.confirmGlobal'))) return;
      persistLayout(next);
      const message = isGlobal
        ? t('layout.changedGlobal', { layout: next })
        : t('layout.changedDevice', {
            layout: next,
            devices: selectedDevices.map((d) => d.name).join(', '),
          });
      toast.info(message, {
        duration: 8000,
        action: {
          label: t('toast.undo'),
          onClick: () => {
            persistLayout(previous);
            toast.info(t('layout.undone'));
          },
        },
      });
    },
    [layout, selectedDevices, persistLayout, toast]
  );

  return { layout, layoutKeys, chooseLayout };
}
