import { useState, useMemo } from 'react';
import { Card } from '../components/Card';
import { Button } from '../components/Button';
import { LayoutDropdown } from '../components/LayoutDropdown';
import { Modal } from '../components/Modal';
import { LoadingSkeleton } from '../components/LoadingSkeleton';
import {
  useDevices,
  useForgetDevice,
  useRenameDevice,
  useGlobalLayout,
  useSetGlobalLayout,
} from '../hooks/useDevices';
import { getErrorMessage } from '../utils/errorUtils';
import { usePageTitle } from '../hooks/usePageTitle';
import { LAYOUT_OPTIONS } from '../contexts/LayoutPreviewContext';
import type { DeviceEntry } from '../types';
import { DeviceRow } from '../components/devices/DeviceRow';
import { t } from '../i18n';

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
  usePageTitle(t('devices.title'));
  // Fetch devices using React Query
  const {
    data: deviceEntries = [],
    isLoading: loading,
    error: fetchError,
    refetch,
  } = useDevices();
  const { mutate: forgetDeviceMutation } = useForgetDevice();
  const { mutate: renameDeviceMutation } = useRenameDevice();

  // Transform DeviceEntry to Device (UI format)
  const devices: Device[] = deviceEntries.map((entry: DeviceEntry) => ({
    id: entry.id,
    name: entry.name,
    identifier: entry.path,
    layout: entry.layout || 'ANSI_104',
    active: entry.active,
    vendorId: entry.path.match(/VID_([0-9A-F]{4})/)?.[1],
    productId: entry.path.match(/PID_([0-9A-F]{4})/)?.[1],
    serial: entry.serial || undefined,
    lastSeen: entry.lastSeen,
  }));

  const error = fetchError
    ? getErrorMessage(fetchError, t('devices.fetchFailed'))
    : null;

  // Global layout via React Query
  const { data: globalLayout = 'ANSI_104' } = useGlobalLayout();
  const {
    mutate: setGlobalLayoutMutation,
    isPending: isSavingGlobalLayout,
    error: globalLayoutMutationError,
  } = useSetGlobalLayout();
  const globalLayoutError = globalLayoutMutationError
    ? getErrorMessage(globalLayoutMutationError, t('devices.layoutSaveFailed'))
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
      result = result.filter(
        (d) =>
          d.name.toLowerCase().includes(q) ||
          d.identifier.toLowerCase().includes(q) ||
          d.vendorId?.toLowerCase().includes(q) ||
          d.productId?.toLowerCase().includes(q)
      );
    }
    result.sort((a, b) => {
      let cmp = 0;
      if (sortBy === 'name') cmp = a.name.localeCompare(b.name);
      else if (sortBy === 'active')
        cmp = (a.active ? 0 : 1) - (b.active ? 0 : 1);
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
      setNameError(t('devices.nameEmpty'));
      return;
    }

    if (editingName.length > 64) {
      setNameError(t('devices.nameTooLong'));
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
          setNameError(getErrorMessage(err, t('devices.renameFailed')));
        },
      }
    );
  };

  const [globalLayoutSaved, setGlobalLayoutSaved] = useState(false);
  const handleGlobalLayoutChange = (newLayout: string) => {
    setGlobalLayoutMutation(newLayout, {
      onSuccess: () => {
        setGlobalLayoutSaved(true);
        // Clear the success indicator after 2 seconds (same as DeviceRow)
        setTimeout(() => setGlobalLayoutSaved(false), 2000);
      },
    });
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
          {t('devices.title')}
        </h1>
        <div className="flex gap-2">
          <Button
            variant="ghost"
            size="sm"
            onClick={() => {
              refetch();
            }}
            aria-label={t('devices.refreshAria')}
            disabled={loading}
          >
            {t('devices.refresh')}
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
              {t('devices.global')}
            </h2>
            {isSavingGlobalLayout && (
              <span className="text-xs text-slate-400 flex items-center gap-1">
                <span className="animate-spin h-3 w-3 border-2 border-slate-400 border-t-transparent rounded-full" />
                {t('devices.saving')}
              </span>
            )}
            {!isSavingGlobalLayout && globalLayoutSaved && (
              <span className="text-xs text-green-500">
                {t('devices.saved')}
              </span>
            )}
            {globalLayoutError && (
              <span
                className="text-xs text-red-500 flex items-center gap-1"
                title={globalLayoutError}
              >
                {t('devices.error')}
              </span>
            )}
          </div>

          <div className="flex flex-col gap-sm">
            <label className="text-sm font-medium text-slate-300">
              {t('devices.defaultLayout')}
            </label>
            <p className="text-xs text-slate-400">
              {t('devices.defaultLayoutHelp')}
            </p>
            <LayoutDropdown
              options={LAYOUT_OPTIONS}
              value={globalLayout}
              onChange={handleGlobalLayoutChange}
              aria-label={t('devices.defaultLayoutAria')}
            />
          </div>
        </div>
      </Card>

      <Card>
        <div className="flex flex-col gap-md">
          <h2 className="text-lg font-semibold text-slate-100">
            {t('devices.list', { count: devices.length })}
          </h2>

          {devices.length === 0 ? (
            <div className="py-xl text-center">
              <p className="text-sm text-slate-400">{t('devices.none')}</p>
            </div>
          ) : (
            <>
              {devices.length > 5 && (
                <div className="flex flex-col sm:flex-row items-stretch sm:items-center gap-3 mb-4">
                  <input
                    type="text"
                    placeholder={t('devices.search')}
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                    className="flex-1 px-3 py-2 bg-slate-700 border border-slate-600 rounded-md text-sm text-slate-200 placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-primary-500"
                    aria-label={t('devices.searchAria')}
                  />
                  <div className="flex items-center gap-2">
                    <select
                      value={sortBy}
                      onChange={(e) =>
                        setSortBy(
                          e.target.value as 'name' | 'active' | 'layout'
                        )
                      }
                      className="px-3 py-2 bg-slate-700 border border-slate-600 rounded-md text-sm text-slate-200 focus:outline-none focus:ring-2 focus:ring-primary-500"
                      aria-label={t('devices.sortBy')}
                    >
                      <option value="name">{t('devices.sortName')}</option>
                      <option value="active">{t('devices.sortStatus')}</option>
                      <option value="layout">{t('devices.sortLayout')}</option>
                    </select>
                    <button
                      onClick={() =>
                        setSortDirection((d) => (d === 'asc' ? 'desc' : 'asc'))
                      }
                      className="px-3 py-2 bg-slate-700 border border-slate-600 rounded-md text-sm text-slate-200 hover:bg-slate-600 transition-colors"
                      aria-label={
                        sortDirection === 'asc'
                          ? t('devices.sortDesc')
                          : t('devices.sortAsc')
                      }
                    >
                      {sortDirection === 'asc' ? '↑' : '↓'}
                    </button>
                  </div>
                </div>
              )}
              {filteredAndSortedDevices.length === 0 &&
                devices.length > 0 &&
                searchQuery && (
                  <p className="text-center text-slate-400 py-8">
                    {t('devices.noMatch', { query: searchQuery })}
                  </p>
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
        title={t('devices.forgetTitle')}
      >
        <div className="flex flex-col gap-lg">
          <p className="text-sm text-slate-300">
            {t('devices.forgetConfirm')}{' '}
            <span className="font-semibold text-slate-100">
              {forgetDevice?.name}
            </span>
            ?
          </p>
          <p className="text-sm text-slate-400">{t('devices.forgetWarn')}</p>
          <div className="flex justify-end gap-sm">
            <Button
              variant="ghost"
              size="md"
              onClick={() => setForgetDeviceId(null)}
              aria-label={t('devices.cancelAria')}
            >
              {t('devices.cancel')}
            </Button>
            <Button
              variant="danger"
              size="md"
              onClick={handleForgetDevice}
              aria-label={t('devices.forgetConfirmAria')}
            >
              {t('devices.forgetTitle')}
            </Button>
          </div>
        </div>
      </Modal>
    </div>
  );
};

export default DevicesPage;
