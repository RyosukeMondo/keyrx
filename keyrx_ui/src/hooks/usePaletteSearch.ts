import { useState, useMemo, useCallback } from 'react';
import { KeyDefinition } from '../data/keyDefinitions';

/**
 * Match result from fuzzy matching
 */
export interface FuzzyMatchResult {
  score: number;
  indices: number[];
}

/**
 * Search match result for a key
 */
export interface SearchMatch {
  key: KeyDefinition;
  score: number;
  matches: {
    field: 'id' | 'label' | 'description' | 'alias';
    text: string;
    indices: number[];
  }[];
}

/**
 * Calculate fuzzy match score and find matching character indices
 * Returns { score, indices } where score is higher for better matches
 */
function fuzzyMatch(text: string, query: string): FuzzyMatchResult | null {
  const textLower = text.toLowerCase();
  const queryLower = query.toLowerCase();

  // Exact match gets highest score
  if (textLower === queryLower) {
    return {
      score: 1000,
      indices: Array.from({ length: text.length }, (_, i) => i),
    };
  }

  // Starts with query gets high score
  if (textLower.startsWith(queryLower)) {
    return {
      score: 500,
      indices: Array.from({ length: query.length }, (_, i) => i),
    };
  }

  // Contains query gets medium score
  const containsIndex = textLower.indexOf(queryLower);
  if (containsIndex >= 0) {
    const indices = Array.from(
      { length: query.length },
      (_, i) => containsIndex + i
    );
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
function searchKeysWithFuzzy(
  keys: KeyDefinition[],
  query: string
): SearchMatch[] {
  if (!query.trim()) return [];

  const results: SearchMatch[] = [];

  for (const key of keys) {
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
      matches.push({
        field: 'label',
        text: key.label,
        indices: labelMatch.indices,
      });
      totalScore += labelMatch.score * 1.5;
    }

    // Search in description
    const descMatch = fuzzyMatch(key.description, query);
    if (descMatch) {
      matches.push({
        field: 'description',
        text: key.description,
        indices: descMatch.indices,
      });
      totalScore += descMatch.score;
    }

    // Search in aliases
    for (const alias of key.aliases) {
      const aliasMatch = fuzzyMatch(alias, query);
      if (aliasMatch) {
        matches.push({
          field: 'alias',
          text: alias,
          indices: aliasMatch.indices,
        });
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
 * Hook for fuzzy searching through key definitions
 * Provides memoized search results sorted by relevance score
 *
 * @param keys - Array of key definitions to search through
 * @returns Search state and handlers
 *
 * @example
 * ```tsx
 * const { query, setQuery, results } = usePaletteSearch(KEY_DEFINITIONS);
 *
 * <input value={query} onChange={(e) => setQuery(e.target.value)} />
 * {results.map(match => (
 *   <div key={match.key.id}>{match.key.label}</div>
 * ))}
 * ```
 */
export function usePaletteSearch(keys: KeyDefinition[]) {
  const [query, setQuery] = useState('');

  // Memoize search results - only recompute when keys or query changes
  const results = useMemo(
    () => searchKeysWithFuzzy(keys, query),
    [keys, query]
  );

  // Memoize setQuery callback
  const handleSetQuery = useCallback((newQuery: string) => {
    setQuery(newQuery);
  }, []);

  return {
    query,
    setQuery: handleSetQuery,
    results,
  };
}
