import React from 'react';
import {
  Check,
  Code2,
  FlaskConical,
  Save,
  SlidersHorizontal,
} from 'lucide-react';
import { SyncStatusIndicator } from '@/components/config/SyncStatusIndicator';
import { HistoryControls } from '@/components/config/HistoryControls';
import type { SyncStatus } from '@/hooks/useConfigSync';
import { t } from '@/i18n';

export type ActiveTab = 'edit' | 'test';

interface ConfigWorkspaceHeaderProps {
  selectedProfileName: string;
  isConnected: boolean;
  activeTab: ActiveTab;
  onTabChange: (tab: ActiveTab) => void;
  mobileSidebarOpen: boolean;
  onToggleSidebar: () => void;
  isCodeOpen: boolean;
  onToggleCode: () => void;
  canUndo: boolean;
  canRedo: boolean;
  canRevert: boolean;
  onUndo: () => void;
  onRedo: () => void;
  onRevert: () => void;
  onSaveClick: () => void;
  profileExists: boolean;
  configMissing: boolean;
  syncStatus: SyncStatus;
  lastSaveTime: Date | null;
  saveBlockedReason: string | null;
}

/** Context, workflow tabs, history and the primary Save action. */
export const ConfigWorkspaceHeader: React.FC<ConfigWorkspaceHeaderProps> = ({
  selectedProfileName,
  isConnected,
  activeTab,
  onTabChange,
  mobileSidebarOpen,
  onToggleSidebar,
  isCodeOpen,
  onToggleCode,
  canUndo,
  canRedo,
  canRevert,
  onUndo,
  onRedo,
  onRevert,
  onSaveClick,
  profileExists,
  configMissing,
  syncStatus,
  lastSaveTime,
  saveBlockedReason,
}) => (
  <>
    {/* Workspace header: context, workflow, and primary action */}
    <div className="flex flex-col gap-4 border-b border-slate-700/80 bg-slate-900/80 p-4 backdrop-blur md:px-6 md:py-5">
      <div className="flex flex-col gap-4 xl:flex-row xl:items-center xl:justify-between">
        <div className="min-w-0">
          {/* Mobile profile list toggle: in the header flow (not fixed) so it
              can never cover the workspace label or title. */}
          <button
            className="md:hidden mb-2 inline-flex items-center gap-2 rounded-md bg-slate-700 px-3 py-2 text-sm font-medium text-slate-100 hover:bg-slate-600"
            onClick={() => onToggleSidebar()}
            aria-label={t('header.toggleProfiles')}
            aria-expanded={mobileSidebarOpen}
          >
            <svg
              className="h-5 w-5"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
              aria-hidden="true"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M15.75 6a3.75 3.75 0 1 1-7.5 0 3.75 3.75 0 0 1 7.5 0ZM4.5 20.118a7.5 7.5 0 0 1 14.998 0"
              />
            </svg>
            {t('header.profiles')}
          </button>
          <div className="mb-1 flex items-center gap-2 text-xs font-medium uppercase tracking-[0.14em] text-primary-300">
            <SlidersHorizontal className="h-3.5 w-3.5" aria-hidden="true" />
            {t('header.eyebrow')}
          </div>
          <div className="flex min-w-0 items-center gap-3">
            {/* `selectedProfileName` is '' while the active-profile
                query is still loading (or indefinitely while
                disconnected) -- an empty <h1> has no accessible name
                (axe empty-heading), so fall back to visible text. */}
            <h1 className="truncate text-xl font-semibold text-slate-50 md:text-2xl">
              {selectedProfileName || t('header.loading')}
            </h1>
            <span
              className={`inline-flex items-center gap-1.5 rounded-full px-2.5 py-1 text-xs font-medium ring-1 ${isConnected ? 'bg-emerald-400/10 text-emerald-300 ring-emerald-400/20' : 'bg-rose-400/10 text-rose-300 ring-rose-400/20'}`}
            >
              <span
                className={`h-1.5 w-1.5 rounded-full ${isConnected ? 'bg-emerald-400' : 'bg-rose-400'}`}
              />
              {isConnected ? t('daemon.connected') : t('status.offline')}
            </span>
          </div>
          <p className="mt-1 text-sm text-slate-400">{t('header.subtitle')}</p>
        </div>

        <div className="flex flex-wrap items-center gap-2 sm:flex-nowrap">
          <div
            className="flex rounded-lg bg-slate-800 p-1 ring-1 ring-slate-700"
            role="tablist"
            aria-label={t('header.workflow')}
          >
            <button
              role="tab"
              aria-selected={activeTab === 'edit'}
              onClick={() => onTabChange('edit')}
              className={`flex items-center gap-2 whitespace-nowrap rounded-md px-3 py-2 text-sm font-medium transition-colors ${
                activeTab === 'edit'
                  ? 'bg-slate-600 text-white shadow-sm'
                  : 'text-slate-400 hover:text-slate-200'
              }`}
            >
              <SlidersHorizontal className="h-4 w-4" aria-hidden="true" />
              {t('tab.map')}
            </button>
            <button
              role="tab"
              aria-selected={activeTab === 'test'}
              onClick={() => onTabChange('test')}
              className={`flex items-center gap-2 whitespace-nowrap rounded-md px-3 py-2 text-sm font-medium transition-colors ${
                activeTab === 'test'
                  ? 'bg-slate-600 text-white shadow-sm'
                  : 'text-slate-400 hover:text-slate-200'
              }`}
            >
              <FlaskConical className="h-4 w-4" aria-hidden="true" />
              {t('tab.test')}
            </button>
          </div>

          {/* Actions (Edit tab only shows code toggle + save) */}
          {activeTab === 'edit' && (
            <button
              onClick={onToggleCode}
              aria-label={isCodeOpen ? t('code.hideAria') : t('code.showAria')}
              className="flex items-center gap-2 whitespace-nowrap rounded-lg border border-slate-700 bg-slate-800 px-3 py-2 text-sm font-medium text-slate-300 transition-colors hover:border-slate-600 hover:bg-slate-700"
              title={isCodeOpen ? t('code.hideAria') : t('code.showAria')}
            >
              <Code2 className="h-4 w-4" aria-hidden="true" />
              <span className="hidden sm:inline">
                {isCodeOpen ? t('code.hide') : t('code.show')}
              </span>
            </button>
          )}

          {activeTab === 'edit' && (
            <HistoryControls
              canUndo={canUndo}
              canRedo={canRedo}
              canRevert={canRevert}
              onUndo={onUndo}
              onRedo={onRedo}
              onRevert={onRevert}
            />
          )}

          <button
            onClick={onSaveClick}
            disabled={
              !isConnected ||
              !profileExists ||
              syncStatus === 'saving' ||
              saveBlockedReason !== null
            }
            aria-describedby={
              saveBlockedReason ? 'save-blocked-reason' : undefined
            }
            className="flex items-center gap-2 whitespace-nowrap rounded-lg bg-primary-500 px-4 py-2 text-sm font-semibold text-white shadow-lg shadow-primary-900/20 transition hover:bg-primary-600 disabled:cursor-not-allowed disabled:opacity-50"
          >
            {syncStatus === 'saved' ? (
              <Check className="h-4 w-4" aria-hidden="true" />
            ) : (
              <Save className="h-4 w-4" aria-hidden="true" />
            )}
            {configMissing
              ? t('save.create')
              : syncStatus === 'saving'
                ? t('save.saving')
                : t('save.save')}
          </button>
        </div>
      </div>

      <div className="flex min-h-5 items-center justify-between gap-3 border-t border-slate-800 pt-3">
        <SyncStatusIndicator
          syncStatus={syncStatus}
          lastSaveTime={lastSaveTime}
          isConnected={isConnected}
        />
        {saveBlockedReason && (
          <span
            id="save-blocked-reason"
            role="status"
            className="min-w-0 truncate text-xs text-rose-300"
            title={saveBlockedReason}
          >
            {saveBlockedReason}
          </span>
        )}
        <span className="hidden text-xs text-slate-400 sm:inline">
          {t('save.tipBefore')}{' '}
          <kbd className="rounded border border-slate-700 bg-slate-800 px-1.5 py-0.5 font-mono text-slate-300">
            Ctrl S
          </kbd>{' '}
          {t('save.tipAfter')}
        </span>
      </div>
    </div>
  </>
);
