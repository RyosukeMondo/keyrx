import React, { useState } from 'react';
import { Button } from '@/components/Button';
import { Input } from '@/components/Input';
import { LayoutDropdown } from '@/components/LayoutDropdown';
import { useUpdateDevice } from '@/hooks/useUpdateDevice';
import { LAYOUT_OPTIONS } from '@/contexts/LayoutPreviewContext';

interface Device {
  id: string;
  name: string;
  identifier: string;
  layout: string;
  active: boolean;
  enabled: boolean;
  vendorId?: string;
  productId?: string;
  serial?: string;
  lastSeen?: number;
}

export interface DeviceRowProps {
  device: Device;
  isEditing: boolean;
  editingName: string;
  nameError: string;
  onRenameClick: (device: Device) => void;
  onRenameCancel: () => void;
  onRenameSave: (deviceId: string) => void;
  onEditingNameChange: (value: string) => void;
  onToggleEnabled: (deviceId: string, enabled: boolean) => void;
  onForgetClick: (deviceId: string) => void;
}

/**
 * DeviceRow Component
 *
 * Compact single-row device display with inline layout selector and enable/disable toggle.
 */
export const DeviceRow: React.FC<DeviceRowProps> = ({
  device,
  isEditing,
  editingName,
  nameError,
  onRenameClick,
  onRenameCancel,
  onRenameSave,
  onEditingNameChange,
  onToggleEnabled,
  onForgetClick,
}) => {
  const {
    mutate: updateDevice,
    isPending: isSaving,
    error: saveError,
  } = useUpdateDevice();
  const [lastSavedAt, setLastSavedAt] = useState<Date | null>(null);

  // Simple: save immediately on change, no auto-save complexity
  const handleLayoutChange = (newLayout: string) => {
    updateDevice(
      { id: device.id, layout: newLayout },
      {
        onSuccess: () => {
          setLastSavedAt(new Date());
          // Clear success indicator after 2 seconds
          setTimeout(() => setLastSavedAt(null), 2000);
        },
      }
    );
  };

  return (
    <div
      className={`flex items-center gap-3 px-4 py-3 bg-slate-800 rounded-lg border border-slate-700 hover:border-slate-600 transition-all ${
        !device.enabled ? 'opacity-50 bg-slate-900' : ''
      }`}
      data-testid="device-card"
    >
      {/* Status indicator */}
      <div
        className={`w-2 h-2 rounded-full flex-shrink-0 ${
          device.active ? 'bg-green-500' : 'bg-slate-500'
        }`}
        title={device.active ? 'Connected' : 'Disconnected'}
        aria-label={device.active ? 'Connected' : 'Disconnected'}
      />

      {/* Device name */}
      <div className="flex-1 min-w-0">
        {isEditing ? (
          <div
            className="flex items-center gap-2"
            onKeyDown={(e) => {
              if (e.key === 'Enter') onRenameSave(device.id);
              else if (e.key === 'Escape') onRenameCancel();
            }}
          >
            <Input
              type="text"
              value={editingName}
              onChange={onEditingNameChange}
              error={nameError}
              maxLength={64}
              aria-label="Device name"
              className="!py-1 !text-sm"
            />
            <Button
              variant="primary"
              size="sm"
              onClick={() => onRenameSave(device.id)}
              aria-label="Save"
            >
              ✓
            </Button>
            <Button
              variant="ghost"
              size="sm"
              onClick={onRenameCancel}
              aria-label="Cancel"
            >
              ✕
            </Button>
          </div>
        ) : (
          <button
            onClick={() => onRenameClick(device)}
            className="text-left w-full group"
            aria-label={`Rename ${device.name}`}
          >
            <div className="flex items-center gap-2">
              <span className="text-sm font-medium text-slate-100 group-hover:text-blue-400 transition-colors truncate block">
                {device.name}
              </span>
              {!device.enabled && (
                <span className="text-xs px-2 py-0.5 bg-slate-700 text-slate-400 rounded-full">
                  Disabled
                </span>
              )}
            </div>
            <span className="text-xs font-mono text-slate-500 truncate block">
              {device.identifier}
            </span>
          </button>
        )}
      </div>

      {/* Layout selector */}
      <div className="flex items-center gap-2 flex-shrink-0">
        <div className="relative">
          <LayoutDropdown
            options={LAYOUT_OPTIONS}
            value={device.layout}
            onChange={handleLayoutChange}
            aria-label="Layout"
            compact
            disabled={isSaving}
          />
        </div>
        {isSaving && (
          <span className="animate-spin h-3 w-3 border-2 border-slate-400 border-t-transparent rounded-full" />
        )}
        {!isSaving && lastSavedAt && (
          <span className="text-green-500 text-xs">✓</span>
        )}
        {saveError && (
          <span
            className="text-red-500 text-xs"
            title={
              saveError instanceof Error ? saveError.message : String(saveError)
            }
          >
            ✗
          </span>
        )}
      </div>

      {/* Enable/Disable toggle */}
      <div className="flex items-center gap-2 flex-shrink-0">
        <button
          onClick={() => onToggleEnabled(device.id, !device.enabled)}
          className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 focus:ring-offset-slate-800 ${
            device.enabled ? 'bg-blue-600' : 'bg-slate-600'
          }`}
          role="switch"
          aria-checked={device.enabled}
          aria-label={`${device.enabled ? 'Disable' : 'Enable'} ${device.name}`}
        >
          <span
            className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
              device.enabled ? 'translate-x-6' : 'translate-x-1'
            }`}
          />
        </button>
        <Button
          variant="ghost"
          size="sm"
          onClick={() => onForgetClick(device.id)}
          aria-label={`Permanently forget ${device.name}`}
          className="text-slate-400 hover:text-red-400"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            className="h-4 w-4"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
            />
          </svg>
        </Button>
      </div>
    </div>
  );
};
