import React from 'react';
import { Star, Clock } from 'lucide-react';
import { PaletteHeader } from './palette/PaletteHeader';
import { QuickAccessSection } from './palette/QuickAccessSection';
import { PaletteSearchInput } from './palette/PaletteSearchInput';
import { CustomKeycodeInput } from './palette/CustomKeycodeInput';
import { CategoryTabs } from './palette/CategoryTabs';
import { SearchResultsList } from './palette/SearchResultsList';
import { KeyCaptureModal } from './palette/KeyCaptureModal';
import { Card } from './Card';
import { KEY_DEFINITIONS } from '../data/keyDefinitions';
import { KeyPaletteItem } from './KeyPaletteItem';
import { useRecentKeys } from '../hooks/useRecentKeys';
import { useFavoriteKeys } from '../hooks/useFavoriteKeys';
import { usePaletteSearch } from '../hooks/usePaletteSearch';
import { useKeyPaletteHandlers } from '../hooks/useKeyPaletteHandlers';
import { usePhysicalKeyCapture } from '../hooks/usePhysicalKeyCapture';
import {
  BASIC_KEYS,
  MODIFIER_KEYS,
  LAYER_KEYS,
  SPECIAL_KEYS,
} from '../data/paletteKeys';
import {
  loadViewMode,
  findKeyById,
  type ViewMode,
} from '../utils/paletteHelpers.tsx';

/**
 * Key Palette - Shows available keys/modifiers/layers for assignment
 * Based on VIA-style categories: Basic, Modifiers, Media, Macro, Layers, Special, Any
 */

export interface PaletteKey {
  id: string;
  label: string;
  category:
    | 'basic'
    | 'modifiers'
    | 'media'
    | 'macro'
    | 'layers'
    | 'special'
    | 'any';
  subcategory?: string;
  description?: string;
}

interface KeyPaletteProps {
  onKeySelect: (key: PaletteKey) => void;
  selectedKey?: PaletteKey | null;
  /** Compact mode for embedding in modals - reduced height, no header/recent/favorites */
  compact?: boolean;
}

