import React, { useState, useCallback, useEffect, useMemo } from 'react';
import { useSearchParams } from 'react-router-dom';
import { DndContext, DragEndEvent, DragOverlay } from '@dnd-kit/core';
import { Card } from '@/components/Card';
import { KeyboardVisualizer } from '@/components/KeyboardVisualizer';
import { KeyAssignmentPanel, AssignableKey } from '@/components/KeyAssignmentPanel';
import { KeyAssignmentPopup } from '@/components/KeyAssignmentPopup';
import { DeviceScopeToggle, MappingScope, DeviceOption } from '@/components/DeviceScopeToggle';
import { LayerSelector, Layer } from '@/components/LayerSelector';
import { KeyMapping } from '@/components/KeyButton';
import { LoadingSkeleton } from '@/components/LoadingSkeleton';
import { MonacoEditor } from '@/components/MonacoEditor';
import { useAutoSave } from '@/hooks/useAutoSave';
import { useGetProfileConfig, useSetProfileConfig } from '@/hooks/useProfileConfig';
import { useWasm, ValidationError } from '@/hooks/useWasm';
import { useUnifiedApi } from '@/hooks/useUnifiedApi';

interface ConfigPageProps {
  profileName?: string;
}

export const ConfigPage: React.FC<ConfigPageProps> = ({
  profileName: propProfileName,
}) => {
  const [searchParams] = useSearchParams();
  const profileNameFromQuery = searchParams.get('profile');
  const profileName = propProfileName || profileNameFromQuery || 'Default';

  const api = useUnifiedApi();
  const { data: profileConfig, isLoading: isLoadingConfig, error: configError } = useGetProfileConfig(profileName);
  const { mutateAsync: setProfileConfig } = useSetProfileConfig();
  const { validateConfig, isWasmReady } = useWasm();

  const [activeTab, setActiveTab] = useState<'visual' | 'code'>('visual');
  const [configCode, setConfigCode] = useState<string>('');
  const [validationErrors, setValidationErrors] = useState<ValidationError[]>([]);
  const [selectedLayout, setSelectedLayout] = useState<'ANSI_104' | 'ISO_105' | 'JIS_109' | 'HHKB' | 'NUMPAD'>('ANSI_104');
  const [selectedLayer, setSelectedLayer] = useState('base');
  const [scope, setScope] = useState<MappingScope>('global');
  const [selectedDevice, setSelectedDevice] = useState<string>('');
  const [keyMappings, setKeyMappings] = useState<Map<string, KeyMapping>>(new Map());
  const [popupState, setPopupState] = useState<{ open: boolean; keyCode: string | null }>({ open: false, keyCode: null });
  const [activeDragKey, setActiveDragKey] = useState<AssignableKey | null>(null);
  const [connectionTimeout, setConnectionTimeout] = useState(false);

  // Mock devices for demo - would come from API
  const availableDevices: DeviceOption[] = useMemo(() => [
    { serial: 'KB001', name: 'Microsoft Ergonomic Keyboard' },
    { serial: 'KB002', name: 'Logitech MX Keys' },
  ], []);

  // Mock layers - would come from profile config parsing
  const availableLayers: Layer[] = useMemo(() => [
    { id: 'base', name: 'Base Layer' },
    { id: 'nav', name: 'Navigation Layer' },
    { id: 'num', name: 'Number Layer' },
    { id: 'fn', name: 'Function Layer' },
    { id: 'gaming', name: 'Gaming Layer' },
  ], []);

  // Layout options
  const layoutOptions = [
    { value: 'ANSI_104', label: 'ANSI 104' },
    { value: 'ISO_105', label: 'ISO 105' },
    { value: 'JIS_109', label: 'JIS 109' },
    { value: 'HHKB', label: 'HHKB' },
    { value: 'NUMPAD', label: 'Numpad' },
  ];

  // Load config code from API
  useEffect(() => {
    if (profileConfig?.source) {
      setConfigCode(profileConfig.source);
    }
  }, [profileConfig]);

  // Auto-save configuration
  const { isSaving, error: saveError, lastSavedAt } = useAutoSave(configCode, {
    saveFn: async (code) => {
      // Validate before saving
      const errors = await validateConfig(code);
      if (errors.length > 0) {
        throw new Error(`Validation failed: ${errors[0].message}`);
      }
      await setProfileConfig({ name: profileName, source: code });
    },
    debounceMs: 500,
    enabled: activeTab === 'code' && !!configCode,
  });

  // Connection timeout
  useEffect(() => {
    if (!api.isConnected && !connectionTimeout) {
      const timer = setTimeout(() => {
        setConnectionTimeout(true);
      }, 10000);
      return () => clearTimeout(timer);
    }
  }, [api.isConnected, connectionTimeout]);

  // Handle validation callback from Monaco
  const handleValidation = useCallback((errors: ValidationError[]) => {
    setValidationErrors(errors);
  }, []);

  // Handle key click - open popup
  const handleKeyClick = useCallback((keyCode: string) => {
    setPopupState({ open: true, keyCode });
  }, []);

  // Handle key drop from drag-and-drop
  const handleKeyDrop = useCallback((keyCode: string, droppedKey: AssignableKey) => {
    // Create a simple mapping from the dropped key
    const mapping: KeyMapping = {
      type: 'simple',
      tapAction: droppedKey.id,
    };

    setKeyMappings(prev => {
      const updated = new Map(prev);
      updated.set(keyCode, mapping);
      return updated;
    });

    // TODO: Update config code with new mapping
    // This would require parsing the config, modifying it, and regenerating
  }, []);

  // Handle drag start
  const handleDragStart = useCallback((event: any) => {
    setActiveDragKey(event.active.data.current as AssignableKey);
  }, []);

  // Handle drag end
  const handleDragEnd = useCallback((event: DragEndEvent) => {
    setActiveDragKey(null);

    const { active, over } = event;
    if (!over) return;

    // Extract keyCode from drop zone ID (format: "drop-KEYCODE")
    const dropZoneId = over.id as string;
    if (dropZoneId.startsWith('drop-')) {
      const keyCode = dropZoneId.slice(5);
      const droppedKey = active.data.current as AssignableKey;
      handleKeyDrop(keyCode, droppedKey);
    }
  }, [handleKeyDrop]);

  // Handle popup save
  const handlePopupSave = useCallback((mapping: KeyMapping) => {
    if (popupState.keyCode) {
      setKeyMappings(prev => {
        const updated = new Map(prev);
        updated.set(popupState.keyCode!, mapping);
        return updated;
      });

      // TODO: Update config code with new mapping
    }
  }, [popupState.keyCode]);

  // Show timeout error
  if (connectionTimeout && !api.isConnected) {
    return (
      <div className="flex flex-col items-center justify-center min-h-screen gap-4 p-4">
        <div className="text-red-400 text-xl">⚠️ Connection Timeout</div>
        <div className="text-slate-300 text-center max-w-md">
          Failed to connect to the daemon WebSocket. Please ensure the daemon is running and try refreshing the page.
        </div>
        <button
          onClick={() => window.location.reload()}
          className="px-6 py-3 bg-primary-500 text-white rounded-md hover:bg-primary-600 transition-colors"
        >
          Reload Page
        </button>
      </div>
    );
  }

  // Show loading state
  if (!api.isConnected || isLoadingConfig) {
    return (
      <div className="flex flex-col gap-4 md:gap-6 p-4 md:p-6 lg:p-8">
        <div className="text-center text-slate-400 py-4">
          {!api.isConnected ? '⏳ Connecting to daemon...' : '⏳ Loading configuration...'}
        </div>

        <div className="flex flex-col lg:flex-row lg:items-center lg:justify-between gap-4">
          <LoadingSkeleton variant="text" width="250px" height="32px" />
          <LoadingSkeleton variant="rectangular" width="180px" height="44px" />
        </div>

        <Card variant="default" padding="lg">
          <LoadingSkeleton variant="rectangular" height="600px" />
        </Card>
      </div>
    );
  }

  // Show error state
  if (configError) {
    return (
      <div className="flex flex-col items-center justify-center min-h-screen gap-4 p-4">
        <div className="text-red-400 text-xl">⚠️ Error Loading Configuration</div>
        <div className="text-slate-300 text-center max-w-md">
          {configError instanceof Error ? configError.message : 'Failed to load profile configuration'}
        </div>
      </div>
    );
  }

  return (
    <DndContext onDragStart={handleDragStart} onDragEnd={handleDragEnd}>
      <div className="flex flex-col gap-4 md:gap-6 p-4 md:p-6 lg:p-8">
        {/* Header with Breadcrumb */}
        <div className="flex flex-col lg:flex-row lg:items-center lg:justify-between gap-4">
          <div className="flex flex-col gap-2">
            <div className="text-sm text-slate-400">
              <span className="hover:text-primary-400 cursor-pointer">Profiles</span>
              <span className="mx-2">→</span>
              <span className="text-slate-300">{profileName}</span>
              <span className="mx-2">→</span>
              <span className="text-slate-300">Configuration</span>
            </div>
            <h1 className="text-xl md:text-2xl lg:text-3xl font-semibold text-slate-100">
              Visual Configuration Editor
            </h1>
          </div>

          {/* Save Status Indicator */}
          <div className="flex items-center gap-2">
            {isSaving && (
              <span className="text-sm text-slate-400 flex items-center gap-2">
                <svg className="animate-spin h-4 w-4" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                  <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
                  <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                </svg>
                Saving...
              </span>
            )}
            {!isSaving && lastSavedAt && (
              <span className="text-sm text-green-400">
                ✓ Saved {lastSavedAt.toLocaleTimeString()}
              </span>
            )}
            {saveError && (
              <span className="text-sm text-red-400">
                ✗ Save failed
              </span>
            )}
          </div>
        </div>

        {/* Tab Buttons */}
        <div className="grid grid-cols-2 sm:flex sm:gap-2" role="tablist" aria-label="Editor mode">
          <button
            id="visual-tab"
            role="tab"
            aria-selected={activeTab === 'visual'}
            aria-controls="visual-panel"
            onClick={() => setActiveTab('visual')}
            className={`px-6 py-3 sm:py-2 rounded-md font-medium transition-colors min-h-[44px] sm:min-h-0 ${
              activeTab === 'visual'
                ? 'bg-primary-500 text-white'
                : 'bg-slate-700 text-slate-400 hover:text-slate-300 hover:bg-slate-600'
            }`}
          >
            🎨 Visual Editor
          </button>
          <button
            id="code-tab"
            role="tab"
            aria-selected={activeTab === 'code'}
            aria-controls="code-panel"
            onClick={() => setActiveTab('code')}
            className={`px-6 py-3 sm:py-2 rounded-md font-medium transition-colors min-h-[44px] sm:min-h-0 ${
              activeTab === 'code'
                ? 'bg-primary-500 text-white'
                : 'bg-slate-700 text-slate-400 hover:text-slate-300 hover:bg-slate-600'
            }`}
          >
            {'</>'}Code Editor
          </button>
        </div>

        {/* Validation/Error Messages */}
        {(validationErrors.length > 0 || saveError) && (
          <div className="bg-red-900/50 border border-red-700 rounded-md p-4">
            <div className="flex flex-col gap-2">
              {validationErrors.length > 0 && (
                <div className="text-sm md:text-base text-red-300">
                  ⚠️ {validationErrors.length} validation {validationErrors.length === 1 ? 'error' : 'errors'}
                </div>
              )}
              {saveError && (
                <div className="text-sm md:text-base text-red-300">
                  ✗ {saveError.message}
                </div>
              )}
            </div>
          </div>
        )}

        {!isWasmReady && (
          <div className="bg-yellow-900/50 border border-yellow-700 rounded-md p-4">
            <div className="text-sm md:text-base text-yellow-300">
              ⚠️ WASM module not loaded. Validation disabled. Run <code>npm run build:wasm</code> to enable.
            </div>
          </div>
        )}

        {/* Visual Editor Panel */}
        {activeTab === 'visual' && (
          <div role="tabpanel" id="visual-panel" aria-labelledby="visual-tab">
            <div className="grid grid-cols-1 lg:grid-cols-[1fr_300px] gap-4">
              {/* Main editor area */}
              <div className="flex flex-col gap-4">
                {/* Device Scope Toggle */}
                <Card variant="default" padding="lg">
                  <DeviceScopeToggle
                    scope={scope}
                    onScopeChange={setScope}
                    devices={availableDevices}
                    selectedDevice={selectedDevice}
                    onDeviceChange={setSelectedDevice}
                  />
                </Card>

                {/* Layer Selector */}
                <Card variant="default" padding="lg">
                  <LayerSelector
                    layers={availableLayers}
                    selectedLayer={selectedLayer}
                    onLayerChange={setSelectedLayer}
                  />
                </Card>

                {/* Keyboard Visualizer */}
                <Card variant="default" padding="lg">
                  <div className="flex flex-col gap-4">
                    <div className="flex items-center justify-between pb-4 border-b border-slate-700">
                      <h2 className="text-lg font-medium text-slate-100">
                        Keyboard Layout
                      </h2>
                      <div className="text-sm text-slate-400">
                        Drag keys from the palette or click to configure
                      </div>
                    </div>

                    <div className="py-4 overflow-x-auto md:overflow-x-visible">
                      <KeyboardVisualizer
                        layout={selectedLayout}
                        keyMappings={keyMappings}
                        onKeyClick={handleKeyClick}
                        onKeyDrop={handleKeyDrop}
                        simulatorMode={false}
                      />
                    </div>
                  </div>
                </Card>
              </div>

              {/* Key Assignment Palette */}
              <div className="lg:sticky lg:top-4 h-fit">
                <KeyAssignmentPanel />
              </div>
            </div>
          </div>
        )}

        {/* Code Editor Panel */}
        {activeTab === 'code' && (
          <div role="tabpanel" id="code-panel" aria-labelledby="code-tab">
            <Card variant="default" padding="lg">
              <div className="flex flex-col gap-4">
                <div className="flex items-center justify-between pb-4 border-b border-slate-700">
                  <h2 className="text-lg font-medium text-slate-100">
                    Rhai Configuration Code
                  </h2>
                  {validationErrors.length === 0 && (
                    <span className="text-sm text-green-400">
                      ✓ No errors
                    </span>
                  )}
                </div>
                <MonacoEditor
                  value={configCode}
                  onChange={setConfigCode}
                  onValidate={handleValidation}
                  height="600px"
                />
              </div>
            </Card>
          </div>
        )}

        {/* Key Assignment Popup */}
        <KeyAssignmentPopup
          open={popupState.open}
          onClose={() => setPopupState({ open: false, keyCode: null })}
          keyCode={popupState.keyCode || ''}
          currentMapping={popupState.keyCode ? keyMappings.get(popupState.keyCode) : undefined}
          onSave={handlePopupSave}
        />

        {/* Drag Overlay */}
        <DragOverlay>
          {activeDragKey ? (
            <div className="px-3 py-2 text-sm font-medium rounded border bg-primary-500 border-primary-400 text-white shadow-lg">
              {activeDragKey.label}
            </div>
          ) : null}
        </DragOverlay>
      </div>
    </DndContext>
  );
};

export default ConfigPage;
