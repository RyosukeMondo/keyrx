import React, { useState } from 'react';
import { Card } from '../components/Card';
import { Button } from '../components/Button';
import { Input } from '../components/Input';
import { Dropdown } from '../components/Dropdown';
import { Modal } from '../components/Modal';
import { LoadingSkeleton } from '../components/LoadingSkeleton';
import { useAutoSave } from '../hooks/useAutoSave';
import { useUpdateDevice } from '../hooks/useUpdateDevice';
import { getErrorMessage } from '../utils/errorUtils';

interface Device {
  id: string;
  name: string;
  identifier: string;
  layout: string;
  active: boolean;
  vendorId?: string;
  productId?: string;
  serial?: string;
  lastSeen?: string;
}

interface ApiDevice {
  id: string;
  name: string;
  path: string;
  layout?: string;
  active: boolean;
  serial?: string;
}

interface DevicesPageProps {
  className?: string;
}

const LAYOUT_OPTIONS = [
  { value: 'ANSI_104', label: 'ANSI 104' },
  { value: 'ISO_105', label: 'ISO 105' },
  { value: 'JIS_109', label: 'JIS 109' },
  { value: 'HHKB', label: 'HHKB' },
  { value: 'NUMPAD', label: 'Numpad' },
];

/**
 * DeviceCard Component
 *
 * Individual device card with auto-save for layout changes.
 */
interface DeviceCardProps {
  device: Device;
  isEditing: boolean;
  editingName: string;
  nameError: string;
  onRenameClick: (device: Device) => void;
  onRenameCancel: () => void;
  onRenameSave: (deviceId: string) => void;
  onEditingNameChange: (value: string) => void;
  onForgetClick: (deviceId: string) => void;
}

