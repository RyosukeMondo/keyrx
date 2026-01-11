import React, { useState, useEffect } from 'react';
import { useSearchParams, useParams, useNavigate } from 'react-router-dom';
import { Card } from '@/components/Card';
import { SimpleCodeEditor } from '@/components/SimpleCodeEditor';
import { KeyPalette, type PaletteKey } from '@/components/KeyPalette';
import { DeviceSelector, type Device } from '@/components/DeviceSelector';
import { KeyConfigModal } from '@/components/KeyConfigModal';
import { LayerSwitcher } from '@/components/LayerSwitcher';
import { KeyboardVisualizer } from '@/components/KeyboardVisualizer';
import { useGetProfileConfig, useSetProfileConfig } from '@/hooks/useProfileConfig';
import { useProfiles, useCreateProfile } from '@/hooks/useProfiles';
import { useUnifiedApi } from '@/hooks/useUnifiedApi';
import { useDevices } from '@/hooks/useDevices';
import { useRhaiSyncEngine } from '@/components/RhaiSyncEngine';
import { extractDevicePatterns, hasGlobalMappings } from '@/utils/rhaiParser';
import type { KeyMapping } from '@/types';
import type { KeyMapping as RhaiKeyMapping } from '@/utils/rhaiParser';

interface ConfigPageProps {
  profileName?: string;
}

