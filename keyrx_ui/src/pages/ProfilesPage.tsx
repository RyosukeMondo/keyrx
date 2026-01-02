import React, { useState, useMemo } from 'react';
import { ProfileCard } from '../components/ProfileCard';
import { Button } from '../components/Button';
import { Modal } from '../components/Modal';
import { Input } from '../components/Input';
import { Plus } from 'lucide-react';
import { SkeletonCard } from '../components/LoadingSkeleton';
import {
  useProfiles,
  useCreateProfile,
  useActivateProfile,
  useDeleteProfile,
} from '../hooks/useProfiles';

interface Profile {
  id: string;
  name: string;
  description?: string;
  isActive: boolean;
  lastModified: string;
}

/**
 * ProfilesPage Component
 *
 * Profile management interface showing:
 * - Grid of profile cards (responsive: 3 cols desktop, 2 cols tablet, 1 col mobile)
 * - Create new profile button
 * - Activate/Edit/Delete actions per profile
 * - Active profile highlighted with green checkmark and border
 *
 * User Flows:
 * - Create: Click "Create Profile" → Modal opens → Enter name/description → Save
 * - Activate: Click "Activate" on inactive profile → Becomes active, previous deactivates
 * - Edit: Click "Edit" → Modal opens with current values → Update
 * - Delete: Click "Delete" → Confirmation modal → Confirm → Profile removed
 *
 * Requirements: Req 6 (Profile Management User Flow)
 */
