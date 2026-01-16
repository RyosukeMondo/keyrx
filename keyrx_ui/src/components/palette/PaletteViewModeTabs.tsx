import React from 'react';
import { Grid3x3, Clock, Star, List } from 'lucide-react';

/**
 * View mode type for the key palette
 */
export type PaletteView = 'basic' | 'recent' | 'favorites' | 'all';

/**
 * Props for PaletteViewModeTabs component
 */
export interface PaletteViewModeTabsProps {
  /** Current active view */
  activeView: PaletteView;
  /** Callback when view changes */
  onChange: (view: PaletteView) => void;
  /** Optional class name for styling */
  className?: string;
}

/**
 * Tab configuration interface
 */
interface TabConfig {
  id: PaletteView;
  label: string;
  icon: React.ReactNode;
  ariaLabel: string;
}

/**
 * PaletteViewModeTabs - Tab navigation for switching between palette views
 *
 * Provides accessible tab buttons to switch between:
 * - Basic: Category-based key browsing
 * - Recent: Recently used keys
 * - Favorites: Starred/favorited keys
 * - All: Complete key list
 *
 * @example
 * ```tsx
 * <PaletteViewModeTabs
 *   activeView="basic"
 *   onChange={(view) => setActiveView(view)}
 * />
 * ```
 */
export function PaletteViewModeTabs({
  activeView,
  onChange,
  className = '',
}: PaletteViewModeTabsProps) {
  const tabs: TabConfig[] = [
    {
      id: 'basic',
      label: 'Basic',
      icon: <Grid3x3 className="w-4 h-4" />,
      ariaLabel: 'Basic category view',
    },
    {
      id: 'recent',
      label: 'Recent',
      icon: <Clock className="w-4 h-4" />,
      ariaLabel: 'Recent keys view',
    },
    {
      id: 'favorites',
      label: 'Favorites',
      icon: <Star className="w-4 h-4" />,
      ariaLabel: 'Favorite keys view',
    },
    {
      id: 'all',
      label: 'All',
      icon: <List className="w-4 h-4" />,
      ariaLabel: 'All keys view',
    },
  ];

  const handleTabClick = (view: PaletteView) => {
    onChange(view);
  };

  const handleKeyDown = (e: React.KeyboardEvent, view: PaletteView) => {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      onChange(view);
    }
  };

  return (
    <div
      className={`flex gap-1 border-b border-slate-700 ${className}`}
      role="tablist"
      aria-label="Palette view modes"
    >
      {tabs.map((tab) => {
        const isActive = activeView === tab.id;

        return (
          <button
            key={tab.id}
            role="tab"
            aria-selected={isActive}
            aria-label={tab.ariaLabel}
            onClick={() => handleTabClick(tab.id)}
            onKeyDown={(e) => handleKeyDown(e, tab.id)}
            tabIndex={isActive ? 0 : -1}
            className={`
              px-3 py-2 text-sm font-medium
              transition-colors
              whitespace-nowrap
              flex items-center gap-2
              ${
                isActive
                  ? 'text-primary-400 border-b-2 border-primary-400'
                  : 'text-slate-400 hover:text-slate-300'
              }
            `}
          >
            {tab.icon}
            <span>{tab.label}</span>
          </button>
        );
      })}
    </div>
  );
}