export function KeyPalette({
  onKeySelect,
  selectedKey,
  compact = false,
}: KeyPaletteProps) {
  const [activeCategory, setActiveCategory] =
    React.useState<PaletteKey['category']>('basic');
  const [activeSubcategory, setActiveSubcategory] = React.useState<
    string | null
  >(null);
  const [selectedSearchIndex, setSelectedSearchIndex] = React.useState(0);
  const searchInputRef = React.useRef<HTMLInputElement>(null);

  // View mode state
  const [viewMode, setViewMode] = React.useState<ViewMode>(() =>
    loadViewMode()
  );

  // Use extracted hooks
  const { recentKeys: recentKeyIds, addRecentKey } = useRecentKeys();
  const {
    favoriteKeys: favoriteKeyIds,
    toggleFavorite,
    isFavorite,
  } = useFavoriteKeys();
  const {
    query: searchQuery,
    setQuery: setSearchQuery,
    results: searchResults,
  } = usePaletteSearch(KEY_DEFINITIONS);

  // Key palette handlers and state
  const {
    customKeycode,
    customValidation,
    isCapturingKey,
    capturedKey,
    setCapturedKey,
    handleKeySelect,
    handleCustomKeycodeChange,
    handleApplyCustomKeycode,
    startKeyCapture,
    cancelKeyCapture,
    confirmCapturedKey,
    toggleViewMode: toggleViewModeCallback,
  } = useKeyPaletteHandlers({ onKeySelect, addRecentKey });

  // Toggle view mode wrapper
  const toggleViewMode = React.useCallback(() => {
    setViewMode((prev) => toggleViewModeCallback(prev));
  }, [toggleViewModeCallback]);

  // Handle physical key press during capture mode
  usePhysicalKeyCapture({
    isCapturingKey,
    onCapturedKey: setCapturedKey,
    onCancel: cancelKeyCapture,
  });

  // Get recent and favorite key objects
  const recentKeys = React.useMemo(() => {
    return recentKeyIds
      .map((id) => findKeyById(id))
      .filter((k): k is PaletteKey => k !== null);
  }, [recentKeyIds]);

  const favoriteKeys = React.useMemo(() => {
    return favoriteKeyIds
      .map((id) => findKeyById(id))
      .filter((k): k is PaletteKey => k !== null);
  }, [favoriteKeyIds]);

  const categories = [
    { id: 'basic' as const, label: 'Basic', keys: BASIC_KEYS, icon: '⌨️' },
    {
      id: 'modifiers' as const,
      label: 'Modifiers',
      keys: MODIFIER_KEYS,
      icon: '⌥',
    },
    {
      id: 'special' as const,
      label: 'Special',
      keys: SPECIAL_KEYS,
      icon: '⭐',
    },
    { id: 'any' as const, label: 'Any', keys: [], icon: '✏️' },
  ];

  // Reset selected index when search changes
  React.useEffect(() => {
    setSelectedSearchIndex(0);
  }, [searchQuery]);

  const activeCategoryData = categories.find((c) => c.id === activeCategory);
  let activeKeys = activeCategoryData?.keys || [];

  // If searching, use search results instead
  const isSearching = searchQuery.trim().length > 0;

  // Filter by subcategory if one is active
  if (
    activeSubcategory &&
    (activeCategory === 'basic' || activeCategory === 'layers') &&
    !isSearching
  ) {
    activeKeys = activeKeys.filter((k) => k.subcategory === activeSubcategory);
  }

  // Get unique subcategories for Basic and Layers categories
  const subcategories =
    activeCategory === 'basic'
      ? (Array.from(
          new Set(BASIC_KEYS.map((k) => k.subcategory).filter(Boolean))
        ) as string[])
      : activeCategory === 'layers'
        ? (Array.from(
            new Set(LAYER_KEYS.map((k) => k.subcategory).filter(Boolean))
          ) as string[])
        : [];

  // Handle keyboard navigation in search results
  const handleSearchKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (searchResults.length === 0) return;

    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        setSelectedSearchIndex((prev) =>
          Math.min(prev + 1, searchResults.length - 1)
        );
        break;
      case 'ArrowUp':
        e.preventDefault();
        setSelectedSearchIndex((prev) => Math.max(prev - 1, 0));
        break;
      case 'Enter':
        e.preventDefault();
        if (searchResults[selectedSearchIndex]) {
          const match = searchResults[selectedSearchIndex];
          handleKeySelect({
            id: match.key.id,
            label: match.key.label,
            category: match.key.category,
            subcategory: match.key.subcategory,
            description: match.key.description,
          });
          setSearchQuery('');
        }
        break;
      case 'Escape':
        e.preventDefault();
        setSearchQuery('');
        searchInputRef.current?.blur();
        break;
    }
  };

  // Drag handlers removed - using click-only interaction now

  // Render a key item with star button using KeyPaletteItem component
  const renderKeyItem = (
    key: PaletteKey,
    onClick: () => void,
    showStar: boolean = true
  ) => {
    const favorite = isFavorite(key.id);

    return (
      <KeyPaletteItem
        key={key.id}
        keyItem={key}
        isSelected={selectedKey?.id === key.id}
        isFavorite={favorite}
        showStar={showStar}
        viewMode={viewMode}
        onClick={onClick}
        onToggleFavorite={showStar ? () => toggleFavorite(key.id) : undefined}
      />
    );
  };

  return (
    <Card className={`flex flex-col ${compact ? 'h-full p-2' : 'h-full'}`}>
      {/* Header with title, capture button, and view toggle - hidden in compact mode */}
      {!compact && (
        <PaletteHeader
          viewMode={viewMode}
          onToggleView={toggleViewMode}
          onCaptureKey={startKeyCapture}
        />
      )}

      {/* Favorites Section - hidden in compact mode */}
      {!compact && (
        <QuickAccessSection
          title="Favorites"
          icon={<Star className="w-4 h-4 text-yellow-400 fill-yellow-400" />}
          keys={favoriteKeys}
          viewMode={viewMode}
          renderKeyItem={renderKeyItem}
          onKeySelect={handleKeySelect}
        />
      )}

      {/* Recent Keys Section - hidden in compact mode */}
      {!compact && (
        <QuickAccessSection
          title="Recent"
          icon={<Clock className="w-4 h-4 text-slate-400" />}
          keys={recentKeys}
          viewMode={viewMode}
          renderKeyItem={renderKeyItem}
          onKeySelect={handleKeySelect}
        />
      )}

      {/* Empty state when no favorites or recent - hidden in compact mode */}
      {!compact &&
        favoriteKeys.length === 0 &&
        recentKeys.length === 0 &&
        !searchQuery && (
          <div className="mb-4 p-3 bg-slate-800/30 rounded-lg border border-slate-700/50">
            <p className="text-xs text-slate-500 text-center">
              Star keys to add favorites. Recent keys will appear automatically.
            </p>
          </div>
        )}

      {/* Search Input */}
      <PaletteSearchInput
        searchQuery={searchQuery}
        onSearchChange={setSearchQuery}
        onKeyDown={handleSearchKeyDown}
        searchResultCount={searchResults.length}
        compact={compact}
        inputRef={searchInputRef}
      />

      {/* Category Tabs */}
      <CategoryTabs
        categories={categories}
        activeCategory={activeCategory}
        onCategoryChange={(catId) => {
          setActiveCategory(catId as PaletteKey['category']);
          setActiveSubcategory(null);
        }}
        subcategories={subcategories}
        activeSubcategory={activeSubcategory}
        onSubcategoryChange={setActiveSubcategory}
        compact={compact}
        isSearching={isSearching}
      />

      {/* Key Grid - Keyboard keycap style */}
      <div className="flex-1 overflow-y-auto">
        {isSearching ? (
          <SearchResultsList
            searchQuery={searchQuery}
            searchResults={searchResults}
            selectedSearchIndex={selectedSearchIndex}
            onSelectResult={(key) => {
              handleKeySelect(key);
              setSearchQuery('');
            }}
          />
        ) : activeCategory === 'any' ? (
          // Custom keycode input (Any category)
          <CustomKeycodeInput
            customKeycode={customKeycode}
            customValidation={customValidation}
            onKeycodeChange={handleCustomKeycodeChange}
            onApplyKeycode={handleApplyCustomKeycode}
          />
        ) : activeKeys.length === 0 ? (
          <div className="p-4 text-center text-slate-400">
            <p>No keys in this category yet</p>
          </div>
        ) : (
          <div
            className={`bg-slate-800/50 rounded-lg ${compact ? 'p-2' : 'p-4'} ${
              viewMode === 'grid'
                ? compact
                  ? 'grid grid-cols-12 gap-1'
                  : 'grid grid-cols-8 gap-2'
                : 'flex flex-col gap-2'
            }`}
          >
            {activeKeys.map((key) =>
              renderKeyItem(key, () => handleKeySelect(key), !compact)
            )}
          </div>
        )}
      </div>

      {/* Hint - hidden in compact mode */}
      {!compact && (
        <p className="text-xs text-slate-500 mt-4">
          {isSearching
            ? 'Use ↑↓ arrows to navigate, Enter to select, Esc to clear'
            : 'Search for keys or browse by category. Click to select.'}
        </p>
      )}

      {/* Key Capture Modal */}
      <KeyCaptureModal
        isCapturingKey={isCapturingKey}
        capturedKey={capturedKey}
        onCancel={cancelKeyCapture}
        onConfirm={confirmCapturedKey}
      />
    </Card>
  );
}
