import React from 'react';
import { Search, X, Star, Clock } from 'lucide-react';
import { Card } from './Card';
import { KEY_DEFINITIONS, KeyDefinition } from '../data/keyDefinitions';

/**
 * Key Palette - Shows available keys/modifiers/layers for assignment
 * Based on VIA-style categories: Basic, Modifiers, Media, Macro, Layers, Special, Any
 */

export interface PaletteKey {
  id: string;
  label: string;
  category: 'basic' | 'modifiers' | 'media' | 'macro' | 'layers' | 'special' | 'any';
  subcategory?: string;
  description?: string;
}

interface SearchMatch {
  key: KeyDefinition;
  score: number;
  matches: {
    field: 'id' | 'label' | 'description' | 'alias';
    text: string;
    indices: number[];
  }[];
}

interface KeyPaletteProps {
  onKeySelect: (key: PaletteKey) => void;
  selectedKey?: PaletteKey | null;
}

/**
 * Calculate fuzzy match score and find matching character indices
 * Returns { score, indices } where score is higher for better matches
 */
function fuzzyMatch(text: string, query: string): { score: number; indices: number[] } | null {
  const textLower = text.toLowerCase();
  const queryLower = query.toLowerCase();

  // Exact match gets highest score
  if (textLower === queryLower) {
    return { score: 1000, indices: Array.from({ length: text.length }, (_, i) => i) };
  }

  // Starts with query gets high score
  if (textLower.startsWith(queryLower)) {
    return { score: 500, indices: Array.from({ length: query.length }, (_, i) => i) };
  }

  // Contains query gets medium score
  const containsIndex = textLower.indexOf(queryLower);
  if (containsIndex >= 0) {
    const indices = Array.from({ length: query.length }, (_, i) => containsIndex + i);
    return { score: 200 - containsIndex, indices };
  }

  // Fuzzy matching: find all query characters in order
  const indices: number[] = [];
  let textIdx = 0;
  let queryIdx = 0;
  let consecutiveMatches = 0;
  let score = 0;

  while (textIdx < textLower.length && queryIdx < queryLower.length) {
    if (textLower[textIdx] === queryLower[queryIdx]) {
      indices.push(textIdx);
      queryIdx++;
      consecutiveMatches++;
      // Bonus for consecutive matches
      score += 10 + consecutiveMatches;
    } else {
      consecutiveMatches = 0;
    }
    textIdx++;
  }

  // All query characters must be found
  if (queryIdx !== queryLower.length) {
    return null;
  }

  // Penalty for gaps between matches
  score -= (textIdx - queryLower.length) * 2;

  return { score: Math.max(score, 1), indices };
}

/**
 * Search keys with fuzzy matching across all fields
 */
function searchKeysWithFuzzy(query: string): SearchMatch[] {
  if (!query.trim()) return [];

  const results: SearchMatch[] = [];

  for (const key of KEY_DEFINITIONS) {
    const matches: SearchMatch['matches'] = [];
    let totalScore = 0;

    // Search in ID
    const idMatch = fuzzyMatch(key.id, query);
    if (idMatch) {
      matches.push({ field: 'id', text: key.id, indices: idMatch.indices });
      totalScore += idMatch.score * 2; // ID matches are important
    }

    // Search in label
    const labelMatch = fuzzyMatch(key.label, query);
    if (labelMatch) {
      matches.push({ field: 'label', text: key.label, indices: labelMatch.indices });
      totalScore += labelMatch.score * 1.5;
    }

    // Search in description
    const descMatch = fuzzyMatch(key.description, query);
    if (descMatch) {
      matches.push({ field: 'description', text: key.description, indices: descMatch.indices });
      totalScore += descMatch.score;
    }

    // Search in aliases
    for (const alias of key.aliases) {
      const aliasMatch = fuzzyMatch(alias, query);
      if (aliasMatch) {
        matches.push({ field: 'alias', text: alias, indices: aliasMatch.indices });
        totalScore += aliasMatch.score * 1.5;
        break; // Only count first matching alias
      }
    }

    if (matches.length > 0) {
      results.push({ key, score: totalScore, matches });
    }
  }

  // Sort by score descending
  return results.sort((a, b) => b.score - a.score);
}

