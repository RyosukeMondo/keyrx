import React, { useState, useEffect, useRef, useMemo } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import {
  Check,
  Code2,
  FlaskConical,
  Save,
  SlidersHorizontal,
} from 'lucide-react';
import {
  useGetProfileConfig,
  useSetProfileConfig,
} from '@/hooks/useProfileConfig';
import { useProfiles, useCreateProfile } from '@/hooks/useProfiles';
import { useUnifiedApi } from '@/hooks/useUnifiedApi';
import { usePageTitle } from '@/hooks/usePageTitle';
import { useConfigStore } from '@/stores/configStore';
import { ProfileTemplate } from '@/types';
import type { LayoutType } from '@/components/KeyboardVisualizer';

// Custom hooks
import { useProfileSelection } from '@/hooks/useProfileSelection';
import { useCodePanel } from '@/hooks/useCodePanel';
import { useKeyboardLayout } from '@/hooks/useKeyboardLayout';
import { useConfigSync } from '@/hooks/useConfigSync';
import { useASTSync } from '@/hooks/useASTSync';
import {
  useKeyboardShortcuts,
  CommonShortcuts,
} from '@/hooks/useKeyboardShortcuts';

// Components
import { CodePanelContainer } from '@/components/config/CodePanelContainer';
import { SyncStatusIndicator } from '@/components/config/SyncStatusIndicator';
import { ProfileSidebar } from '@/components/config/ProfileSidebar';
import { EditTab } from '@/components/config/EditTab';
import { SimulatorTab } from '@/components/config/SimulatorTab';
import { Modal } from '@/components/Modal';
import { NotificationBanners } from '@/components/config/NotificationBanners';
import { ProfileDiffView } from '@/components/config/ProfileDiffView';

/** Auto-detect keyboard layout from Rhai config source */
function detectLayoutFromSource(source: string | undefined): LayoutType {
  if (!source) return 'JIS_109';
  const jisKeys = [
    'VK_Zenkaku',
    'VK_全角',
    'VK_無変換',
    'VK_変換',
    'VK_ひらがな',
    'VK_カタカナ',
    'VK_Ro',
    'VK_Yen',
    'VK_Henkan',
    'VK_Muhenkan',
  ];
  return jisKeys.some((k) => source.includes(k)) ? 'JIS_109' : 'ANSI_104';
}

type ActiveTab = 'edit' | 'test';

