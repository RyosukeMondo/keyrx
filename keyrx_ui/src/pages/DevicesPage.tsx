import React, { useState } from 'react';
import { Card } from '../components/Card';
import { Button } from '../components/Button';
import { Input } from '../components/Input';
import { LayoutDropdown } from '../components/LayoutDropdown';
import { Modal } from '../components/Modal';
import { LoadingSkeleton } from '../components/LoadingSkeleton';
import { useAutoSave } from '../hooks/useAutoSave';
import { useUpdateDevice } from '../hooks/useUpdateDevice';
import {
  useDevices,
  useSetDeviceEnabled,
  useForgetDevice,
} from '../hooks/useDevices';
import { getErrorMessage } from '../utils/errorUtils';
import { LAYOUT_OPTIONS } from '../contexts/LayoutPreviewContext';
import type { DeviceEntry } from '../types';

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
  lastSeen?: string;
}

interface DevicesPageProps {
  className?: string;
}

/**
 * DeviceRow Component
 *
 * Compact single-row device display with inline layout selector and enable/disable toggle.
 */
interface DeviceRowProps {
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

const DeviceRow: React.FC<DeviceRowProps> = ({
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
  const [localLayout, setLocalLayout] = useState(device.layout);
  const serverLayoutRef = React.useRef(device.layout);
  const [hasUserChanges, setHasUserChanges] = useState(false);
  const {
    mutate: updateDevice,
    isPending: isUpdating,
    error: updateError,
  } = useUpdateDevice();
  const [lastSavedAt, setLastSavedAt] = useState<Date | null>(null);

  const saveLayout = React.useCallback(
    async (layout: string) => {
      await new Promise<void>((resolve, reject) => {
        updateDevice(
          { id: device.id, layout },
          {
            onSuccess: () => {
              serverLayoutRef.current = layout;
              setLastSavedAt(new Date());
              setHasUserChanges(false);
              resolve();
            },
            onError: (error) => reject(error),
          }
        );
      });
    },
    [device.id, updateDevice]
  );

  const { isSaving: isSavingLayout, error: layoutSaveError } = useAutoSave(
    localLayout,
    { saveFn: saveLayout, debounceMs: 500, enabled: hasUserChanges }
  );

  const isSaving = isSavingLayout || isUpdating;
  const saveError = layoutSaveError || updateError;

  React.useEffect(() => {
    if (device.layout !== serverLayoutRef.current) {
      serverLayoutRef.current = device.layout;
      setLocalLayout(device.layout);
      setHasUserChanges(false);
    }
  }, [device.layout]);

  const handleLayoutChange = (newLayout: string) => {
    setLocalLayout(newLayout);
    if (newLayout !== serverLayoutRef.current) {
      setHasUserChanges(true);
    }
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
            value={localLayout}
            onChange={handleLayoutChange}
            aria-label="Layout"
            compact
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

/**
 * DevicesPage Component
 *
 * Device management interface with:
 * - Global settings card with default layout selector
 * - Device list showing all connected keyboards
 * - Inline rename functionality (click Rename → input → Enter saves)
 * - Layout selector dropdown with auto-save
 * - Forget device with confirmation dialog
 *
 * Layout: From design.md Layout 2
 * Requirements: Req 5 (Device Management User Flows), Req 2 (Global Layout Selection)
 *
 * Note: Device scope (global vs device-specific) is now determined by the Rhai configuration,
 * not by a UI setting. See ConfigPage for device-aware editing.
 */
export const DevicesPage: React.FC<DevicesPageProps> = ({ className = '' }) => {
  // Fetch devices using React Query
  const {
    data: deviceEntries = [],
    isLoading: loading,
    error: fetchError,
  } = useDevices();
  const { mutate: setDeviceEnabledMutation } = useSetDeviceEnabled();
  const { mutate: forgetDeviceMutation } = useForgetDevice();

  // Transform DeviceEntry to Device (UI format)
  const devices: Device[] = deviceEntries.map((entry: DeviceEntry) => ({
    id: entry.id,
    name: entry.name,
    identifier: entry.path,
    layout: entry.layout || 'ANSI_104',
    active: entry.active,
    enabled: entry.enabled,
    vendorId: entry.path.match(/VID_([0-9A-F]{4})/)?.[1],
    productId: entry.path.match(/PID_([0-9A-F]{4})/)?.[1],
    serial: entry.serial || undefined,
    lastSeen: 'Just now',
  }));

  const error = fetchError
    ? getErrorMessage(fetchError, 'Failed to fetch devices')
    : null;

  // Global layout state
  const [globalLayout, setGlobalLayout] = useState<string>('ANSI_104');
  const [isSavingGlobalLayout, setIsSavingGlobalLayout] = useState(false);
  const [globalLayoutError, setGlobalLayoutError] = useState<string | null>(
    null
  );
  const [globalLayoutSavedAt, setGlobalLayoutSavedAt] = useState<Date | null>(
    null
  );

  // Fetch global layout on mount
  React.useEffect(() => {
    const fetchGlobalLayout = async () => {
      try {
        const layoutResponse = await fetch('/api/settings/global-layout');
        if (layoutResponse.ok) {
          const layoutData = await layoutResponse.json();
          setGlobalLayout(layoutData.layout || 'ANSI_104');
        }
      } catch (layoutErr) {
        // Endpoint may not exist yet, use default
        // Silent fail - this is expected if backend hasn't implemented the endpoint yet
      }
    };

    fetchGlobalLayout();
  }, []);

  const [editingDeviceId, setEditingDeviceId] = useState<string | null>(null);
  const [editingName, setEditingName] = useState('');
  const [nameError, setNameError] = useState('');
  const [forgetDeviceId, setForgetDeviceId] = useState<string | null>(null);

  const handleRenameClick = (device: Device) => {
    setEditingDeviceId(device.id);
    setEditingName(device.name);
    setNameError('');
  };

  const handleRenameCancel = () => {
    setEditingDeviceId(null);
    setEditingName('');
    setNameError('');
  };

  const handleRenameSave = (_deviceId: string) => {
    // Validate name
    if (!editingName.trim()) {
      setNameError('Device name cannot be empty');
      return;
    }

    if (editingName.length > 64) {
      setNameError('Device name cannot exceed 64 characters');
      return;
    }

    // TODO: Call API to rename device
    // For now, just update local state
    // setDevices((prev) =>
    //   prev.map((d) => (d.id === _deviceId ? { ...d, name: editingName } : d))
    // );

    // Reset editing state
    setEditingDeviceId(null);
    setEditingName('');
    setNameError('');
  };

  const handleToggleEnabled = (deviceId: string, enabled: boolean) => {
    setDeviceEnabledMutation({ id: deviceId, enabled });
  };

  const handleGlobalLayoutChange = async (newLayout: string) => {
    setGlobalLayout(newLayout);
    setIsSavingGlobalLayout(true);
    setGlobalLayoutError(null);

    try {
      const response = await fetch('/api/settings/global-layout', {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ layout: newLayout }),
      });

      if (!response.ok) {
        throw new Error(`Failed to save global layout: ${response.statusText}`);
      }

      setGlobalLayoutSavedAt(new Date());

      // Auto-clear success indicator after 3 seconds
      setTimeout(() => {
        setGlobalLayoutSavedAt(null);
      }, 3000);
    } catch (err) {
      console.error('Failed to save global layout:', err);
      setGlobalLayoutError(
        getErrorMessage(err, 'Failed to save global layout')
      );
    } finally {
      setIsSavingGlobalLayout(false);
    }
  };

  const handleForgetDevice = () => {
    if (forgetDeviceId) {
      forgetDeviceMutation(forgetDeviceId);
      setForgetDeviceId(null);
    }
  };

  const forgetDevice = devices.find((d) => d.id === forgetDeviceId);

  if (loading) {
    return (
      <div
        className={`flex flex-col gap-4 md:gap-6 p-4 md:p-6 lg:p-8 ${className}`}
      >
        <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
          <LoadingSkeleton variant="text" width="150px" height="32px" />
          <LoadingSkeleton variant="rectangular" width="100px" height="36px" />
        </div>

        <Card>
          <div className="flex flex-col gap-md">
            <LoadingSkeleton variant="text" width="200px" height="24px" />
            <div className="flex flex-col gap-md">
              <LoadingSkeleton variant="rectangular" height="120px" />
              <LoadingSkeleton variant="rectangular" height="120px" />
            </div>
          </div>
        </Card>
      </div>
    );
  }

  return (
    <div
      className={`flex flex-col gap-4 md:gap-6 p-4 md:p-6 lg:p-8 ${className}`}
    >
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <h1 className="text-xl md:text-2xl lg:text-3xl font-semibold text-slate-100">
          Devices
        </h1>
        <div className="flex gap-2">
          <Button
            variant="ghost"
            size="sm"
            onClick={() => {
              window.location.reload();
            }}
            aria-label="Refresh device list"
            disabled={loading}
          >
            Refresh
          </Button>
        </div>
      </div>

      {error && (
        <div className="bg-red-900/20 border border-red-700 rounded-lg p-4">
          <p className="text-sm text-red-400">{error}</p>
        </div>
      )}

      {/* Global Settings Card */}
      <Card variant="elevated" className="bg-slate-800">
        <div className="flex flex-col gap-md">
          <div className="flex items-center justify-between">
            <h2 className="text-lg font-semibold text-slate-100">
              Global Settings
            </h2>
            {isSavingGlobalLayout && (
              <span className="text-xs text-slate-400 flex items-center gap-1">
                <span className="animate-spin h-3 w-3 border-2 border-slate-400 border-t-transparent rounded-full" />
                Saving...
              </span>
            )}
            {!isSavingGlobalLayout && globalLayoutSavedAt && (
              <span className="text-xs text-green-500 flex items-center gap-1">
                ✓ Saved
              </span>
            )}
            {globalLayoutError && (
              <span
                className="text-xs text-red-500 flex items-center gap-1"
                title={globalLayoutError}
              >
                ✗ Error
              </span>
            )}
          </div>

          <div className="flex flex-col gap-sm">
            <label className="text-sm font-medium text-slate-300">
              Default Keyboard Layout
            </label>
            <p className="text-xs text-slate-400">
              New devices will inherit this layout by default. You can override
              it for specific devices below.
            </p>
            <LayoutDropdown
              options={LAYOUT_OPTIONS}
              value={globalLayout}
              onChange={handleGlobalLayoutChange}
              aria-label="Select default keyboard layout"
            />
          </div>

          {globalLayoutError && (
            <div className="bg-red-900/20 border border-red-700 rounded-lg p-3">
              <p className="text-xs text-red-400">{globalLayoutError}</p>
            </div>
          )}
        </div>
      </Card>

      <Card>
        <div className="flex flex-col gap-md">
          <h2 className="text-lg font-semibold text-slate-100">
            Device List ({devices.length} connected)
          </h2>

          {devices.length === 0 ? (
            <div className="py-xl text-center">
              <p className="text-sm text-slate-400">
                No devices connected. Connect a keyboard to get started.
              </p>
            </div>
          ) : (
            <div className="flex flex-col gap-2">
              {devices.map((device) => (
                <DeviceRow
                  key={device.id}
                  device={device}
                  isEditing={editingDeviceId === device.id}
                  editingName={editingName}
                  nameError={nameError}
                  onRenameClick={handleRenameClick}
                  onRenameCancel={handleRenameCancel}
                  onRenameSave={handleRenameSave}
                  onEditingNameChange={setEditingName}
                  onToggleEnabled={handleToggleEnabled}
                  onForgetClick={setForgetDeviceId}
                />
              ))}
            </div>
          )}
        </div>
      </Card>

      {/* Forget device confirmation modal */}
      <Modal
        open={forgetDeviceId !== null}
        onClose={() => setForgetDeviceId(null)}
        title="Forget Device"
      >
        <div className="flex flex-col gap-lg">
          <p className="text-sm text-slate-300">
            Are you sure you want to forget device{' '}
            <span className="font-semibold text-slate-100">
              {forgetDevice?.name}
            </span>
            ?
          </p>
          <p className="text-sm text-slate-400">
            This will remove all device-specific configuration and mappings.
            This action cannot be undone.
          </p>
          <div className="flex justify-end gap-sm">
            <Button
              variant="ghost"
              size="md"
              onClick={() => setForgetDeviceId(null)}
              aria-label="Cancel forget device"
            >
              Cancel
            </Button>
            <Button
              variant="danger"
              size="md"
              onClick={handleForgetDevice}
              aria-label="Confirm forget device"
            >
              Forget Device
            </Button>
          </div>
        </div>
      </Modal>
    </div>
  );
};

export default DevicesPage;