/**
 * Highlight matching characters in text
 */
function highlightMatches(text: string, indices: number[]): React.ReactNode {
  if (indices.length === 0) return text;

  const result: React.ReactNode[] = [];
  let lastIndex = 0;

  // Create set for O(1) lookup
  const indexSet = new Set(indices);

  for (let i = 0; i < text.length; i++) {
    if (indexSet.has(i)) {
      // Add non-highlighted text before this match
      if (i > lastIndex) {
        result.push(text.slice(lastIndex, i));
      }
      // Add highlighted character
      result.push(
        <mark key={i} className="bg-yellow-400/40 text-yellow-200 font-semibold">
          {text[i]}
        </mark>
      );
      lastIndex = i + 1;
    }
  }

  // Add remaining text
  if (lastIndex < text.length) {
    result.push(text.slice(lastIndex));
  }

  return <>{result}</>;
}

// VIA-style key definitions with categories and subcategories
const BASIC_KEYS: PaletteKey[] = [
  // Letters subcategory
  { id: 'A', label: 'A', category: 'basic', subcategory: 'letters' },
  { id: 'B', label: 'B', category: 'basic', subcategory: 'letters' },
  { id: 'C', label: 'C', category: 'basic', subcategory: 'letters' },
  { id: 'D', label: 'D', category: 'basic', subcategory: 'letters' },
  { id: 'E', label: 'E', category: 'basic', subcategory: 'letters' },
  { id: 'F', label: 'F', category: 'basic', subcategory: 'letters' },
  { id: 'G', label: 'G', category: 'basic', subcategory: 'letters' },
  { id: 'H', label: 'H', category: 'basic', subcategory: 'letters' },
  { id: 'I', label: 'I', category: 'basic', subcategory: 'letters' },
  { id: 'J', label: 'J', category: 'basic', subcategory: 'letters' },
  { id: 'K', label: 'K', category: 'basic', subcategory: 'letters' },
  { id: 'L', label: 'L', category: 'basic', subcategory: 'letters' },
  { id: 'M', label: 'M', category: 'basic', subcategory: 'letters' },
  { id: 'N', label: 'N', category: 'basic', subcategory: 'letters' },
  { id: 'O', label: 'O', category: 'basic', subcategory: 'letters' },
  { id: 'P', label: 'P', category: 'basic', subcategory: 'letters' },
  { id: 'Q', label: 'Q', category: 'basic', subcategory: 'letters' },
  { id: 'R', label: 'R', category: 'basic', subcategory: 'letters' },
  { id: 'S', label: 'S', category: 'basic', subcategory: 'letters' },
  { id: 'T', label: 'T', category: 'basic', subcategory: 'letters' },
  { id: 'U', label: 'U', category: 'basic', subcategory: 'letters' },
  { id: 'V', label: 'V', category: 'basic', subcategory: 'letters' },
  { id: 'W', label: 'W', category: 'basic', subcategory: 'letters' },
  { id: 'X', label: 'X', category: 'basic', subcategory: 'letters' },
  { id: 'Y', label: 'Y', category: 'basic', subcategory: 'letters' },
  { id: 'Z', label: 'Z', category: 'basic', subcategory: 'letters' },
  // Numbers subcategory
  { id: '0', label: '0', category: 'basic', subcategory: 'numbers' },
  { id: '1', label: '1', category: 'basic', subcategory: 'numbers' },
  { id: '2', label: '2', category: 'basic', subcategory: 'numbers' },
  { id: '3', label: '3', category: 'basic', subcategory: 'numbers' },
  { id: '4', label: '4', category: 'basic', subcategory: 'numbers' },
  { id: '5', label: '5', category: 'basic', subcategory: 'numbers' },
  { id: '6', label: '6', category: 'basic', subcategory: 'numbers' },
  { id: '7', label: '7', category: 'basic', subcategory: 'numbers' },
  { id: '8', label: '8', category: 'basic', subcategory: 'numbers' },
  { id: '9', label: '9', category: 'basic', subcategory: 'numbers' },
  // Function keys
  { id: 'F1', label: 'F1', category: 'basic', subcategory: 'function' },
  { id: 'F2', label: 'F2', category: 'basic', subcategory: 'function' },
  { id: 'F3', label: 'F3', category: 'basic', subcategory: 'function' },
  { id: 'F4', label: 'F4', category: 'basic', subcategory: 'function' },
  { id: 'F5', label: 'F5', category: 'basic', subcategory: 'function' },
  { id: 'F6', label: 'F6', category: 'basic', subcategory: 'function' },
  { id: 'F7', label: 'F7', category: 'basic', subcategory: 'function' },
  { id: 'F8', label: 'F8', category: 'basic', subcategory: 'function' },
  { id: 'F9', label: 'F9', category: 'basic', subcategory: 'function' },
  { id: 'F10', label: 'F10', category: 'basic', subcategory: 'function' },
  { id: 'F11', label: 'F11', category: 'basic', subcategory: 'function' },
  { id: 'F12', label: 'F12', category: 'basic', subcategory: 'function' },
  // Navigation subcategory
  { id: 'Escape', label: 'Esc', category: 'basic', subcategory: 'navigation', description: 'Escape key' },
  { id: 'Enter', label: 'Enter', category: 'basic', subcategory: 'navigation', description: 'Enter/Return' },
  { id: 'Space', label: 'Space', category: 'basic', subcategory: 'navigation', description: 'Space bar' },
  { id: 'Backspace', label: 'BS', category: 'basic', subcategory: 'navigation', description: 'Backspace' },
  { id: 'Tab', label: 'Tab', category: 'basic', subcategory: 'navigation', description: 'Tab key' },
  { id: 'Delete', label: 'Del', category: 'basic', subcategory: 'navigation', description: 'Delete' },
  { id: 'Insert', label: 'Ins', category: 'basic', subcategory: 'navigation', description: 'Insert' },
  { id: 'Home', label: 'Home', category: 'basic', subcategory: 'navigation', description: 'Home' },
  { id: 'End', label: 'End', category: 'basic', subcategory: 'navigation', description: 'End' },
  { id: 'PageUp', label: 'PgUp', category: 'basic', subcategory: 'navigation', description: 'Page Up' },
  { id: 'PageDown', label: 'PgDn', category: 'basic', subcategory: 'navigation', description: 'Page Down' },
  { id: 'Up', label: '↑', category: 'basic', subcategory: 'navigation', description: 'Arrow Up' },
  { id: 'Down', label: '↓', category: 'basic', subcategory: 'navigation', description: 'Arrow Down' },
  { id: 'Left', label: '←', category: 'basic', subcategory: 'navigation', description: 'Arrow Left' },
  { id: 'Right', label: '→', category: 'basic', subcategory: 'navigation', description: 'Arrow Right' },
  // Punctuation subcategory
  { id: 'Minus', label: '-', category: 'basic', subcategory: 'punctuation' },
  { id: 'Equal', label: '=', category: 'basic', subcategory: 'punctuation' },
  { id: 'LeftBracket', label: '[', category: 'basic', subcategory: 'punctuation' },
  { id: 'RightBracket', label: ']', category: 'basic', subcategory: 'punctuation' },
  { id: 'Backslash', label: '\\', category: 'basic', subcategory: 'punctuation' },
  { id: 'Semicolon', label: ';', category: 'basic', subcategory: 'punctuation' },
  { id: 'Quote', label: "'", category: 'basic', subcategory: 'punctuation' },
  { id: 'Comma', label: ',', category: 'basic', subcategory: 'punctuation' },
  { id: 'Period', label: '.', category: 'basic', subcategory: 'punctuation' },
  { id: 'Slash', label: '/', category: 'basic', subcategory: 'punctuation' },
];

