import React from 'react';
import {
  Keyboard,
  Command,
  Music,
  Wrench,
  Layers,
  Sparkles,
  Edit3,
  Star,
} from 'lucide-react';
import type { PaletteKey } from './KeyPalette';

/**
 * Props for KeyPaletteItem component
 */
interface KeyPaletteItemProps {
  keyItem: PaletteKey;
  isSelected?: boolean;
  isFavorite?: boolean;
  showStar?: boolean;
  viewMode?: 'grid' | 'list';
  onClick: () => void;
  onToggleFavorite?: () => void;
}

/**
 * Get category-specific icon component
 */
function getCategoryIcon(category: PaletteKey['category']) {
  const iconProps = { className: 'w-3.5 h-3.5' };

  switch (category) {
    case 'basic':
      return <Keyboard {...iconProps} />;
    case 'modifiers':
      return <Command {...iconProps} />;
    case 'media':
      return <Music {...iconProps} />;
    case 'macro':
      return <Wrench {...iconProps} />;
    case 'layers':
      return <Layers {...iconProps} />;
    case 'special':
      return <Sparkles {...iconProps} />;
    case 'any':
      return <Edit3 {...iconProps} />;
    default:
      return <Keyboard {...iconProps} />;
  }
}

/**
 * Get category-specific color classes
 */
function getCategoryColors(category: PaletteKey['category']) {
  switch (category) {
    case 'basic':
      return {
        border: 'border-blue-500/50',
        bg: 'bg-blue-500/10',
        icon: 'text-blue-400',
        hover: 'hover:border-blue-400',
      };
    case 'modifiers':
      return {
        border: 'border-cyan-500/50',
        bg: 'bg-cyan-500/10',
        icon: 'text-cyan-400',
        hover: 'hover:border-cyan-400',
      };
    case 'media':
      return {
        border: 'border-pink-500/50',
        bg: 'bg-pink-500/10',
        icon: 'text-pink-400',
        hover: 'hover:border-pink-400',
      };
    case 'macro':
      return {
        border: 'border-green-500/50',
        bg: 'bg-green-500/10',
        icon: 'text-green-400',
        hover: 'hover:border-green-400',
      };
    case 'layers':
      return {
        border: 'border-yellow-500/50',
        bg: 'bg-yellow-500/10',
        icon: 'text-yellow-400',
        hover: 'hover:border-yellow-400',
      };
    case 'special':
      return {
        border: 'border-purple-500/50',
        bg: 'bg-purple-500/10',
        icon: 'text-purple-400',
        hover: 'hover:border-purple-400',
      };
    case 'any':
      return {
        border: 'border-slate-500/50',
        bg: 'bg-slate-500/10',
        icon: 'text-slate-400',
        hover: 'hover:border-slate-400',
      };
    default:
      return {
        border: 'border-slate-600',
        bg: 'bg-slate-700',
        icon: 'text-slate-400',
        hover: 'hover:border-slate-500',
      };
  }
}

/**
 * KeyPaletteItem - Individual key button with icon, label, and tooltip
 *
 * Features:
 * - Category-specific icon and color coding
 * - Primary label (key label)
 * - Secondary label (key ID if different)
 * - Hover tooltip with full description
 * - Optional favorite star button
 * - 44px minimum touch target for accessibility
 */