export const ProfilesPage: React.FC = () => {
  // Fetch profiles from real API
  const { data: profilesData, isLoading, error } = useProfiles();
  const createProfileMutation = useCreateProfile();
  const activateProfileMutation = useActivateProfile();
  const deleteProfileMutation = useDeleteProfile();

  // Transform API data to component format
  const profiles: Profile[] = useMemo(() => {
    if (!profilesData) return [];

    return profilesData.map((p) => ({
      id: p.name, // Use name as ID since it's unique
      name: p.name,
      description: undefined, // API doesn't provide description yet
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

  const [createModalOpen, setCreateModalOpen] = useState(false);
  const [editModalOpen, setEditModalOpen] = useState(false);
  const [deleteModalOpen, setDeleteModalOpen] = useState(false);
  const [selectedProfile, setSelectedProfile] = useState<Profile | null>(null);

  const [newProfileName, setNewProfileName] = useState('');
  const [newProfileDescription, setNewProfileDescription] = useState('');
  const [nameError, setNameError] = useState('');
  const [activationError, setActivationError] = useState<string | null>(null);

  const validateProfileName = (name: string): boolean => {
    if (!name.trim()) {
      setNameError('Profile name is required');
      return false;
    }
    if (name.length > 50) {
      setNameError('Profile name must be 50 characters or less');
      return false;
    }
    if (profiles.some((p) => p.name === name && p.id !== selectedProfile?.id)) {
      setNameError('Profile name already exists');
      return false;
    }
    setNameError('');
    return true;
  };

  const handleCreateProfile = async () => {
    if (!validateProfileName(newProfileName)) return;

    try {
      await createProfileMutation.mutateAsync({
        name: newProfileName,
        template: 'blank', // Default to blank template
      });

      setCreateModalOpen(false);
      setNewProfileName('');
      setNewProfileDescription('');
      setNameError('');
    } catch (err) {
      setNameError(err instanceof Error ? err.message : 'Failed to create profile');
    }
  };

  const handleActivateProfile = async (profileId: string) => {
    // Clear any previous activation errors
    setActivationError(null);

    try {
      const result = await activateProfileMutation.mutateAsync(profileId);

      // Check for compilation errors in the result
      if (result.errors && result.errors.length > 0) {
        const errorMessage = result.errors.join('\n');
        setActivationError(`Compilation failed:\n${errorMessage}`);
        console.error('Compilation errors:', result.errors);
      }
    } catch (err) {
      // Handle API errors
      const errorMessage = err instanceof Error ? err.message : 'Unknown error occurred';
      setActivationError(`Failed to activate profile: ${errorMessage}`);
      console.error('Failed to activate profile:', err);
    }
  };

  const handleEditProfile = (profile: Profile) => {
    setSelectedProfile(profile);
    setNewProfileName(profile.name);
    setNewProfileDescription(profile.description || '');
    setEditModalOpen(true);
  };

  const handleSaveEdit = async () => {
    if (!selectedProfile || !validateProfileName(newProfileName)) return;

    // TODO: Implement profile rename/update API
    // For now, just close the modal
    console.warn('Profile editing not yet implemented in API');

    setEditModalOpen(false);
    setSelectedProfile(null);
    setNewProfileName('');
    setNewProfileDescription('');
    setNameError('');
  };

  const handleDeleteProfile = (profile: Profile) => {
    setSelectedProfile(profile);
    setDeleteModalOpen(true);
  };

  const handleConfirmDelete = async () => {
    if (!selectedProfile) return;

    try {
      await deleteProfileMutation.mutateAsync(selectedProfile.name);
      setDeleteModalOpen(false);
      setSelectedProfile(null);
    } catch (err) {
      console.error('Failed to delete profile:', err);
      // Show error to user (could add error state here)
    }
  };

  const handleCancelCreate = () => {
    setCreateModalOpen(false);
    setNewProfileName('');
    setNewProfileDescription('');
    setNameError('');
  };

  const handleCancelEdit = () => {
    setEditModalOpen(false);
    setSelectedProfile(null);
    setNewProfileName('');
    setNewProfileDescription('');
    setNameError('');
  };

  // Show loading state while fetching
  if (isLoading) {
    return (
      <div className="flex flex-col gap-4 md:gap-6 p-4 md:p-6 lg:p-8">
        <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
          <SkeletonCard className="h-8 w-32" />
          <SkeletonCard className="h-10 w-40" />
        </div>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 md:gap-6">
          <SkeletonCard />
          <SkeletonCard />
          <SkeletonCard />
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-4 md:gap-6 p-4 md:p-6 lg:p-8">
      {/* Error Display */}
      {error && (
        <div className="p-4 bg-red-500/10 border border-red-500/20 rounded-lg">
          <p className="text-sm text-red-400">
            Failed to load profiles: {error instanceof Error ? error.message : 'Unknown error'}
          </p>
        </div>
      )}

      {/* Activation Error Display */}
      {activationError && (
        <div className="p-4 bg-red-500/10 border border-red-500/20 rounded-lg">
          <div className="flex justify-between items-start">
            <div className="flex-1">
              <p className="text-sm font-semibold text-red-400 mb-2">
                Profile Activation Error
              </p>
              <pre className="text-xs text-red-300 whitespace-pre-wrap font-mono">
                {activationError}
              </pre>
            </div>
            <button
              onClick={() => setActivationError(null)}
              className="text-red-400 hover:text-red-300 ml-4"
              aria-label="Dismiss error"
            >
              ×
            </button>
          </div>
        </div>
      )}

      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <h1 className="text-xl md:text-2xl lg:text-3xl font-semibold text-slate-100">
          Profiles
        </h1>
        <Button
          variant="primary"
          size="md"
          onClick={() => setCreateModalOpen(true)}
          aria-label="Create new profile"
          className="w-full sm:w-auto min-h-[44px] sm:min-h-0"
        >
          <Plus size={20} className="mr-2" aria-hidden="true" />
          Create Profile
        </Button>
      </div>

      {/* Profile Grid - responsive: 1 col mobile, 2 cols tablet, 3 cols desktop */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 md:gap-6">
        {profiles.map((profile) => (
          <ProfileCard
            key={profile.id}
            name={profile.name}
            description={profile.description}
            isActive={profile.isActive}
            lastModified={profile.lastModified}
            onActivate={() => handleActivateProfile(profile.id)}
            onEdit={() => handleEditProfile(profile)}
            onDelete={() => handleDeleteProfile(profile)}
          />
        ))}
      </div>

      {/* Create Profile Modal */}
      <Modal
        open={createModalOpen}
        onClose={handleCancelCreate}
        title="Create New Profile"
      >
        <div className="flex flex-col gap-md">
          <Input
            type="text"
            value={newProfileName}
            onChange={(value) => {
              setNewProfileName(value);
              if (nameError) validateProfileName(value);
            }}
            aria-label="Profile name"
            placeholder="Profile name"
            error={nameError}
            maxLength={50}
          />
          <Input
            type="text"
            value={newProfileDescription}
            onChange={(value) => setNewProfileDescription(value)}
            aria-label="Profile description"
            placeholder="Description (optional)"
            maxLength={200}
          />
          <div className="flex gap-2 justify-end mt-2">
            <Button
              variant="secondary"
              size="md"
              onClick={handleCancelCreate}
              aria-label="Cancel creating profile"
            >
              Cancel
            </Button>
            <Button
              variant="primary"
              size="md"
              onClick={handleCreateProfile}
              aria-label="Save new profile"
            >
              Create
            </Button>
          </div>
        </div>
      </Modal>

      {/* Edit Profile Modal */}
      <Modal
        open={editModalOpen}
        onClose={handleCancelEdit}
        title="Edit Profile"
      >
        <div className="flex flex-col gap-md">
          <Input
            type="text"
            value={newProfileName}
            onChange={(value) => {
              setNewProfileName(value);
              if (nameError) validateProfileName(value);
            }}
            aria-label="Profile name"
            placeholder="Profile name"
            error={nameError}
            maxLength={50}
          />
          <Input
            type="text"
            value={newProfileDescription}
            onChange={(value) => setNewProfileDescription(value)}
            aria-label="Profile description"
            placeholder="Description (optional)"
            maxLength={200}
          />
          <div className="flex gap-2 justify-end mt-2">
            <Button
              variant="secondary"
              size="md"
              onClick={handleCancelEdit}
              aria-label="Cancel editing profile"
            >
              Cancel
            </Button>
            <Button
              variant="primary"
              size="md"
              onClick={handleSaveEdit}
              aria-label="Save profile changes"
            >
              Save
            </Button>
          </div>
        </div>
      </Modal>

      {/* Delete Confirmation Modal */}
      <Modal
        open={deleteModalOpen}
        onClose={() => setDeleteModalOpen(false)}
        title="Delete Profile"
      >
        <div className="flex flex-col gap-md">
          <p className="text-slate-300">
            Are you sure you want to delete the profile{' '}
            <strong className="text-white">{selectedProfile?.name}</strong>?
            This action cannot be undone.
          </p>
          <div className="flex gap-2 justify-end mt-2">
            <Button
              variant="secondary"
              size="md"
              onClick={() => setDeleteModalOpen(false)}
              aria-label="Cancel deleting profile"
            >
              Cancel
            </Button>
            <Button
              variant="danger"
              size="md"
              onClick={handleConfirmDelete}
              aria-label="Confirm delete profile"
            >
              Delete
            </Button>
          </div>
        </div>
      </Modal>
    </div>
  );
};

export default ProfilesPage;