const ConfigPage: React.FC = () => {
  const navigate = useNavigate();
  const { name: routeProfileName } = useParams<{ name: string }>();
  const api = useUnifiedApi();
  usePageTitle('Config');

  // Profile selection (route param feeds into priority chain)
  const { selectedProfileName, setSelectedProfileName } =
    useProfileSelection(routeProfileName);

  // Code panel state
  const {
    isOpen: isCodePanelOpen,
    height: _codePanelHeight,
    toggleOpen: toggleCodePanel,
  } = useCodePanel();

  // Sync engine
  const {
    syncEngine,
    syncStatus,
    lastSaveTime,
    setSyncStatus,
    setLastSaveTime,
  } = useConfigSync(selectedProfileName);

  // Profiles
  const { data: profiles, isLoading: isLoadingProfiles } = useProfiles();
  const { mutateAsync: createProfile } = useCreateProfile();

  // Config store (Zustand)
  const configStore = useConfigStore();

  // Profile config query
  const {
    data: profileConfig,
    isLoading,
    error,
  } = useGetProfileConfig(selectedProfileName);
  const { mutateAsync: setProfileConfig } = useSetProfileConfig();

  // Keyboard layout detection
  const detectedLayout = useMemo(
    () => detectLayoutFromSource(profileConfig?.source),
    [profileConfig?.source]
  );
  const {
    layout: keyboardLayout,
    setLayout,
    layoutKeys,
  } = useKeyboardLayout(detectedLayout);

  useEffect(() => {
    setLayout(detectedLayout);
  }, [detectedLayout, setLayout]);

  // AST sync (visual editor state from parsed config)
  useASTSync({
    syncEngine,
    configStore,
    globalSelected: configStore.globalSelected,
    selectedDevices: configStore.selectedDevices,
  });

  // Local UI state
  const [activeTab, setActiveTab] = useState<ActiveTab>('edit');
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false);
  const [mobileSidebarOpen, setMobileSidebarOpen] = useState(false);
  const [showDiffModal, setShowDiffModal] = useState(false);

  // Profile existence checks
  const profileExists =
    profiles?.some((p) => p.name === selectedProfileName) ?? false;
  const configMissing =
    !isLoading && !error && profileExists && !profileConfig?.source;

  // Track profile changes for config loading
  const lastProfileRef = useRef<string>(selectedProfileName);
  const configLoadedRef = useRef<boolean>(false);

  // Auto-select first profile if selected doesn't exist
  useEffect(() => {
    if (
      profiles &&
      profiles.length > 0 &&
      !profiles.some((p) => p.name === selectedProfileName)
    ) {
      setSelectedProfileName(profiles[0].name);
    }
  }, [profiles, selectedProfileName, setSelectedProfileName]);

  // Load config into sync engine when profile config arrives
  useEffect(() => {
    const profileChanged = lastProfileRef.current !== selectedProfileName;
    if (profileChanged) {
      lastProfileRef.current = selectedProfileName;
      configLoadedRef.current = false;
    }

    const shouldLoadConfig = profileConfig?.source && !configLoadedRef.current;
    if (shouldLoadConfig) {
      syncEngine.loadServerConfig(profileConfig.source);
      setSyncStatus('saved');
      configLoadedRef.current = true;
    } else if (profileChanged && configMissing) {
      const defaultTemplate = `// Configuration for profile: ${selectedProfileName}\n// Add your key mappings here...\n`;
      syncEngine.onCodeChange(defaultTemplate);
      setSyncStatus('unsaved');
      configLoadedRef.current = true;
    }
  }, [
    profileConfig,
    configMissing,
    selectedProfileName,
    syncEngine,
    setSyncStatus,
  ]);

  // Track code changes for unsaved status
  useEffect(() => {
    if (syncStatus === 'saved' && syncEngine.state === 'idle') {
      const currentCode = syncEngine.getCode();
      const originalCode = profileConfig?.source || '';
      if (currentCode !== originalCode) {
        setSyncStatus('unsaved');
      }
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [
    syncEngine.state,
    syncEngine.getCode,
    profileConfig?.source,
    setSyncStatus,
  ]);

  // Handlers
  const handleProfileSelect = (name: string) => {
    setSelectedProfileName(name);
    navigate(`/profiles/${name}/config`, { replace: true });
    setMobileSidebarOpen(false);
  };

  const handleCreateProfile = async () => {
    try {
      await createProfile({
        name: selectedProfileName,
        template: ProfileTemplate.Blank,
      });
    } catch {
      // Global MutationCache.onError handles the toast
    }
  };

  const handleSaveConfig = async () => {
    try {
      setSyncStatus('saving');
      await setProfileConfig({
        name: selectedProfileName,
        source: syncEngine.getCode(),
      });
      setSyncStatus('saved');
      setLastSaveTime(new Date());
    } catch {
      setSyncStatus('unsaved');
    }
  };

  useKeyboardShortcuts(
    [CommonShortcuts.save(handleSaveConfig)],
    activeTab === 'edit' && api.isConnected && profileExists
  );

  return (
    <div className="flex h-full min-h-[calc(100vh-4rem)]">
      {/* Mobile sidebar backdrop */}
      {mobileSidebarOpen && (
        <div
          className="md:hidden fixed inset-0 bg-black/50 z-40"
          onClick={() => setMobileSidebarOpen(false)}
          aria-hidden="true"
        />
      )}

      {/* Profile Sidebar */}
      <div
        className={`
          fixed md:relative z-50 md:z-auto
          h-full md:h-auto
          transition-transform duration-300
          ${mobileSidebarOpen ? 'translate-x-0' : '-translate-x-full md:translate-x-0'}
          ${sidebarCollapsed ? 'md:w-12' : 'md:w-64'}
          flex-shrink-0
        `}
      >
        <ProfileSidebar
          selectedProfileName={selectedProfileName}
          onSelectProfile={handleProfileSelect}
          isCollapsed={sidebarCollapsed}
          onToggleCollapse={() => setSidebarCollapsed(!sidebarCollapsed)}
        />
      </div>

      {/* Main Content */}
      <div className="flex-1 flex flex-col min-w-0 overflow-hidden">
        {/* Workspace header: context, workflow, and primary action */}
        <div className="flex flex-col gap-4 border-b border-slate-700/80 bg-slate-900/80 p-4 backdrop-blur md:px-6 md:py-5">
          <div className="flex flex-col gap-4 xl:flex-row xl:items-center xl:justify-between">
            <div className="min-w-0">
              {/* Mobile profile list toggle: in the header flow (not fixed) so it
                  can never cover the workspace label or title. */}
              <button
                className="md:hidden mb-2 inline-flex items-center gap-2 rounded-md bg-slate-700 px-3 py-2 text-sm font-medium text-slate-100 hover:bg-slate-600"
                onClick={() => setMobileSidebarOpen(!mobileSidebarOpen)}
                aria-label="Toggle profile sidebar"
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
                Profiles
              </button>
              <div className="mb-1 flex items-center gap-2 text-xs font-medium uppercase tracking-[0.14em] text-primary-300">
                <SlidersHorizontal className="h-3.5 w-3.5" aria-hidden="true" />
                Keymap workspace
              </div>
              <div className="flex min-w-0 items-center gap-3">
                {/* `selectedProfileName` is '' while the active-profile
                    query is still loading (or indefinitely while
                    disconnected) -- an empty <h1> has no accessible name
                    (axe empty-heading), so fall back to visible text. */}
                <h1 className="truncate text-xl font-semibold text-slate-50 md:text-2xl">
                  {selectedProfileName || 'Loading profile…'}
                </h1>
                <span
                  className={`inline-flex items-center gap-1.5 rounded-full px-2.5 py-1 text-xs font-medium ring-1 ${api.isConnected ? 'bg-emerald-400/10 text-emerald-300 ring-emerald-400/20' : 'bg-rose-400/10 text-rose-300 ring-rose-400/20'}`}
                >
                  <span
                    className={`h-1.5 w-1.5 rounded-full ${api.isConnected ? 'bg-emerald-400' : 'bg-rose-400'}`}
                  />
                  {api.isConnected ? 'Daemon connected' : 'Daemon offline'}
                </span>
              </div>
              <p className="mt-1 text-sm text-slate-400">
                Choose where it applies, map the keys, then test before saving.
              </p>
            </div>

            <div className="flex flex-wrap items-center gap-2 sm:flex-nowrap">
              <div
                className="flex rounded-lg bg-slate-800 p-1 ring-1 ring-slate-700"
                role="tablist"
                aria-label="Configuration workflow"
              >
                <button
                  role="tab"
                  aria-selected={activeTab === 'edit'}
                  onClick={() => setActiveTab('edit')}
                  className={`flex items-center gap-2 rounded-md px-3 py-2 text-sm font-medium transition-colors ${
                    activeTab === 'edit'
                      ? 'bg-slate-600 text-white shadow-sm'
                      : 'text-slate-400 hover:text-slate-200'
                  }`}
                >
                  <SlidersHorizontal className="h-4 w-4" aria-hidden="true" />
                  1. Map
                </button>
                <button
                  role="tab"
                  aria-selected={activeTab === 'test'}
                  onClick={() => setActiveTab('test')}
                  className={`flex items-center gap-2 rounded-md px-3 py-2 text-sm font-medium transition-colors ${
                    activeTab === 'test'
                      ? 'bg-slate-600 text-white shadow-sm'
                      : 'text-slate-400 hover:text-slate-200'
                  }`}
                >
                  <FlaskConical className="h-4 w-4" aria-hidden="true" />
                  2. Test
                </button>
              </div>

              {/* Actions (Edit tab only shows code toggle + save) */}
              {activeTab === 'edit' && (
                <button
                  onClick={toggleCodePanel}
                  aria-label={isCodePanelOpen ? 'Hide Code' : 'Show Code'}
                  className="flex items-center gap-2 rounded-lg border border-slate-700 bg-slate-800 px-3 py-2 text-sm font-medium text-slate-300 transition-colors hover:border-slate-600 hover:bg-slate-700"
                  title={isCodePanelOpen ? 'Hide Code' : 'Show Code'}
                >
                  <Code2 className="h-4 w-4" aria-hidden="true" />
                  <span className="hidden sm:inline">
                    {isCodePanelOpen ? 'Hide code' : 'Code'}
                  </span>
                </button>
              )}

              <button
                onClick={() => {
                  const currentCode = syncEngine.getCode();
                  const originalCode = profileConfig?.source || '';
                  if (currentCode === originalCode) {
                    handleSaveConfig();
                  } else {
                    setShowDiffModal(true);
                  }
                }}
                disabled={
                  !api.isConnected || !profileExists || syncStatus === 'saving'
                }
                className="flex items-center gap-2 rounded-lg bg-primary-500 px-4 py-2 text-sm font-semibold text-white shadow-lg shadow-primary-900/20 transition hover:bg-primary-600 disabled:cursor-not-allowed disabled:opacity-50"
              >
                {syncStatus === 'saved' ? (
                  <Check className="h-4 w-4" aria-hidden="true" />
                ) : (
                  <Save className="h-4 w-4" aria-hidden="true" />
                )}
                {configMissing
                  ? 'Create'
                  : syncStatus === 'saving'
                    ? 'Saving…'
                    : 'Save'}
              </button>
            </div>
          </div>

          <div className="flex min-h-5 items-center justify-between gap-3 border-t border-slate-800 pt-3">
            <SyncStatusIndicator
              syncStatus={syncStatus}
              lastSaveTime={lastSaveTime}
              isConnected={api.isConnected}
            />
            <span className="hidden text-xs text-slate-400 sm:inline">
              Tip: press{' '}
              <kbd className="rounded border border-slate-700 bg-slate-800 px-1.5 py-0.5 font-mono text-slate-300">
                Ctrl S
              </kbd>{' '}
              to save
            </span>
          </div>
        </div>

        {/* Notifications — visible on all tabs */}
        <div className="px-4 md:px-6 pt-4 md:pt-6">
          <NotificationBanners
            profileName={selectedProfileName}
            profileExists={profileExists}
            configMissing={configMissing}
            error={error}
            isLoading={isLoading || isLoadingProfiles}
            isConnected={api.isConnected}
            onCreateProfile={handleCreateProfile}
          />
        </div>

        {/* Tab Content — both stay mounted for state preservation */}
        <div className="flex-1 overflow-y-auto">
          <div className={activeTab === 'edit' ? '' : 'hidden'}>
            <div className="flex flex-col gap-4 md:gap-6 p-4 md:p-6">
              <EditTab
                key={selectedProfileName}
                selectedProfileName={selectedProfileName}
                profileConfig={profileConfig}
                syncEngine={syncEngine}
                syncStatus={syncStatus}
                setSyncStatus={setSyncStatus}
                configStore={configStore}
                keyboardLayout={keyboardLayout}
                onKeyboardLayoutChange={setLayout}
                layoutKeys={layoutKeys}
                onOpenAdvanced={() => {
                  if (!isCodePanelOpen) toggleCodePanel();
                  window.requestAnimationFrame(() => {
                    document.getElementById('code-panel')?.scrollIntoView({
                      behavior: 'smooth',
                      block: 'start',
                    });
                  });
                }}
              />

              {/* Code Panel (Edit tab only) */}
              <div id="code-panel" className="scroll-mt-4">
                <CodePanelContainer
                  profileName={selectedProfileName}
                  rhaiCode={syncEngine.getCode()}
                  onChange={(value) => syncEngine.onCodeChange(value)}
                  syncEngine={syncEngine}
                  isOpen={isCodePanelOpen}
                  onToggle={toggleCodePanel}
                />
              </div>
            </div>
          </div>

          <div className={activeTab === 'test' ? '' : 'hidden'}>
            <div className="p-4 md:p-6">
              <SimulatorTab
                profileName={selectedProfileName}
                profileConfig={profileConfig}
              />
            </div>
          </div>
        </div>
      </div>

      {showDiffModal && (
        <Modal
          open={showDiffModal}
          onClose={() => setShowDiffModal(false)}
          title="Review Changes"
          size="xl"
        >
          <ProfileDiffView
            original={profileConfig?.source || ''}
            modified={syncEngine.getCode()}
          />
          <div className="flex justify-end gap-3 mt-4">
            <button
              onClick={() => setShowDiffModal(false)}
              className="px-4 py-2 bg-slate-700 text-slate-200 text-sm font-medium rounded-md hover:bg-slate-600 transition-colors"
            >
              Cancel
            </button>
            <button
              onClick={() => {
                setShowDiffModal(false);
                handleSaveConfig();
              }}
              className="px-4 py-2 bg-primary-500 text-white text-sm font-medium rounded-md hover:bg-primary-600 transition-colors"
            >
              Confirm Save
            </button>
          </div>
        </Modal>
      )}
    </div>
  );
};

export default ConfigPage;