const MODIFIER_KEYS: PaletteKey[] = [
  { id: 'LCtrl', label: 'LCtrl', category: 'modifiers', description: 'Left Control' },
  { id: 'RCtrl', label: 'RCtrl', category: 'modifiers', description: 'Right Control' },
  { id: 'LShift', label: 'LShift', category: 'modifiers', description: 'Left Shift' },
  { id: 'RShift', label: 'RShift', category: 'modifiers', description: 'Right Shift' },
  { id: 'LAlt', label: 'LAlt', category: 'modifiers', description: 'Left Alt' },
  { id: 'RAlt', label: 'RAlt', category: 'modifiers', description: 'Right Alt' },
  { id: 'LMeta', label: 'LWin', category: 'modifiers', description: 'Left Windows/Super' },
  { id: 'RMeta', label: 'RWin', category: 'modifiers', description: 'Right Windows/Super' },
  { id: 'MD_00', label: 'MD_00', category: 'modifiers', description: 'Custom Modifier 0' },
  { id: 'MD_01', label: 'MD_01', category: 'modifiers', description: 'Custom Modifier 1' },
  { id: 'MD_02', label: 'MD_02', category: 'modifiers', description: 'Custom Modifier 2' },
  { id: 'MD_03', label: 'MD_03', category: 'modifiers', description: 'Custom Modifier 3' },
  { id: 'MD_04', label: 'MD_04', category: 'modifiers', description: 'Custom Modifier 4' },
  { id: 'MD_05', label: 'MD_05', category: 'modifiers', description: 'Custom Modifier 5' },
  { id: 'MD_06', label: 'MD_06', category: 'modifiers', description: 'Custom Modifier 6' },
  { id: 'MD_07', label: 'MD_07', category: 'modifiers', description: 'Custom Modifier 7' },
  { id: 'MD_08', label: 'MD_08', category: 'modifiers', description: 'Custom Modifier 8' },
  { id: 'MD_09', label: 'MD_09', category: 'modifiers', description: 'Custom Modifier 9' },
];

