import React from 'react';
import { useNavigate } from 'react-router-dom';
import { Card } from './Card';
import { Button } from './Button';
import { LoadingSkeleton } from './LoadingSkeleton';

interface ActiveProfile {
  name: string;
  layers: number;
  mappings: number;
  modifiedAt: string;
}

interface ActiveProfileCardProps {
  profile?: ActiveProfile;
  loading?: boolean;
  className?: string;
}

/**
 * ActiveProfileCard Component
 *
 * Displays the currently active profile with:
 * - Profile name and icon
 * - Number of layers and key mappings
 * - Last modified timestamp
 * - Edit button to navigate to config page
 *
 * Used on: HomePage (dashboard)
 * Design: From design.md Layout 1 - Active Profile section
 */
export const ActiveProfileCard: React.FC<ActiveProfileCardProps> = ({
  profile,
  loading = false,
  className = '',
}) => {
  const navigate = useNavigate();

  const handleEdit = () => {
    if (profile) {
      navigate('/config');
    }
  };

  if (loading) {
    return (
      <Card className={className}>
        <div className="flex flex-col gap-md">
          <LoadingSkeleton variant="text" width="40%" height="24px" />
          <div className="flex items-center gap-sm">
            <LoadingSkeleton variant="circular" width="32px" height="32px" />
            <LoadingSkeleton variant="text" width="60%" height="28px" />
          </div>
          <div className="flex gap-md">
            <LoadingSkeleton variant="text" width="80px" height="16px" />
            <LoadingSkeleton variant="text" width="120px" height="16px" />
            <LoadingSkeleton variant="text" width="100px" height="16px" />
          </div>
        </div>
      </Card>
    );
  }

  if (!profile) {
    return (
      <Card className={className}>
        <div className="flex flex-col gap-md">
          <h2 className="text-lg font-semibold text-slate-100">
            Active Profile
          </h2>
          <p className="text-sm text-slate-400">
            No profile is currently active. Create or activate a profile to get
            started.
          </p>
          <div>
            <Button
              variant="primary"
              size="md"
              onClick={() => navigate('/profiles')}
              aria-label="Go to profiles page"
            >
              Manage Profiles
            </Button>
          </div>
        </div>
      </Card>
    );
  }

  return (
    <Card className={className}>
      <div className="flex flex-col gap-md">
        <div className="flex items-center justify-between">
          <h2 className="text-lg font-semibold text-slate-100">
            Active Profile
          </h2>
          <Button
            variant="secondary"
            size="sm"
            onClick={handleEdit}
            aria-label={`Edit profile ${profile.name}`}
          >
            Edit
          </Button>
        </div>

        <div className="flex items-center gap-sm">
          <span className="text-2xl" role="img" aria-label="Profile icon">
            🎮
          </span>
          <h3 className="text-xl font-semibold text-slate-100">
            {profile.name}
          </h3>
        </div>

        <div className="flex flex-wrap items-center gap-md text-sm text-slate-400">
          <span>• {profile.layers} Layers</span>
          <span>• Modified: {profile.modifiedAt}</span>
          <span>• {profile.mappings} key mappings</span>
        </div>
      </div>
    </Card>
  );
};
