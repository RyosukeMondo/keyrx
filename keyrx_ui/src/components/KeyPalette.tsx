import React from 'react';
import {
  Search,
  X,
  Star,
  Clock,
  Check,
  AlertCircle,
  HelpCircle,
  Grid3x3,
  List,
  Keyboard,
} from 'lucide-react';
import { Card } from './Card';
import { KEY_DEFINITIONS, KeyDefinition } from '../data/keyDefinitions';
import { KeyPaletteItem } from './KeyPaletteItem';
import { useRecentKeys } from '../hooks/useRecentKeys';
import { useFavoriteKeys } from '../hooks/useFavoriteKeys';
import { usePaletteSearch } from '../hooks/usePaletteSearch';
import { KeyCategorySection } from './palette/KeyCategorySection';
import { PaletteViewModeTabs, PaletteView } from './palette/PaletteViewModeTabs';
import {
  BASIC_KEYS,
  MODIFIER_KEYS,
  MEDIA_KEYS,
  MACRO_KEYS,
  LAYER_KEYS,
  SPECIAL_KEYS,
} from '../data/paletteKeys';
import {
  highlightMatches,
  loadViewMode,
  saveViewMode,
  findKeyById,
  mapDomCodeToKeyId,
  validateCustomKeycode,
  type ViewMode,
  type ValidationResult,
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
  const { favoriteKeys: favoriteKeyIds, toggleFavorite, isFavorite } =
    useFavoriteKeys();
  const { query: searchQuery, setQuery: setSearchQuery, results: searchResults } =
    usePaletteSearch(KEY_DEFINITIONS);

  // Custom keycode input state (for "Any" category)
  const [customKeycode, setCustomKeycode] = React.useState('');
  const [customValidation, setCustomValidation] =
    React.useState<ValidationResult>({ valid: false });

  // Physical key capture state
  const [isCapturingKey, setIsCapturingKey] = React.useState(false);
  const [capturedKey, setCapturedKey] = React.useState<PaletteKey | null>(null);


  // Toggle view mode
  const toggleViewMode = React.useCallback(() => {
    setViewMode((prev) => {
      const newMode = prev === 'grid' ? 'list' : 'grid';
      saveViewMode(newMode);
      return newMode;
    });
  }, []);

  // Handle key selection with recent tracking
  const handleKeySelect = React.useCallback(
    (key: PaletteKey) => {
      addRecentKey(key.id);
      onKeySelect(key);
    },
    [addRecentKey, onKeySelect]
  );

  // Handle custom keycode input change
  const handleCustomKeycodeChange = React.useCallback((value: string) => {
    setCustomKeycode(value);
    const validation = validateCustomKeycode(value);
    setCustomValidation(validation);
  }, []);

  // Apply custom keycode
  const handleApplyCustomKeycode = React.useCallback(() => {
    if (
      customValidation.valid &&
      customValidation.normalizedId &&
      customValidation.label
    ) {
      const customKey: PaletteKey = {
        id: customValidation.normalizedId,
        label: customValidation.label,
        category: 'any',
        description: `Custom keycode: ${customValidation.normalizedId}`,
      };
      handleKeySelect(customKey);
      setCustomKeycode('');
      setCustomValidation({ valid: false });
    }
  }, [customValidation, handleKeySelect]);

  // Start physical key capture mode
  const startKeyCapture = React.useCallback(() => {
    setIsCapturingKey(true);
    setCapturedKey(null);
  }, []);

  // Cancel key capture mode
  const cancelKeyCapture = React.useCallback(() => {
    setIsCapturingKey(false);
    setCapturedKey(null);
  }, []);

  // Confirm captured key
  const confirmCapturedKey = React.useCallback(() => {
    if (capturedKey) {
      handleKeySelect(capturedKey);
      setIsCapturingKey(false);
      setCapturedKey(null);
    }
  }, [capturedKey, handleKeySelect]);

  // Handle physical key press during capture mode
  React.useEffect(() => {
    if (!isCapturingKey) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      e.preventDefault();
      e.stopPropagation();

      // Allow Escape to cancel
      if (e.code === 'Escape') {
        cancelKeyCapture();
        return;
      }

      // Map the DOM code to our key ID
      const mappedKey = mapDomCodeToKeyId(e.code);
      if (mappedKey) {
        setCapturedKey(mappedKey);
      } else {
        // Unknown key - show error state
        console.warn('Unknown key code:', e.code);
      }
    };

    // Add listener at document level to capture all keys
    document.addEventListener('keydown', handleKeyDown, true);

    return () => {
      document.removeEventListener('keydown', handleKeyDown, true);
    };
  }, [isCapturingKey, cancelKeyCapture]);

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
      ? Array.from(
          new Set(BASIC_KEYS.map((k) => k.subcategory).filter(Boolean))
        )
      : activeCategory === 'layers'
        ? Array.from(
            new Set(LAYER_KEYS.map((k) => k.subcategory).filter(Boolean))
          )
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
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold text-slate-100">Key Palette</h3>
          <div className="flex gap-2">
            {/* Capture Key button */}
            <button
              onClick={startKeyCapture}
              className="px-3 py-2 bg-primary-500 hover:bg-primary-600 text-white rounded-lg transition-colors flex items-center gap-2 text-sm font-medium"
              title="Press any physical key to select it"
              aria-label="Capture physical key"
            >
              <Keyboard className="w-4 h-4" />
              <span>Capture Key</span>
            </button>

            {/* View toggle buttons */}
            <div className="flex gap-1">
              <button
                onClick={toggleViewMode}
                className={`p-2 rounded-lg transition-colors ${
                  viewMode === 'grid'
                    ? 'bg-primary-500 text-white'
                    : 'bg-slate-700 text-slate-400 hover:bg-slate-600 hover:text-slate-300'
                }`}
                title="Grid view"
                aria-label="Grid view"
              >
                <Grid3x3 className="w-4 h-4" />
              </button>
              <button
                onClick={toggleViewMode}
                className={`p-2 rounded-lg transition-colors ${
                  viewMode === 'list'
                    ? 'bg-primary-500 text-white'
                    : 'bg-slate-700 text-slate-400 hover:bg-slate-600 hover:text-slate-300'
                }`}
                title="List view"
                aria-label="List view"
              >
                <List className="w-4 h-4" />
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Favorites Section - hidden in compact mode */}
      {!compact && favoriteKeys.length > 0 && (
        <div className="mb-4">
          <div className="flex items-center gap-2 mb-2">
            <Star className="w-4 h-4 text-yellow-400 fill-yellow-400" />
            <h4 className="text-sm font-semibold text-slate-300">Favorites</h4>
          </div>
          <div
            className={`p-3 bg-slate-800/50 rounded-lg ${
              viewMode === 'grid'
                ? 'grid grid-cols-8 gap-2'
                : 'flex flex-col gap-2'
            }`}
          >
            {favoriteKeys.map((key) =>
              renderKeyItem(key, () => handleKeySelect(key), true)
            )}
          </div>
        </div>
      )}

      {/* Recent Keys Section - hidden in compact mode */}
      {!compact && recentKeys.length > 0 && (
        <div className="mb-4">
          <div className="flex items-center gap-2 mb-2">
            <Clock className="w-4 h-4 text-slate-400" />
            <h4 className="text-sm font-semibold text-slate-300">Recent</h4>
          </div>
          <div
            className={`p-3 bg-slate-800/50 rounded-lg ${
              viewMode === 'grid'
                ? 'grid grid-cols-8 gap-2'
                : 'flex flex-col gap-2'
            }`}
          >
            {recentKeys.map((key) =>
              renderKeyItem(key, () => handleKeySelect(key), true)
            )}
          </div>
        </div>
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
      <div className={`relative ${compact ? 'mb-2' : 'mb-4'}`}>
        <Search
          className={`absolute left-3 top-1/2 -translate-y-1/2 text-slate-400 ${
            compact ? 'w-3 h-3' : 'w-4 h-4'
          }`}
        />
        <input
          ref={searchInputRef}
          type="text"
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          onKeyDown={handleSearchKeyDown}
          placeholder={
            compact
              ? 'Search keys...'
              : 'Search keys (e.g., ctrl, enter, KC_A)...'
          }
          className={`w-full bg-slate-800 border border-slate-700 rounded-lg text-slate-100 placeholder-slate-500 focus:outline-none focus:border-primary-500 focus:ring-1 focus:ring-primary-500 ${
            compact ? 'pl-8 pr-8 py-1.5 text-xs' : 'pl-10 pr-10 py-2 text-sm'
          }`}
        />
        {searchQuery && (
          <button
            onClick={() => setSearchQuery('')}
            className="absolute right-3 top-1/2 -translate-y-1/2 text-slate-400 hover:text-slate-300 transition-colors"
            aria-label="Clear search"
          >
            <X className="w-4 h-4" />
          </button>
        )}
        {/* Search result count */}
        {searchQuery && (
          <div className="absolute -bottom-5 left-0 text-xs text-slate-400">
            {searchResults.length}{' '}
            {searchResults.length === 1 ? 'result' : 'results'}
          </div>
        )}
      </div>

      {/* Category Tabs */}
      {!isSearching && (
        <div
          className={`flex gap-1 border-b border-slate-700 overflow-x-auto ${
            compact ? 'mb-2' : 'mb-4'
          }`}
        >
          {categories.map((cat) => (
            <button
              key={cat.id}
              onClick={() => {
                setActiveCategory(cat.id);
                setActiveSubcategory(null);
              }}
              className={`font-medium transition-colors whitespace-nowrap flex items-center gap-1 ${
                compact ? 'px-2 py-1 text-xs' : 'px-3 py-2 text-sm'
              } ${
                activeCategory === cat.id
                  ? 'text-primary-400 border-b-2 border-primary-400'
                  : 'text-slate-400 hover:text-slate-300'
              }`}
            >
              {!compact && <span>{cat.icon}</span>}
              <span>{cat.label}</span>
            </button>
          ))}
        </div>
      )}

      {/* Subcategory Pills (for Basic and Layers categories) - hidden in compact mode */}
      {!compact && !isSearching && subcategories.length > 0 && (
        <div className="flex gap-2 mb-3 flex-wrap">
          <button
            onClick={() => setActiveSubcategory(null)}
            className={`px-3 py-1 text-xs rounded-full transition-colors ${
              activeSubcategory === null
                ? 'bg-primary-500 text-white'
                : 'bg-slate-700 text-slate-300 hover:bg-slate-600'
            }`}
          >
            All
          </button>
          {subcategories.map((sub) => (
            <button
              key={sub}
              onClick={() => setActiveSubcategory(sub || null)}
              className={`px-3 py-1 text-xs rounded-full transition-colors capitalize ${
                activeSubcategory === sub
                  ? 'bg-primary-500 text-white'
                  : 'bg-slate-700 text-slate-300 hover:bg-slate-600'
              }`}
            >
              {sub}
            </button>
          ))}
        </div>
      )}

      {/* Key Grid - Keyboard keycap style */}
      <div className="flex-1 overflow-y-auto">
        {isSearching ? (
          // Search Results View
          searchResults.length === 0 ? (
            <div className="p-4 text-center text-slate-400">
              <p className="mb-2">No results found for "{searchQuery}"</p>
              <p className="text-xs text-slate-500">
                Try different search terms like "ctrl", "enter", or "KC_A"
              </p>
            </div>
          ) : (
            <div className="space-y-2 p-4">
              {searchResults.map((result, idx) => {
                const match = result.matches[0]; // Primary match for highlighting
                const isSelected = idx === selectedSearchIndex;

                return (
                  <button
                    key={`search-${result.key.id}`}
                    onClick={() => {
                      handleKeySelect({
                        id: result.key.id,
                        label: result.key.label,
                        category: result.key.category,
                        subcategory: result.key.subcategory,
                        description: result.key.description,
                      });
                      setSearchQuery('');
                    }}
                    className={`
                      w-full text-left p-3 rounded-lg border transition-all
                      ${
                        isSelected
                          ? 'border-primary-500 bg-primary-500/10 ring-2 ring-primary-500/50'
                          : 'border-slate-700 bg-slate-800 hover:border-slate-600 hover:bg-slate-750'
                      }
                    `}
                  >
                    <div className="flex items-start justify-between gap-3">
                      <div className="flex-1">
                        {/* Key label with highlighting */}
                        <div className="text-lg font-bold text-white font-mono mb-1">
                          {match.field === 'label'
                            ? highlightMatches(result.key.label, match.indices)
                            : result.key.label}
                        </div>

                        {/* Key ID */}
                        <div className="text-xs text-slate-400 font-mono mb-1">
                          {match.field === 'id'
                            ? highlightMatches(result.key.id, match.indices)
                            : result.key.id}
                        </div>

                        {/* Description */}
                        <div className="text-sm text-slate-300">
                          {match.field === 'description'
                            ? highlightMatches(
                                result.key.description,
                                match.indices
                              )
                            : result.key.description}
                        </div>

                        {/* Matched alias */}
                        {match.field === 'alias' && (
                          <div className="text-xs text-slate-500 mt-1">
                            Alias: {highlightMatches(match.text, match.indices)}
                          </div>
                        )}
                      </div>

                      {/* Category badge */}
                      <div
                        className={`
                        px-2 py-1 text-xs rounded capitalize whitespace-nowrap
                        ${
                          result.key.category === 'basic'
                            ? 'bg-blue-500/20 text-blue-300'
                            : ''
                        }
                        ${
                          result.key.category === 'modifiers'
                            ? 'bg-cyan-500/20 text-cyan-300'
                            : ''
                        }
                        ${
                          result.key.category === 'media'
                            ? 'bg-pink-500/20 text-pink-300'
                            : ''
                        }
                        ${
                          result.key.category === 'macro'
                            ? 'bg-green-500/20 text-green-300'
                            : ''
                        }
                        ${
                          result.key.category === 'layers'
                            ? 'bg-yellow-500/20 text-yellow-300'
                            : ''
                        }
                        ${
                          result.key.category === 'special'
                            ? 'bg-purple-500/20 text-purple-300'
                            : ''
                        }
                      `}
                      >
                        {result.key.category}
                      </div>
                    </div>
                  </button>
                );
              })}
            </div>
          )
        ) : activeCategory === 'any' ? (
          // Custom keycode input (Any category)
          <div className="p-6 space-y-6">
            <div>
              <h4 className="text-lg font-semibold text-slate-200 mb-2">
                Custom Keycode
              </h4>
              <p className="text-sm text-slate-400 mb-4">
                Enter any valid QMK-style keycode for advanced customization.
              </p>
            </div>

            {/* Input field with validation */}
            <div className="space-y-3">
              <div className="relative">
                <input
                  type="text"
                  value={customKeycode}
                  onChange={(e) => handleCustomKeycodeChange(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter' && customValidation.valid) {
                      handleApplyCustomKeycode();
                    }
                  }}
                  placeholder="e.g., KC_A, LCTL(KC_C), MO(1), LT(2,KC_SPC)"
                  className={`
                    w-full px-4 py-3 pr-10
                    bg-slate-800 border-2 rounded-lg
                    text-slate-100 font-mono text-base
                    placeholder-slate-500
                    focus:outline-none focus:ring-2
                    transition-colors
                    ${
                      customValidation.valid
                        ? 'border-green-500 focus:border-green-400 focus:ring-green-500/50'
                        : customKeycode && !customValidation.valid
                          ? 'border-red-500 focus:border-red-400 focus:ring-red-500/50'
                          : 'border-slate-700 focus:border-primary-500 focus:ring-primary-500/50'
                    }
                  `}
                />
                {/* Validation icon */}
                <div className="absolute right-3 top-1/2 -translate-y-1/2">
                  {customValidation.valid ? (
                    <Check className="w-5 h-5 text-green-400" />
                  ) : customKeycode && !customValidation.valid ? (
                    <AlertCircle className="w-5 h-5 text-red-400" />
                  ) : null}
                </div>
              </div>

              {/* Validation message */}
              {customKeycode && (
                <div
                  className={`text-sm ${
                    customValidation.valid ? 'text-green-400' : 'text-red-400'
                  }`}
                >
                  {customValidation.valid ? (
                    <div className="flex items-start gap-2">
                      <Check className="w-4 h-4 mt-0.5 flex-shrink-0" />
                      <div>
                        <p className="font-medium">Valid keycode</p>
                        <p className="text-slate-400 text-xs mt-0.5">
                          Will be mapped as:{' '}
                          <span className="font-mono">
                            {customValidation.label}
                          </span>
                        </p>
                      </div>
                    </div>
                  ) : (
                    <div className="flex items-start gap-2">
                      <AlertCircle className="w-4 h-4 mt-0.5 flex-shrink-0" />
                      <p>{customValidation.error}</p>
                    </div>
                  )}
                </div>
              )}

              {/* Apply button */}
              <button
                onClick={handleApplyCustomKeycode}
                disabled={!customValidation.valid}
                className={`
                  w-full px-4 py-3 rounded-lg font-medium
                  transition-all
                  ${
                    customValidation.valid
                      ? 'bg-primary-500 hover:bg-primary-600 text-white shadow-lg hover:shadow-xl'
                      : 'bg-slate-700 text-slate-500 cursor-not-allowed'
                  }
                `}
              >
                {customValidation.valid
                  ? 'Apply Keycode'
                  : 'Enter a valid keycode'}
              </button>
            </div>

            {/* Help section */}
            <div className="pt-6 border-t border-slate-700">
              <div className="flex items-start gap-2 mb-3">
                <HelpCircle className="w-4 h-4 text-slate-400 mt-0.5 flex-shrink-0" />
                <h5 className="text-sm font-semibold text-slate-300">
                  Supported Syntax
                </h5>
              </div>
              <div className="space-y-3 text-sm text-slate-400">
                <div>
                  <p className="font-mono text-xs text-primary-400 mb-1">
                    Simple Keys
                  </p>
                  <p className="text-xs">
                    <span className="font-mono text-slate-300">A</span>,{' '}
                    <span className="font-mono text-slate-300">KC_A</span>,{' '}
                    <span className="font-mono text-slate-300">KC_ENTER</span>
                  </p>
                </div>
                <div>
                  <p className="font-mono text-xs text-primary-400 mb-1">
                    Modifier Combinations
                  </p>
                  <p className="text-xs">
                    <span className="font-mono text-slate-300">LCTL(KC_C)</span>
                    , <span className="font-mono text-slate-300">LSFT(A)</span>
                  </p>
                </div>
                <div>
                  <p className="font-mono text-xs text-primary-400 mb-1">
                    Layer Functions
                  </p>
                  <p className="text-xs">
                    <span className="font-mono text-slate-300">MO(1)</span> =
                    Hold layer,{' '}
                    <span className="font-mono text-slate-300">TO(2)</span> =
                    Switch to layer
                    <br />
                    <span className="font-mono text-slate-300">TG(3)</span> =
                    Toggle layer,{' '}
                    <span className="font-mono text-slate-300">OSL(4)</span> =
                    One-shot layer
                  </p>
                </div>
                <div>
                  <p className="font-mono text-xs text-primary-400 mb-1">
                    Layer-Tap
                  </p>
                  <p className="text-xs">
                    <span className="font-mono text-slate-300">
                      LT(2,KC_SPC)
                    </span>{' '}
                    = Hold for layer 2, tap for Space
                  </p>
                </div>
              </div>
            </div>
          </div>
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
      {isCapturingKey && (
        <div
          className="fixed inset-0 bg-black/70 backdrop-blur-sm flex items-center justify-center z-50"
          onClick={cancelKeyCapture}
        >
          <div
            className="bg-slate-800 border-2 border-primary-500 rounded-xl p-8 shadow-2xl max-w-md w-full mx-4"
            onClick={(e) => e.stopPropagation()}
          >
            {/* Waiting for key press */}
            {!capturedKey ? (
              <div className="text-center">
                <div className="mb-6">
                  <Keyboard className="w-16 h-16 text-primary-400 mx-auto animate-pulse" />
                </div>
                <h3 className="text-2xl font-bold text-white mb-3">
                  Press any key...
                </h3>
                <p className="text-slate-400 mb-6">
                  Press the physical key you want to select.
                </p>
                <div className="text-xs text-slate-500">
                  Press{' '}
                  <kbd className="px-2 py-1 bg-slate-700 rounded border border-slate-600 font-mono">
                    Esc
                  </kbd>{' '}
                  to cancel
                </div>
              </div>
            ) : (
              /* Key captured - show confirmation */
              <div className="text-center">
                <div className="mb-6">
                  <div className="inline-block p-4 bg-green-500/20 rounded-full">
                    <Check className="w-12 h-12 text-green-400" />
                  </div>
                </div>
                <h3 className="text-2xl font-bold text-white mb-3">
                  Key Captured!
                </h3>
                <div className="mb-6 p-4 bg-slate-900/50 rounded-lg border border-slate-700">
                  <div className="text-3xl font-bold text-white font-mono mb-2">
                    {capturedKey.label}
                  </div>
                  <div className="text-sm text-slate-400 font-mono mb-1">
                    {capturedKey.id}
                  </div>
                  <div className="text-xs text-slate-500">
                    {capturedKey.description}
                  </div>
                </div>
                <p className="text-slate-400 mb-6">Use this key for mapping?</p>
                <div className="flex gap-3">
                  <button
                    onClick={cancelKeyCapture}
                    className="flex-1 px-4 py-3 bg-slate-700 hover:bg-slate-600 text-slate-300 rounded-lg transition-colors font-medium"
                  >
                    Cancel
                  </button>
                  <button
                    onClick={confirmCapturedKey}
                    className="flex-1 px-4 py-3 bg-primary-500 hover:bg-primary-600 text-white rounded-lg transition-colors font-medium"
                  >
                    Use This Key
                  </button>
                </div>
              </div>
            )}
          </div>
        </div>
      )}
    </Card>
  );
}
