import React, { useState, useEffect, useCallback } from 'react';
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
  const [dismissed, setDismissed] = useState<Set<string>>(new Set());

  // Reset dismissed state when profile changes
  useEffect(() => {
    setDismissed(new Set());
  }, [profileName]);

  // Auto-dismiss timers
  useEffect(() => {
    const timers: ReturnType<typeof setTimeout>[] = [];
    if (configMissing && !dismissed.has('configMissing')) {
      timers.push(setTimeout(() => dismiss('configMissing'), 8000));
    }
    return () => timers.forEach(clearTimeout);
  }, [configMissing, dismissed]);

  const dismiss = useCallback((key: string) => {
    setDismissed(prev => new Set(prev).add(key));
  }, []);

  const DismissButton: React.FC<{ bannerKey: string }> = ({ bannerKey }) => (
    <button
      onClick={() => dismiss(bannerKey)}
      className="ml-auto flex-shrink-0 p-1 rounded hover:bg-white/10 transition-colors"
      aria-label="Dismiss notification"
    >
      <X className="w-4 h-4" />
    </button>
  );

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
          <DismissButton bannerKey="noProfile" />
        </div>
      )}

      {/* Config file missing */}
      {configMissing && !dismissed.has('configMissing') && (
        <div className="p-3 bg-blue-900/20 border border-blue-500 rounded-md flex items-start gap-2">
          <p className="text-sm text-blue-300 flex-1">
            No configuration file found for "{profileName}". A template has
            been loaded — click <strong>Save</strong> to create it.
          </p>
          <DismissButton bannerKey="configMissing" />
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
          <DismissButton bannerKey="error" />
        </div>
      )}
    </>
  );
};
