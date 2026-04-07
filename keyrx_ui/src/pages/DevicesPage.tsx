import { useState, useMemo } from 'react';
import { Card } from '../components/Card';
import { Button } from '../components/Button';
import { LayoutDropdown } from '../components/LayoutDropdown';
import { Modal } from '../components/Modal';
import { LoadingSkeleton } from '../components/LoadingSkeleton';
import {
  useDevices,
  useSetDeviceEnabled,
  useForgetDevice,
  useRenameDevice,
  useGlobalLayout,
  useSetGlobalLayout,
} from '../hooks/useDevices';
import { getErrorMessage } from '../utils/errorUtils';
import { LAYOUT_OPTIONS } from '../contexts/LayoutPreviewContext';
import type { DeviceEntry } from '../types';
import { DeviceRow } from '../components/devices/DeviceRow';

interface DevicesPageProps {
  className?: string;
}

// Device interface used in DevicesPage - matches DeviceRow's Device interface
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
    refetch,
  } = useDevices();
  const { mutate: setDeviceEnabledMutation } = useSetDeviceEnabled();
  const { mutate: forgetDeviceMutation } = useForgetDevice();
  const { mutate: renameDeviceMutation } = useRenameDevice();

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
    lastSeen: entry.lastSeen,
  }));

  const error = fetchError
    ? getErrorMessage(fetchError, 'Failed to fetch devices')
    : null;

  // Global layout via React Query
  const { data: globalLayout = 'ANSI_104' } = useGlobalLayout();
  const {
    mutate: setGlobalLayoutMutation,
    isPending: isSavingGlobalLayout,
    error: globalLayoutMutationError,
  } = useSetGlobalLayout();
  const globalLayoutError = globalLayoutMutationError
    ? getErrorMessage(globalLayoutMutationError, 'Failed to save global layout')
    : null;

  const [editingDeviceId, setEditingDeviceId] = useState<string | null>(null);
  const [editingName, setEditingName] = useState('');
  const [nameError, setNameError] = useState('');
  const [forgetDeviceId, setForgetDeviceId] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [sortBy, setSortBy] = useState<'name' | 'active' | 'layout'>('name');
  const [sortDirection, setSortDirection] = useState<'asc' | 'desc'>('asc');

  const filteredAndSortedDevices = useMemo(() => {
    let result = [...devices];
    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase();
      result = result.filter(d =>
        d.name.toLowerCase().includes(q) ||
        d.identifier.toLowerCase().includes(q) ||
        (d.vendorId?.toLowerCase().includes(q)) ||
        (d.productId?.toLowerCase().includes(q))
      );
    }
    result.sort((a, b) => {
      let cmp = 0;
      if (sortBy === 'name') cmp = a.name.localeCompare(b.name);
      else if (sortBy === 'active') cmp = (a.active ? 0 : 1) - (b.active ? 0 : 1);
      else if (sortBy === 'layout') cmp = a.layout.localeCompare(b.layout);
      return sortDirection === 'asc' ? cmp : -cmp;
    });
    return result;
  }, [devices, searchQuery, sortBy, sortDirection]);

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

    // Call API to rename device
    renameDeviceMutation(
      { id: deviceId, name: editingName.trim() },
      {
        onSuccess: () => {
          // Reset editing state on success
          setEditingDeviceId(null);
          setEditingName('');
          setNameError('');
        },
        onError: (err) => {
          // Show error message
          setNameError(getErrorMessage(err, 'Failed to rename device'));
        },
      }
    );
  };

  const handleToggleEnabled = (deviceId: string, enabled: boolean) => {
    setDeviceEnabledMutation({ id: deviceId, enabled });
  };

  const handleGlobalLayoutChange = (newLayout: string) => {
    setGlobalLayoutMutation(newLayout);
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
              refetch();
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
            <>
              {devices.length > 5 && (
                <div className="flex flex-col sm:flex-row items-stretch sm:items-center gap-3 mb-4">
                  <input
                    type="text"
                    placeholder="Search devices..."
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                    className="flex-1 px-3 py-2 bg-slate-700 border border-slate-600 rounded-md text-sm text-slate-200 placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-primary-500"
                    aria-label="Search devices"
                  />
                  <div className="flex items-center gap-2">
                    <select
                      value={sortBy}
                      onChange={(e) => setSortBy(e.target.value as 'name' | 'active' | 'layout')}
                      className="px-3 py-2 bg-slate-700 border border-slate-600 rounded-md text-sm text-slate-200 focus:outline-none focus:ring-2 focus:ring-primary-500"
                      aria-label="Sort by"
                    >
                      <option value="name">Name</option>
                      <option value="active">Status</option>
                      <option value="layout">Layout</option>
                    </select>
                    <button
                      onClick={() => setSortDirection(d => d === 'asc' ? 'desc' : 'asc')}
                      className="px-3 py-2 bg-slate-700 border border-slate-600 rounded-md text-sm text-slate-200 hover:bg-slate-600 transition-colors"
                      aria-label={`Sort ${sortDirection === 'asc' ? 'descending' : 'ascending'}`}
                    >
                      {sortDirection === 'asc' ? '↑' : '↓'}
                    </button>
                  </div>
                </div>
              )}
              {filteredAndSortedDevices.length === 0 && devices.length > 0 && searchQuery && (
                <p className="text-center text-slate-400 py-8">No devices match "{searchQuery}"</p>
              )}
              <div className="flex flex-col gap-2">
                {filteredAndSortedDevices.map((device) => (
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
            </>
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
