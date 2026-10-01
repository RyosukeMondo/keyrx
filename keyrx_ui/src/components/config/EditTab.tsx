import React, { useState } from 'react';
import { Card } from '@/components/Card';
import { type Device } from '@/components/DeviceSelector';
import { KeyConfigPanel } from '@/components/KeyConfigPanel';
import type { KeyMapping } from '@/types';
import type { LayoutType } from '@/components/KeyboardVisualizer';
import type { RhaiSyncEngineResult } from '@/components/RhaiSyncEngine';
import type { SyncStatus } from '@/hooks/useConfigSync';
import type { SVGKeyData } from '@/utils/kle-parser';
import { useDeviceMerging } from '@/hooks/useDeviceMerging';
import { useASTRebuild } from '@/hooks/useASTRebuild';
import {
  useKeyboardShortcuts,
  CommonShortcuts,
} from '@/hooks/useKeyboardShortcuts';
import { ConfigurationLayout } from '@/components/config/ConfigurationLayout';
import { DeviceSelectionPanel } from '@/components/config/DeviceSelectionPanel';
import { ConfigScopeTabs } from '@/components/config/ConfigScopeTabs';
import { GlobalKeyboardPanel } from '@/components/config/GlobalKeyboardPanel';
import { DeviceKeyboardPanel } from '@/components/config/DeviceKeyboardPanel';
import { UseCaseGuide } from '@/components/config/UseCaseGuide';
import { SwapKeysPanel } from '@/components/config/SwapKeysPanel';
import {
  ImeKeysPanel,
  type ImeAssignments,
} from '@/components/config/ImeKeysPanel';
import { friendlyKeyName } from '@/utils/keyNames';
import { useToast } from '@/hooks/useToast';
import { t } from '@/i18n';
import { describeKey } from '@/components/SVGKeyboard';
import { formatLayerName } from '@/components/LayerSwitcher';

interface EditTabProps {
  selectedProfileName: string;
  profileConfig: { source: string } | undefined;
  syncEngine: RhaiSyncEngineResult;
  syncStatus: SyncStatus;
  setSyncStatus: (status: SyncStatus) => void;
  configStore: {
    activeLayer: string;
    globalSelected: boolean;
    selectedDevices: string[];
    getLayerMappings: (layerId: string) => Map<string, KeyMapping>;
    getAllLayers: () => string[];
    setActiveLayer: (layerId: string) => void;
    setGlobalSelected: (selected: boolean) => void;
    setSelectedDevices: (deviceIds: string[]) => void;
    setKeyMapping: (key: string, mapping: KeyMapping, layerId?: string) => void;
    deleteKeyMapping: (key: string, layerId?: string) => void;
  };
  keyboardLayout: LayoutType;
  onKeyboardLayoutChange: (layout: LayoutType) => void;
  layoutKeys: SVGKeyData[];
  onOpenAdvanced: () => void;
}

/** Put the keyboard cursor in the key editor and bring it on screen. */
function moveFocusToKeyEditor(): void {
  window.requestAnimationFrame(() => {
    const editor = document.getElementById('key-editor');
    if (!editor) return;
    editor.focus({ preventScroll: true });
    const reduce = window.matchMedia?.('(prefers-reduced-motion: reduce)');
    editor.scrollIntoView({
      behavior: reduce?.matches ? 'auto' : 'smooth',
      block: 'start',
    });
  });
}

/** Return keyboard focus to a keycap after leaving the editor with Escape. */
function focusKeycap(keyCode: string): void {
  window.requestAnimationFrame(() => {
    document
      .querySelector<SVGGElement>(
        `#keyboard-editor [data-key-code="${keyCode}"]`
      )
      ?.focus();
  });
}

/**
 * Visual editor tab for the ConfigPage.
 *
 * Renders the keyboard visualizer, device selection, layer switcher,
 * key configuration panel, and notification banners. Owns the
 * selectedPhysicalKey and activePane local state, and internally
 * calls useDeviceMerging / useASTRebuild hooks.
 */
