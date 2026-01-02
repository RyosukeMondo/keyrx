import React, { useState } from 'react';
import { Card } from '../components/Card';
import { Button } from '../components/Button';
import { Input } from '../components/Input';
import { Dropdown } from '../components/Dropdown';
import { Modal } from '../components/Modal';
import { LoadingSkeleton } from '../components/LoadingSkeleton';

interface Device {
  id: string;
  name: string;
  identifier: string;
  scope: 'global' | 'device-specific';
  layout: string;
  active: boolean;
  vendorId?: string;
  productId?: string;
  serial?: string;
  lastSeen?: string;
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
 * DevicesPage Component
 *
 * Device management interface with:
 * - Device list showing all connected keyboards
 * - Inline rename functionality (click Rename → input → Enter saves)
 * - Scope toggle (Global / Device-Specific)
 * - Layout selector dropdown
 * - Forget device with confirmation dialog
 *
 * Layout: From design.md Layout 2
 * Requirements: Req 5 (Device Management User Flows)
 */
export const DevicesPage: React.FC<DevicesPageProps> = ({ className = '' }) => {
  const [loading, setLoading] = useState(true);
  const [devices, setDevices] = useState<Device[]>([]);
  const [error, setError] = useState<string | null>(null);

  // Fetch devices from API on mount
  React.useEffect(() => {
    const fetchDevices = async () => {
      try {
        setLoading(true);
        setError(null);
        const response = await fetch('/api/devices');
        if (!response.ok) {
          throw new Error(`Failed to fetch devices: ${response.statusText}`);
        }
        const data = await response.json();

        // Transform API response to UI format
        const transformedDevices: Device[] = (data.devices || []).map((device: any) => ({
          id: device.id,
          name: device.name,
          identifier: device.path,
          scope: device.scope || 'global',
          layout: device.layout || 'ANSI_104',
          active: device.active,
          vendorId: device.path.match(/VID_([0-9A-F]{4})/)?.[1],
          productId: device.path.match(/PID_([0-9A-F]{4})/)?.[1],
          serial: device.serial,
          lastSeen: 'Just now',
        }));

        setDevices(transformedDevices);
      } catch (err) {
        console.error('Failed to fetch devices:', err);
        setError(err instanceof Error ? err.message : 'Failed to fetch devices');
      } finally {
        setLoading(false);
      }
    };

    fetchDevices();
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

  const handleScopeChange = (deviceId: string, newScope: 'global' | 'device-specific') => {
    setDevices((prev) =>
      prev.map((d) => (d.id === deviceId ? { ...d, scope: newScope } : d))
    );
  };

  const handleLayoutChange = (deviceId: string, newLayout: string) => {
    setDevices((prev) =>
      prev.map((d) => (d.id === deviceId ? { ...d, layout: newLayout } : d))
    );
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
                  <Card key={device.id} variant="elevated" className="bg-slate-800">
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
                                      handleRenameSave(device.id);
                                    } else if (e.key === 'Escape') {
                                      handleRenameCancel();
                                    }
                                  }}
                                >
                                  <Input
                                    type="text"
                                    value={editingName}
                                    onChange={(value) => setEditingName(value)}
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
                                  onClick={() => handleRenameSave(device.id)}
                                  aria-label="Save device name"
                                  className="flex-1 sm:flex-none min-h-[44px] sm:min-h-0"
                                >
                                  Save
                                </Button>
                                <Button
                                  variant="ghost"
                                  size="sm"
                                  onClick={handleRenameCancel}
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
                                onClick={() => handleRenameClick(device)}
                                aria-label={`Rename device ${device.name}`}
                                className="w-full sm:w-auto min-h-[44px] sm:min-h-0"
                              >
                                Rename
                              </Button>
                            </>
                          )}
                        </div>
                      </div>

                      {/* Scope selector */}
                      <div className="flex flex-col gap-2">
                        <label className="text-sm font-medium text-slate-300">Scope</label>
                        <div className="flex flex-col sm:flex-row gap-2 sm:gap-4">
                          <button
                            onClick={() => handleScopeChange(device.id, 'global')}
                            className={`flex items-center gap-2 rounded-md border px-4 py-3 sm:py-2 text-sm transition-colors min-h-[44px] sm:min-h-0 focus:outline focus:outline-2 focus:outline-primary-500 focus:outline-offset-2 ${
                              device.scope === 'global'
                                ? 'border-primary-500 bg-primary-500/10 text-primary-500'
                                : 'border-slate-700 bg-slate-900 text-slate-400 hover:border-slate-600 hover:text-slate-300'
                            }`}
                            aria-label="Set scope to global"
                            role="radio"
                            aria-checked={device.scope === 'global'}
                          >
                            <span
                              className={`h-4 w-4 rounded-full border-2 flex-shrink-0 ${
                                device.scope === 'global'
                                  ? 'border-primary-500 bg-primary-500'
                                  : 'border-slate-500'
                              }`}
                              aria-hidden="true"
                            >
                              {device.scope === 'global' && (
                                <span className="block h-full w-full rounded-full bg-white" />
                              )}
                            </span>
                            Global
                          </button>
                          <button
                            onClick={() => handleScopeChange(device.id, 'device-specific')}
                            className={`flex items-center gap-2 rounded-md border px-4 py-3 sm:py-2 text-sm transition-colors min-h-[44px] sm:min-h-0 focus:outline focus:outline-2 focus:outline-primary-500 focus:outline-offset-2 ${
                              device.scope === 'device-specific'
                                ? 'border-primary-500 bg-primary-500/10 text-primary-500'
                                : 'border-slate-700 bg-slate-900 text-slate-400 hover:border-slate-600 hover:text-slate-300'
                            }`}
                            aria-label="Set scope to device-specific"
                            role="radio"
                            aria-checked={device.scope === 'device-specific'}
                          >
                            <span
                              className={`h-4 w-4 rounded-full border-2 flex-shrink-0 ${
                                device.scope === 'device-specific'
                                  ? 'border-primary-500 bg-primary-500'
                                  : 'border-slate-500'
                              }`}
                              aria-hidden="true"
                            >
                              {device.scope === 'device-specific' && (
                                <span className="block h-full w-full rounded-full bg-white" />
                              )}
                            </span>
                            Device-Specific
                          </button>
                        </div>
                      </div>

                      {/* Layout selector */}
                      <div className="flex flex-col gap-sm">
                        <label className="text-sm font-medium text-slate-300">Layout</label>
                        <Dropdown
                          options={LAYOUT_OPTIONS}
                          value={device.layout}
                          onChange={(value) => handleLayoutChange(device.id, value)}
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
                          onClick={() => setForgetDeviceId(device.id)}
                          aria-label={`Forget device ${device.name}`}
                        >
                          Forget Device
                        </Button>
                      </div>
                    </div>
                  </Card>
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
