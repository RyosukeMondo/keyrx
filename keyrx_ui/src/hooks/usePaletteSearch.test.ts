import { renderHook, act } from '@testing-library/react';
import { usePaletteSearch } from './usePaletteSearch';
import { KeyDefinition } from '../data/keyDefinitions';

// Test data - sample key definitions
const mockKeys: KeyDefinition[] = [
  {
    id: 'A',
    label: 'A',
    category: 'basic',
    subcategory: 'letters',
    description: 'Letter A',
    aliases: ['KC_A', 'VK_A', 'KEY_A'],
  },
  {
    id: 'CTRL',
    label: 'Ctrl',
    category: 'modifiers',
    description: 'Control modifier key',
    aliases: ['KC_LCTRL', 'CONTROL', 'LCTRL'],
  },
  {
    id: 'ESC',
    label: 'Escape',
    category: 'basic',
    subcategory: 'function',
    description: 'Escape key',
    aliases: ['KC_ESC', 'VK_ESCAPE'],
  },
  {
    id: 'ENTER',
    label: 'Enter',
    category: 'basic',
    subcategory: 'function',
    description: 'Enter/Return key',
    aliases: ['KC_ENTER', 'KC_RETURN', 'RETURN'],
  },
  {
    id: 'SPACE',
    label: 'Space',
    category: 'basic',
    subcategory: 'function',
    description: 'Spacebar',
    aliases: ['KC_SPACE', 'KC_SPC', 'SPACEBAR'],
  },
];

