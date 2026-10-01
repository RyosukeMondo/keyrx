import React, { useState, useMemo, useEffect, useRef } from 'react';
import { Plus, ChevronLeft, ChevronRight, User } from 'lucide-react';
import {
  useProfiles,
  useCreateProfile,
  useActivateProfile,
  useDeleteProfile,
} from '@/hooks/useProfiles';
import { useToast } from '@/hooks/useToast';
import { getErrorMessage } from '@/utils/errorUtils';
import { ProfileTemplate } from '@/types';
import { t } from '@/i18n';
import { Modal } from '@/components/Modal';
import { Input } from '@/components/Input';
import { Button } from '@/components/Button';
import { ProfileSidebarItem } from './ProfileSidebarItem';

export interface ProfileSidebarProps {
  selectedProfileName: string;
  onSelectProfile: (name: string) => void;
  isCollapsed: boolean;
  onToggleCollapse: () => void;
}

interface ProfileEntry {
  name: string;
  isActive: boolean;
  lastModified: string;
}

/**
 * Collapsible sidebar panel listing all profiles with CRUD actions.
 *
 * When expanded (default) it shows a scrollable list of ProfileSidebarItem
 * rows plus a "Create Profile" button. When collapsed it renders a narrow
 * icon strip. Auto-creates a default profile on first load if the list is
 * empty (guarded by a ref to run only once).
 */
