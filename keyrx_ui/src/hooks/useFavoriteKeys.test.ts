import { renderHook, act } from '@testing-library/react';
import { useFavoriteKeys } from './useFavoriteKeys';

describe('useFavoriteKeys', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  afterEach(() => {
    localStorage.clear();
  });

  it('initializes with empty array when no stored data', () => {
    const { result } = renderHook(() => useFavoriteKeys());
    expect(result.current.favoriteKeys).toEqual([]);
  });

  it('loads favorites from localStorage on initialization', () => {
    localStorage.setItem(
      'keyrx_favorite_keys',
      JSON.stringify(['KeyA', 'KeyB'])
    );
    const { result } = renderHook(() => useFavoriteKeys());
    expect(result.current.favoriteKeys).toEqual(['KeyA', 'KeyB']);
  });

  it('adds a key to favorites', () => {
    const { result } = renderHook(() => useFavoriteKeys());

    act(() => {
      result.current.toggleFavorite('KeyA');
    });

    expect(result.current.favoriteKeys).toEqual(['KeyA']);
    expect(result.current.isFavorite('KeyA')).toBe(true);
  });

  it('removes a key from favorites', () => {
    localStorage.setItem(
      'keyrx_favorite_keys',
      JSON.stringify(['KeyA', 'KeyB'])
    );
    const { result } = renderHook(() => useFavoriteKeys());

    act(() => {
      result.current.toggleFavorite('KeyA');
    });

    expect(result.current.favoriteKeys).toEqual(['KeyB']);
    expect(result.current.isFavorite('KeyA')).toBe(false);
  });

  it('toggles a key on and off', () => {
    const { result } = renderHook(() => useFavoriteKeys());

    // Toggle on
    act(() => {
      result.current.toggleFavorite('KeyA');
    });
    expect(result.current.favoriteKeys).toEqual(['KeyA']);
    expect(result.current.isFavorite('KeyA')).toBe(true);

    // Toggle off
    act(() => {
      result.current.toggleFavorite('KeyA');
    });
    expect(result.current.favoriteKeys).toEqual([]);
    expect(result.current.isFavorite('KeyA')).toBe(false);
  });

  it('persists favorites to localStorage', () => {
    const { result } = renderHook(() => useFavoriteKeys());

    act(() => {
      result.current.toggleFavorite('KeyA');
      result.current.toggleFavorite('KeyB');
    });

    const stored = localStorage.getItem('keyrx_favorite_keys');
    expect(stored).toBe(JSON.stringify(['KeyA', 'KeyB']));
  });

  it('maintains favorites order', () => {
    const { result } = renderHook(() => useFavoriteKeys());

    act(() => {
      result.current.toggleFavorite('KeyA');
      result.current.toggleFavorite('KeyB');
      result.current.toggleFavorite('KeyC');
    });

    expect(result.current.favoriteKeys).toEqual(['KeyA', 'KeyB', 'KeyC']);
  });

  it('isFavorite returns false for non-favorite keys', () => {
    const { result } = renderHook(() => useFavoriteKeys());

    act(() => {
      result.current.toggleFavorite('KeyA');
    });

    expect(result.current.isFavorite('KeyA')).toBe(true);
    expect(result.current.isFavorite('KeyB')).toBe(false);
    expect(result.current.isFavorite('KeyC')).toBe(false);
  });

  it('handles corrupted localStorage data gracefully', () => {
    localStorage.setItem('keyrx_favorite_keys', 'invalid json');
    const { result } = renderHook(() => useFavoriteKeys());
    expect(result.current.favoriteKeys).toEqual([]);
  });

  it('handles non-array data in localStorage', () => {
    localStorage.setItem(
      'keyrx_favorite_keys',
      JSON.stringify({ not: 'array' })
    );
    const { result } = renderHook(() => useFavoriteKeys());
    expect(result.current.favoriteKeys).toEqual([]);
  });

  it('handles localStorage errors during save', () => {
    const consoleErrorSpy = vi
      .spyOn(console, 'error')
      .mockImplementation(() => {});
    const setItemSpy = vi
      .spyOn(Storage.prototype, 'setItem')
      .mockImplementation(() => {
        throw new Error('Storage quota exceeded');
      });

    const { result } = renderHook(() => useFavoriteKeys());

    act(() => {
      result.current.toggleFavorite('KeyA');
    });

    expect(consoleErrorSpy).toHaveBeenCalledWith(
      'Failed to save keyrx_favorite_keys to localStorage:',
      expect.any(Error)
    );

    consoleErrorSpy.mockRestore();
    setItemSpy.mockRestore();
  });

  it('handles localStorage errors during load', () => {
    const consoleWarnSpy = vi
      .spyOn(console, 'warn')
      .mockImplementation(() => {});
    const getItemSpy = vi
      .spyOn(Storage.prototype, 'getItem')
      .mockImplementation(() => {
        throw new Error('Storage unavailable');
      });

    const { result } = renderHook(() => useFavoriteKeys());

    expect(result.current.favoriteKeys).toEqual([]);
    expect(consoleWarnSpy).toHaveBeenCalledWith(
      'Failed to load keyrx_favorite_keys from localStorage:',
      expect.any(Error)
    );

    consoleWarnSpy.mockRestore();
    getItemSpy.mockRestore();
  });

  it('toggleFavorite callback remains stable', () => {
    const { result, rerender } = renderHook(() => useFavoriteKeys());
    const firstToggleFavorite = result.current.toggleFavorite;

    act(() => {
      result.current.toggleFavorite('KeyA');
    });

    rerender();

    expect(result.current.toggleFavorite).toBe(firstToggleFavorite);
  });

  it('isFavorite callback updates with state changes', () => {
    const { result } = renderHook(() => useFavoriteKeys());
    const firstIsFavorite = result.current.isFavorite;

    act(() => {
      result.current.toggleFavorite('KeyA');
    });

    // isFavorite is memoized on favoriteKeys, so it should be a new reference
    expect(result.current.isFavorite).not.toBe(firstIsFavorite);
    expect(result.current.isFavorite('KeyA')).toBe(true);
  });

  it('handles multiple favorite toggles in sequence', () => {
    const { result } = renderHook(() => useFavoriteKeys());

    act(() => {
      result.current.toggleFavorite('KeyA');
      result.current.toggleFavorite('KeyB');
      result.current.toggleFavorite('KeyC');
      result.current.toggleFavorite('KeyA'); // Remove KeyA
      result.current.toggleFavorite('KeyD');
    });

    expect(result.current.favoriteKeys).toEqual(['KeyB', 'KeyC', 'KeyD']);
    expect(result.current.isFavorite('KeyA')).toBe(false);
    expect(result.current.isFavorite('KeyB')).toBe(true);
    expect(result.current.isFavorite('KeyC')).toBe(true);
    expect(result.current.isFavorite('KeyD')).toBe(true);
  });
});
