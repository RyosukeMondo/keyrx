import React, { useRef, useEffect } from 'react';
import { Search, X } from 'lucide-react';
import { SearchMatch } from '../../hooks/usePaletteSearch';

/**
 * Props for PaletteSearch component
 */
export interface PaletteSearchProps {
  /** Current search query value */
  value: string;
  /** Callback when search query changes */
  onChange: (value: string) => void;
  /** Array of search results with match highlighting data */
  results: SearchMatch[];
  /** Callback when a search result is selected */
  onSelect: (match: SearchMatch) => void;
  /** Placeholder text for search input */
  placeholder?: string;
  /** Compact mode for reduced size */
  compact?: boolean;
}

/**
 * Highlight matching characters in text
 * Uses match indices to wrap matched characters in <mark> tags
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
        <mark
          key={i}
          className="bg-yellow-400/40 text-yellow-200 font-semibold"
        >
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

/**
 * PaletteSearch Component
 *
 * Search input with dropdown showing fuzzy match results.
 * Supports keyboard navigation (Arrow keys, Enter, Escape) and
 * highlights matching characters in results.
 *
 * @example
 * ```tsx
 * const { query, setQuery, results } = usePaletteSearch(keys);
 *
 * <PaletteSearch
 *   value={query}
 *   onChange={setQuery}
 *   results={results}
 *   onSelect={(match) => handleKeySelect(match.key)}
 * />
 * ```
 */
export function PaletteSearch({
  value,
  onChange,
  results,
  onSelect,
  placeholder = 'Search keys...',
  compact = false,
}: PaletteSearchProps) {
  const [selectedIndex, setSelectedIndex] = React.useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  // Reset selected index when results change
  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect
    setSelectedIndex(0);
  }, [results]);

  // Handle keyboard navigation
  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (results.length === 0) return;

    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        setSelectedIndex((prev) => Math.min(prev + 1, results.length - 1));
        break;
      case 'ArrowUp':
        e.preventDefault();
        setSelectedIndex((prev) => Math.max(prev - 1, 0));
        break;
      case 'Enter':
        e.preventDefault();
        if (results[selectedIndex]) {
          onSelect(results[selectedIndex]);
          onChange('');
        }
        break;
      case 'Escape':
        e.preventDefault();
        onChange('');
        inputRef.current?.blur();
        break;
    }
  };

  // Handle clear button click
  const handleClear = () => {
    onChange('');
    inputRef.current?.focus();
  };

  const showDropdown = value.trim().length > 0;

  return (
    <div className="relative">
      {/* Search Input */}
      <div className="relative">
        <Search
          className={`absolute left-3 top-1/2 -translate-y-1/2 text-slate-400 ${
            compact ? 'w-3 h-3' : 'w-4 h-4'
          }`}
        />
        <input
          ref={inputRef}
          type="text"
          value={value}
          onChange={(e) => onChange(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder={placeholder}
          className={`w-full bg-slate-800 border border-slate-700 rounded-lg text-slate-100 placeholder-slate-500 focus:outline-none focus:border-primary-500 focus:ring-1 focus:ring-primary-500 ${
            compact ? 'pl-8 pr-8 py-1.5 text-xs' : 'pl-10 pr-10 py-2 text-sm'
          }`}
          aria-label="Search keys"
          aria-autocomplete="list"
          aria-controls="search-results"
          aria-expanded={showDropdown}
        />
        {value && (
          <button
            onClick={handleClear}
            className="absolute right-3 top-1/2 -translate-y-1/2 text-slate-400 hover:text-slate-300 transition-colors"
            aria-label="Clear search"
          >
            <X className="w-4 h-4" />
          </button>
        )}
      </div>

      {/* Result count */}
      {showDropdown && (
        <div className="absolute -bottom-5 left-0 text-xs text-slate-400">
          {results.length} {results.length === 1 ? 'result' : 'results'}
        </div>
      )}

      {/* Dropdown Results */}
      {showDropdown && (
        <div
          id="search-results"
          role="listbox"
          className="absolute top-full left-0 right-0 mt-6 bg-slate-800 border border-slate-700 rounded-lg shadow-2xl max-h-96 overflow-y-auto z-50"
        >
          {results.length === 0 ? (
            <div className="p-4 text-center text-slate-400">
              <p className="mb-2">No results found for "{value}"</p>
              <p className="text-xs text-slate-500">
                Try different search terms like "ctrl", "enter", or "KC_A"
              </p>
            </div>
          ) : (
            <div className="p-2">
              {results.map((result, idx) => {
                const match = result.matches[0]; // Primary match
                const isSelected = idx === selectedIndex;

                return (
                  <button
                    key={`search-${result.key.id}-${idx}`}
                    role="option"
                    aria-selected={isSelected}
                    onClick={() => {
                      onSelect(result);
                      onChange('');
                    }}
                    className={`
                      w-full text-left p-3 rounded-lg border transition-all
                      ${
                        isSelected
                          ? 'border-primary-500 bg-primary-500/10 ring-2 ring-primary-500/50'
                          : 'border-transparent bg-slate-800 hover:border-slate-600 hover:bg-slate-750'
                      }
                    `}
                  >
                    <div className="flex items-start justify-between gap-3">
                      <div className="flex-1 min-w-0">
                        {/* Key label with highlighting */}
                        <div className="text-lg font-bold text-white font-mono mb-1 truncate">
                          {match.field === 'label'
                            ? highlightMatches(result.key.label, match.indices)
                            : result.key.label}
                        </div>

                        {/* Key ID */}
                        <div className="text-xs text-slate-400 font-mono mb-1 truncate">
                          {match.field === 'id'
                            ? highlightMatches(result.key.id, match.indices)
                            : result.key.id}
                        </div>

                        {/* Description */}
                        <div className="text-sm text-slate-300 truncate">
                          {match.field === 'description'
                            ? highlightMatches(
                                result.key.description,
                                match.indices
                              )
                            : result.key.description}
                        </div>

                        {/* Matched alias */}
                        {match.field === 'alias' && (
                          <div className="text-xs text-slate-500 mt-1 truncate">
                            Alias: {highlightMatches(match.text, match.indices)}
                          </div>
                        )}
                      </div>

                      {/* Category badge */}
                      <div
                        className={`
                        px-2 py-1 text-xs rounded capitalize whitespace-nowrap flex-shrink-0
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
          )}
        </div>
      )}
    </div>
  );
}
