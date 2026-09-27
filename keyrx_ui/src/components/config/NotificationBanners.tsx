import React, { useState, useCallback, useEffect, useMemo } from 'react';
import { X } from 'lucide-react';

interface NotificationBannersProps {
  profileName: string;
  profileExists: boolean;
  configMissing: boolean;
  error: Error | null;
  isLoading: boolean;
  isConnected: boolean;
  onCreateProfile: () => void;
}

/** Dismiss (X) button for a banner. Declared outside the render body so it
 * isn't re-created as a new component on every render. */
const DismissButton: React.FC<{ onDismiss: () => void }> = ({ onDismiss }) => (
  <button
    onClick={onDismiss}
    className="ml-auto flex-shrink-0 p-1 rounded hover:bg-white/10 transition-colors"
    aria-label="Dismiss notification"
  >
    <X className="w-4 h-4" />
  </button>
);

/**
 * Displays informational banners for profile and config status.
 * Info banners auto-dismiss after 8s, error banners after 12s.
 * All banners can be manually dismissed with the X button.
 * Dismissed state resets when the profile changes.
 */
export const NotificationBanners: React.FC<NotificationBannersProps> = ({
  profileName,
  profileExists,
  configMissing,
  error,
  isLoading,
  isConnected,
  onCreateProfile,
}) => {
  // Dismissed keys are tagged with the profile they were dismissed under.
  // When `profileName` changes, the tag no longer matches, so `dismissed`
  // derives back to empty on the next render — no effect needed to reset it.
  const [dismissedState, setDismissedState] = useState<{
    profileName: string;
    keys: Set<string>;
  }>({ profileName, keys: new Set() });

  // Memoized so the derived Set has a stable identity across renders that
  // don't actually change it — otherwise the auto-dismiss effect below would
  // see a "new" dependency (and re-run) on every render while dismissedState
  // is stale for the current profile.
  const dismissed = useMemo(
    () =>
      dismissedState.profileName === profileName
        ? dismissedState.keys
        : new Set<string>(),
    [dismissedState, profileName]
  );

  const dismiss = useCallback(
    (key: string) => {
      setDismissedState((prev) => {
        const keys =
          prev.profileName === profileName ? prev.keys : new Set<string>();
        return { profileName, keys: new Set(keys).add(key) };
      });
    },
    [profileName]
  );

  // Auto-dismiss timers
  useEffect(() => {
    const timers: ReturnType<typeof setTimeout>[] = [];
    if (configMissing && !dismissed.has('configMissing')) {
      timers.push(setTimeout(() => dismiss('configMissing'), 8000));
    }
    return () => timers.forEach(clearTimeout);
  }, [configMissing, dismissed, dismiss]);

  return (
    <>
      {/* Profile doesn't exist — not auto-dismissed (requires user action) */}
      {!profileExists && !isLoading && isConnected && !dismissed.has('noProfile') && (
        <div className="p-3 bg-orange-900/20 border border-orange-500 rounded-md flex items-start gap-2">
          <div className="flex-1">
            <p className="text-sm text-orange-300 mb-2">
              Profile "{profileName}" does not exist.
            </p>
            <button
              onClick={onCreateProfile}
              className="px-4 py-1.5 bg-orange-600 hover:bg-orange-500 text-white text-sm font-medium rounded transition-colors"
            >
              Create Profile "{profileName}"
            </button>
          </div>
          <DismissButton onDismiss={() => dismiss('noProfile')} />
        </div>
      )}

      {/* Config file missing */}
      {configMissing && !dismissed.has('configMissing') && (
        <div className="p-3 bg-blue-900/20 border border-blue-500 rounded-md flex items-start gap-2">
          <p className="text-sm text-blue-300 flex-1">
            No configuration file found for "{profileName}". A template has
            been loaded — click <strong>Save</strong> to create it.
          </p>
          <DismissButton onDismiss={() => dismiss('configMissing')} />
        </div>
      )}

      {/* Error loading config */}
      {error && !dismissed.has('error') && (
        <div className="p-3 bg-red-900/20 border border-red-500 rounded-md flex items-start gap-2">
          <p className="text-sm text-red-300 flex-1">
            {error instanceof Error
              ? error.message
              : 'Failed to load configuration'}
          </p>
          <DismissButton onDismiss={() => dismiss('error')} />
        </div>
      )}
    </>
  );
};
