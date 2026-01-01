import React from 'react';
import { Card } from './Card';
import { Button } from './Button';
import { Check } from 'lucide-react';

export interface ProfileCardProps {
  name: string;
  description?: string;
  isActive: boolean;
  lastModified?: string;
  onActivate: () => void;
  onEdit: () => void;
  onDelete: () => void;
}

/**
 * ProfileCard Component
 *
 * Displays a single profile in a card format with:
 * - Profile name and optional description
 * - Active state indicator (green checkmark + "ACTIVE" badge)
 * - Action buttons: Activate, Edit, Delete
 * - Last modified timestamp
 *
 * Used in ProfilesPage grid layout
 */
export const ProfileCard = React.memo<ProfileCardProps>(
  ({
    name,
    description,
    isActive,
    lastModified,
    onActivate,
    onEdit,
    onDelete,
  }) => {
    return (
      <Card
        variant="default"
        padding="md"
        className={`relative ${isActive ? 'border-green-500 border-2' : ''}`}
      >
        {/* Active Badge */}
        {isActive && (
          <div className="absolute top-2 right-2 flex items-center gap-1 bg-green-500 text-white px-2 py-1 rounded text-xs font-semibold">
            <Check size={14} aria-hidden="true" />
            <span>ACTIVE</span>
          </div>
        )}

        {/* Profile Name */}
        <div className="flex items-start gap-2 mb-2">
          {isActive && (
            <Check
              size={20}
              className="text-green-500 flex-shrink-0 mt-1"
              aria-label="Active profile indicator"
            />
          )}
          <h3 className="text-lg font-semibold text-slate-100">{name}</h3>
        </div>

        {/* Description */}
        {description && (
          <p className="text-sm text-slate-400 mb-3 line-clamp-2">
            {description}
          </p>
        )}

        {/* Last Modified */}
        {lastModified && (
          <p className="text-xs text-slate-500 mb-4">
            Modified: {lastModified}
          </p>
        )}

        {/* Action Buttons */}
        <div className="flex gap-2 flex-wrap">
          {!isActive && (
            <Button
              variant="primary"
              size="sm"
              onClick={onActivate}
              aria-label={`Activate profile ${name}`}
            >
              Activate
            </Button>
          )}
          <Button
            variant="secondary"
            size="sm"
            onClick={onEdit}
            aria-label={`Edit profile ${name}`}
          >
            Edit
          </Button>
          <Button
            variant="danger"
            size="sm"
            onClick={onDelete}
            aria-label={`Delete profile ${name}`}
            disabled={isActive}
          >
            Delete
          </Button>
        </div>
      </Card>
    );
  }
);

ProfileCard.displayName = 'ProfileCard';