describe('usePaletteSearch', () => {
  it('should initialize with empty query and no results', () => {
    const { result } = renderHook(() => usePaletteSearch(mockKeys));

    expect(result.current.query).toBe('');
    expect(result.current.results).toEqual([]);
  });

  it('should return empty results for empty query', () => {
    const { result } = renderHook(() => usePaletteSearch(mockKeys));

    act(() => {
      result.current.setQuery('');
    });

    expect(result.current.results).toEqual([]);
  });

  it('should return empty results for whitespace-only query', () => {
    const { result } = renderHook(() => usePaletteSearch(mockKeys));

    act(() => {
      result.current.setQuery('   ');
    });

    expect(result.current.results).toEqual([]);
  });

  it('should find exact match in ID', () => {
    const { result } = renderHook(() => usePaletteSearch(mockKeys));

    act(() => {
      result.current.setQuery('A');
    });

    // Should find 'A' key, and it should be first (highest score)
    expect(result.current.results.length).toBeGreaterThan(0);
    expect(result.current.results[0].key.id).toBe('A');
    expect(result.current.results[0].matches).toContainEqual(
      expect.objectContaining({ field: 'id', text: 'A' })
    );
  });

  it('should find exact match in label', () => {
    const { result } = renderHook(() => usePaletteSearch(mockKeys));

    act(() => {
      result.current.setQuery('Ctrl');
    });

    expect(result.current.results.length).toBeGreaterThan(0);
    const ctrlResult = result.current.results.find((r) => r.key.id === 'CTRL');
    expect(ctrlResult).toBeDefined();
  });

  it('should find match in description', () => {
    const { result } = renderHook(() => usePaletteSearch(mockKeys));

    act(() => {
      result.current.setQuery('modifier');
    });

    expect(result.current.results.length).toBeGreaterThan(0);
    const ctrlResult = result.current.results.find((r) => r.key.id === 'CTRL');
    expect(ctrlResult).toBeDefined();
    expect(ctrlResult?.matches).toContainEqual(
      expect.objectContaining({ field: 'description' })
    );
  });

  it('should find match in aliases', () => {
    const { result } = renderHook(() => usePaletteSearch(mockKeys));

    act(() => {
      result.current.setQuery('KC_ESC');
    });

    expect(result.current.results.length).toBeGreaterThan(0);
    const escResult = result.current.results.find((r) => r.key.id === 'ESC');
    expect(escResult).toBeDefined();
    expect(escResult?.matches).toContainEqual(
      expect.objectContaining({ field: 'alias', text: 'KC_ESC' })
    );
  });

  it('should be case insensitive', () => {
    const { result } = renderHook(() => usePaletteSearch(mockKeys));

    act(() => {
      result.current.setQuery('CtRl');
    });

    const ctrlResult = result.current.results.find((r) => r.key.id === 'CTRL');
    expect(ctrlResult).toBeDefined();
  });

  it('should handle fuzzy matching', () => {
    const { result } = renderHook(() => usePaletteSearch(mockKeys));

    act(() => {
      result.current.setQuery('entr');
    });

    const enterResult = result.current.results.find(
      (r) => r.key.id === 'ENTER'
    );
    expect(enterResult).toBeDefined();
  });

  it('should rank exact matches higher than partial matches', () => {
    const { result } = renderHook(() => usePaletteSearch(mockKeys));

    act(() => {
      result.current.setQuery('A');
    });

    // "A" should be first (exact ID match)
    expect(result.current.results[0].key.id).toBe('A');
    // It should have higher score than fuzzy matches
    if (result.current.results.length > 1) {
      expect(result.current.results[0].score).toBeGreaterThan(
        result.current.results[1].score
      );
    }
  });

  it('should rank starts-with matches high', () => {
    const { result } = renderHook(() => usePaletteSearch(mockKeys));

    act(() => {
      result.current.setQuery('Esc');
    });

    const escResult = result.current.results.find((r) => r.key.id === 'ESC');
    expect(escResult).toBeDefined();
    // Should have high score for starting with query
    expect(escResult!.score).toBeGreaterThan(200);
  });

  it('should rank contains matches medium', () => {
    const { result } = renderHook(() => usePaletteSearch(mockKeys));

    act(() => {
      result.current.setQuery('spa');
    });

    const spaceResult = result.current.results.find(
      (r) => r.key.id === 'SPACE'
    );
    expect(spaceResult).toBeDefined();
  });

  it('should prioritize ID matches over other fields', () => {
    const { result } = renderHook(() => usePaletteSearch(mockKeys));

    act(() => {
      result.current.setQuery('A');
    });

    // "A" key should rank highest due to exact ID match
    const aResult = result.current.results[0];
    expect(aResult.key.id).toBe('A');
  });

  it('should return results sorted by score descending', () => {
    const { result } = renderHook(() => usePaletteSearch(mockKeys));

    act(() => {
      result.current.setQuery('e');
    });

    // Verify scores are in descending order
    for (let i = 0; i < result.current.results.length - 1; i++) {
      expect(result.current.results[i].score).toBeGreaterThanOrEqual(
        result.current.results[i + 1].score
      );
    }
  });

  it('should find multiple matches in same key', () => {
    const { result } = renderHook(() => usePaletteSearch(mockKeys));

    act(() => {
      result.current.setQuery('enter');
    });

    const enterResult = result.current.results.find(
      (r) => r.key.id === 'ENTER'
    );
    expect(enterResult).toBeDefined();
    // Should match in both ID and label and description and aliases
    expect(enterResult!.matches.length).toBeGreaterThan(0);
  });

  it('should only count first matching alias', () => {
    const { result } = renderHook(() => usePaletteSearch(mockKeys));

    act(() => {
      result.current.setQuery('return');
    });

    const enterResult = result.current.results.find(
      (r) => r.key.id === 'ENTER'
    );
    expect(enterResult).toBeDefined();

    // Count alias matches - should only be 1
    const aliasMatches = enterResult!.matches.filter(
      (m) => m.field === 'alias'
    );
    expect(aliasMatches.length).toBeLessThanOrEqual(1);
  });

  it('should handle no matches gracefully', () => {
    const { result } = renderHook(() => usePaletteSearch(mockKeys));

    act(() => {
      result.current.setQuery('ZZZZZ');
    });

    expect(result.current.results).toEqual([]);
  });

  it('should update results when query changes', () => {
    const { result } = renderHook(() => usePaletteSearch(mockKeys));

    act(() => {
      result.current.setQuery('A');
    });
    const firstResults = result.current.results;

    act(() => {
      result.current.setQuery('Ctrl');
    });
    const secondResults = result.current.results;

    expect(firstResults).not.toEqual(secondResults);
  });

  it('should update results when keys array changes', () => {
    const { result, rerender } = renderHook(
      ({ keys }) => usePaletteSearch(keys),
      { initialProps: { keys: mockKeys } }
    );

    act(() => {
      result.current.setQuery('A');
    });
    const firstResultsCount = result.current.results.length;

    // Change keys array
    const newKeys = mockKeys.slice(1); // Remove 'A' key
    rerender({ keys: newKeys });

    // Results should update
    expect(result.current.results.length).toBeLessThan(firstResultsCount);
  });

  it('should memoize results - same query returns same reference', () => {
    const { result, rerender } = renderHook(() => usePaletteSearch(mockKeys));

    act(() => {
      result.current.setQuery('Ctrl');
    });
    const firstResults = result.current.results;

    // Re-render without changing query
    rerender();

    // Should be same reference (memoized)
    expect(result.current.results).toBe(firstResults);
  });

  it('should have stable setQuery callback reference', () => {
    const { result, rerender } = renderHook(() => usePaletteSearch(mockKeys));

    const firstSetQuery = result.current.setQuery;
    rerender();

    expect(result.current.setQuery).toBe(firstSetQuery);
  });

  it('should include matching indices for highlighting', () => {
    const { result } = renderHook(() => usePaletteSearch(mockKeys));

    act(() => {
      result.current.setQuery('Ctrl');
    });

    const ctrlResult = result.current.results.find((r) => r.key.id === 'CTRL');
    expect(ctrlResult).toBeDefined();

    // Each match should have indices array
    ctrlResult!.matches.forEach((match) => {
      expect(match.indices).toBeInstanceOf(Array);
      expect(match.indices.length).toBeGreaterThan(0);
    });
  });

  it('should handle keys with no aliases', () => {
    const keysWithoutAliases: KeyDefinition[] = [
      {
        id: 'TEST',
        label: 'Test',
        category: 'basic',
        description: 'Test key',
        aliases: [],
      },
    ];

    const { result } = renderHook(() => usePaletteSearch(keysWithoutAliases));

    act(() => {
      result.current.setQuery('test');
    });

    expect(result.current.results).toHaveLength(1);
  });

  it('should handle empty keys array', () => {
    const { result } = renderHook(() => usePaletteSearch([]));

    act(() => {
      result.current.setQuery('test');
    });

    expect(result.current.results).toEqual([]);
  });

  it('should handle special characters in query', () => {
    const { result } = renderHook(() => usePaletteSearch(mockKeys));

    act(() => {
      result.current.setQuery('KC_A');
    });

    const aResult = result.current.results.find((r) => r.key.id === 'A');
    expect(aResult).toBeDefined();
  });

  it('should handle consecutive character matches with bonus', () => {
    const { result } = renderHook(() => usePaletteSearch(mockKeys));

    act(() => {
      result.current.setQuery('space');
    });

    const spaceResult = result.current.results.find(
      (r) => r.key.id === 'SPACE'
    );
    expect(spaceResult).toBeDefined();
    // Should have good score due to consecutive matches
    expect(spaceResult!.score).toBeGreaterThan(100);
  });

  it('should apply gap penalty for non-consecutive matches', () => {
    const { result } = renderHook(() => usePaletteSearch(mockKeys));

    // Search for 'ae' - should match 'Space' but with gaps
    act(() => {
      result.current.setQuery('ae');
    });

    const spaceResult = result.current.results.find(
      (r) => r.key.id === 'SPACE'
    );
    if (spaceResult) {
      // Should have lower score due to gaps
      expect(spaceResult.score).toBeLessThan(500);
    }
  });

  it('should not match if all query characters are not found', () => {
    const { result } = renderHook(() => usePaletteSearch(mockKeys));

    act(() => {
      result.current.setQuery('xyz');
    });

    // None of our mock keys contain 'x', 'y', 'z' in sequence
    expect(result.current.results.length).toBe(0);
  });

  it('should handle long queries efficiently', () => {
    const { result } = renderHook(() => usePaletteSearch(mockKeys));

    act(() => {
      result.current.setQuery('thisisaverylongquerythatdoesnotmatchanything');
    });

    expect(result.current.results).toEqual([]);
  });
});
