import React, { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { Card } from '@/components/Card';
import { type Device } from '@/components/DeviceSelector';
import { KeyConfigPanel } from '@/components/KeyConfigPanel';
import { LayerSwitcher } from '@/components/LayerSwitcher';
import { useGetProfileConfig, useSetProfileConfig } from '@/hooks/useProfileConfig';
import { useProfiles, useCreateProfile } from '@/hooks/useProfiles';
import { useUnifiedApi } from '@/hooks/useUnifiedApi';
import { useDevices } from '@/hooks/useDevices';
import { extractDevicePatterns, hasGlobalMappings } from '@/utils/rhaiParser';
import { useConfigStore } from '@/stores/configStore';
import type { KeyMapping } from '@/types';
import type { KeyMapping as RhaiKeyMapping } from '@/utils/rhaiParser';

// Import custom hooks
import { useProfileSelection } from '@/hooks/useProfileSelection';
import { useCodePanel } from '@/hooks/useCodePanel';
import { useKeyboardLayout } from '@/hooks/useKeyboardLayout';
import { useConfigSync } from '@/hooks/useConfigSync';

// Import container components
import { CodePanelContainer } from '@/components/config/CodePanelContainer';
import { KeyboardVisualizerContainer } from '@/components/config/KeyboardVisualizerContainer';
import { ProfileSelector } from '@/components/config/ProfileSelector';

interface ConfigPageProps {
  profileName?: string;
}

const ConfigPage: React.FC<ConfigPageProps> = ({
  profileName: propProfileName,
}) => {
  const navigate = useNavigate();
  const api = useUnifiedApi();

  // Custom hooks for state management
  const { selectedProfileName, setSelectedProfileName } = useProfileSelection(propProfileName);
  const { isOpen: isCodePanelOpen, height: codePanelHeight, toggleOpen: toggleCodePanel, setHeight: setCodePanelHeight } = useCodePanel();
  const { layout: keyboardLayout, setLayout: setKeyboardLayout, layoutKeys } = useKeyboardLayout('ANSI_104');
  const { syncEngine, syncStatus, lastSaveTime, setSyncStatus, setLastSaveTime } = useConfigSync(selectedProfileName);

  // Fetch available profiles
  const { data: profiles, isLoading: isLoadingProfiles } = useProfiles();
  const { mutateAsync: createProfile } = useCreateProfile();

  // Visual editor state - now using Zustand store for layer-aware mappings
  const configStore = useConfigStore();
  const [selectedPhysicalKey, setSelectedPhysicalKey] = useState<string | null>(null);

  // Computed: Get current layer's mappings for display
  const keyMappings = configStore.getLayerMappings(configStore.activeLayer);
  const activeLayer = configStore.activeLayer;
  const globalSelected = configStore.globalSelected;
  const selectedDevices = configStore.selectedDevices;

  // Available layers
  const availableLayers = ['base', 'md-00', 'md-01', 'md-02', 'md-03', 'md-04', 'md-05'];

  // Responsive layout state: 'global' or 'device' for mobile/tablet views
  const [activePane, setActivePane] = useState<'global' | 'device'>('global');

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
      setSyncStatus('saved');
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
      setSyncStatus('unsaved');
    }
  }, [profileConfig, configMissing, selectedProfileName]);

  // Track code changes to update sync status
  useEffect(() => {
    // Mark as unsaved when code changes (except during save)
    if (syncStatus === 'saved' && syncEngine.state === 'idle') {
      const currentCode = syncEngine.getCode();
      const originalCode = profileConfig?.source || '';
      if (currentCode !== originalCode) {
        setSyncStatus('unsaved');
      }
    }
  }, [syncEngine.state, syncEngine.getCode()]);

  // Rhai-driven device detection: Extract devices from parsed Rhai and merge with connected devices
  useEffect(() => {
    const ast = syncEngine.getAST();
    if (!ast) {
      // No AST yet, just use connected devices (filter out disabled devices)
      setMergedDevices(
        devicesData
          ?.filter((d) => d.enabled !== false) // Exclude disabled devices
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
    const connectedDeviceMap = new Map<string, NonNullable<typeof devicesData>[number]>();
    devicesData
      ?.filter((device) => device.enabled !== false) // Exclude disabled devices
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
            name: pattern, // Use pattern as name for disconnected devices
            serial: pattern,
            connected: false,
          });
        }
      });

    // Add connected devices not in Rhai (filter out disabled devices)
    devicesData
      ?.filter((device) => device.enabled !== false) // Exclude disabled devices
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

    // Auto-populate device selector based on Rhai content
    // If Rhai has global mappings OR device_start("*"), select global
    // device_start("*") is equivalent to global - applies to all devices
    const hasWildcardDevice = devicePatternsInRhai.includes('*');
    if (hasGlobalMappings(ast) || hasWildcardDevice) {
      configStore.setGlobalSelected(true);
    }

    // If Rhai has device blocks, auto-select those devices (excluding "*" which is handled by global)
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
  }, [syncEngine.state, devicesData]); // Re-run when sync state changes (parsing complete) or devices change

  // Sync visual editor state from parsed AST - LAYER-AWARE VERSION
  useEffect(() => {
    // Only sync when state is idle (parsing complete)
    if (syncEngine.state !== 'idle') return;

    const ast = syncEngine.getAST();
    if (!ast) return;

    // Normalize key codes to VK_ format for consistent lookup
    // Handles: "1" -> "VK_1", "KC_A" -> "VK_A", "VK_A" -> "VK_A"
    const normalizeKeyCode = (key: string): string => {
      if (!key) return key;
      // Already VK_ format
      if (key.startsWith('VK_')) return key;
      // Convert KC_ to VK_
      if (key.startsWith('KC_')) return key.replace(/^KC_/, 'VK_');
      // Single character or number - add VK_ prefix
      if (/^[A-Z0-9]$/i.test(key)) return `VK_${key.toUpperCase()}`;
      // Named keys without prefix
      const knownKeys = ['ESCAPE', 'ENTER', 'SPACE', 'TAB', 'BACKSPACE', 'DELETE',
        'INSERT', 'HOME', 'END', 'PAGEUP', 'PAGEDOWN', 'UP', 'DOWN', 'LEFT', 'RIGHT',
        'CAPSLOCK', 'NUMLOCK', 'SCROLLLOCK', 'LEFTSHIFT', 'RIGHTSHIFT',
        'LEFTCONTROL', 'RIGHTCONTROL', 'LEFTALT', 'RIGHTALT', 'LEFTMETA', 'RIGHTMETA'];
      if (knownKeys.includes(key.toUpperCase())) return `VK_${key.toUpperCase()}`;
      // Already has some prefix or unknown - return as-is
      return key;
    };

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
      const wildcardBlock = ast.deviceBlocks.find((block) => block.pattern === '*');
      if (wildcardBlock) {
        wildcardBlock.mappings.forEach((m) => {
          baseMap.set(normalizeKeyCode(m.sourceKey), convertToVisualMapping(m));
        });

        // Also process layers from wildcard block
        wildcardBlock.layers.forEach((layer) => {
          const layerModifiers = Array.isArray(layer.modifiers) ? layer.modifiers : [layer.modifiers];
          layerModifiers.forEach((mod: string) => {
            const layerId = mod.toLowerCase().replace('_', '-');
            if (!layerMappings.has(layerId)) {
              layerMappings.set(layerId, new Map());
            }
            const layerMap = layerMappings.get(layerId)!;
            layer.mappings.forEach((m: RhaiKeyMapping) => {
              layerMap.set(normalizeKeyCode(m.sourceKey), convertToVisualMapping(m));
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
          ? selectedDevices.includes('disconnected-*') || selectedDevices.length > 0
          : (devicesData?.some((device) => {
              const isSelected = selectedDevices.includes(device.id);
              const matchesPattern =
                block.pattern === device.serial ||
                block.pattern === device.name ||
                block.pattern === device.id;
              return isSelected && matchesPattern;
            }) ?? false);

        if (matchesSelectedDevice) {
          // Add base mappings
          const baseMap = layerMappings.get('base')!;
          block.mappings.forEach((m) => {
            baseMap.set(normalizeKeyCode(m.sourceKey), convertToVisualMapping(m));
          });

          // Add layer-specific mappings
          block.layers.forEach((layer) => {
            const layerModifiers = Array.isArray(layer.modifiers) ? layer.modifiers : [layer.modifiers];

            // Convert each modifier to layer ID format (MD_00 -> md-00)
            layerModifiers.forEach((mod: string) => {
              const layerId = mod.toLowerCase().replace('_', '-'); // MD_00 -> md-00

              if (!layerMappings.has(layerId)) {
                layerMappings.set(layerId, new Map());
              }

              const layerMap = layerMappings.get(layerId)!;
              layer.mappings.forEach((m: RhaiKeyMapping) => {
                layerMap.set(normalizeKeyCode(m.sourceKey), convertToVisualMapping(m));
              });
            });
          });
        }
      });
    }

    // Load into store
    configStore.loadLayerMappings(layerMappings);
  }, [syncEngine.state, globalSelected, selectedDevices, devicesData]);

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
      setSyncStatus('saving');
      await setProfileConfig({ name: selectedProfileName, source: syncEngine.getCode() });
      setSyncStatus('saved');
      setLastSaveTime(new Date());
    } catch (err) {
      console.error('Failed to save config:', err);
      setSyncStatus('unsaved');
    }
  };

  // Handle key click: select key for inline configuration
  const handlePhysicalKeyClick = (keyCode: string) => {
    setSelectedPhysicalKey(keyCode);
  };

  // Handle clear mapping from summary - LAYER-AWARE
  const handleClearMapping = (keyCode: string) => {
    configStore.deleteKeyMapping(keyCode, activeLayer);
    setSyncStatus('unsaved');
    rebuildAndSyncAST();
  };

  // Handle save from modal - LAYER-AWARE
  const handleSaveMapping = (mapping: KeyMapping) => {
    if (!selectedPhysicalKey) return;

    // Save to active layer in store
    configStore.setKeyMapping(selectedPhysicalKey, mapping, activeLayer);
    setSyncStatus('unsaved');
    rebuildAndSyncAST();
  };

  // Helper: Rebuild AST from store and sync to code editor
  const rebuildAndSyncAST = () => {
    // Convert a KeyMapping to RhaiKeyMapping
    const convertToRhaiMapping = (key: string, m: KeyMapping): RhaiKeyMapping => {
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
          keys: m.macroSteps.filter(s => s.key).map(s => s.key!),
          delayMs: m.macroSteps.find(s => s.delayMs)?.delayMs,
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

    // Build global mappings (base layer only)
    const globalMappings: RhaiKeyMapping[] = [];
    if (globalSelected) {
      const baseMappings = configStore.getLayerMappings('base');
      baseMappings.forEach((mapping, key) => {
        globalMappings.push(convertToRhaiMapping(key, mapping));
      });
    }

    // Build device blocks with layer structures
    const deviceBlocks = selectedDevices.map((deviceId) => {
      const device = devices.find((d) => d.id === deviceId);
      if (!device) return null;

      // Base mappings for this device
      const baseMappings = configStore.getLayerMappings('base');
      const deviceBaseMappings: RhaiKeyMapping[] = [];
      baseMappings.forEach((mapping, key) => {
        deviceBaseMappings.push(convertToRhaiMapping(key, mapping));
      });

      // Layer-specific mappings
      const layers = allLayers
        .filter(layerId => layerId !== 'base')
        .map(layerId => {
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
        .filter(layer => layer.mappings.length > 0); // Only include layers with mappings

      return {
        pattern: device.serial || device.name,
        mappings: deviceBaseMappings,
        layers,
        startLine: 0,
        endLine: 0,
      };
    }).filter((block): block is NonNullable<typeof block> => block !== null);

    // Update sync engine with new AST
    syncEngine.onVisualChange({
      imports: [],
      globalMappings,
      deviceBlocks,
      comments: [],
    });
  };

  // Use merged device list (connected + disconnected from Rhai)
  const devices: Device[] = mergedDevices;

  return (
    <div className="flex flex-col gap-4 md:gap-6 p-4 md:p-6 lg:p-8">
      {/* Streamlined Header */}
      <div className="flex flex-col sm:flex-row items-stretch sm:items-center justify-between gap-3 pb-4 border-b border-slate-700">
        {/* Left: Profile Selector */}
        <ProfileSelector
          value={selectedProfileName}
          onChange={handleProfileChange}
          profiles={profiles}
          isLoading={isLoadingProfiles}
          disabled={!api.isConnected}
        />

        {/* Right: Sync Status and Save Button */}
        <div className="flex items-center gap-3">
          {/* Sync Status Indicator */}
          <div className="flex items-center gap-2">
            {syncStatus === 'saved' && (
              <div className="flex items-center gap-2 text-xs text-green-400" title="All changes saved">
                <span className="w-2 h-2 rounded-full bg-green-400"></span>
                <span className="hidden sm:inline">Saved</span>
                {lastSaveTime && (
                  <span className="text-slate-500 hidden md:inline">
                    {new Date().getTime() - lastSaveTime.getTime() < 60000
                      ? 'just now'
                      : `${Math.floor((new Date().getTime() - lastSaveTime.getTime()) / 60000)}m ago`}
                  </span>
                )}
              </div>
            )}
            {syncStatus === 'unsaved' && (
              <div className="flex items-center gap-2 text-xs text-yellow-400" title="Unsaved changes">
                <span className="w-2 h-2 rounded-full bg-yellow-400"></span>
                <span className="hidden sm:inline">Unsaved</span>
              </div>
            )}
            {syncStatus === 'saving' && (
              <div className="flex items-center gap-2 text-xs text-blue-400" title="Saving...">
                <span className="w-2 h-2 rounded-full bg-blue-400 animate-pulse"></span>
                <span className="hidden sm:inline">Saving...</span>
              </div>
            )}
            {!api.isConnected && (
              <div className="flex items-center gap-2 text-xs text-red-400" title="Disconnected from daemon">
                <span className="w-2 h-2 rounded-full bg-red-400"></span>
                <span className="hidden sm:inline">Disconnected</span>
              </div>
            )}
          </div>

          {/* Code Panel Toggle and Save Button */}
          <button
            onClick={toggleCodePanel}
            className="px-4 py-2 bg-slate-700 text-slate-200 text-sm font-medium rounded-md hover:bg-slate-600 transition-colors whitespace-nowrap border border-slate-600"
            title={isCodePanelOpen ? 'Hide Code' : 'Show Code'}
          >
            {isCodePanelOpen ? '▼ Hide Code' : '▲ Show Code'}
          </button>

          <button
            onClick={handleSaveConfig}
            disabled={!api.isConnected || !profileExists || syncStatus === 'saving'}
            className="px-4 py-2 bg-primary-500 text-white text-sm font-medium rounded-md hover:bg-primary-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors whitespace-nowrap"
          >
            {configMissing ? 'Create' : 'Save'}
          </button>
        </div>
      </div>

      {/* Error/Info Messages */}
      {!profileExists && !isLoading && api.isConnected && (
        <div className="p-3 bg-orange-900/20 border border-orange-500 rounded-md">
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
        <div className="p-3 bg-blue-900/20 border border-blue-500 rounded-md">
          <p className="text-sm text-blue-300">
            📝 No configuration file found for "{selectedProfileName}". A template has been loaded - click <strong>Save</strong> to create it.
          </p>
        </div>
      )}

      {error && (
        <div className="p-3 bg-red-900/20 border border-red-500 rounded-md">
          <p className="text-sm text-red-300">
            {error instanceof Error ? error.message : 'Failed to load configuration'}
          </p>
        </div>
      )}

      {/* Visual Editor Content (Always visible) */}
      <div className="flex flex-col gap-4">
          {/* Device Selection Panel (compact at top) */}
          <Card>
            <div className="flex items-center gap-4 flex-wrap" data-testid="device-selector">
              <label className="flex items-center gap-2">
                <input
                  type="checkbox"
                  checked={globalSelected}
                  onChange={(e) => configStore.setGlobalSelected(e.target.checked)}
                  className="w-4 h-4 text-primary-600 bg-slate-700 border-slate-600 rounded focus:ring-primary-500 focus:ring-2"
                  aria-label="Enable global configuration"
                  data-testid="global-checkbox"
                />
                <span className="text-sm font-medium text-slate-200">
                  Global (All Devices)
                </span>
              </label>

              <div className="h-5 w-px bg-slate-700"></div>

              <div className="flex items-center gap-2 flex-wrap">
                <span className="text-sm font-medium text-slate-300">Devices:</span>
                {/* Filter out "*" device - it's represented by the Global checkbox */}
                {devices.filter((d) => d.name !== '*' && d.serial !== '*').length > 0 ? (
                  devices.filter((d) => d.name !== '*' && d.serial !== '*').map((device) => (
                    <label
                      key={device.id}
                      className="flex items-center gap-2 px-3 py-1.5 bg-slate-700/50 rounded-md hover:bg-slate-700 cursor-pointer transition-colors"
                      data-testid={device.connected === false ? `disconnected-${device.id}` : undefined}
                    >
                      <input
                        type="checkbox"
                        checked={selectedDevices.includes(device.id)}
                        onChange={(e) => {
                          if (e.target.checked) {
                            configStore.setSelectedDevices([...selectedDevices, device.id]);
                          } else {
                            configStore.setSelectedDevices(selectedDevices.filter((id) => id !== device.id));
                          }
                        }}
                        className="w-4 h-4 text-primary-600 bg-slate-700 border-slate-600 rounded focus:ring-primary-500 focus:ring-2"
                        aria-label={`Select device ${device.name}`}
                      />
                      <span className="text-sm text-slate-200">{device.name}</span>
                      {device.connected !== undefined && (
                        <span
                          className={`w-2 h-2 rounded-full ${
                            device.connected ? 'bg-green-400' : 'bg-gray-500'
                          }`}
                          title={device.connected ? 'Connected' : 'Disconnected'}
                          aria-label={device.connected ? 'Connected' : 'Disconnected'}
                        />
                      )}
                    </label>
                  ))
                ) : (
                  <span className="text-sm text-slate-500">No devices detected</span>
                )}
              </div>
            </div>
          </Card>

          {/* Tab Navigation - Accessible tabs for Global/Device switching */}
          {globalSelected && selectedDevices.length > 0 && (
            <div
              role="tablist"
              aria-label="Keyboard configuration scope"
              className="flex gap-2 border-b border-slate-700"
              data-testid="pane-switcher"
            >
              <button
                role="tab"
                aria-selected={activePane === 'global'}
                aria-controls="panel-global"
                id="tab-global"
                onClick={() => setActivePane('global')}
                data-testid="pane-global"
                className={`flex-1 px-4 py-2 font-medium transition-colors focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-2 focus:ring-offset-slate-900 ${
                  activePane === 'global'
                    ? 'text-primary-400 border-b-2 border-primary-400'
                    : 'text-slate-400 hover:text-slate-300'
                }`}
              >
                Global Keys
              </button>
              <button
                role="tab"
                aria-selected={activePane === 'device'}
                aria-controls="panel-device"
                id="tab-device"
                onClick={() => setActivePane('device')}
                data-testid="pane-device"
                className={`flex-1 px-4 py-2 font-medium transition-colors focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-2 focus:ring-offset-slate-900 ${
                  activePane === 'device'
                    ? 'text-primary-400 border-b-2 border-primary-400'
                    : 'text-slate-400 hover:text-slate-300'
                }`}
              >
                Device Keys
              </button>
            </div>
          )}

          {/* Single-Pane Layout: Show one pane at a time (tabs control visibility) */}
          <div className="flex flex-col gap-4">
            {/* Global Keyboard Panel */}
            {globalSelected && (
              <div
                role="tabpanel"
                id="panel-global"
                aria-labelledby="tab-global"
                className={`flex flex-col gap-3 ${
                  // Show only when selected (or when device tabs don't exist)
                  selectedDevices.length > 0 && activePane !== 'global' ? 'hidden' : 'flex'
                }`}
              >
                {/* Global Pane Header */}
                <div className="flex items-center justify-between px-4 py-2 bg-slate-800/50 border border-slate-700 rounded-md">
                  <h2 className="text-lg font-semibold text-slate-200">Global Keys</h2>
                  <div className="flex items-center gap-2">
                    <input
                      type="checkbox"
                      id="global-checkbox"
                      checked={globalSelected}
                      onChange={(e) => configStore.setGlobalSelected(e.target.checked)}
                      className="w-4 h-4 text-primary-600 bg-slate-700 border-slate-600 rounded focus:ring-primary-500 focus:ring-2"
                    />
                    <label htmlFor="global-checkbox" className="text-sm text-slate-300">
                      Enable
                    </label>
                  </div>
                </div>

                {/* Global Keyboard Content */}
                <div className="flex gap-2 flex-1 bg-slate-900/30 rounded-lg p-3">
                  <LayerSwitcher
                    activeLayer={activeLayer}
                    availableLayers={availableLayers}
                    onLayerChange={configStore.setActiveLayer}
                  />
                  <Card className="bg-gradient-to-br from-slate-800 to-slate-900 flex-1">
                    <h3 className="text-xl font-bold text-primary-400 mb-4">
                      Global Keyboard (All Devices)
                    </h3>
                    <KeyboardVisualizerContainer
                      profileName={selectedProfileName}
                      activeLayer={activeLayer}
                      mappings={keyMappings}
                      onKeyClick={handlePhysicalKeyClick}
                      selectedKeyCode={selectedPhysicalKey}
                      initialLayout={keyboardLayout}
                    />
                  </Card>
                </div>
              </div>
            )}

            {/* Device-Specific Keyboard Panel */}
            {selectedDevices.length > 0 && devices
              .filter((d) => selectedDevices.includes(d.id))
              .map((device) => (
                <div
                  key={device.id}
                  role="tabpanel"
                  id="panel-device"
                  aria-labelledby="tab-device"
                  className={`flex flex-col gap-3 ${
                    // Show only when selected (or when global is not selected)
                    globalSelected && activePane !== 'device' ? 'hidden' : 'flex'
                  }`}
                >
                  {/* Device Pane Header */}
                  <div className="flex items-center justify-between px-4 py-2 bg-zinc-800/50 border border-zinc-700 rounded-md">
                    <div className="flex items-center gap-2">
                      <label htmlFor={`device-selector-${device.id}`} className="text-lg font-semibold text-slate-200">
                        Device:
                      </label>
                      <select
                        id={`device-selector-${device.id}`}
                        value={device.id}
                        onChange={(e) => {
                          const newDeviceId = e.target.value;
                          // Replace current device with new selection
                          const updatedDevices = selectedDevices.filter(id => id !== device.id);
                          configStore.setSelectedDevices([...updatedDevices, newDeviceId]);
                        }}
                        className="px-3 py-1.5 bg-zinc-700 border border-zinc-600 rounded-md text-slate-100 text-sm font-medium focus:outline-none focus:ring-2 focus:ring-primary-500"
                        aria-label="Select device to configure"
                      >
                        {devices.map((d) => (
                          <option key={d.id} value={d.id}>
                            {d.name} {d.serial ? `(${d.serial})` : ''}
                          </option>
                        ))}
                      </select>
                    </div>
                    <div className="flex items-center gap-2">
                      <span className={`text-xs px-2 py-1 rounded ${
                        device.connected
                          ? 'bg-green-900/30 border border-green-500 text-green-400'
                          : 'bg-gray-900/30 border border-gray-500 text-gray-400'
                      }`}>
                        {device.connected ? '● Connected' : '○ Disconnected'}
                      </span>
                    </div>
                  </div>

                  {/* Device Keyboard Content */}
                  <div className="flex gap-2 flex-1 bg-zinc-900/30 rounded-lg p-3">
                    <LayerSwitcher
                      activeLayer={activeLayer}
                      availableLayers={availableLayers}
                      onLayerChange={configStore.setActiveLayer}
                    />
                    <Card className="bg-gradient-to-br from-zinc-800 to-zinc-900 flex-1">
                      <h3 className="text-xl font-bold text-primary-400 mb-4">
                        {device.name}
                        {device.serial && (
                          <span className="ml-2 text-sm text-slate-400 font-normal">
                            ({device.serial})
                          </span>
                        )}
                      </h3>
                      <KeyboardVisualizerContainer
                        profileName={selectedProfileName}
                        activeLayer={activeLayer}
                        mappings={keyMappings}
                        onKeyClick={handlePhysicalKeyClick}
                        selectedKeyCode={selectedPhysicalKey}
                        initialLayout={keyboardLayout}
                      />
                    </Card>
                  </div>
                </div>
              ))}

            {/* Warning if no selection */}
            {!globalSelected && selectedDevices.length === 0 && (
              <Card className="bg-yellow-900/20 border border-yellow-700/50 flex-1 block">
                <div className="text-center py-8">
                  <p className="text-yellow-200 text-lg mb-2">⚠️ No devices selected</p>
                  <p className="text-yellow-300 text-sm">
                    Select at least one device or enable "Global Keys" to configure key mappings
                  </p>
                </div>
              </Card>
            )}
          </div>

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

          {/* Current Mappings Summary - Shows active mappings with edit/delete */}
          {/* Inline Key Configuration Panel */}
          <KeyConfigPanel
            physicalKey={selectedPhysicalKey}
            currentMapping={selectedPhysicalKey ? keyMappings.get(selectedPhysicalKey) : undefined}
            onSave={handleSaveMapping}
            onClearMapping={handleClearMapping}
            onEditMapping={handlePhysicalKeyClick}
            activeLayer={activeLayer}
            keyMappings={keyMappings}
            layoutKeys={layoutKeys}
          />
      </div>

      {/* Collapsible Code Panel */}
      <CodePanelContainer
        profileName={selectedProfileName}
        rhaiCode={syncEngine.getCode()}
        onChange={(value) => syncEngine.onCodeChange(value)}
        syncEngine={syncEngine}
      />
    </div>
  );
};

export default ConfigPage;