const ConfigPage: React.FC<ConfigPageProps> = ({
  profileName: propProfileName,
}) => {
  const [searchParams] = useSearchParams();
  const navigate = useNavigate();
  const { name: profileNameFromRoute } = useParams<{ name: string }>();
  const profileNameFromQuery = searchParams.get('profile');
  const defaultProfileName = propProfileName || profileNameFromRoute || profileNameFromQuery || 'Default';

  const [selectedProfileName, setSelectedProfileName] = useState<string>(defaultProfileName);

  const api = useUnifiedApi();
  const [activeTab, setActiveTab] = useState<'visual' | 'code'>('visual');

  // Initialize RhaiSyncEngine for bidirectional sync
  const syncEngine = useRhaiSyncEngine({
    storageKey: `profile-${selectedProfileName}`,
    debounceMs: 500,
    onStateChange: (state) => {
      console.debug('Sync state changed:', state);
    },
    onError: (error, direction) => {
      console.error('Sync error:', { error, direction });
    },
  });

  // Fetch available profiles
  const { data: profiles, isLoading: isLoadingProfiles } = useProfiles();
  const { mutateAsync: createProfile } = useCreateProfile();

  // Visual editor state
  const [selectedPaletteKey, setSelectedPaletteKey] = useState<PaletteKey | null>(null);
  const [selectedPhysicalKey, setSelectedPhysicalKey] = useState<string | null>(null);
  const [keyMappings, setKeyMappings] = useState<Map<string, KeyMapping>>(new Map());
  const [selectedDevices, setSelectedDevices] = useState<string[]>([]);
  const [globalSelected, setGlobalSelected] = useState<boolean>(true);
  const [activeLayer, setActiveLayer] = useState<string>('base');
  const [isModalOpen, setIsModalOpen] = useState(false);

  // Available layers
  const availableLayers = ['base', 'md-00', 'md-01', 'md-02', 'md-03', 'md-04', 'md-05'];

  // Query for profile config - doesn't block rendering
  const { data: profileConfig, isLoading, error } = useGetProfileConfig(selectedProfileName);
  const { mutateAsync: setProfileConfig } = useSetProfileConfig();

  // Fetch devices
  const { data: devicesData } = useDevices();

  // Merged device list: connected devices + devices from Rhai (even if disconnected)
  const [mergedDevices, setMergedDevices] = useState<Device[]>([]);

  // Auto-select first profile if "Default" doesn't exist and profiles are loaded
  useEffect(() => {
    if (profiles && profiles.length > 0 && !profiles.some(p => p.name === selectedProfileName)) {
      setSelectedProfileName(profiles[0].name);
    }
  }, [profiles, selectedProfileName]);

  // Check if selected profile exists
  const profileExists = profiles?.some(p => p.name === selectedProfileName);

  // Check if config file exists
  const configExists = !isLoading && !error && profileConfig?.source;
  const configMissing = !isLoading && !error && profileExists && !profileConfig?.source;

  // Update sync engine when profile config loads
  useEffect(() => {
    if (profileConfig?.source) {
      // Initialize sync engine with loaded config
      syncEngine.onCodeChange(profileConfig.source);
    } else if (configMissing) {
      // Default config template when config file doesn't exist
      const defaultTemplate = `// Configuration for profile: ${selectedProfileName}
// This is a new configuration file

// Example: Simple key remapping
// map("A", "B");  // Press A → outputs B

// Example: Tap/Hold behavior
// tap_hold("CapsLock", "Escape", "LCtrl", 200);

// Add your key mappings here...
`;
      syncEngine.onCodeChange(defaultTemplate);
    }
  }, [profileConfig, configMissing, selectedProfileName]);

  // Rhai-driven device detection: Extract devices from parsed Rhai and merge with connected devices
  useEffect(() => {
    const ast = syncEngine.getAST();
    if (!ast) {
      // No AST yet, just use connected devices
      setMergedDevices(
        devicesData?.map((d) => ({
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

    // Create a map of connected devices by serial/name/id
    const connectedDeviceMap = new Map<string, NonNullable<typeof devicesData>[number]>();
    devicesData?.forEach((device) => {
      if (device.serial) connectedDeviceMap.set(device.serial, device);
      connectedDeviceMap.set(device.name, device);
      connectedDeviceMap.set(device.id, device);
    });

    // Build merged device list
    const merged: Device[] = [];
    const addedPatterns = new Set<string>();

    // Add devices from Rhai (may be disconnected)
    devicePatternsInRhai.forEach((pattern) => {
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
          name: pattern, // Use pattern as name for disconnected devices
          serial: pattern,
          connected: false,
        });
      }
    });

    // Add connected devices not in Rhai
    devicesData?.forEach((device) => {
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

    // Auto-populate device selector based on Rhai content
    // If Rhai has global mappings, select global
    if (hasGlobalMappings(ast)) {
      setGlobalSelected(true);
    }

    // If Rhai has device blocks, auto-select those devices
    if (devicePatternsInRhai.length > 0) {
      const devicesToSelect = merged
        .filter((device) => {
          const pattern = device.serial || device.name;
          return devicePatternsInRhai.includes(pattern);
        })
        .map((device) => device.id);

      if (devicesToSelect.length > 0) {
        setSelectedDevices(devicesToSelect);
      }
    }
  }, [syncEngine.state, devicesData]); // Re-run when sync state changes (parsing complete) or devices change

  // Sync visual editor state from parsed AST when transitioning to visual tab or when code changes
  useEffect(() => {
    // Only sync when on visual tab and state is idle (parsing complete)
    if (activeTab !== 'visual' || syncEngine.state !== 'idle') return;

    const ast = syncEngine.getAST();
    if (!ast) return;

    // Helper to convert RhaiKeyMapping to visual KeyMapping
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

    // Convert RhaiAST to visual editor KeyMapping format
    const newMappings = new Map<string, KeyMapping>();

    // Add global mappings if global is selected
    if (globalSelected) {
      ast.globalMappings.forEach((m) => {
        newMappings.set(m.sourceKey, convertToVisualMapping(m));
      });
    }

    // Add device-specific mappings for selected devices
    if (selectedDevices.length > 0 && devicesData) {
      ast.deviceBlocks.forEach((block) => {
        // Check if this device block matches any selected device
        const matchesSelectedDevice = devicesData.some((device) => {
          const isSelected = selectedDevices.includes(device.id);
          const matchesPattern =
            block.pattern === device.serial ||
            block.pattern === device.name ||
            block.pattern === device.id;
          return isSelected && matchesPattern;
        });

        if (matchesSelectedDevice) {
          block.mappings.forEach((m) => {
            // Device-specific mappings override global mappings
            newMappings.set(m.sourceKey, convertToVisualMapping(m));
          });
        }
      });
    }

    setKeyMappings(newMappings);
  }, [activeTab, syncEngine.state, globalSelected, selectedDevices, devicesData]);

  // Handle profile selection change
  const handleProfileChange = (newProfileName: string) => {
    setSelectedProfileName(newProfileName);
    navigate(`/config?profile=${newProfileName}`);
  };

  // Handle profile creation
  const handleCreateProfile = async () => {
    try {
      await createProfile({ name: selectedProfileName, template: 'blank' });
    } catch (err) {
      console.error('Failed to create profile:', err);
    }
  };

  const handleSaveConfig = async () => {
    try {
      await setProfileConfig({ name: selectedProfileName, source: syncEngine.getCode() });
    } catch (err) {
      console.error('Failed to save config:', err);
    }
  };

  // Handle key click: open modal for configuration
  const handlePhysicalKeyClick = (keyCode: string) => {
    setSelectedPhysicalKey(keyCode);
    setIsModalOpen(true);
  };

  // Handle save from modal
  const handleSaveMapping = (mapping: KeyMapping) => {
    if (selectedPhysicalKey) {
      const newMappings = new Map(keyMappings);
      newMappings.set(selectedPhysicalKey, mapping);
      setKeyMappings(newMappings);

      // Convert visual editor state to RhaiAST and trigger sync
      const convertToRhaiMappings = (mappings: Map<string, KeyMapping>): RhaiKeyMapping[] => {
        return Array.from(mappings.entries()).map(([key, m]) => {
          const baseMapping: RhaiKeyMapping = {
            type: m.type,
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
              keys: m.macroSteps.filter(s => s.key).map(s => s.key!),
              delayMs: m.macroSteps.find(s => s.delayMs)?.delayMs,
            };
          } else if (m.type === 'layer_switch' && m.targetLayer) {
            baseMapping.layerSwitch = {
              layerId: m.targetLayer,
            };
          }

          return baseMapping;
        });
      };

      // Build device blocks based on current selection
      const deviceBlocks = selectedDevices.map((deviceId, index) => {
        const device = devices.find((d) => d.id === deviceId);
        if (!device) return null;

        return {
          pattern: device.serial || device.name,
          mappings: convertToRhaiMappings(newMappings),
          layers: [],
          startLine: 0,
          endLine: 0,
        };
      }).filter((block): block is NonNullable<typeof block> => block !== null);

      // Update sync engine with new AST
      syncEngine.onVisualChange({
        imports: [],
        globalMappings: globalSelected ? convertToRhaiMappings(newMappings) : [],
        deviceBlocks,
        comments: [],
      });
    }
  };

  // Use merged device list (connected + disconnected from Rhai)
  const devices: Device[] = mergedDevices;

  // Get all available keys for modal (will be passed from KeyPalette component data)
  // For now, use a simplified list
  const getAllAvailableKeys = (): PaletteKey[] => {
    // This should include all VK_, MD_, LK_ keys
    // For now returning empty array, will be populated from KeyPalette
    return [];
  };

  return (
    <div className="flex flex-col gap-4 md:gap-6 p-4 md:p-6 lg:p-8">
      {/* Header with Profile Selector */}
      <div className="flex flex-col lg:flex-row lg:items-start lg:justify-between gap-4">
        <div className="flex-1">
          <h1 className="text-2xl md:text-3xl font-bold text-slate-100 mb-3">
            Configuration Editor
          </h1>

          {/* Profile Selector */}
          <div className="flex items-center gap-3 flex-wrap">
            <label htmlFor="profile-selector" className="text-sm font-medium text-slate-300">
              Profile:
            </label>
            <select
              id="profile-selector"
              value={selectedProfileName}
              onChange={(e) => handleProfileChange(e.target.value)}
              disabled={isLoadingProfiles || !api.isConnected}
              className="px-4 py-2 bg-slate-700 border border-slate-600 rounded-md text-slate-100 font-medium focus:outline-none focus:ring-2 focus:ring-primary-500 disabled:opacity-50"
            >
              {profiles?.map((profile) => (
                <option key={profile.name} value={profile.name}>
                  {profile.name}
                </option>
              ))}
            </select>

            {/* Status indicators */}
            {!api.isConnected && (
              <span className="text-xs px-2 py-1 bg-yellow-900/30 border border-yellow-500 text-yellow-400 rounded">
                ⚠️ Disconnected
              </span>
            )}
            {isLoading && api.isConnected && (
              <span className="text-xs px-2 py-1 bg-blue-900/30 border border-blue-500 text-blue-400 rounded">
                ⏳ Loading configuration...
              </span>
            )}
            {configExists && (
              <span className="text-xs px-2 py-1 bg-green-900/30 border border-green-500 text-green-400 rounded">
                ✅ Loaded
              </span>
            )}
            {configMissing && (
              <span className="text-xs px-2 py-1 bg-orange-900/30 border border-orange-500 text-orange-400 rounded">
                📝 New configuration
              </span>
            )}
            {error && (
              <span className="text-xs px-2 py-1 bg-red-900/30 border border-red-500 text-red-400 rounded">
                ❌ Error
              </span>
            )}
            {!profileExists && !isLoading && api.isConnected && (
              <span className="text-xs px-2 py-1 bg-orange-900/30 border border-orange-500 text-orange-400 rounded">
                ⚠️ Profile not found
              </span>
            )}
          </div>

          {/* Error message with action */}
          {!profileExists && !isLoading && api.isConnected && (
            <div className="mt-3 p-3 bg-orange-900/20 border border-orange-500 rounded-md">
              <p className="text-sm text-orange-300 mb-2">
                Profile "{selectedProfileName}" does not exist.
              </p>
              <button
                onClick={handleCreateProfile}
                className="px-4 py-1.5 bg-orange-600 hover:bg-orange-500 text-white text-sm font-medium rounded transition-colors"
              >
                Create Profile "{selectedProfileName}"
              </button>
            </div>
          )}

          {configMissing && (
            <div className="mt-3 p-3 bg-blue-900/20 border border-blue-500 rounded-md">
              <p className="text-sm text-blue-300">
                📝 No configuration file found for "{selectedProfileName}". A template has been loaded - click <strong>Save Configuration</strong> to create it.
              </p>
            </div>
          )}

          {error && (
            <div className="mt-3 p-3 bg-red-900/20 border border-red-500 rounded-md">
              <p className="text-sm text-red-300">
                {error instanceof Error ? error.message : 'Failed to load configuration'}
              </p>
            </div>
          )}
        </div>

        <button
          onClick={handleSaveConfig}
          disabled={!api.isConnected || !profileExists}
          className="px-6 py-3 bg-primary-500 text-white font-medium rounded-md hover:bg-primary-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        >
          {configMissing ? 'Create Configuration' : 'Save Configuration'}
        </button>
      </div>

      {/* Tabs */}
      <div className="flex gap-2 border-b border-slate-700">
        <button
          onClick={() => setActiveTab('visual')}
          className={`px-4 py-2 font-medium transition-colors ${
            activeTab === 'visual'
              ? 'text-primary-400 border-b-2 border-primary-400'
              : 'text-slate-400 hover:text-slate-300'
          }`}
        >
          Visual Editor
        </button>
        <button
          onClick={() => setActiveTab('code')}
          className={`px-4 py-2 font-medium transition-colors ${
            activeTab === 'code'
              ? 'text-primary-400 border-b-2 border-primary-400'
              : 'text-slate-400 hover:text-slate-300'
          }`}
        >
          Code Editor
        </button>
      </div>

      {/* Content */}
      {activeTab === 'visual' ? (
        <>
          {/* Device Selector (compact at top) */}
          <DeviceSelector
            devices={devices}
            selectedDevices={selectedDevices}
            globalSelected={globalSelected}
            onSelectionChange={(deviceIds, global) => {
              setSelectedDevices(deviceIds);
              setGlobalSelected(global);
            }}
          />

          {/* KEYBOARD DISPLAY - Shows Global and/or Device-Specific Keyboards */}
          {globalSelected && (
            <Card className="bg-gradient-to-br from-slate-800 to-slate-900">
              <h3 className="text-xl font-bold text-primary-400 mb-4">
                Global Keyboard (All Devices)
              </h3>
              <div className="flex justify-center p-4">
                <KeyboardVisualizer
                  layout="ANSI_104"
                  keyMappings={keyMappings}
                  onKeyClick={handlePhysicalKeyClick}
                  simulatorMode={false}
                />
              </div>
              <p className="text-center text-sm text-slate-400 mt-4">
                Click any key to configure global mappings
              </p>
            </Card>
          )}

          {/* Device-Specific Keyboards */}
          {selectedDevices.length > 0 && devices
            .filter((d) => selectedDevices.includes(d.id))
            .map((device) => (
              <Card key={device.id} className="bg-gradient-to-br from-slate-800 to-slate-900">
                <h3 className="text-xl font-bold text-primary-400 mb-4">
                  {device.name}
                  {device.serial && (
                    <span className="ml-2 text-sm text-slate-400 font-normal">
                      ({device.serial})
                    </span>
                  )}
                </h3>
                <div className="flex justify-center p-4">
                  <KeyboardVisualizer
                    layout="ANSI_104"
                    keyMappings={keyMappings}
                    onKeyClick={handlePhysicalKeyClick}
                    simulatorMode={false}
                  />
                </div>
                <p className="text-center text-sm text-slate-400 mt-4">
                  Click any key to configure device-specific mappings for {device.name}
                </p>
              </Card>
            ))}

          {/* Warning if no selection */}
          {!globalSelected && selectedDevices.length === 0 && (
            <Card className="bg-yellow-900/20 border border-yellow-700/50">
              <div className="text-center py-8">
                <p className="text-yellow-200 text-lg mb-2">⚠️ No devices selected</p>
                <p className="text-yellow-300 text-sm">
                  Select at least one device or "Global" to configure key mappings
                </p>
              </div>
            </Card>
          )}

          {/* Layer Switcher */}
          <LayerSwitcher
            activeLayer={activeLayer}
            availableLayers={availableLayers}
            onLayerChange={setActiveLayer}
          />

          {/* Legend - Color coding */}
          <div className="flex gap-4 flex-wrap text-xs text-slate-400 px-2">
            <div className="flex items-center gap-2">
              <div className="w-4 h-4 rounded bg-green-500"></div>
              <span>Simple</span>
            </div>
            <div className="flex items-center gap-2">
              <div className="w-4 h-4 rounded bg-primary-500"></div>
              <span>Modifier</span>
            </div>
            <div className="flex items-center gap-2">
              <div className="w-4 h-4 rounded bg-purple-500"></div>
              <span>Lock</span>
            </div>
            <div className="flex items-center gap-2">
              <div className="w-4 h-4 rounded bg-red-500"></div>
              <span>Tap/Hold</span>
            </div>
            <div className="flex items-center gap-2">
              <div className="w-4 h-4 rounded bg-yellow-500"></div>
              <span>Layer Active</span>
            </div>
          </div>

          {/* Key Palette - Comprehensive list at bottom */}
          <Card>
            <KeyPalette
              onKeySelect={setSelectedPaletteKey}
              selectedKey={selectedPaletteKey}
            />
          </Card>

          {/* Configuration Modal */}
          {isModalOpen && selectedPhysicalKey && (
            <KeyConfigModal
              isOpen={isModalOpen}
              onClose={() => setIsModalOpen(false)}
              physicalKey={selectedPhysicalKey}
              currentMapping={keyMappings.get(selectedPhysicalKey)}
              onSave={handleSaveMapping}
              availableKeys={getAllAvailableKeys()}
            />
          )}
        </>
      ) : (
        <div className="flex flex-col gap-4">
          {/* Sync status indicator */}
          {syncEngine.state !== 'idle' && (
            <div className="flex items-center gap-2 px-4 py-2 bg-slate-700 border border-slate-600 rounded-md">
              {syncEngine.state === 'parsing' && (
                <>
                  <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-primary-400"></div>
                  <span className="text-sm text-slate-300">Parsing Rhai script...</span>
                </>
              )}
              {syncEngine.state === 'generating' && (
                <>
                  <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-primary-400"></div>
                  <span className="text-sm text-slate-300">Generating code...</span>
                </>
              )}
              {syncEngine.state === 'syncing' && (
                <>
                  <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-primary-400"></div>
                  <span className="text-sm text-slate-300">Syncing...</span>
                </>
              )}
            </div>
          )}

          {/* Parse error display */}
          {syncEngine.error && (
            <div className="p-4 bg-red-900/20 border border-red-500 rounded-md">
              <div className="flex items-start gap-3">
                <span className="text-red-400 text-xl">⚠️</span>
                <div className="flex-1">
                  <h4 className="text-red-400 font-semibold mb-1">Parse Error</h4>
                  <p className="text-sm text-red-300 mb-2">
                    Line {syncEngine.error.line}, Column {syncEngine.error.column}: {syncEngine.error.message}
                  </p>
                  {syncEngine.error.suggestion && (
                    <p className="text-sm text-slate-300 italic">
                      💡 {syncEngine.error.suggestion}
                    </p>
                  )}
                </div>
                <button
                  onClick={() => syncEngine.clearError()}
                  className="text-slate-400 hover:text-slate-300 transition-colors"
                  aria-label="Clear error"
                >
                  ✕
                </button>
              </div>
            </div>
          )}

          <Card variant="default" padding="lg">
            <SimpleCodeEditor
              value={syncEngine.getCode()}
              onChange={(value) => syncEngine.onCodeChange(value)}
              height="600px"
              language="javascript"
            />
          </Card>
        </div>
      )}
    </div>
  );
};

export default ConfigPage;
