import React, { useState, useEffect, useMemo, useRef } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import {
  useGetProfileConfig,
  useSetProfileConfig,
} from '@/hooks/useProfileConfig';
import { useProfiles, useCreateProfile } from '@/hooks/useProfiles';
import { useUnifiedApi } from '@/hooks/useUnifiedApi';
import { usePageTitle } from '@/hooks/usePageTitle';
import { useConfigStore } from '@/stores/configStore';
import { ProfileTemplate } from '@/types';

// Custom hooks
import { useProfileSelection } from '@/hooks/useProfileSelection';
import { useCodePanel } from '@/hooks/useCodePanel';
import { useLayoutPreference } from '@/hooks/useLayoutPreference';
import { useConfigValidation } from '@/hooks/useConfigValidation';
import { useConfigSync } from '@/hooks/useConfigSync';
import { useCodeHistory } from '@/hooks/useCodeHistory';
import { useToast } from '@/hooks/useToast';
import { previousVersion, recordSavedVersion } from '@/utils/profileVersions';
import { useASTSync } from '@/hooks/useASTSync';
import {
  useKeyboardShortcuts,
  CommonShortcuts,
} from '@/hooks/useKeyboardShortcuts';

// Components
import { CodePanelContainer } from '@/components/config/CodePanelContainer';
import { ProfileSidebar } from '@/components/config/ProfileSidebar';
import { EditTab } from '@/components/config/EditTab';
import { SimulatorTab } from '@/components/config/SimulatorTab';
import { NotificationBanners } from '@/components/config/NotificationBanners';
import { SaveReviewModal } from '@/components/config/SaveReviewModal';
import {
  ConfigWorkspaceHeader,
  type ActiveTab,
} from '@/components/config/ConfigWorkspaceHeader';
import { t } from '@/i18n';
import { friendlyErrorMessage } from '@/utils/errorUtils';

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

  // Keyboard layout: remembered per device, detected only as a fallback
  const {
    layout: keyboardLayout,
    layoutKeys,
    chooseLayout,
  } = useLayoutPreference({
    source: profileConfig?.source,
    ast: syncEngine.getAST(),
    globalSelected: configStore.globalSelected,
    selectedScopeIds: configStore.selectedDevices,
  });

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
  // Bumped after each save so the "previous version" button re-reads storage.
  const [versionTick, setVersionTick] = useState(0);
  const toast = useToast();
  const history = useCodeHistory(syncEngine, activeTab === 'edit');
  const { reset: resetHistory } = history;

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
      resetHistory(profileConfig.source);
      setSyncStatus('saved');
      configLoadedRef.current = true;
    } else if (profileChanged && configMissing) {
      const defaultTemplate = `// Configuration for profile: ${selectedProfileName}\n// Add your key mappings here...\n`;
      syncEngine.onCodeChange(defaultTemplate);
      resetHistory(defaultTemplate);
      setSyncStatus('unsaved');
      configLoadedRef.current = true;
    }
  }, [
    profileConfig,
    configMissing,
    selectedProfileName,
    syncEngine,
    setSyncStatus,
    resetHistory,
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

  // Save is blocked while the editor shows errors: same validator, same facts.
  const validationErrors = useConfigValidation(syncEngine.getCode());
  const firstIssue = syncEngine.error ?? validationErrors[0] ?? null;
  const issueCount = validationErrors.length + (syncEngine.error ? 1 : 0);
  const saveBlockedReason = firstIssue
    ? `${t('save.blocked', {
        reason: `line ${firstIssue.line}: ${friendlyErrorMessage(firstIssue.message)}`,
      })}${issueCount > 1 ? ` (+${issueCount - 1})` : ''}`
    : null;

  const handleSaveConfig = async () => {
    if (saveBlockedReason) return;
    try {
      setSyncStatus('saving');
      const replaced = profileConfig?.source ?? '';
      const source = syncEngine.getCode();
      await setProfileConfig({ name: selectedProfileName, source });
      if (replaced !== source) {
        recordSavedVersion(selectedProfileName, replaced);
        setVersionTick((n) => n + 1);
      }
      setSyncStatus('saved');
      setLastSaveTime(new Date());
    } catch {
      setSyncStatus('unsaved');
    }
  };

  // Last saved version that differs from what is saved now (client-side
  // history, see utils/profileVersions.ts).
  const savedNow = profileConfig?.source ?? '';
  const priorVersion = useMemo(
    // versionTick: re-read storage after each save (the source string alone
    // may not change when the server echoes it back).
    () =>
      versionTick >= 0 ? previousVersion(selectedProfileName, savedNow) : null,
    [selectedProfileName, savedNow, versionTick]
  );

  const handleSaveClick = () => {
    if (syncEngine.getCode() === savedNow) handleSaveConfig();
    else setShowDiffModal(true);
  };

  const handleRevert = () => {
    if (!priorVersion) return;
    syncEngine.loadServerConfig(priorVersion.source);
    setSyncStatus('unsaved');
    toast.info(
      t('history.reverted', {
        when: new Date(priorVersion.savedAt).toLocaleString(),
      })
    );
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
        <ConfigWorkspaceHeader
          selectedProfileName={selectedProfileName}
          isConnected={api.isConnected}
          activeTab={activeTab}
          onTabChange={setActiveTab}
          mobileSidebarOpen={mobileSidebarOpen}
          onToggleSidebar={() => setMobileSidebarOpen(!mobileSidebarOpen)}
          isCodeOpen={isCodePanelOpen}
          onToggleCode={toggleCodePanel}
          canUndo={history.canUndo}
          canRedo={history.canRedo}
          canRevert={priorVersion !== null}
          onUndo={history.undo}
          onRedo={history.redo}
          onRevert={handleRevert}
          onSaveClick={handleSaveClick}
          profileExists={profileExists}
          configMissing={configMissing}
          syncStatus={syncStatus}
          lastSaveTime={lastSaveTime}
          saveBlockedReason={saveBlockedReason}
        />

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
                onKeyboardLayoutChange={chooseLayout}
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

      <SaveReviewModal
        open={showDiffModal}
        original={profileConfig?.source || ''}
        modified={showDiffModal ? syncEngine.getCode() : ''}
        onCancel={() => setShowDiffModal(false)}
        onConfirm={() => {
          setShowDiffModal(false);
          handleSaveConfig();
        }}
      />
    </div>
  );
};

export default ConfigPage;