export const ProfileSidebar: React.FC<ProfileSidebarProps> = ({
  selectedProfileName,
  onSelectProfile,
  isCollapsed,
  onToggleCollapse,
}) => {
  // --- data hooks ---
  const { data: profilesData, isLoading, error } = useProfiles();
  const createMutation = useCreateProfile();
  const activateMutation = useActivateProfile();
  const deleteMutation = useDeleteProfile();
  const toast = useToast();

  // --- derived profile list ---
  const profiles: ProfileEntry[] = useMemo(() => {
    if (!profilesData) return [];
    return profilesData.map((p) => ({
      name: p.name,
      isActive: p.isActive,
      lastModified: new Date(p.modifiedAt).toLocaleString('en-CA', {
        year: 'numeric',
        month: '2-digit',
        day: '2-digit',
        hour: '2-digit',
        minute: '2-digit',
        hour12: false,
      }),
    }));
  }, [profilesData]);

  // --- auto-create default profile ---
  const hasAutoCreated = useRef(false);

  useEffect(() => {
    if (!isLoading && !error && !hasAutoCreated.current && profiles.length === 0) {
      hasAutoCreated.current = true;
      autoCreateDefault();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [isLoading, error, profiles.length]);

  const autoCreateDefault = async () => {
    try {
      await createMutation.mutateAsync({ name: 'default', template: ProfileTemplate.Blank });
      await activateMutation.mutateAsync('default');
      onSelectProfile('default');
      toast.success(t('toast.defaultCreated'));
    } catch (err) {
      const msg = getErrorMessage(err, 'Failed to create default profile');
      const isOffline = msg.includes('Failed to fetch') || msg.includes('NetworkError');
      toast.error(isOffline ? t('toast.offline') : msg);
    }
  };

  // --- create modal state ---
  const [createOpen, setCreateOpen] = useState(false);
  const [newName, setNewName] = useState('');
  const [newDescription, setNewDescription] = useState('');
  const [template, setTemplate] = useState<ProfileTemplate>(ProfileTemplate.Blank);
  const [nameError, setNameError] = useState('');

  // --- delete modal state ---
  const [deleteOpen, setDeleteOpen] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<string | null>(null);

  // --- validation ---
  const validateName = (name: string): boolean => {
    if (!name.trim()) {
      setNameError(t('prof.nameRequired'));
      return false;
    }
    if (name.length > 50) {
      setNameError(t('prof.nameTooLong'));
      return false;
    }
    if (profiles.some((p) => p.name === name)) {
      setNameError(t('prof.nameExists'));
      return false;
    }
    setNameError('');
    return true;
  };

  // --- handlers ---
  const handleCreate = async () => {
    if (!validateName(newName)) return;
    try {
      await createMutation.mutateAsync({ name: newName, template });
      resetCreateModal();
      onSelectProfile(newName);
    } catch (err) {
      setNameError(getErrorMessage(err, 'Failed to create profile'));
    }
  };

  const resetCreateModal = () => {
    setCreateOpen(false);
    setNewName('');
    setNewDescription('');
    setTemplate(ProfileTemplate.Blank);
    setNameError('');
  };

  const handleActivate = async (name: string) => {
    if (activateMutation.isPending) return;
    try {
      const result = await activateMutation.mutateAsync(name);
      if (result.errors && result.errors.length > 0) {
        toast.error(t('toast.compileFailed'), {
          description: result.errors.join('\n'),
          duration: 8000,
        });
      } else {
        toast.success(t('toast.applied', { name }));
      }
    } catch {
      // Global MutationCache.onError handles the toast
    }
  };

  const handleDeleteConfirm = async () => {
    if (!deleteTarget) return;
    try {
      await deleteMutation.mutateAsync(deleteTarget);
      if (selectedProfileName === deleteTarget && profiles.length > 1) {
        const next = profiles.find((p) => p.name !== deleteTarget);
        if (next) onSelectProfile(next.name);
      }
    } catch {
      // Global MutationCache.onError handles the toast
    }
    setDeleteOpen(false);
    setDeleteTarget(null);
  };

  // --- collapsed view ---
  if (isCollapsed) {
    return (
      <div className="flex flex-col items-center w-12 bg-slate-800 border-r border-slate-700 h-full py-3 gap-2">
        <button
          type="button"
          aria-label={t('prof.expand')}
          title={t('prof.expand')}
          onClick={onToggleCollapse}
          className="p-2 rounded text-slate-400 hover:text-white hover:bg-slate-700 transition-colors"
        >
          <ChevronRight size={16} />
        </button>
        {profiles.map((p) => (
          <button
            key={p.name}
            type="button"
            title={p.name}
            aria-label={`Select profile ${p.name}`}
            onClick={() => onSelectProfile(p.name)}
            className={`p-2 rounded transition-colors ${
              p.name === selectedProfileName
                ? 'bg-primary-600/30 text-primary-400'
                : 'text-slate-400 hover:text-white hover:bg-slate-700'
            }`}
          >
            <User size={16} />
          </button>
        ))}
      </div>
    );
  }

  // --- expanded view ---
  return (
    <>
      <div className="flex flex-col w-64 bg-slate-800 border-r border-slate-700 h-full">
        {/* Header */}
        <div className="flex items-center justify-between px-3 py-3 border-b border-slate-700">
          <h2 className="text-sm font-semibold text-slate-100">{t('prof.title')}</h2>
          <div className="flex items-center gap-1">
            <button
              type="button"
              aria-label={t('prof.create')}
              title={t('prof.create')}
              onClick={() => setCreateOpen(true)}
              className="p-1.5 rounded text-slate-400 hover:text-white hover:bg-slate-700 transition-colors"
            >
              <Plus size={16} />
            </button>
            <button
              type="button"
              aria-label={t('prof.collapse')}
              title={t('prof.collapse')}
              onClick={onToggleCollapse}
              className="p-1.5 rounded text-slate-400 hover:text-white hover:bg-slate-700 transition-colors"
            >
              <ChevronLeft size={16} />
            </button>
          </div>
        </div>

        {/* Error */}
        {error && (
          <div className="px-3 py-2 text-xs text-red-400 bg-red-500/10">
            {error instanceof Error ? error.message : 'Failed to load profiles'}
          </div>
        )}

        {/* Scrollable list */}
        <div className="flex-1 overflow-y-auto px-1 py-2 space-y-0.5">
          {isLoading
            ? Array.from({ length: 3 }).map((_, i) => (
                <div
                  key={i}
                  className="animate-pulse rounded-md bg-slate-700/50 h-12 mx-2 mb-1"
                />
              ))
            : profiles.map((p) => (
                <ProfileSidebarItem
                  key={p.name}
                  name={p.name}
                  isActive={p.isActive}
                  isSelected={p.name === selectedProfileName}
                  lastModified={p.lastModified}
                  onSelect={() => onSelectProfile(p.name)}
                  onActivate={() => handleActivate(p.name)}
                  onDelete={() => {
                    setDeleteTarget(p.name);
                    setDeleteOpen(true);
                  }}
                />
              ))}
        </div>
      </div>

      {/* Create Profile Modal */}
      <Modal open={createOpen} onClose={resetCreateModal} title={t('prof.createTitle')}>
        <div className="flex flex-col gap-md">
          <Input
            type="text"
            value={newName}
            onChange={(value) => {
              setNewName(value);
              if (nameError) validateName(value);
            }}
            aria-label={t('prof.name')}
            placeholder={t('prof.name')}
            error={nameError}
            maxLength={50}
          />

          <div className="flex flex-col gap-2">
            <label htmlFor="sidebar-template-select" className="text-sm font-medium text-slate-300">
              {t('prof.template')}
            </label>
            <select
              id="sidebar-template-select"
              value={template}
              onChange={(e) => setTemplate(e.target.value as ProfileTemplate)}
              className="w-full px-3 py-2 bg-slate-800 border border-slate-700 rounded-md text-slate-100 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
              aria-label={t('prof.templateAria')}
            >
              <option value="blank">{t('prof.tBlank')}</option>
              <option value="simple_remap">{t('prof.tSimple')}</option>
              <option value="capslock_escape">{t('prof.tCaps')}</option>
              <option value="vim_navigation">{t('prof.tVim')}</option>
              <option value="gaming">{t('prof.tGaming')}</option>
            </select>
          </div>

          <Input
            type="text"
            value={newDescription}
            onChange={(value) => setNewDescription(value)}
            aria-label={t('prof.description')}
            placeholder={t('prof.descriptionPh')}
            maxLength={200}
          />

          <div className="flex gap-2 justify-end mt-2">
            <Button variant="secondary" size="md" onClick={resetCreateModal} aria-label={t('prof.cancelCreate')}>
              {t('prof.cancel')}
            </Button>
            <Button variant="primary" size="md" onClick={handleCreate} aria-label={t('prof.createBtnAria')}>
              {t('prof.createBtn')}
            </Button>
          </div>
        </div>
      </Modal>

      {/* Delete Confirmation Modal */}
      <Modal
        open={deleteOpen}
        onClose={() => {
          setDeleteOpen(false);
          setDeleteTarget(null);
        }}
        title={t('prof.deleteTitle')}
      >
        <div className="flex flex-col gap-md">
          <p className="text-slate-300">
            {t('prof.deleteConfirm')}{' '}
            <strong className="text-white">{deleteTarget}</strong>? {t('prof.deleteWarn')}
          </p>
          <div className="flex gap-2 justify-end mt-2">
            <Button
              variant="secondary"
              size="md"
              onClick={() => {
                setDeleteOpen(false);
                setDeleteTarget(null);
              }}
              aria-label={t('prof.cancelDelete')}
            >
              {t('prof.cancel')}
            </Button>
            <Button variant="danger" size="md" onClick={handleDeleteConfirm} aria-label={t('prof.deleteConfirmAria')}>
              {t('prof.deleteBtn')}
            </Button>
          </div>
        </div>
      </Modal>
    </>
  );
};

ProfileSidebar.displayName = 'ProfileSidebar';