export const EditTab: React.FC<EditTabProps> = ({
  selectedProfileName,
  profileConfig: _profileConfig,
  syncEngine,
  syncStatus: _syncStatus,
  setSyncStatus,
  configStore,
  keyboardLayout,
  onKeyboardLayoutChange,
  layoutKeys,
  onOpenAdvanced,
}) => {
  // Local state owned by EditTab
  const [selectedPhysicalKey, setSelectedPhysicalKey] = useState<string | null>(
    null
  );

  const toast = useToast();

  // Screen-reader announcement for selection / edit results (aria-live).
  const [announcement, setAnnouncement] = useState('');

  useKeyboardShortcuts([
    CommonShortcuts.escape(() => {
      const previous = selectedPhysicalKey;
      setSelectedPhysicalKey(null);
      if (previous) focusKeycap(previous);
    }),
  ]);

  const [activePane, setActivePane] = useState<'global' | 'device'>('global');

  // First-run guide step the user chose from the cards (visible next-step cue)
  const [guide, setGuide] = useState<null | 'pick-key' | 'swap' | 'ime'>(null);

  // Derived values from configStore
  const keyMappings = configStore.getLayerMappings(configStore.activeLayer);
  const { activeLayer, globalSelected, selectedDevices } = configStore;
  const usedLayers = configStore.getAllLayers();

  // Merged device list: connected devices + devices from Rhai config
  const mergedDevices = useDeviceMerging({ syncEngine, configStore });
  const devices: Device[] = mergedDevices;

  // AST rebuild hook for syncing visual editor changes back to code
  const rebuildAndSyncAST = useASTRebuild({
    configStore,
    syncEngine,
    globalSelected,
    selectedDevices,
    devices,
  });

  // Handlers
  const handlePhysicalKeyClick = (keyCode: string) => {
    setSelectedPhysicalKey(keyCode);
    setGuide((g) => (g === 'pick-key' ? null : g));
    setAnnouncement(
      `${describeKey(keyCode, keyMappings.get(keyCode))}. ${t('kc.layer')} ${formatLayerName(activeLayer)}.`
    );
    moveFocusToKeyEditor();
  };

  const handleClearMapping = (keyCode: string) => {
    const previous = keyMappings.get(keyCode);
    const layer = activeLayer;
    configStore.deleteKeyMapping(keyCode, layer);
    setSyncStatus('unsaved');
    rebuildAndSyncAST();
    const name = friendlyKeyName(keyCode);
    setAnnouncement(t('toast.cleared', { key: name }));
    if (!previous) return;
    // Deleting is instant, so offer the way back right where the click was.
    toast.info(t('toast.cleared', { key: name }), {
      duration: 8000,
      action: {
        label: t('toast.undo'),
        onClick: () => {
          configStore.setKeyMapping(keyCode, previous, layer);
          setSyncStatus('unsaved');
          rebuildAndSyncAST();
          setAnnouncement(t('toast.restored', { key: name }));
        },
      },
    });
  };

  const handleSaveMapping = (mapping: KeyMapping) => {
    if (!selectedPhysicalKey) return;
    configStore.setKeyMapping(selectedPhysicalKey, mapping, activeLayer);
    setSyncStatus('unsaved');
    rebuildAndSyncAST();
    setAnnouncement(`Updated ${describeKey(selectedPhysicalKey, mapping)}.`);
  };

  /** Scroll the keyboard into view and put the keyboard cursor on it. */
  const focusEditor = () => {
    window.requestAnimationFrame(() => {
      const editor = document.getElementById('keyboard-editor');
      if (!editor) return;
      const reduce = window.matchMedia?.('(prefers-reduced-motion: reduce)');
      editor.scrollIntoView({
        behavior: reduce?.matches ? 'auto' : 'smooth',
        block: 'start',
      });
      editor
        .querySelector<SVGGElement>('[data-key-code][tabindex="0"]')
        ?.focus({ preventScroll: true });
    });
  };

  /**
   * Make sure edits have somewhere to go without changing an existing scope:
   * a profile already scoped to a device must stay scoped to it.
   */
  const ensureScope = () => {
    if (!globalSelected && selectedDevices.length === 0) {
      configStore.setGlobalSelected(true);
      setActivePane('global');
    }
  };

  const startSimpleRemap = () => {
    ensureScope();
    setGuide('pick-key');
    setAnnouncement(t('guide.step.pick'));
    focusEditor();
  };

  const startSwap = () => {
    ensureScope();
    setGuide('swap');
    setAnnouncement(t('guide.step.swap'));
  };

  /** Exchange two keys on the base layer: each acts like the other. */
  const applySwap = (a: string, b: string) => {
    configStore.setActiveLayer('base');
    configStore.setKeyMapping(a, { type: 'simple', tapAction: b }, 'base');
    configStore.setKeyMapping(b, { type: 'simple', tapAction: a }, 'base');
    setSyncStatus('unsaved');
    rebuildAndSyncAST();
    setGuide(null);
    setAnnouncement(
      `${friendlyKeyName(a)} → ${friendlyKeyName(b)}, ${friendlyKeyName(b)} → ${friendlyKeyName(a)}. Review and save to apply.`
    );
    focusEditor();
  };

  /** Apply the IME on/off keys chosen in ImeKeysPanel on the base layer. */
  const applyIme = (assignments: ImeAssignments) => {
    configStore.setActiveLayer('base');
    assignments.forEach(([key, target]) =>
      configStore.setKeyMapping(
        key,
        { type: 'simple', tapAction: target },
        'base'
      )
    );
    setSyncStatus('unsaved');
    rebuildAndSyncAST();
    setGuide(null);
    setAnnouncement(t('ime.done'));
    focusEditor();
  };

  const startCommandPad = () => {
    const firstDevice = devices.find(
      (device) => device.name !== '*' && device.serial !== '*'
    );
    if (!firstDevice) return;
    configStore.setGlobalSelected(false);
    configStore.setSelectedDevices([firstDevice.id]);
    setActivePane('device');
    focusEditor();
  };

  return (
    <div className="flex flex-col gap-4 md:gap-6">
      {/* Visual Editor Content */}
      <ConfigurationLayout profileName={selectedProfileName}>
        <UseCaseGuide
          hasDevice={devices.some(
            (device) => device.name !== '*' && device.serial !== '*'
          )}
          onStartSwap={startSwap}
          onStartSimple={startSimpleRemap}
          onStartCommandPad={startCommandPad}
          onStartAdvanced={onOpenAdvanced}
        />

        {guide === 'swap' && (
          <SwapKeysPanel
            layoutKeys={layoutKeys}
            onSwap={applySwap}
            onCancel={() => setGuide(null)}
          />
        )}

        {guide === 'ime' && (
          <ImeKeysPanel
            layoutKeys={layoutKeys}
            onApply={applyIme}
            onCancel={() => setGuide(null)}
          />
        )}

        {/* Device Selection Panel */}
        <DeviceSelectionPanel
          devices={devices}
          globalSelected={globalSelected}
          selectedDevices={selectedDevices}
          onToggleGlobal={(selected) => configStore.setGlobalSelected(selected)}
          onToggleDevice={(deviceId, selected) => {
            if (selected) {
              configStore.setSelectedDevices([...selectedDevices, deviceId]);
            } else {
              configStore.setSelectedDevices(
                selectedDevices.filter((id) => id !== deviceId)
              );
            }
          }}
        />

        {/* Tab Navigation for Global/Device switching */}
        {globalSelected && selectedDevices.length > 0 && (
          <ConfigScopeTabs
            activePane={activePane}
            onPaneChange={setActivePane}
          />
        )}

        {/* Single-Pane Layout: tabs control visibility */}
        <div
          id="keyboard-editor"
          className={`flex scroll-mt-4 flex-col gap-4 rounded-xl transition-shadow ${
            guide === 'pick-key'
              ? 'ring-2 ring-primary-300 ring-offset-4 ring-offset-slate-900'
              : ''
          }`}
        >
          {guide === 'pick-key' && (
            <div
              role="status"
              className="flex items-center justify-between gap-3 rounded-lg bg-primary-500/15 px-4 py-3 text-sm text-primary-100"
            >
              <span>{t('guide.step.pick')}</span>
              <span className="flex items-center gap-2">
                <button
                  type="button"
                  onClick={() => setGuide('ime')}
                  className="rounded bg-primary-500/30 px-2 py-1 text-xs font-medium text-primary-50 hover:bg-primary-500/50"
                >
                  {t('ime.open')}
                </button>
                <button
                  type="button"
                  onClick={() => setGuide(null)}
                  className="rounded px-2 py-1 text-xs text-primary-100 underline"
                >
                  {t('swap.cancel')}
                </button>
              </span>
            </div>
          )}
          {/* Global Keyboard Panel */}
          <GlobalKeyboardPanel
            profileName={selectedProfileName}
            activeLayer={activeLayer}
            availableLayers={usedLayers}
            onLayerChange={configStore.setActiveLayer}
            globalSelected={globalSelected}
            onToggleGlobal={configStore.setGlobalSelected}
            keyMappings={keyMappings}
            onKeyClick={handlePhysicalKeyClick}
            selectedKeyCode={selectedPhysicalKey}
            layout={keyboardLayout}
            onLayoutChange={onKeyboardLayoutChange}
            isVisible={selectedDevices.length === 0 || activePane === 'global'}
          />

          {/* Device-Specific Keyboard Panel */}
          <DeviceKeyboardPanel
            profileName={selectedProfileName}
            activeLayer={activeLayer}
            availableLayers={usedLayers}
            onLayerChange={configStore.setActiveLayer}
            devices={devices}
            selectedDevices={selectedDevices}
            onDeviceChange={(oldDeviceId, newDeviceId) => {
              const updatedDevices = selectedDevices.filter(
                (id) => id !== oldDeviceId
              );
              configStore.setSelectedDevices([...updatedDevices, newDeviceId]);
            }}
            keyMappings={keyMappings}
            onKeyClick={handlePhysicalKeyClick}
            selectedKeyCode={selectedPhysicalKey}
            layout={keyboardLayout}
            onLayoutChange={onKeyboardLayoutChange}
            isVisible={!globalSelected || activePane === 'device'}
          />

          {/* Warning if no selection */}
          {!globalSelected && selectedDevices.length === 0 && (
            <Card
              className="bg-yellow-900/20 border border-yellow-700/50 flex-1 block"
              aria-label={t('edit.configWarning')}
            >
              <div className="text-center py-8">
                <p className="text-yellow-200 text-lg mb-2">
                  {t('edit.noDevices')}
                </p>
                <p className="text-yellow-300 text-sm">
                  {t('edit.noDevicesHelp')}
                </p>
              </div>
            </Card>
          )}
        </div>

        {/* Legend - Color coding */}
        <div className="flex gap-4 flex-wrap text-xs text-slate-400 px-2">
          <div className="flex items-center gap-2">
            <div className="w-4 h-4 rounded bg-green-500"></div>
            <span>{t('edit.legend.simple')}</span>
          </div>
          <div className="flex items-center gap-2">
            <div className="w-4 h-4 rounded bg-primary-500"></div>
            <span>{t('edit.legend.modifier')}</span>
          </div>
          <div className="flex items-center gap-2">
            <div className="w-4 h-4 rounded bg-purple-500"></div>
            <span>{t('edit.legend.lock')}</span>
          </div>
          <div className="flex items-center gap-2">
            <div className="w-4 h-4 rounded bg-red-500"></div>
            <span>{t('edit.legend.tapHold')}</span>
          </div>
          <div className="flex items-center gap-2">
            <div className="w-4 h-4 rounded bg-yellow-500"></div>
            <span>{t('edit.legend.layer')}</span>
          </div>
        </div>

        {/* Polite live region: announces key selection and edit results */}
        <div role="status" aria-live="polite" className="sr-only">
          {announcement}
        </div>

        {/* Inline Key Configuration Panel */}
        <section
          id="key-editor"
          tabIndex={-1}
          aria-label="Key editor"
          className="scroll-mt-4 focus:outline-none focus-visible:outline-2"
        >
          <KeyConfigPanel
            physicalKey={selectedPhysicalKey}
            currentMapping={
              selectedPhysicalKey
                ? keyMappings.get(selectedPhysicalKey)
                : undefined
            }
            onSave={handleSaveMapping}
            onClearMapping={handleClearMapping}
            onEditMapping={handlePhysicalKeyClick}
            activeLayer={activeLayer}
            keyMappings={keyMappings}
            layoutKeys={layoutKeys}
          />
        </section>
      </ConfigurationLayout>
    </div>
  );
};
