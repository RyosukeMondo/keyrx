import React, { useState, useCallback } from 'react';
import { Card } from './Card';
import { Button } from './Button';
import { Tooltip } from './Tooltip';
import { InlineEdit } from './InlineEdit';
import {
  Check,
  AlertTriangle,
  CheckCircle2,
  FileCode,
  Copy,
  CheckCheck,
} from 'lucide-react';
import { useProfileValidation } from '../hooks/useProfileValidation';
import { truncatePath } from '../utils/pathUtils';

export interface ProfileCardProps {
  name: string;
  description?: string;
  isActive: boolean;
  isActivating?: boolean;
  lastModified?: string;
  rhaiPath?: string;
  fileExists?: boolean;
  onActivate: () => void;
  onDelete: () => void;
  onPathClick?: () => void;
  onUpdateName?: (newName: string) => void;
  onUpdateDescription?: (newDescription: string) => void;
}

/**
 * ProfileCard Component
 *
 * Displays a single profile in a card format with:
 * - Inline-editable profile name and description (click to edit)
 * - Active state indicator (green checkmark + "ACTIVE" badge)
 * - Action buttons: Activate, Delete
 * - Last modified timestamp
 *
 * Used in ProfilesPage grid layout
 */
export const ProfileCard = React.memo<ProfileCardProps>(
  ({
    name,
    description,
    isActive,
    isActivating = false,
    lastModified,
    rhaiPath,
    fileExists = true,
    onActivate,
    onDelete,
    onPathClick,
    onUpdateName,
    onUpdateDescription,
  }) => {
    // Fetch validation status for this profile
    const { data: validationResult, isLoading: isValidating } =
      useProfileValidation(name);

    // Determine if profile is valid
    const isValid = validationResult?.valid ?? true; // Default to valid if not yet loaded
    const validationErrors = validationResult?.errors ?? [];
    const firstError = validationErrors[0];

    // State for copy-to-clipboard feedback
    const [copied, setCopied] = useState(false);

    // Handler to copy error to clipboard
    const handleCopyError = useCallback(async () => {
      if (!firstError) return;

      const errorText = `Configuration Error - Line ${firstError.line}: ${firstError.message}`;
      try {
        await navigator.clipboard.writeText(errorText);
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
      } catch (err) {
        console.error('Failed to copy error:', err);
      }
    }, [firstError]);

    // Format Rhai path for display
    const displayPath = rhaiPath ? truncatePath(rhaiPath, 40) : null;

    return (
      <Card
        variant="default"
        padding="md"
        className={`relative ${
          isActive
            ? 'border-l-4 border-l-blue-500 bg-gradient-to-r from-blue-500/10 to-transparent'
            : ''
        }`}
        data-profile={name}
      >
        {/* Active Badge */}
        {isActive && (
          <div className="absolute top-2 right-2 flex items-center gap-1.5 bg-blue-500 text-white px-3 py-1.5 rounded-md text-sm font-bold shadow-lg">
            <Check size={16} aria-hidden="true" />
            <span>ACTIVE</span>
          </div>
        )}

        {/* Profile Name - Inline Editable */}
        <div className="flex items-start gap-2 mb-2">
          {isActive && (
            <Check
              size={20}
              className="text-blue-500 flex-shrink-0 mt-1"
              aria-label="Active profile indicator"
            />
          )}
          <div className="flex-1">
            <InlineEdit
              value={name}
              onSave={onUpdateName || (() => {})}
              className="text-lg font-semibold text-slate-100"
              placeholder="Profile name"
              maxLength={50}
              disabled={!onUpdateName}
              ariaLabel={`Edit profile name: ${name}`}
            />
          </div>
        </div>

        {/* Description - Inline Editable */}
        <div className="mb-3">
          <InlineEdit
            value={description || ''}
            onSave={onUpdateDescription || (() => {})}
            className="text-sm text-slate-400 line-clamp-2"
            placeholder="Add a description (click to edit)"
            maxLength={200}
            multiline={true}
            disabled={!onUpdateDescription}
            ariaLabel={`Edit profile description: ${name}`}
          />
        </div>

        {/* Rhai File Path */}
        {rhaiPath && (
          <div className="mb-3">
            <Tooltip content={rhaiPath} position="top">
              <button
                onClick={onPathClick}
                className="inline-flex items-center gap-1.5 text-xs text-slate-400 hover:text-blue-400 transition-colors group max-w-full"
                aria-label={`Open configuration file: ${rhaiPath}`}
              >
                <FileCode
                  size={14}
                  className="flex-shrink-0 group-hover:text-blue-400"
                  aria-hidden="true"
                />
                <span className="truncate font-mono">{displayPath}</span>
                {!fileExists && (
                  <AlertTriangle
                    size={14}
                    className="flex-shrink-0 text-yellow-500"
                    aria-label="File not found"
                  />
                )}
              </button>
            </Tooltip>
          </div>
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
                  firstError ? (
                    <div className="space-y-1">
                      <div className="text-yellow-300 font-bold text-sm">
                        Configuration Error:
                      </div>
                      <div className="text-white">
                        <span className="text-yellow-400 font-semibold">
                          Line {firstError.line}:
                        </span>{' '}
                        {firstError.message}
                      </div>
                      <div className="text-xs text-slate-400 mt-2">
                        {copied ? '✓ Copied!' : 'Click to copy error'}
                      </div>
                    </div>
                  ) : (
                    'Invalid configuration'
                  )
                }
                position="top"
              >
                <button
                  onClick={handleCopyError}
                  className="inline-flex items-center gap-2 bg-yellow-500/20 text-yellow-200 border-2 border-yellow-500/60 px-3 py-1.5 rounded-md text-sm font-semibold cursor-pointer shadow-sm hover:bg-yellow-500/30 transition-colors"
                  aria-label={`Copy error: Line ${firstError?.line}: ${firstError?.message}`}
                >
                  <AlertTriangle
                    size={16}
                    className="flex-shrink-0"
                    aria-hidden="true"
                  />
                  <span>Invalid Configuration</span>
                  {copied ? (
                    <CheckCheck
                      size={14}
                      className="text-green-400"
                      aria-hidden="true"
                    />
                  ) : (
                    <Copy size={14} className="opacity-50" aria-hidden="true" />
                  )}
                </button>
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
              disabled={!isValid || isActivating}
              aria-label={
                isActivating
                  ? 'Activating profile...'
                  : !isValid
                    ? `Cannot activate invalid profile ${name}`
                    : `Activate profile ${name}`
              }
            >
              {isActivating ? 'Activating...' : 'Activate'}
            </Button>
          )}
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
