# Design: Refactor KeyPalette Component

## Target Architecture
```
KeyPalette (orchestrator, <200 lines)
├── PaletteSearch (search input with fuzzy matching)
├── PaletteViewModeTabs (Basic/Recent/Favorites/All tabs)
├── KeyCategorySection (reusable category renderer)
└── Hooks
    ├── useRecentKeys (recent keys storage/management)
    ├── useFavoriteKeys (favorites storage/management)
    └── usePaletteSearch (fuzzy search logic)
```

## Components

### 1. PaletteSearch (`src/components/palette/PaletteSearch.tsx`)
**Props**: `{ value, onChange, onSelect, results }`
**Extracts**: Lines 602-700 (search UI + fuzzy matching)

### 2. PaletteViewModeTabs (`src/components/palette/PaletteViewModeTabs.tsx`)
**Props**: `{ activeView, onChange }`
**Extracts**: View mode tabs UI

### 3. KeyCategorySection (`src/components/palette/KeyCategorySection.tsx`)
**Props**: `{ title, keys, onKeySelect, favorites, onToggleFavorite }`
**Extracts**: Category rendering logic (reusable)

### 4. useRecentKeys Hook (`src/hooks/useRecentKeys.ts`)
**API**: `{ recentKeys, addRecentKey, clearRecentKeys }`
**Extracts**: Lines 259-289 (localStorage for recent keys)

### 5. useFavoriteKeys Hook (`src/hooks/useFavoriteKeys.ts`)
**API**: `{ favoriteKeys, toggleFavorite, isFavorite }`
**Extracts**: Lines 291-323 (localStorage for favorites)

### 6. usePaletteSearch Hook (`src/hooks/usePaletteSearch.ts`)
**API**: `{ search, results, fuzzyMatch }`
**Extracts**: Fuzzy search logic (lines 61-118)

## Migration Strategy
1. Extract hooks (useRecentKeys, useFavoriteKeys, usePaletteSearch)
2. Extract PaletteSearch component
3. Extract KeyCategorySection component
4. Extract PaletteViewModeTabs component
5. Refactor KeyPalette to orchestrate components
6. Tests for all extracted code

## Success Criteria
- KeyPalette <500 lines
- All functions ≤50 lines
- 6 new files created
- All tests pass