const MEDIA_KEYS: PaletteKey[] = [
  // Placeholder for media keys (to be expanded in task 1.2)
];

const MACRO_KEYS: PaletteKey[] = [
  // User-defined macros (M0-M15)
  { id: 'M0', label: 'M0', category: 'macro', description: 'Macro 0' },
  { id: 'M1', label: 'M1', category: 'macro', description: 'Macro 1' },
  { id: 'M2', label: 'M2', category: 'macro', description: 'Macro 2' },
  { id: 'M3', label: 'M3', category: 'macro', description: 'Macro 3' },
  { id: 'M4', label: 'M4', category: 'macro', description: 'Macro 4' },
  { id: 'M5', label: 'M5', category: 'macro', description: 'Macro 5' },
  { id: 'M6', label: 'M6', category: 'macro', description: 'Macro 6' },
  { id: 'M7', label: 'M7', category: 'macro', description: 'Macro 7' },
  { id: 'M8', label: 'M8', category: 'macro', description: 'Macro 8' },
  { id: 'M9', label: 'M9', category: 'macro', description: 'Macro 9' },
  { id: 'M10', label: 'M10', category: 'macro', description: 'Macro 10' },
  { id: 'M11', label: 'M11', category: 'macro', description: 'Macro 11' },
  { id: 'M12', label: 'M12', category: 'macro', description: 'Macro 12' },
  { id: 'M13', label: 'M13', category: 'macro', description: 'Macro 13' },
  { id: 'M14', label: 'M14', category: 'macro', description: 'Macro 14' },
  { id: 'M15', label: 'M15', category: 'macro', description: 'Macro 15' },
];

