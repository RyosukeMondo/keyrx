import React from 'react';
import { RadioGroup } from '@headlessui/react';
import { cn } from '@/utils/cn';
import { Dropdown } from './Dropdown';

/**
 * Scope type for key mappings
 */
export type MappingScope = 'global' | 'device-specific';

/**
 * Device option for the device selector
 */
export interface DeviceOption {
  /** Device serial number or unique identifier */
  serial: string;
  /** Device display name */
  name: string;
}

export interface DeviceScopeToggleProps {
  /** Current scope selection */
  scope: MappingScope;
  /** Callback when scope changes */
  onScopeChange: (scope: MappingScope) => void;
  /** Available devices for device-specific mapping */
  devices: DeviceOption[];
  /** Currently selected device (required when scope is 'device-specific') */
  selectedDevice?: string;
  /** Callback when selected device changes */
  onDeviceChange?: (deviceSerial: string) => void;
  /** Whether the component is disabled */
  disabled?: boolean;
  /** CSS class name for styling */
  className?: string;
}

/**
 * DeviceScopeToggle component for switching between global and device-specific key mapping scopes.
 *
 * Features:
 * - Segmented control for Global vs Device-Specific toggle
 * - Device selector dropdown (shown only when device-specific mode is active)
 * - Accessible with proper ARIA labels and radio group semantics
 * - Responsive design that works on mobile and desktop
 * - Controlled component pattern (value/onChange props)
 *
 * @example
 * ```tsx
 * const [scope, setScope] = useState<MappingScope>('global');
 * const [device, setDevice] = useState<string>('');
 *
 * <DeviceScopeToggle
 *   scope={scope}
 *   onScopeChange={setScope}
 *   devices={availableDevices}
 *   selectedDevice={device}
 *   onDeviceChange={setDevice}
 * />
 * ```
 */
export const DeviceScopeToggle: React.FC<DeviceScopeToggleProps> = ({
  scope,
  onScopeChange,
  devices,
  selectedDevice,
  onDeviceChange,
  disabled = false,
  className = '',
}) => {
  const handleDeviceChange = (deviceSerial: string) => {
    if (onDeviceChange) {
      onDeviceChange(deviceSerial);
    }
  };

  // Convert devices to dropdown options
  const deviceOptions = devices.map((device) => ({
    value: device.serial,
    label: device.name,
  }));

  return (
    <div className={cn('space-y-3', className)}>
      {/* Scope selector - Segmented control */}
      <div>
        <label className="block text-sm font-medium text-slate-300 mb-2">
          Mapping Scope
        </label>
        <RadioGroup
          value={scope}
          onChange={onScopeChange}
          disabled={disabled}
          className="w-full"
        >
          <RadioGroup.Label className="sr-only">
            Select mapping scope
          </RadioGroup.Label>
          <div
            className="inline-flex w-full rounded-md border border-slate-600 bg-slate-800 p-1"
            role="group"
            aria-label="Mapping scope selector"
          >
            {/* Global option */}
            <RadioGroup.Option value="global">
              {({ checked }) => (
                <button
                  type="button"
                  className={cn(
                    'flex-1 rounded px-4 py-2 text-sm font-medium transition-all duration-150',
                    'focus:outline focus:outline-2 focus:outline-primary-500 focus:outline-offset-2',
                    checked
                      ? 'bg-primary-500 text-white shadow-sm'
                      : 'bg-transparent text-slate-300 hover:text-slate-100',
                    disabled && 'opacity-50 cursor-not-allowed'
                  )}
                  aria-pressed={checked}
                  disabled={disabled}
                >
                  Global
                </button>
              )}
            </RadioGroup.Option>

            {/* Device-specific option */}
            <RadioGroup.Option value="device-specific">
              {({ checked }) => (
                <button
                  type="button"
                  className={cn(
                    'flex-1 rounded px-4 py-2 text-sm font-medium transition-all duration-150',
                    'focus:outline focus:outline-2 focus:outline-primary-500 focus:outline-offset-2',
                    checked
                      ? 'bg-primary-500 text-white shadow-sm'
                      : 'bg-transparent text-slate-300 hover:text-slate-100',
                    disabled && 'opacity-50 cursor-not-allowed'
                  )}
                  aria-pressed={checked}
                  disabled={disabled}
                >
                  Device-Specific
                </button>
              )}
            </RadioGroup.Option>
          </div>
        </RadioGroup>
      </div>

      {/* Device selector - Only shown when device-specific is selected */}
      {scope === 'device-specific' && (
        <div>
          <label
            htmlFor="device-selector"
            className="block text-sm font-medium text-slate-300 mb-2"
          >
            Select Device
          </label>
          {devices.length === 0 ? (
            <div
              className="rounded-md border border-slate-600 bg-slate-800/50 px-4 py-3 text-sm text-slate-400"
              role="alert"
            >
              No devices available. Connect a device to use device-specific
              mappings.
            </div>
          ) : (
            <Dropdown
              options={deviceOptions}
              value={selectedDevice || ''}
              onChange={handleDeviceChange}
              aria-label="Select device for device-specific mapping"
              placeholder="Select a device..."
              disabled={disabled}
            />
          )}
        </div>
      )}

      {/* Help text */}
      <p className="text-xs text-slate-400">
        {scope === 'global' ? (
          <>Global mappings apply to all connected devices.</>
        ) : (
          <>
            Device-specific mappings apply only to the selected device.
            {devices.length === 0 && ' Connect a device to get started.'}
          </>
        )}
      </p>
    </div>
  );
};

DeviceScopeToggle.displayName = 'DeviceScopeToggle';
