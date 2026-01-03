import React from 'react';
import { Card } from './Card';
import { Button } from './Button';
import { Tooltip } from './Tooltip';
import { Check, AlertTriangle, CheckCircle2 } from 'lucide-react';
import { useProfileValidation } from '../hooks/useProfileValidation';

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
    // Fetch validation status for this profile
    const { data: validationResult, isLoading: isValidating } = useProfileValidation(name);

    // Determine if profile is valid
    const isValid = validationResult?.valid ?? true; // Default to valid if not yet loaded
    const validationErrors = validationResult?.errors ?? [];
    const firstError = validationErrors[0];

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

        {/* Validation Status Badge */}
        {!isValidating && (
          <div className="mb-3">
            {isValid ? (
              <div className="inline-flex items-center gap-1 bg-green-900/30 text-green-400 border border-green-700 px-2 py-1 rounded text-xs font-medium">
                <CheckCircle2 size={14} aria-hidden="true" />
                <span>Valid</span>
              </div>
            ) : (
              <Tooltip
                content={
                  firstError
                    ? `Line ${firstError.line}: ${firstError.message}`
                    : 'Invalid configuration'
                }
                position="top"
              >
                <div className="inline-flex items-center gap-1 bg-yellow-900/30 text-yellow-400 border border-yellow-700 px-2 py-1 rounded text-xs font-medium cursor-help">
                  <AlertTriangle size={14} aria-hidden="true" />
                  <span>Invalid Configuration</span>
                </div>
              </Tooltip>
            )}
          </div>
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
              disabled={!isValid}
              aria-label={
                !isValid
                  ? `Cannot activate invalid profile ${name}`
                  : `Activate profile ${name}`
              }
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