const LAYER_KEYS: PaletteKey[] = [
  { id: 'MD_00', label: 'Base (MD_00)', category: 'layers', description: 'Base layer' },
  { id: 'MD_01', label: 'Layer 1', category: 'layers', description: 'Layer 1' },
  { id: 'MD_02', label: 'Layer 2', category: 'layers', description: 'Layer 2' },
  { id: 'MD_03', label: 'Layer 3', category: 'layers', description: 'Layer 3' },
  { id: 'MD_04', label: 'Layer 4', category: 'layers', description: 'Layer 4' },
  { id: 'MD_05', label: 'Layer 5', category: 'layers', description: 'Layer 5' },
  { id: 'MD_06', label: 'Layer 6', category: 'layers', description: 'Layer 6' },
  { id: 'MD_07', label: 'Layer 7', category: 'layers', description: 'Layer 7' },
  { id: 'MD_08', label: 'Layer 8', category: 'layers', description: 'Layer 8' },
  { id: 'MD_09', label: 'Layer 9', category: 'layers', description: 'Layer 9' },
];

const SPECIAL_KEYS: PaletteKey[] = [
  { id: 'LK_00', label: 'CapsLock', category: 'special', description: 'Caps Lock (LK_00)' },
  { id: 'LK_01', label: 'NumLock', category: 'special', description: 'Num Lock (LK_01)' },
  { id: 'LK_02', label: 'ScrollLock', category: 'special', description: 'Scroll Lock (LK_02)' },
  { id: 'LK_03', label: 'LK_03', category: 'special', description: 'Custom Lock 3' },
  { id: 'LK_04', label: 'LK_04', category: 'special', description: 'Custom Lock 4' },
  { id: 'LK_05', label: 'LK_05', category: 'special', description: 'Custom Lock 5' },
  { id: 'LK_06', label: 'LK_06', category: 'special', description: 'Custom Lock 6' },
  { id: 'LK_07', label: 'LK_07', category: 'special', description: 'Custom Lock 7' },
  { id: 'LK_08', label: 'LK_08', category: 'special', description: 'Custom Lock 8' },
  { id: 'LK_09', label: 'LK_09', category: 'special', description: 'Custom Lock 9' },
];

/**
 * LocalStorage keys for persistence
 */
const STORAGE_KEY_RECENT = 'keyrx_recent_keys';
const STORAGE_KEY_FAVORITES = 'keyrx_favorite_keys';
const MAX_RECENT_KEYS = 10;

/**
 * Load array from localStorage with error handling
 */
function loadFromStorage(key: string): string[] {
  try {
    const stored = localStorage.getItem(key);
    if (stored) {
      const parsed = JSON.parse(stored);
      return Array.isArray(parsed) ? parsed : [];
    }
  } catch (err) {
    console.warn(`Failed to load ${key} from localStorage:`, err);
  }
  return [];
}

/**
 * Save array to localStorage with error handling
 */
function saveToStorage(key: string, data: string[]): void {
  try {
    localStorage.setItem(key, JSON.stringify(data));
  } catch (err) {
    console.error(`Failed to save ${key} to localStorage:`, err);
  }
}

/**
 * Find a key definition by ID
 */
function findKeyById(keyId: string): PaletteKey | null {
  // Search in KEY_DEFINITIONS first
  const keyDef = KEY_DEFINITIONS.find(k => k.id === keyId);
  if (keyDef) {
    return {
      id: keyDef.id,
      label: keyDef.label,
      category: keyDef.category,
      subcategory: keyDef.subcategory,
      description: keyDef.description,
    };
  }

  // Fallback: search in static key arrays
  const allKeys = [...BASIC_KEYS, ...MODIFIER_KEYS, ...MEDIA_KEYS, ...MACRO_KEYS, ...LAYER_KEYS, ...SPECIAL_KEYS];
  return allKeys.find(k => k.id === keyId) || null;
}

