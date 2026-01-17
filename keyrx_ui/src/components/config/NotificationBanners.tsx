import React from 'react';

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
 * Displays informational banners for profile and config status
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
  return (
    <>
      {/* Profile doesn't exist */}
      {!profileExists && !isLoading && isConnected && (
        <div className="p-3 bg-orange-900/20 border border-orange-500 rounded-md">
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
      )}

      {/* Config file missing */}
      {configMissing && (
        <div className="p-3 bg-blue-900/20 border border-blue-500 rounded-md">
          <p className="text-sm text-blue-300">
            📝 No configuration file found for "{profileName}". A template has
            been loaded - click <strong>Save</strong> to create it.
          </p>
        </div>
      )}

      {/* Error loading config */}
      {error && (
        <div className="p-3 bg-red-900/20 border border-red-500 rounded-md">
          <p className="text-sm text-red-300">
            {error instanceof Error
              ? error.message
              : 'Failed to load configuration'}
          </p>
        </div>
      )}
    </>
  );
};