const DeviceCard: React.FC<DeviceCardProps> = ({
  device,
  isEditing,
  editingName,
  nameError,
  onRenameClick,
  onRenameCancel,
  onRenameSave,
  onEditingNameChange,
  onForgetClick,
}) => {
  // Local state for layout - tracks user's current selection
  const [localLayout, setLocalLayout] = useState(device.layout);

  // Track the server value to detect user changes vs external updates
  const serverLayoutRef = React.useRef(device.layout);

  // Track whether user has made a change that needs saving
  const [hasUserChanges, setHasUserChanges] = useState(false);

  // Device update mutation hook
  const { mutate: updateDevice, isPending: isUpdating, error: updateError } = useUpdateDevice();

  // Track last saved timestamp for feedback
  const [lastSavedAt, setLastSavedAt] = useState<Date | null>(null);

  // Save function - only called when user makes changes
  const saveLayout = React.useCallback(async (layout: string) => {
    await new Promise<void>((resolve, reject) => {
      updateDevice(
        { id: device.id, layout },
        {
          onSuccess: () => {
            serverLayoutRef.current = layout; // Update server value after successful save
            setLastSavedAt(new Date());
            setHasUserChanges(false);
            resolve();
          },
          onError: (error) => reject(error),
        }
      );
    });
  }, [device.id, updateDevice]);

  // Auto-save hook - only enabled when user has made changes
  const { isSaving: isSavingLayout, error: layoutSaveError } = useAutoSave(
    localLayout,
    {
      saveFn: saveLayout,
      debounceMs: 500,
      enabled: hasUserChanges, // Only save when user changed the value
    }
  );

  // Combine saving and error states
  const isSaving = isSavingLayout || isUpdating;
  const saveError = layoutSaveError || updateError;

  // Update local state when device changes externally (e.g., from server refresh)
  React.useEffect(() => {
    // Only update if server value changed (external update, not our save)
    if (device.layout !== serverLayoutRef.current) {
      serverLayoutRef.current = device.layout;
      setLocalLayout(device.layout);
      setHasUserChanges(false); // External update, no user changes
    }
  }, [device.layout]);

  const handleLayoutChange = (newLayout: string) => {
    setLocalLayout(newLayout);
    // Mark as user change only if different from server value
    if (newLayout !== serverLayoutRef.current) {
      setHasUserChanges(true);
    }
  };

  return (
    <Card key={device.id} variant="elevated" className="bg-slate-800" data-testid="device-card">
      <div className="flex flex-col gap-md">
        {/* Device header */}
        <div className="flex items-start justify-between">
          <div className="flex items-center gap-sm">
            <span className="text-xl" role="img" aria-label="Keyboard">
              🖮
            </span>
            <div className="flex flex-col">
              <div className="flex items-center gap-sm">
                <span className="text-base font-medium text-slate-100">
                  {device.name}
                </span>
                {device.active && (
                  <span
                    className="text-xs text-green-500"
                    aria-label="Connected"
                  >
                    ✓ Connected
                  </span>
                )}
              </div>
              <span className="text-xs font-mono text-slate-500">
                {device.identifier}
              </span>
            </div>
          </div>
        </div>

        {/* Rename section */}
        <div className="flex flex-col gap-2">
          <label className="text-sm font-medium text-slate-300">Name</label>
          <div className="flex flex-col sm:flex-row items-start gap-2">
            {isEditing ? (
              <>
                <div className="flex-1 w-full">
                  <div
                    onKeyDown={(e) => {
                      if (e.key === 'Enter') {
                        onRenameSave(device.id);
                      } else if (e.key === 'Escape') {
                        onRenameCancel();
                      }
                    }}
                  >
                    <Input
                      type="text"
                      value={editingName}
                      onChange={(value) => onEditingNameChange(value)}
                      error={nameError}
                      maxLength={64}
                      aria-label="Device name"
                    />
                  </div>
                </div>
                <div className="flex gap-2 w-full sm:w-auto">
                  <Button
                    variant="primary"
                    size="sm"
                    onClick={() => onRenameSave(device.id)}
                    aria-label="Save device name"
                    className="flex-1 sm:flex-none min-h-[44px] sm:min-h-0"
                  >
                    Save
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={onRenameCancel}
                    aria-label="Cancel rename"
                    className="flex-1 sm:flex-none min-h-[44px] sm:min-h-0"
                  >
                    Cancel
                  </Button>
                </div>
              </>
            ) : (
              <>
                <div className="flex-1 w-full rounded-md border border-slate-700 bg-slate-900 px-4 py-3 text-sm text-slate-100">
                  {device.name}
                </div>
                <Button
                  variant="secondary"
                  size="sm"
                  onClick={() => onRenameClick(device)}
                  aria-label={`Rename device ${device.name}`}
                  className="w-full sm:w-auto min-h-[44px] sm:min-h-0"
                >
                  Rename
                </Button>
              </>
            )}
          </div>
        </div>

        {/* Layout selector with save feedback */}
        <div className="flex flex-col gap-2">
          <div className="flex items-center justify-between">
            <label className="text-sm font-medium text-slate-300">Layout</label>
            <div className="flex items-center gap-2">
              {isSaving && (
                <span className="text-xs text-slate-400 flex items-center gap-1">
                  <span className="animate-spin h-3 w-3 border-2 border-slate-400 border-t-transparent rounded-full" />
                  Saving...
                </span>
              )}
              {!isSaving && lastSavedAt && (
                <span className="text-xs text-green-500 flex items-center gap-1">
                  ✓ Saved
                </span>
              )}
              {saveError && (
                <span className="text-xs text-red-500 flex items-center gap-1" title={saveError instanceof Error ? saveError.message : String(saveError)}>
                  ✗ Error
                </span>
              )}
            </div>
          </div>
          <Dropdown
            options={LAYOUT_OPTIONS}
            value={localLayout}
            onChange={handleLayoutChange}
            aria-label="Select keyboard layout"
          />
        </div>

        {/* Device details */}
        <div className="flex flex-wrap gap-md text-xs text-slate-400">
          {device.serial && <span>Serial: {device.serial}</span>}
          {device.vendorId && <span>• Vendor: {device.vendorId}</span>}
          {device.productId && <span>• Product: {device.productId}</span>}
          {device.lastSeen && <span>• Last seen: {device.lastSeen}</span>}
        </div>

        {/* Forget device button */}
        <div className="flex justify-end border-t border-slate-700 pt-md">
          <Button
            variant="danger"
            size="sm"
            onClick={() => onForgetClick(device.id)}
            aria-label={`Forget device ${device.name}`}
          >
            Forget Device
          </Button>
        </div>
      </div>
    </Card>
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
  const [loading, setLoading] = useState(true);
  const [devices, setDevices] = useState<Device[]>([]);
  const [error, setError] = useState<string | null>(null);

  // Global layout state
  const [globalLayout, setGlobalLayout] = useState<string>('ANSI_104');
  const [isSavingGlobalLayout, setIsSavingGlobalLayout] = useState(false);
  const [globalLayoutError, setGlobalLayoutError] = useState<string | null>(null);
  const [globalLayoutSavedAt, setGlobalLayoutSavedAt] = useState<Date | null>(null);

  // Fetch devices and global layout from API on mount
  React.useEffect(() => {
    const fetchData = async () => {
      try {
        setLoading(true);
        setError(null);

        // Fetch devices
        const devicesResponse = await fetch('/api/devices');
        if (!devicesResponse.ok) {
          throw new Error(`Failed to fetch devices: ${devicesResponse.statusText}`);
        }
        const devicesData = await devicesResponse.json();

        // Transform API response to UI format
        const transformedDevices: Device[] = (devicesData.devices || []).map((device: ApiDevice) => ({
          id: device.id,
          name: device.name,
          identifier: device.path,
          layout: device.layout || 'ANSI_104',
          active: device.active,
          vendorId: device.path.match(/VID_([0-9A-F]{4})/)?.[1],
          productId: device.path.match(/PID_([0-9A-F]{4})/)?.[1],
          serial: device.serial,
          lastSeen: 'Just now',
        }));

        setDevices(transformedDevices);

        // Fetch global layout (if endpoint exists)
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
      } catch (err) {
        console.error('Failed to fetch data:', err);
        setError(getErrorMessage(err, 'Failed to fetch data'));
      } finally {
        setLoading(false);
      }
    };

    fetchData();
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

  const handleRenameSave = (deviceId: string) => {
    // Validate name
    if (!editingName.trim()) {
      setNameError('Device name cannot be empty');
      return;
    }

    if (editingName.length > 64) {
      setNameError('Device name cannot exceed 64 characters');
      return;
    }

    // Save the new name
    setDevices((prev) =>
      prev.map((d) => (d.id === deviceId ? { ...d, name: editingName } : d))
    );

    // Reset editing state
    setEditingDeviceId(null);
    setEditingName('');
    setNameError('');
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
      setGlobalLayoutError(getErrorMessage(err, 'Failed to save global layout'));
    } finally {
      setIsSavingGlobalLayout(false);
    }
  };

  const handleForgetDevice = () => {
    if (forgetDeviceId) {
      setDevices((prev) => prev.filter((d) => d.id !== forgetDeviceId));
      setForgetDeviceId(null);
    }
  };

  const forgetDevice = devices.find((d) => d.id === forgetDeviceId);

  if (loading) {
    return (
      <div className={`flex flex-col gap-4 md:gap-6 p-4 md:p-6 lg:p-8 ${className}`}>
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
    <div className={`flex flex-col gap-4 md:gap-6 p-4 md:p-6 lg:p-8 ${className}`}>
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
            <h2 className="text-lg font-semibold text-slate-100">Global Settings</h2>
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
              <span className="text-xs text-red-500 flex items-center gap-1" title={globalLayoutError}>
                ✗ Error
              </span>
            )}
          </div>

          <div className="flex flex-col gap-sm">
            <label className="text-sm font-medium text-slate-300">
              Default Keyboard Layout
            </label>
            <p className="text-xs text-slate-400">
              New devices will inherit this layout by default. You can override it for specific devices below.
            </p>
            <Dropdown
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
            <div className="flex flex-col gap-md">
              {devices.map((device) => {
                const isEditing = editingDeviceId === device.id;

                return (
                  <DeviceCard
                    key={device.id}
                    device={device}
                    isEditing={isEditing}
                    editingName={editingName}
                    nameError={nameError}
                    onRenameClick={handleRenameClick}
                    onRenameCancel={handleRenameCancel}
                    onRenameSave={handleRenameSave}
                    onEditingNameChange={setEditingName}
                    onForgetClick={setForgetDeviceId}
                  />
                );
              })}
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
            <span className="font-semibold text-slate-100">{forgetDevice?.name}</span>?
          </p>
          <p className="text-sm text-slate-400">
            This will remove all device-specific configuration and mappings. This action cannot be
            undone.
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
