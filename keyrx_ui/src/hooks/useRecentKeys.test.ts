import { renderHook, act } from '@testing-library/react';
import { useRecentKeys } from './useRecentKeys';

// Mock localStorage
const localStorageMock = (() => {
  let store: Record<string, string> = {};

  return {
    getItem: (key: string) => store[key] || null,
    setItem: (key: string, value: string) => {
      store[key] = value;
    },
    removeItem: (key: string) => {
      delete store[key];
    },
    clear: () => {
      store = {};
    },
  };
})();

Object.defineProperty(window, 'localStorage', {
  value: localStorageMock,
});

describe('useRecentKeys', () => {
  beforeEach(() => {
    localStorageMock.clear();
  });

  it('should initialize with empty array when localStorage is empty', () => {
    const { result } = renderHook(() => useRecentKeys());
    expect(result.current.recentKeys).toEqual([]);
  });

  it('should load existing recent keys from localStorage', () => {
    const existingKeys = ['KEY_A', 'KEY_B', 'KEY_C'];
    localStorageMock.setItem('keyrx_recent_keys', JSON.stringify(existingKeys));

    const { result } = renderHook(() => useRecentKeys());
    expect(result.current.recentKeys).toEqual(existingKeys);
  });

  it('should add a key to recent keys', () => {
    const { result } = renderHook(() => useRecentKeys());

    act(() => {
      result.current.addRecentKey('KEY_A');
    });

    expect(result.current.recentKeys).toEqual(['KEY_A']);
  });

  it('should add multiple keys in order', () => {
    const { result } = renderHook(() => useRecentKeys());

    act(() => {
      result.current.addRecentKey('KEY_A');
      result.current.addRecentKey('KEY_B');
      result.current.addRecentKey('KEY_C');
    });

    expect(result.current.recentKeys).toEqual(['KEY_C', 'KEY_B', 'KEY_A']);
  });

  it('should move existing key to front when added again', () => {
    const { result } = renderHook(() => useRecentKeys());

    act(() => {
      result.current.addRecentKey('KEY_A');
      result.current.addRecentKey('KEY_B');
      result.current.addRecentKey('KEY_C');
      result.current.addRecentKey('KEY_A'); // Re-add KEY_A
    });

    expect(result.current.recentKeys).toEqual(['KEY_A', 'KEY_C', 'KEY_B']);
  });

  it('should enforce FIFO with max 10 keys', () => {
    const { result } = renderHook(() => useRecentKeys());

    act(() => {
      // Add 12 keys
      for (let i = 0; i < 12; i++) {
        result.current.addRecentKey(`KEY_${i}`);
      }
    });

    // Should only keep last 10 keys (most recent first)
    expect(result.current.recentKeys).toHaveLength(10);
    expect(result.current.recentKeys[0]).toBe('KEY_11');
    expect(result.current.recentKeys[9]).toBe('KEY_2');
  });

  it('should persist to localStorage when adding keys', () => {
    const { result } = renderHook(() => useRecentKeys());

    act(() => {
      result.current.addRecentKey('KEY_A');
      result.current.addRecentKey('KEY_B');
    });

    const stored = localStorageMock.getItem('keyrx_recent_keys');
    expect(stored).toBe(JSON.stringify(['KEY_B', 'KEY_A']));
  });

  it('should clear all recent keys', () => {
    const { result } = renderHook(() => useRecentKeys());

    act(() => {
      result.current.addRecentKey('KEY_A');
      result.current.addRecentKey('KEY_B');
    });

    expect(result.current.recentKeys).toHaveLength(2);

    act(() => {
      result.current.clearRecentKeys();
    });

    expect(result.current.recentKeys).toEqual([]);
    expect(localStorageMock.getItem('keyrx_recent_keys')).toBe(
      JSON.stringify([])
    );
  });

  it('should handle localStorage errors gracefully on load', () => {
    // Mock getItem to throw an error
    const originalGetItem = localStorageMock.getItem;
    localStorageMock.getItem = () => {
      throw new Error('Storage error');
    };

    const { result } = renderHook(() => useRecentKeys());
    expect(result.current.recentKeys).toEqual([]);

    // Restore
    localStorageMock.getItem = originalGetItem;
  });

  it('should handle localStorage errors gracefully on save', () => {
    // Mock setItem to throw an error
    const originalSetItem = localStorageMock.setItem;
    localStorageMock.setItem = () => {
      throw new Error('Storage error');
    };

    const { result } = renderHook(() => useRecentKeys());

    // Should not throw
    act(() => {
      result.current.addRecentKey('KEY_A');
    });

    // State should still update
    expect(result.current.recentKeys).toEqual(['KEY_A']);

    // Restore
    localStorageMock.setItem = originalSetItem;
  });

  it('should handle invalid JSON in localStorage', () => {
    localStorageMock.setItem('keyrx_recent_keys', 'invalid json');

    const { result } = renderHook(() => useRecentKeys());
    expect(result.current.recentKeys).toEqual([]);
  });

  it('should handle non-array data in localStorage', () => {
    localStorageMock.setItem(
      'keyrx_recent_keys',
      JSON.stringify({ key: 'value' })
    );

    const { result } = renderHook(() => useRecentKeys());
    expect(result.current.recentKeys).toEqual([]);
  });

  it('should have stable callback references', () => {
    const { result, rerender } = renderHook(() => useRecentKeys());

    const firstAddRef = result.current.addRecentKey;
    const firstClearRef = result.current.clearRecentKeys;

    rerender();

    expect(result.current.addRecentKey).toBe(firstAddRef);
    expect(result.current.clearRecentKeys).toBe(firstClearRef);
  });
});
