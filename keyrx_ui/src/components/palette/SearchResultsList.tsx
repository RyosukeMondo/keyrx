import React from 'react';
import { highlightMatches } from '@/utils/paletteHelpers.tsx';
import type { PaletteKey } from '../KeyPalette';
import type { SearchMatch } from '@/hooks/usePaletteSearch';

interface SearchResultsListProps {
  searchQuery: string;
  searchResults: SearchMatch[];
  selectedSearchIndex: number;
  onSelectResult: (key: PaletteKey) => void;
}

/**
 * Displays search results with highlighted matches
 */
export const SearchResultsList: React.FC<SearchResultsListProps> = ({
  searchQuery,
  searchResults,
  selectedSearchIndex,
  onSelectResult,
}) => {
  if (searchResults.length === 0) {
    return (
      <div className="p-4 text-center text-slate-400">
        <p className="mb-2">No results found for "{searchQuery}"</p>
        <p className="text-xs text-slate-500">
          Try different search terms like "ctrl", "enter", or "KC_A"
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-2 p-4">
      {searchResults.map((result, idx) => {
        const match = result.matches[0];
        const isSelected = idx === selectedSearchIndex;

        return (
          <button
            key={`search-${result.key.id}`}
            onClick={() => onSelectResult(result.key)}
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
                    ? highlightMatches(result.key.description, match.indices)
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
  );
};
