import React from 'react';
import { ChevronDown, ChevronRight } from 'lucide-react';
import type { PaletteKey } from '../KeyPalette';
import { KeyPaletteItem } from '../KeyPaletteItem';

/**
 * Props for KeyCategorySection component
 */
export interface KeyCategorySectionProps {
  /** Category title displayed in header */
  title: string;
  /** Icon element to display next to title */
  icon?: React.ReactNode;
  /** Array of keys to render in the section */
  keys: PaletteKey[];
  /** Callback when a key is selected */
  onKeySelect: (key: PaletteKey) => void;
  /** Currently selected key ID */
  selectedKeyId?: string | null;
  /** Array of favorite key IDs */
  favoriteKeyIds?: string[];
  /** Callback to toggle favorite status */
  onToggleFavorite?: (keyId: string) => void;
  /** Whether the section can be collapsed */
  collapsible?: boolean;
  /** Initial collapsed state (only used if collapsible) */
  defaultCollapsed?: boolean;
  /** View mode for rendering keys */
  viewMode?: 'grid' | 'list';
  /** Whether to show favorite star buttons */
  showFavorites?: boolean;
}

/**
 * KeyCategorySection - Reusable category renderer for key palette
 *
 * Renders a titled section with a grid or list of keys. Supports:
 * - Grid and list view modes
 * - Collapsible sections with expand/collapse animation
 * - Favorite toggling per key
 * - Keyboard navigation support
 *
 * @example
 * ```tsx
 * <KeyCategorySection
 *   title="Modifiers"
 *   icon={<Command className="w-4 h-4" />}
 *   keys={modifierKeys}
 *   onKeySelect={(key) => handleSelect(key)}
 *   favoriteKeyIds={favorites}
 *   onToggleFavorite={(id) => toggleFavorite(id)}
 *   collapsible
 * />
 * ```
 */
export function KeyCategorySection({
  title,
  icon,
  keys,
  onKeySelect,
  selectedKeyId,
  favoriteKeyIds = [],
  onToggleFavorite,
  collapsible = false,
  defaultCollapsed = false,
  viewMode = 'grid',
  showFavorites = true,
}: KeyCategorySectionProps) {
  const [isCollapsed, setIsCollapsed] = React.useState(defaultCollapsed);

  const toggleCollapsed = React.useCallback(() => {
    setIsCollapsed((prev) => !prev);
  }, []);

  const isFavorite = React.useCallback(
    (keyId: string) => favoriteKeyIds.includes(keyId),
    [favoriteKeyIds]
  );

  // Early return if no keys
  if (keys.length === 0) {
    return null;
  }

  return (
    <div className="mb-4">
      {/* Header */}
      <div className="flex items-center gap-2 mb-2">
        {collapsible ? (
          <button
            onClick={toggleCollapsed}
            className="flex items-center gap-2 text-sm font-semibold text-slate-300 hover:text-slate-100 transition-colors"
            aria-expanded={!isCollapsed}
            aria-label={`${
              isCollapsed ? 'Expand' : 'Collapse'
            } ${title} section`}
          >
            {isCollapsed ? (
              <ChevronRight className="w-4 h-4" />
            ) : (
              <ChevronDown className="w-4 h-4" />
            )}
            {icon && <span className="flex-shrink-0">{icon}</span>}
            <h4>{title}</h4>
            <span className="text-xs text-slate-500 font-normal">
              ({keys.length})
            </span>
          </button>
        ) : (
          <div className="flex items-center gap-2">
            {icon && <span className="flex-shrink-0">{icon}</span>}
            <h4 className="text-sm font-semibold text-slate-300">{title}</h4>
            <span className="text-xs text-slate-500">({keys.length})</span>
          </div>
        )}
      </div>

      {/* Content */}
      {!isCollapsed && (
        <div
          className={`p-3 bg-slate-800/50 rounded-lg ${
            viewMode === 'grid'
              ? 'grid grid-cols-8 gap-2'
              : 'flex flex-col gap-2'
          }`}
          role="group"
          aria-label={`${title} keys`}
        >
          {keys.map((key) => (
            <KeyPaletteItem
              key={key.id}
              keyItem={key}
              isSelected={selectedKeyId === key.id}
              isFavorite={isFavorite(key.id)}
              showStar={showFavorites}
              viewMode={viewMode}
              onClick={() => onKeySelect(key)}
              onToggleFavorite={
                showFavorites && onToggleFavorite
                  ? () => onToggleFavorite(key.id)
                  : undefined
              }
            />
          ))}
        </div>
      )}
    </div>
  );
}
