import React from 'react';
import { Search, X } from 'lucide-react';

interface PaletteSearchInputProps {
  searchQuery: string;
  onSearchChange: (query: string) => void;
  onKeyDown: (e: React.KeyboardEvent<HTMLInputElement>) => void;
  searchResultCount: number;
  compact: boolean;
  inputRef: React.RefObject<HTMLInputElement>;
}

/**
 * Search input for key palette with result count
 */
export const PaletteSearchInput: React.FC<PaletteSearchInputProps> = ({
  searchQuery,
  onSearchChange,
  onKeyDown,
  searchResultCount,
  compact,
  inputRef,
}) => {
  return (
    <div className={`relative ${compact ? 'mb-2' : 'mb-4'}`}>
      <Search
        className={`absolute left-3 top-1/2 -translate-y-1/2 text-slate-400 ${
          compact ? 'w-3 h-3' : 'w-4 h-4'
        }`}
      />
      <input
        ref={inputRef}
        type="text"
        value={searchQuery}
        onChange={(e) => onSearchChange(e.target.value)}
        onKeyDown={onKeyDown}
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
          onClick={() => onSearchChange('')}
          className="absolute right-3 top-1/2 -translate-y-1/2 text-slate-400 hover:text-slate-300 transition-colors"
          aria-label="Clear search"
        >
          <X className="w-4 h-4" />
        </button>
      )}
      {/* Search result count */}
      {searchQuery && (
        <div className="absolute -bottom-5 left-0 text-xs text-slate-400">
          {searchResultCount} {searchResultCount === 1 ? 'result' : 'results'}
        </div>
      )}
    </div>
  );
};