export function KeyPalette({ onKeySelect, selectedKey }: KeyPaletteProps) {
  const [activeCategory, setActiveCategory] = React.useState<PaletteKey['category']>('basic');
  const [activeSubcategory, setActiveSubcategory] = React.useState<string | null>(null);
  const [searchQuery, setSearchQuery] = React.useState('');
  const [selectedSearchIndex, setSelectedSearchIndex] = React.useState(0);
  const searchInputRef = React.useRef<HTMLInputElement>(null);

  // Recent and Favorite keys state
  const [recentKeyIds, setRecentKeyIds] = React.useState<string[]>(() => loadFromStorage(STORAGE_KEY_RECENT));
  const [favoriteKeyIds, setFavoriteKeyIds] = React.useState<string[]>(() => loadFromStorage(STORAGE_KEY_FAVORITES));

  // Add key to recent list (max 10, most recent first)
  const addToRecent = React.useCallback((keyId: string) => {
    setRecentKeyIds(prev => {
      const filtered = prev.filter(id => id !== keyId);
      const updated = [keyId, ...filtered].slice(0, MAX_RECENT_KEYS);
      saveToStorage(STORAGE_KEY_RECENT, updated);
      return updated;
    });
  }, []);

  // Toggle favorite status
  const toggleFavorite = React.useCallback((keyId: string) => {
    setFavoriteKeyIds(prev => {
      const isFavorite = prev.includes(keyId);
      const updated = isFavorite
        ? prev.filter(id => id !== keyId)
        : [...prev, keyId];
      saveToStorage(STORAGE_KEY_FAVORITES, updated);
      return updated;
    });
  }, []);

  // Check if key is favorite
  const isFavorite = React.useCallback((keyId: string) => {
    return favoriteKeyIds.includes(keyId);
  }, [favoriteKeyIds]);

  // Handle key selection with recent tracking
  const handleKeySelect = React.useCallback((key: PaletteKey) => {
    addToRecent(key.id);
    onKeySelect(key);
  }, [addToRecent, onKeySelect]);

  // Get recent and favorite key objects
  const recentKeys = React.useMemo(() => {
    return recentKeyIds.map(id => findKeyById(id)).filter((k): k is PaletteKey => k !== null);
  }, [recentKeyIds]);

  const favoriteKeys = React.useMemo(() => {
    return favoriteKeyIds.map(id => findKeyById(id)).filter((k): k is PaletteKey => k !== null);
  }, [favoriteKeyIds]);

  const categories = [
    { id: 'basic' as const, label: 'Basic', keys: BASIC_KEYS, icon: '⌨️' },
    { id: 'modifiers' as const, label: 'Modifiers', keys: MODIFIER_KEYS, icon: '⌥' },
    { id: 'media' as const, label: 'Media', keys: MEDIA_KEYS, icon: '🎵' },
    { id: 'macro' as const, label: 'Macro', keys: MACRO_KEYS, icon: '🔧' },
    { id: 'layers' as const, label: 'Layers', keys: LAYER_KEYS, icon: '📚' },
    { id: 'special' as const, label: 'Special', keys: SPECIAL_KEYS, icon: '⭐' },
    { id: 'any' as const, label: 'Any', keys: [], icon: '✏️' },
  ];

  // Search results (if query is active)
  const searchResults = React.useMemo(() => {
    return searchQuery.trim() ? searchKeysWithFuzzy(searchQuery) : [];
  }, [searchQuery]);

  // Reset selected index when search changes
  React.useEffect(() => {
    setSelectedSearchIndex(0);
  }, [searchQuery]);

  const activeCategoryData = categories.find(c => c.id === activeCategory);
  let activeKeys = activeCategoryData?.keys || [];

  // If searching, use search results instead
  const isSearching = searchQuery.trim().length > 0;

  // Filter by subcategory if one is active
  if (activeSubcategory && activeCategory === 'basic' && !isSearching) {
    activeKeys = activeKeys.filter(k => k.subcategory === activeSubcategory);
  }

  // Get unique subcategories for Basic category
  const subcategories = activeCategory === 'basic'
    ? Array.from(new Set(BASIC_KEYS.map(k => k.subcategory).filter(Boolean)))
    : [];

  // Handle keyboard navigation in search results
  const handleSearchKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (searchResults.length === 0) return;

    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        setSelectedSearchIndex(prev => Math.min(prev + 1, searchResults.length - 1));
        break;
      case 'ArrowUp':
        e.preventDefault();
        setSelectedSearchIndex(prev => Math.max(prev - 1, 0));
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

  // Render a key item with star button
  const renderKeyItem = (key: PaletteKey, onClick: () => void, showStar: boolean = true) => {
    const favorite = isFavorite(key.id);

    return (
      <div key={key.id} className="relative group">
        <button
          onClick={onClick}
          className={`
            w-full relative flex flex-col items-center justify-center
            min-h-[50px] px-2 py-2
            rounded border transition-all
            hover:brightness-110 hover:-translate-y-0.5 hover:shadow-lg
            ${
              selectedKey?.id === key.id
                ? 'border-primary-500 bg-primary-500/20 shadow-lg shadow-primary-500/50'
                : 'border-slate-600 bg-slate-700 hover:border-slate-500'
            }
            ${key.category === 'modifiers' ? 'border-cyan-500/50' : ''}
            ${key.category === 'special' ? 'border-purple-500/50' : ''}
            ${key.category === 'layers' ? 'border-yellow-500/50' : ''}
            ${key.category === 'macro' ? 'border-green-500/50' : ''}
            ${key.category === 'media' ? 'border-pink-500/50' : ''}
          `}
          title={key.description || key.id}
        >
          {/* Key label (main) */}
          <div className="text-sm font-bold text-white font-mono">
            {key.label}
          </div>
          {/* Key ID (small, below) */}
          {key.id !== key.label && (
            <div className="text-[9px] text-slate-400 mt-0.5 font-mono">
              {key.id}
            </div>
          )}
        </button>
        {/* Star button */}
        {showStar && (
          <button
            onClick={(e) => {
              e.stopPropagation();
              toggleFavorite(key.id);
            }}
            className="absolute top-1 right-1 opacity-0 group-hover:opacity-100 transition-opacity"
            title={favorite ? "Remove from favorites" : "Add to favorites"}
          >
            <Star
              className={`w-3 h-3 ${
                favorite
                  ? 'fill-yellow-400 text-yellow-400'
                  : 'text-slate-400 hover:text-yellow-400'
              }`}
            />
          </button>
        )}
      </div>
    );
  };

  return (
    <Card className="h-full flex flex-col">
      <h3 className="text-lg font-semibold text-slate-100 mb-4">Key Palette</h3>

      {/* Favorites Section */}
      {favoriteKeys.length > 0 && (
        <div className="mb-4">
          <div className="flex items-center gap-2 mb-2">
            <Star className="w-4 h-4 text-yellow-400 fill-yellow-400" />
            <h4 className="text-sm font-semibold text-slate-300">Favorites</h4>
          </div>
          <div className="grid grid-cols-8 gap-2 p-3 bg-slate-800/50 rounded-lg">
            {favoriteKeys.map(key => renderKeyItem(key, () => handleKeySelect(key), true))}
          </div>
        </div>
      )}

      {/* Recent Keys Section */}
      {recentKeys.length > 0 && (
        <div className="mb-4">
          <div className="flex items-center gap-2 mb-2">
            <Clock className="w-4 h-4 text-slate-400" />
            <h4 className="text-sm font-semibold text-slate-300">Recent</h4>
          </div>
          <div className="grid grid-cols-8 gap-2 p-3 bg-slate-800/50 rounded-lg">
            {recentKeys.map(key => renderKeyItem(key, () => handleKeySelect(key), true))}
          </div>
        </div>
      )}

      {/* Empty state when no favorites or recent */}
      {favoriteKeys.length === 0 && recentKeys.length === 0 && !searchQuery && (
        <div className="mb-4 p-3 bg-slate-800/30 rounded-lg border border-slate-700/50">
          <p className="text-xs text-slate-500 text-center">
            Star keys to add favorites. Recent keys will appear automatically.
          </p>
        </div>
      )}

      {/* Search Input */}
      <div className="relative mb-4">
        <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-400" />
        <input
          ref={searchInputRef}
          type="text"
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          onKeyDown={handleSearchKeyDown}
          placeholder="Search keys (e.g., ctrl, enter, KC_A)..."
          className="w-full pl-10 pr-10 py-2 bg-slate-800 border border-slate-700 rounded-lg text-slate-100 text-sm placeholder-slate-500 focus:outline-none focus:border-primary-500 focus:ring-1 focus:ring-primary-500"
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
            {searchResults.length} {searchResults.length === 1 ? 'result' : 'results'}
          </div>
        )}
      </div>

      {/* Category Tabs */}
      {!isSearching && (
        <div className="flex gap-1 mb-4 border-b border-slate-700 overflow-x-auto">
        {categories.map(cat => (
          <button
            key={cat.id}
            onClick={() => {
              setActiveCategory(cat.id);
              setActiveSubcategory(null);
            }}
            className={`px-3 py-2 text-sm font-medium transition-colors whitespace-nowrap flex items-center gap-1 ${
              activeCategory === cat.id
                ? 'text-primary-400 border-b-2 border-primary-400'
                : 'text-slate-400 hover:text-slate-300'
            }`}
          >
            <span>{cat.icon}</span>
            <span>{cat.label}</span>
          </button>
        ))}
        </div>
      )}

      {/* Subcategory Pills (for Basic category) */}
      {!isSearching && subcategories.length > 0 && (
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
          {subcategories.map(sub => (
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
              <p className="text-xs text-slate-500">Try different search terms like "ctrl", "enter", or "KC_A"</p>
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
                      ${isSelected
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
                            : result.key.label
                          }
                        </div>

                        {/* Key ID */}
                        <div className="text-xs text-slate-400 font-mono mb-1">
                          {match.field === 'id'
                            ? highlightMatches(result.key.id, match.indices)
                            : result.key.id
                          }
                        </div>

                        {/* Description */}
                        <div className="text-sm text-slate-300">
                          {match.field === 'description'
                            ? highlightMatches(result.key.description, match.indices)
                            : result.key.description
                          }
                        </div>

                        {/* Matched alias */}
                        {match.field === 'alias' && (
                          <div className="text-xs text-slate-500 mt-1">
                            Alias: {highlightMatches(match.text, match.indices)}
                          </div>
                        )}
                      </div>

                      {/* Category badge */}
                      <div className={`
                        px-2 py-1 text-xs rounded capitalize whitespace-nowrap
                        ${result.key.category === 'basic' ? 'bg-blue-500/20 text-blue-300' : ''}
                        ${result.key.category === 'modifiers' ? 'bg-cyan-500/20 text-cyan-300' : ''}
                        ${result.key.category === 'media' ? 'bg-pink-500/20 text-pink-300' : ''}
                        ${result.key.category === 'macro' ? 'bg-green-500/20 text-green-300' : ''}
                        ${result.key.category === 'layers' ? 'bg-yellow-500/20 text-yellow-300' : ''}
                        ${result.key.category === 'special' ? 'bg-purple-500/20 text-purple-300' : ''}
                      `}>
                        {result.key.category}
                      </div>
                    </div>
                  </button>
                );
              })}
            </div>
          )
        ) : activeCategory === 'any' ? (
          <div className="p-4 text-center text-slate-400">
            <p className="mb-2">Custom keycode input coming soon...</p>
            <p className="text-xs text-slate-500">Task 3.1: Add custom keycode input field</p>
          </div>
        ) : activeKeys.length === 0 ? (
          <div className="p-4 text-center text-slate-400">
            <p>No keys in this category yet</p>
          </div>
        ) : (
          <div className="grid grid-cols-8 gap-2 p-4 bg-slate-800/50 rounded-lg">
            {activeKeys.map(key => renderKeyItem(key, () => handleKeySelect(key), true))}
          </div>
        )}
      </div>

      {/* Hint */}
      <p className="text-xs text-slate-500 mt-4">
        {isSearching
          ? 'Use ↑↓ arrows to navigate, Enter to select, Esc to clear'
          : 'Search for keys or browse by category. Click to select.'
        }
      </p>
    </Card>
  );
}