export function KeyPaletteItem({
  keyItem,
  isSelected = false,
  isFavorite = false,
  showStar = true,
  viewMode = 'grid',
  onClick,
  onToggleFavorite,
}: KeyPaletteItemProps) {
  const colors = getCategoryColors(keyItem.category);
  const icon = getCategoryIcon(keyItem.category);

  // Build tooltip content
  const tooltipText = [
    keyItem.description || keyItem.label,
    keyItem.id !== keyItem.label ? `ID: ${keyItem.id}` : null,
    keyItem.subcategory ? `Category: ${keyItem.category} / ${keyItem.subcategory}` : null,
  ]
    .filter(Boolean)
    .join('\n');

  // Grid view: Compact vertical layout
  if (viewMode === 'grid') {
    return (
      <div className="relative group">
        <button
          onClick={onClick}
          className={`
            w-full aspect-square relative flex flex-col items-center justify-center gap-1
            px-2 py-2
            rounded-lg border-2 transition-all duration-200
            hover:brightness-110 hover:-translate-y-0.5 hover:shadow-lg
            ${
              isSelected
                ? 'border-primary-500 bg-primary-500/20 shadow-lg shadow-primary-500/50 scale-105'
                : `${colors.border} ${colors.bg} ${colors.hover}`
            }
          `}
          title={tooltipText}
          aria-label={`Select key ${keyItem.label}`}
        >
          {/* Icon (category indicator) */}
          <div className={`${colors.icon} opacity-70`}>
            {icon}
          </div>

          {/* Key label (main) */}
          <div className="text-sm font-bold text-white font-mono leading-none">
            {keyItem.label}
          </div>

          {/* Key ID (small, below label, only if different) */}
          {keyItem.id !== keyItem.label && (
            <div className="text-[9px] text-slate-400 font-mono leading-none truncate max-w-full px-1">
              {keyItem.id}
            </div>
          )}
        </button>

        {/* Star button (favorite toggle) */}
        {showStar && onToggleFavorite && (
          <button
            onClick={(e) => {
              e.stopPropagation();
              onToggleFavorite();
            }}
            className="absolute top-1 right-1 opacity-0 group-hover:opacity-100 transition-opacity duration-200 z-10"
            title={isFavorite ? 'Remove from favorites' : 'Add to favorites'}
            aria-label={isFavorite ? 'Remove from favorites' : 'Add to favorites'}
          >
            <Star
              className={`w-3 h-3 ${
                isFavorite
                  ? 'fill-yellow-400 text-yellow-400'
                  : 'text-slate-400 hover:text-yellow-400'
              }`}
            />
          </button>
        )}
      </div>
    );
  }

  // List view: Horizontal layout with description
  return (
    <div className="relative group">
      <button
        onClick={onClick}
        className={`
          w-full relative flex items-center gap-3
          min-h-[56px] px-4 py-3
          rounded-lg border-2 transition-all duration-200
          hover:brightness-110 hover:shadow-lg
          ${
            isSelected
              ? 'border-primary-500 bg-primary-500/20 shadow-lg shadow-primary-500/50'
              : `${colors.border} ${colors.bg} ${colors.hover}`
          }
        `}
        title={tooltipText}
        aria-label={`Select key ${keyItem.label}`}
      >
        {/* Icon (category indicator) */}
        <div className={`${colors.icon} opacity-70 flex-shrink-0`}>
          {icon}
        </div>

        {/* Content (left aligned) */}
        <div className="flex-1 text-left">
          {/* Key label and ID */}
          <div className="flex items-baseline gap-2 mb-1">
            <div className="text-lg font-bold text-white font-mono">
              {keyItem.label}
            </div>
            {keyItem.id !== keyItem.label && (
              <div className="text-xs text-slate-400 font-mono">
                {keyItem.id}
              </div>
            )}
          </div>

          {/* Description (visible in list view) */}
          {keyItem.description && (
            <div className="text-sm text-slate-300">
              {keyItem.description}
            </div>
          )}
        </div>
      </button>

      {/* Star button (favorite toggle) - positioned in top right */}
      {showStar && onToggleFavorite && (
        <button
          onClick={(e) => {
            e.stopPropagation();
            onToggleFavorite();
          }}
          className="absolute top-3 right-3 opacity-0 group-hover:opacity-100 transition-opacity duration-200 z-10"
          title={isFavorite ? 'Remove from favorites' : 'Add to favorites'}
          aria-label={isFavorite ? 'Remove from favorites' : 'Add to favorites'}
        >
          <Star
            className={`w-4 h-4 ${
              isFavorite
                ? 'fill-yellow-400 text-yellow-400'
                : 'text-slate-400 hover:text-yellow-400'
            }`}
          />
        </button>
      )}
    </div>
  );
}
