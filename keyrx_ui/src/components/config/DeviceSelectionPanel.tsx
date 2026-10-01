import React from 'react';
import { Card } from '../Card';

import type { Device } from '../DeviceSelector';
import { t } from '@/i18n';

export type { Device };

/** Human-readable status for a pattern scope, or null for a plain device. */
function patternSummary(device: Device): string | null {
  if (device.pattern === undefined || device.matchedNames === undefined) {
    return null;
  }
  const n = device.matchedNames.length;
  if (n === 0) return t('devsel.noMatch');
  return t('devsel.matches', {
    count: n,
    name: device.matchedNames[0] ?? '',
  });
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
    <Card aria-label={t('devsel.aria')}>
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
            aria-label={t('devsel.globalAria')}
            data-testid="global-checkbox"
          />
          <span className="text-sm font-medium text-slate-200">
            {t('devsel.global')}
          </span>
        </label>

        <div className="h-5 w-px bg-slate-700"></div>

        <div className="flex items-center gap-2 flex-wrap">
          <span className="text-sm font-medium text-slate-300">
            {t('devsel.devices')}
          </span>
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
                  aria-label={t('devsel.select', { name: device.name })}
                />
                <span className="text-sm text-slate-200">
                  {device.name}
                  {patternSummary(device) && (
                    <span className="ml-1.5 text-xs text-slate-400">
                      ({patternSummary(device)})
                    </span>
                  )}
                </span>
                {device.connected !== undefined && (
                  <span
                    role="status"
                    className={`w-2 h-2 rounded-full ${
                      device.connected ? 'bg-green-400' : 'bg-gray-500'
                    }`}
                    title={
                      device.connected
                        ? t('device.connected')
                        : t('device.disconnected')
                    }
                    aria-label={
                      device.connected
                        ? t('device.connected')
                        : t('device.disconnected')
                    }
                  />
                )}
              </label>
            ))
          ) : (
            <span className="text-sm text-slate-400">{t('devsel.none')}</span>
          )}
        </div>
      </div>
    </Card>
  );
};
