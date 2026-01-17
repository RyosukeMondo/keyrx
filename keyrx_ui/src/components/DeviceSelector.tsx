import React from 'react';
import { Card } from './Card';

/**
 * Device Selector - Multi-device selection with global option
 * Replaces old scope-based selection with device-aware configuration
 */

export interface Device {
  id: string;
  name: string;
  serial?: string;
  connected?: boolean;
  layout?: string;
}

interface DeviceSelectorProps {
  /** List of available devices */
  devices: Device[];
  /** Whether to show global checkbox option */
  showGlobalOption?: boolean;
  /** Whether to allow multiple device selection */
  multiSelect?: boolean;
  /** Currently selected device IDs */
  selectedDevices: string[];
  /** Whether global is selected */
  globalSelected: boolean;
  /** Called when selection changes */
  onSelectionChange: (devices: string[], global: boolean) => void;
  /** Called when device edit is requested */
  onEditDevice?: (deviceId: string) => void;
}

export function DeviceSelector({
  devices,
  showGlobalOption = true,
  multiSelect = true,
  selectedDevices,
  globalSelected,
  onSelectionChange,
  onEditDevice,
}: DeviceSelectorProps) {
  const handleGlobalToggle = () => {
    onSelectionChange(selectedDevices, !globalSelected);
  };

  const handleDeviceToggle = (deviceId: string) => {
    if (multiSelect) {
      const newSelected = selectedDevices.includes(deviceId)
        ? selectedDevices.filter((id) => id !== deviceId)
        : [...selectedDevices, deviceId];
      onSelectionChange(newSelected, globalSelected);
    } else {
      onSelectionChange([deviceId], globalSelected);
    }
  };

  const hasSelection = globalSelected || selectedDevices.length > 0;

  return (
    <Card>
      <h3 className="text-sm font-semibold text-slate-100 mb-3">
        Device Selection
      </h3>

      {/* Warning if no selection */}
      {!hasSelection && (
        <div
          className="mb-3 px-3 py-2 bg-yellow-900/20 border border-yellow-700/50 rounded-md text-yellow-200 text-xs"
          role="alert"
          aria-live="polite"
        >
          ⚠ Select at least one device or global to configure
        </div>
      )}

      {/* Global checkbox */}
      {showGlobalOption && (
        <label
          className="flex items-center gap-3 px-3 py-2.5 mb-2 rounded-md bg-slate-700/50 hover:bg-slate-700 cursor-pointer transition-colors"
          htmlFor="device-global"
        >
          <input
            type="checkbox"
            id="device-global"
            checked={globalSelected}
            onChange={handleGlobalToggle}
            className="w-4 h-4 rounded border-slate-500 bg-slate-600 text-primary-500 focus:ring-2 focus:ring-primary-500 focus:ring-offset-0 cursor-pointer"
            aria-label="Apply configuration globally to all devices"
          />
          <div className="flex-1">
            <div className="text-sm font-medium text-slate-100">
              Global (All Devices)
            </div>
            <div className="text-xs text-slate-400">
              Configuration applies to all devices
            </div>
          </div>
        </label>
      )}

      {/* Device list */}
      {devices.length > 0 ? (
        <div className="space-y-2">
          {devices.map((device) => (
            <label
              key={device.id}
              className="flex items-center gap-3 px-3 py-2.5 rounded-md bg-slate-700/50 hover:bg-slate-700 cursor-pointer transition-colors"
              htmlFor={`device-${device.id}`}
            >
              <input
                type="checkbox"
                id={`device-${device.id}`}
                checked={selectedDevices.includes(device.id)}
                onChange={() => handleDeviceToggle(device.id)}
                className="w-4 h-4 rounded border-slate-500 bg-slate-600 text-primary-500 focus:ring-2 focus:ring-primary-500 focus:ring-offset-0 cursor-pointer"
                aria-label={`Select device ${device.name}`}
              />
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2">
                  <span className="text-sm font-medium text-slate-100 truncate">
                    {device.name}
                  </span>
                  {/* Connection status badge */}
                  {device.connected !== undefined && (
                    <span
                      className={`px-2 py-0.5 text-xs font-medium rounded-full ${
                        device.connected
                          ? 'bg-green-900/30 text-green-300 border border-green-700/50'
                          : 'bg-gray-700/30 text-gray-400 border border-gray-600/50'
                      }`}
                      aria-label={
                        device.connected
                          ? 'Device connected'
                          : 'Device disconnected'
                      }
                    >
                      {device.connected ? 'Connected' : 'Disconnected'}
                    </span>
                  )}
                </div>
                {device.serial && (
                  <div className="text-xs text-slate-500 truncate">
                    Serial: {device.serial}
                  </div>
                )}
                {device.layout && (
                  <div className="text-xs text-slate-500">
                    Layout: {device.layout}
                  </div>
                )}
              </div>
              {/* Edit button */}
              {onEditDevice && (
                <button
                  onClick={(e) => {
                    e.preventDefault();
                    onEditDevice(device.id);
                  }}
                  className="px-2 py-1 text-xs text-primary-400 hover:text-primary-300 hover:bg-slate-600 rounded transition-colors"
                  aria-label={`Edit device ${device.name}`}
                  type="button"
                >
                  Edit
                </button>
              )}
            </label>
          ))}
        </div>
      ) : (
        <div className="text-sm text-slate-500 text-center py-4">
          No devices detected. Connect a keyboard to get started.
        </div>
      )}

      {/* Info text */}
      <p className="text-xs text-slate-500 mt-3">
        {multiSelect
          ? 'Select one or more devices to configure. Device-specific mappings will be generated in Rhai script.'
          : 'Select a device to configure.'}
      </p>
    </Card>
  );
}
