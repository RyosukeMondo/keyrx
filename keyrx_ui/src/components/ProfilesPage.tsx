import { useState, useEffect } from 'react';
import { ProfileCard } from './ProfileCard';
import { ProfileDialog } from './ProfileDialog';
import './ProfilesPage.css';

export interface Profile {
  name: string;
  rhai_path: string;
  krx_path: string;
  modified_at: number;
  layer_count: number;
  is_active: boolean;
}

interface ProfilesListResponse {
  profiles: Profile[];
}

export function ProfilesPage() {
  const [profiles, setProfiles] = useState<Profile[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showCreateDialog, setShowCreateDialog] = useState(false);
  const [renameProfile, setRenameProfile] = useState<Profile | null>(null);

  useEffect(() => {
    loadProfiles();
  }, []);

  const loadProfiles = async () => {
    try {
      setLoading(true);
      setError(null);
      const response = await fetch('http://localhost:3030/api/profiles');
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      const data: ProfilesListResponse = await response.json();
      setProfiles(data.profiles);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load profiles');
    } finally {
      setLoading(false);
    }
  };

  const handleCreateProfile = async (name: string, template: string) => {
    try {
      const response = await fetch('http://localhost:3030/api/profiles', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name, template }),
      });
      if (!response.ok) {
        const data = await response.json();
        throw new Error(data.error || 'Failed to create profile');
      }
      setShowCreateDialog(false);
      loadProfiles();
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to create profile');
    }
  };

  const handleActivateProfile = async (profile: Profile) => {
    try {
      const response = await fetch(
        `http://localhost:3030/api/profiles/${encodeURIComponent(profile.name)}/activate`,
        { method: 'POST' }
      );
      if (!response.ok) {
        throw new Error('Failed to activate profile');
      }
      loadProfiles();
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to activate profile');
    }
  };

  const handleDeleteProfile = async (profile: Profile) => {
    if (!confirm(`Delete profile "${profile.name}"?`)) {
      return;
    }
    try {
      const response = await fetch(
        `http://localhost:3030/api/profiles/${encodeURIComponent(profile.name)}`,
        { method: 'DELETE' }
      );
      if (!response.ok) {
        throw new Error('Failed to delete profile');
      }
      loadProfiles();
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to delete profile');
    }
  };

  const handleDuplicateProfile = async (profile: Profile) => {
    const newName = prompt(`Enter name for duplicated profile:`, `${profile.name}-copy`);
    if (!newName) return;

    try {
      const response = await fetch(
        `http://localhost:3030/api/profiles/${encodeURIComponent(profile.name)}/duplicate`,
        {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ dest: newName }),
        }
      );
      if (!response.ok) {
        throw new Error('Failed to duplicate profile');
      }
      loadProfiles();
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to duplicate profile');
    }
  };

  const handleExportProfile = async (profile: Profile) => {
    try {
      // For export, we'll download the .rhai file
      const response = await fetch(profile.rhai_path);
      if (!response.ok) {
        throw new Error('Failed to export profile');
      }
      const blob = await response.blob();
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `${profile.name}.rhai`;
      document.body.appendChild(a);
      a.click();
      window.URL.revokeObjectURL(url);
      document.body.removeChild(a);
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to export profile');
    }
  };

  if (loading) {
    return (
      <div className="profiles-page">
        <div className="profiles-loading">Loading profiles...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="profiles-page">
        <div className="profiles-error">
          Error: {error}
          <button onClick={loadProfiles} className="retry-button">
            Retry
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="profiles-page">
      <div className="profiles-header">
        <h2>Profiles</h2>
        <button onClick={() => setShowCreateDialog(true)} className="create-profile-button">
          + New Profile
        </button>
      </div>

      <div className="profiles-list">
        {profiles.length === 0 ? (
          <div className="no-profiles">
            <p>No profiles found.</p>
            <button onClick={() => setShowCreateDialog(true)} className="create-first-button">
              Create your first profile
            </button>
          </div>
        ) : (
          profiles.map((profile) => (
            <ProfileCard
              key={profile.name}
              profile={profile}
              onActivate={() => handleActivateProfile(profile)}
              onDelete={() => handleDeleteProfile(profile)}
              onDuplicate={() => handleDuplicateProfile(profile)}
              onExport={() => handleExportProfile(profile)}
              onRename={() => setRenameProfile(profile)}
            />
          ))
        )}
      </div>

      {showCreateDialog && (
        <ProfileDialog
          mode="create"
          onClose={() => setShowCreateDialog(false)}
          onSubmit={handleCreateProfile}
        />
      )}

      {renameProfile && (
        <ProfileDialog
          mode="rename"
          initialName={renameProfile.name}
          onClose={() => setRenameProfile(null)}
          onSubmit={(name) => {
            // Rename is not implemented in the API yet - would need to be added
            alert('Rename functionality not yet implemented in API');
            setRenameProfile(null);
          }}
        />
      )}
    </div>
  );
}
