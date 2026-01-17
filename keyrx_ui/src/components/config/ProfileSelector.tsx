import React, { useState } from 'react';
import type { ProfileMetadata } from '@/types';

/**
 * ProfileSelector component
 *
 * Renders a dropdown for profile selection with loading states
 * and a button for creating new profiles.
 *
 * @param props.value - Currently selected profile name
 * @param props.onChange - Callback when profile selection changes
 * @param props.profiles - Array of available profiles
 * @param props.isLoading - Loading state indicator
 * @param props.onCreateProfile - Callback to create a new profile
 * @param props.disabled - Whether the selector is disabled (e.g., when disconnected)
 *
 * @example
 * ```tsx
 * <ProfileSelector
 *   value={selectedProfileName}
 *   onChange={handleProfileChange}
 *   profiles={profiles}
 *   isLoading={isLoading}
 *   onCreateProfile={handleCreateProfile}
 *   disabled={!api.isConnected}
 * />
 * ```
 */
interface ProfileSelectorProps {
  value: string;
  onChange: (profileName: string) => void;
  profiles: ProfileMetadata[] | undefined;
  isLoading: boolean;
  onCreateProfile?: () => void;
  disabled?: boolean;
}

export const ProfileSelector: React.FC<ProfileSelectorProps> = ({
  value,
  onChange,
  profiles,
  isLoading,
  onCreateProfile,
  disabled = false,
}) => {
  const [isCreating, setIsCreating] = useState(false);
  const [newProfileName, setNewProfileName] = useState('');
  const [createError, setCreateError] = useState<string | null>(null);

  const handleCreateClick = () => {
    setIsCreating(true);
    setNewProfileName('');
    setCreateError(null);
  };

  const handleCreateConfirm = () => {
    if (!newProfileName.trim()) {
      setCreateError('Profile name cannot be empty');
      return;
    }

    // Check for duplicate names
    if (profiles?.some((p) => p.name === newProfileName.trim())) {
      setCreateError('Profile name already exists');
      return;
    }

    onCreateProfile?.();
    setIsCreating(false);
    setNewProfileName('');
    setCreateError(null);
  };

  const handleCreateCancel = () => {
    setIsCreating(false);
    setNewProfileName('');
    setCreateError(null);
  };

  return (
    <div className="flex items-center gap-3">
      <label
        htmlFor="profile-selector"
        className="text-sm font-medium text-slate-300 whitespace-nowrap"
      >
        Profile:
      </label>

      {isCreating ? (
        <div className="flex items-center gap-2">
          <input
            type="text"
            value={newProfileName}
            onChange={(e) => {
              setNewProfileName(e.target.value);
              setCreateError(null);
            }}
            onKeyDown={(e) => {
              if (e.key === 'Enter') {
                handleCreateConfirm();
              } else if (e.key === 'Escape') {
                handleCreateCancel();
              }
            }}
            placeholder="Enter profile name"
            autoFocus
            className="px-3 py-2 bg-slate-700 border border-slate-600 rounded-md text-slate-100 text-sm font-medium focus:outline-none focus:ring-2 focus:ring-primary-500"
            aria-label="New profile name"
            aria-invalid={!!createError}
            aria-describedby={createError ? 'create-error' : undefined}
          />
          <button
            onClick={handleCreateConfirm}
            className="px-3 py-2 bg-green-600 text-white text-sm font-medium rounded-md hover:bg-green-500 transition-colors"
            aria-label="Confirm create profile"
          >
            ✓
          </button>
          <button
            onClick={handleCreateCancel}
            className="px-3 py-2 bg-slate-600 text-white text-sm font-medium rounded-md hover:bg-slate-500 transition-colors"
            aria-label="Cancel create profile"
          >
            ✕
          </button>
          {createError && (
            <span
              id="create-error"
              className="text-xs text-red-400"
              role="alert"
            >
              {createError}
            </span>
          )}
        </div>
      ) : (
        <>
          <select
            id="profile-selector"
            value={value}
            onChange={(e) => onChange(e.target.value)}
            disabled={isLoading || disabled}
            className="px-3 py-2 bg-slate-700 border border-slate-600 rounded-md text-slate-100 text-sm font-medium focus:outline-none focus:ring-2 focus:ring-primary-500 disabled:opacity-50"
            aria-label="Select profile"
          >
            {profiles?.map((profile) => (
              <option key={profile.name} value={profile.name}>
                {profile.name}
              </option>
            ))}
          </select>

          {onCreateProfile && (
            <button
              onClick={handleCreateClick}
              disabled={disabled}
              className="px-3 py-2 bg-primary-500 text-white text-sm font-medium rounded-md hover:bg-primary-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
              aria-label="Create new profile"
            >
              + New
            </button>
          )}
        </>
      )}
    </div>
  );
};
