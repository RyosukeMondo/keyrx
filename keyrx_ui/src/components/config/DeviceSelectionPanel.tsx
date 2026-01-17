import React from 'react';
import { Card } from '../Card';

export interface Device {
  id: string;
  name: string;
  serial?: string;
  connected?: boolean;
}

interface DeviceSelectionPanelProps {
  devices: Device[];
  globalSelected: boolean;
  selectedDevices: string[];
  onToggleGlobal: (selected: boolean) => void;
  onToggleDevice: (deviceId: string, selected: boolean) => void;
}

/**
 * Panel for selecting global or specific devices for configuration
 */
export const DeviceSelectionPanel: React.FC<DeviceSelectionPanelProps> = ({
  devices,
  globalSelected,
  selectedDevices,
  onToggleGlobal,
  onToggleDevice,
}) => {
  const filteredDevices = devices.filter(
    (d) => d.name !== '*' && d.serial !== '*'
  );

  return (
    <Card aria-label="Device Selection">
      <div
        className="flex items-center gap-4 flex-wrap"
        data-testid="device-selector"
      >
        <label className="flex items-center gap-2">
          <input
            type="checkbox"
            checked={globalSelected}
            onChange={(e) => onToggleGlobal(e.target.checked)}
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
          {filteredDevices.length > 0 ? (
            filteredDevices.map((device) => (
              <label
                key={device.id}
                className="flex items-center gap-2 px-3 py-1.5 bg-slate-700/50 rounded-md hover:bg-slate-700 cursor-pointer transition-colors"
                data-testid={
                  device.connected === false
                    ? `disconnected-${device.id}`
                    : undefined
                }
              >
                <input
                  type="checkbox"
                  checked={selectedDevices.includes(device.id)}
                  onChange={(e) => onToggleDevice(device.id, e.target.checked)}
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
            <span className="text-sm text-slate-500">No devices</span>
          )}
        </div>
      </div>
    </Card>
  );
};
