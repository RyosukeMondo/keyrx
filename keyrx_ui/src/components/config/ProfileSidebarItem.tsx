import React from 'react';
import { Trash2, Play } from 'lucide-react';
import { cn } from '@/utils/cn';

export interface ProfileSidebarItemProps {
  name: string;
  isActive: boolean;
  isSelected: boolean;
  lastModified: string;
  onSelect: () => void;
  onActivate: () => void;
  onDelete: () => void;
}

/**
 * Compact profile row for the sidebar list.
 *
 * Displays name (truncated), last-modified timestamp, an active-profile
 * indicator dot, and hover-visible action buttons (activate / delete).
 */
export const ProfileSidebarItem: React.FC<ProfileSidebarItemProps> = ({
  name,
  isActive,
  isSelected,
  lastModified,
  onSelect,
  onActivate,
  onDelete,
}) => {
  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      onSelect();
    }
  };

  const stopPropagation = (
    e: React.MouseEvent,
    handler: () => void,
  ) => {
    e.stopPropagation();
    handler();
  };

  return (
    <div
      role="button"
      tabIndex={0}
      aria-label={`Select profile ${name}`}
      aria-current={isSelected ? 'true' : undefined}
      onClick={onSelect}
      onKeyDown={handleKeyDown}
      className={cn(
        'group relative flex items-center gap-2 px-3 py-2 cursor-pointer',
        'rounded-md transition-colors duration-100',
        'focus:outline focus:outline-2 focus:outline-primary-500 focus:outline-offset-[-2px]',
        isSelected
          ? 'bg-primary-600/20 border-l-2 border-primary-500'
          : 'border-l-2 border-transparent hover:bg-slate-700',
      )}
    >
      {/* Active dot */}
      {isActive && (
        <span
          className="shrink-0 w-2 h-2 rounded-full bg-green-500"
          aria-label="Active profile"
        />
      )}

      {/* Name + timestamp */}
      <div className="min-w-0 flex-1">
        <p className="text-sm font-medium text-slate-100 truncate">{name}</p>
        <p className="text-xs text-slate-400 truncate">{lastModified}</p>
      </div>

      {/* Action buttons -- visible on hover, hidden for the active profile */}
      {!isActive && (
        <div className="shrink-0 flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
          <button
            type="button"
            aria-label={`Activate profile ${name}`}
            title="Activate"
            onClick={(e) => stopPropagation(e, onActivate)}
            className="p-1 rounded text-slate-400 hover:text-green-400 hover:bg-slate-600 transition-colors"
          >
            <Play size={14} />
          </button>
          <button
            type="button"
            aria-label={`Delete profile ${name}`}
            title="Delete"
            onClick={(e) => stopPropagation(e, onDelete)}
            className="p-1 rounded text-slate-400 hover:text-red-400 hover:bg-slate-600 transition-colors"
          >
            <Trash2 size={14} />
          </button>
        </div>
      )}
    </div>
  );
};

ProfileSidebarItem.displayName = 'ProfileSidebarItem';
