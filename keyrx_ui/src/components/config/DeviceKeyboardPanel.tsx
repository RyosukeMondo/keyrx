import React from 'react';
import { Card } from '@/components/Card';
import { LayerSwitcher } from '@/components/LayerSwitcher';
import { KeyboardVisualizerContainer } from './KeyboardVisualizerContainer';
import type { LayoutType } from '@/components/KeyboardVisualizer';
import type { Device } from '@/components/DeviceSelector';
import type { KeyMapping } from '@/types';

interface DeviceKeyboardPanelProps {
  profileName: string;
  activeLayer: string;
  availableLayers: string[];
  onLayerChange: (layer: string) => void;
  devices: Device[];
  selectedDevices: string[];
  onDeviceChange: (oldDeviceId: string, newDeviceId: string) => void;
  keyMappings: Map<string, KeyMapping>;
  onKeyClick: (keyCode: string) => void;
  selectedKeyCode: string | null;
  initialLayout: LayoutType | undefined;
  isVisible: boolean;
}

/**
 * Panel for configuring device-specific keyboard mappings
 */
export const DeviceKeyboardPanel: React.FC<DeviceKeyboardPanelProps> = ({
  profileName,
  activeLayer,
  availableLayers,
  onLayerChange,
  devices,
  selectedDevices,
  onDeviceChange,
  keyMappings,
  onKeyClick,
  selectedKeyCode,
  initialLayout,
  isVisible,
}) => {
  if (selectedDevices.length === 0) return null;

  return (
    <>
      {devices
        .filter((d) => selectedDevices.includes(d.id))
        .map((device) => (
          <div
            key={device.id}
            role="tabpanel"
            id="panel-device"
            aria-labelledby="tab-device"
            className={`flex flex-col gap-3 ${isVisible ? 'flex' : 'hidden'}`}
          >
            {/* Device Pane Header */}
            <div className="flex items-center justify-between px-4 py-2 bg-zinc-800/50 border border-zinc-700 rounded-md">
              <div className="flex items-center gap-2">
                <label
                  htmlFor={`device-selector-${device.id}`}
                  className="text-lg font-semibold text-slate-200"
                >
                  Device:
                </label>
                <select
                  id={`device-selector-${device.id}`}
                  value={device.id}
                  onChange={(e) => onDeviceChange(device.id, e.target.value)}
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
                <span
                  className={`text-xs px-2 py-1 rounded ${
                    device.connected
                      ? 'bg-green-900/30 border border-green-500 text-green-400'
                      : 'bg-gray-900/30 border border-gray-500 text-gray-400'
                  }`}
                >
                  {device.connected ? '● Connected' : '○ Disconnected'}
                </span>
              </div>
            </div>

            {/* Device Keyboard Content */}
            <div className="flex gap-2 flex-1 bg-zinc-900/30 rounded-lg p-3">
              <LayerSwitcher
                activeLayer={activeLayer}
                availableLayers={availableLayers}
                onLayerChange={onLayerChange}
              />
              <Card
                className="bg-gradient-to-br from-zinc-800 to-zinc-900 flex-1"
                aria-label="Device-Specific Keyboard Configuration"
              >
                <h3 className="text-xl font-bold text-primary-400 mb-4">
                  {device.name}
                  {device.serial && (
                    <span className="ml-2 text-sm text-slate-400 font-normal">
                      ({device.serial})
                    </span>
                  )}
                </h3>
                <KeyboardVisualizerContainer
                  profileName={profileName}
                  activeLayer={activeLayer}
                  mappings={keyMappings}
                  onKeyClick={onKeyClick}
                  selectedKeyCode={selectedKeyCode}
                  initialLayout={initialLayout}
                />
              </Card>
            </div>
          </div>
        ))}
    </>
  );
};
