import React, { useState, useEffect, useRef, useMemo } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import {
  useGetProfileConfig,
  useSetProfileConfig,
} from '@/hooks/useProfileConfig';
import { useProfiles, useCreateProfile } from '@/hooks/useProfiles';
import { useUnifiedApi } from '@/hooks/useUnifiedApi';
import { useConfigStore } from '@/stores/configStore';
import { ProfileTemplate } from '@/types';
import type { LayoutType } from '@/components/KeyboardVisualizer';

// Custom hooks
import { useProfileSelection } from '@/hooks/useProfileSelection';
import { useCodePanel } from '@/hooks/useCodePanel';
import { useKeyboardLayout } from '@/hooks/useKeyboardLayout';
import { useConfigSync } from '@/hooks/useConfigSync';
import { useASTSync } from '@/hooks/useASTSync';
import { useKeyboardShortcuts, CommonShortcuts } from '@/hooks/useKeyboardShortcuts';

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

  useKeyboardShortcuts([
    CommonShortcuts.save(handleSaveConfig),
  ], activeTab === 'edit' && api.isConnected && profileExists);

  return (
    <div className="flex h-full min-h-[calc(100vh-4rem)]">
      {/* Mobile profile sidebar toggle — pinned to top-left below the app header */}
      <button
        className="md:hidden fixed top-[4.5rem] left-2 z-30 p-2 bg-slate-700 rounded-md text-slate-300 hover:bg-slate-600 shadow-lg"
        onClick={() => setMobileSidebarOpen(!mobileSidebarOpen)}
        aria-label="Toggle profile sidebar"
        aria-expanded={mobileSidebarOpen}
      >
        <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15.75 6a3.75 3.75 0 1 1-7.5 0 3.75 3.75 0 0 1 7.5 0ZM4.5 20.118a7.5 7.5 0 0 1 14.998 0" />
        </svg>
      </button>

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
        {/* Header: Tab switcher + actions */}
        <div className="flex flex-col sm:flex-row items-stretch sm:items-center justify-between gap-3 p-4 border-b border-slate-700 flex-shrink-0">
          {/* Tabs */}
          <div className="flex rounded-lg bg-slate-800 p-1" role="tablist">
            <button
              role="tab"
              aria-selected={activeTab === 'edit'}
              onClick={() => setActiveTab('edit')}
              className={`px-4 py-2 text-sm font-medium rounded-md transition-colors ${
                activeTab === 'edit'
                  ? 'bg-primary-600 text-white'
                  : 'text-slate-400 hover:text-slate-200'
              }`}
            >
              Edit
            </button>
            <button
              role="tab"
              aria-selected={activeTab === 'test'}
              onClick={() => setActiveTab('test')}
              className={`px-4 py-2 text-sm font-medium rounded-md transition-colors ${
                activeTab === 'test'
                  ? 'bg-primary-600 text-white'
                  : 'text-slate-400 hover:text-slate-200'
              }`}
            >
              Test
            </button>
          </div>

          {/* Actions (Edit tab only shows code toggle + save) */}
          <div className="flex items-center gap-3">
            <SyncStatusIndicator
              syncStatus={syncStatus}
              lastSaveTime={lastSaveTime}
              isConnected={api.isConnected}
            />

            {activeTab === 'edit' && (
              <>
                <button
                  onClick={toggleCodePanel}
                  className="px-4 py-2 bg-slate-700 text-slate-200 text-sm font-medium rounded-md hover:bg-slate-600 transition-colors whitespace-nowrap border border-slate-600"
                  title={isCodePanelOpen ? 'Hide Code' : 'Show Code'}
                >
                  {isCodePanelOpen ? '▲ Hide Code' : '▼ Show Code'}
                </button>

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
                    !api.isConnected ||
                    !profileExists ||
                    syncStatus === 'saving'
                  }
                  className="px-4 py-2 bg-primary-500 text-white text-sm font-medium rounded-md hover:bg-primary-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors whitespace-nowrap"
                >
                  {configMissing ? 'Create' : 'Save'}
                </button>
              </>
            )}
          </div>
        </div>

        {/* Notifications — visible on all tabs */}
        <div className="px-4 md:px-6 pt-4 md:pt-6">
          <NotificationBanners
            profileName={selectedProfileName}
            profileExists={profileExists}
            configMissing={configMissing}
            error={error}
            isLoading={isLoading}
            isConnected={api.isConnected}
            onCreateProfile={handleCreateProfile}
          />
        </div>

        {/* Tab Content — both stay mounted for state preservation */}
        <div className="flex-1 overflow-y-auto">
          <div className={activeTab === 'edit' ? '' : 'hidden'}>
            <div className="flex flex-col gap-4 md:gap-6 p-4 md:p-6">
              <EditTab
                selectedProfileName={selectedProfileName}
                profileConfig={profileConfig}
                syncEngine={syncEngine}
                syncStatus={syncStatus}
                setSyncStatus={setSyncStatus}
                configStore={configStore}
                keyboardLayout={keyboardLayout}
                layoutKeys={layoutKeys}
              />

              {/* Code Panel (Edit tab only) */}
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
