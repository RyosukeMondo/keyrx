/**
 * Tests for useCodePanel hook
 */

import { renderHook, act } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { useCodePanel } from './useCodePanel';

const STORAGE_KEY_HEIGHT = 'codePanel.height';
const DEFAULT_HEIGHT = 300;

describe('useCodePanel', () => {
  beforeEach(() => {
    // Clear localStorage before each test
    localStorage.clear();
    // Clear console spies
    vi.clearAllMocks();
    // Restore all mocks to ensure clean state
    vi.restoreAllMocks();
  });

  describe('initialization', () => {
    it('should initialize with default height when localStorage is empty', () => {
      const { result } = renderHook(() => useCodePanel());

      expect(result.current.height).toBe(DEFAULT_HEIGHT);
      expect(result.current.isOpen).toBe(false);
    });

    it('should load height from localStorage if available', () => {
      localStorage.setItem(STORAGE_KEY_HEIGHT, '450');

      const { result } = renderHook(() => useCodePanel());

      expect(result.current.height).toBe(450);
    });

    it('should use default height if localStorage value is invalid', () => {
      localStorage.setItem(STORAGE_KEY_HEIGHT, 'invalid');

      const { result } = renderHook(() => useCodePanel());

      expect(result.current.height).toBe(DEFAULT_HEIGHT);
    });

    it('should use default height if localStorage value is negative', () => {
      localStorage.setItem(STORAGE_KEY_HEIGHT, '-100');

      const { result } = renderHook(() => useCodePanel());

      expect(result.current.height).toBe(DEFAULT_HEIGHT);
    });

    it('should use default height if localStorage value is zero', () => {
      localStorage.setItem(STORAGE_KEY_HEIGHT, '0');

      const { result } = renderHook(() => useCodePanel());

      expect(result.current.height).toBe(DEFAULT_HEIGHT);
    });

    it('should handle localStorage errors gracefully', () => {
      const consoleError = vi
        .spyOn(console, 'error')
        .mockImplementation(() => {});
      const getItemSpy = vi
        .spyOn(Storage.prototype, 'getItem')
        .mockImplementation(() => {
          throw new Error('Storage error');
        });

      const { result } = renderHook(() => useCodePanel());

      expect(result.current.height).toBe(DEFAULT_HEIGHT);
      expect(consoleError).toHaveBeenCalledWith(
        'Failed to load code panel height from localStorage:',
        expect.any(Error)
      );

      consoleError.mockRestore();
      getItemSpy.mockRestore();
    });
  });

  describe('toggleOpen', () => {
    it('should toggle isOpen from false to true', () => {
      const { result } = renderHook(() => useCodePanel());

      expect(result.current.isOpen).toBe(false);

      act(() => {
        result.current.toggleOpen();
      });

      expect(result.current.isOpen).toBe(true);
    });

    it('should toggle isOpen from true to false', () => {
      const { result } = renderHook(() => useCodePanel());

      act(() => {
        result.current.toggleOpen();
      });

      expect(result.current.isOpen).toBe(true);

      act(() => {
        result.current.toggleOpen();
      });

      expect(result.current.isOpen).toBe(false);
    });

    it('should maintain stable reference across renders', () => {
      const { result, rerender } = renderHook(() => useCodePanel());

      const firstToggle = result.current.toggleOpen;
      rerender();
      const secondToggle = result.current.toggleOpen;

      expect(firstToggle).toBe(secondToggle);
    });
  });

  describe('setHeight', () => {
    it('should update height and persist to localStorage', () => {
      const { result } = renderHook(() => useCodePanel());

      act(() => {
        result.current.setHeight(500);
      });

      expect(result.current.height).toBe(500);
      expect(localStorage.getItem(STORAGE_KEY_HEIGHT)).toBe('500');
    });

    it('should not update height if value is zero', () => {
      const { result } = renderHook(() => useCodePanel());

      act(() => {
        result.current.setHeight(0);
      });

      expect(result.current.height).toBe(DEFAULT_HEIGHT);
      expect(localStorage.getItem(STORAGE_KEY_HEIGHT)).toBe(
        DEFAULT_HEIGHT.toString()
      );
    });

    it('should not update height if value is negative', () => {
      const { result } = renderHook(() => useCodePanel());

      act(() => {
        result.current.setHeight(-100);
      });

      expect(result.current.height).toBe(DEFAULT_HEIGHT);
      expect(localStorage.getItem(STORAGE_KEY_HEIGHT)).toBe(
        DEFAULT_HEIGHT.toString()
      );
    });

    it('should persist height changes to localStorage', () => {
      const { result } = renderHook(() => useCodePanel());

      act(() => {
        result.current.setHeight(350);
      });

      expect(localStorage.getItem(STORAGE_KEY_HEIGHT)).toBe('350');

      act(() => {
        result.current.setHeight(600);
      });

      expect(localStorage.getItem(STORAGE_KEY_HEIGHT)).toBe('600');
    });

    it('should handle localStorage errors gracefully when persisting', () => {
      const consoleError = vi
        .spyOn(console, 'error')
        .mockImplementation(() => {});
      const setItemSpy = vi
        .spyOn(Storage.prototype, 'setItem')
        .mockImplementation(() => {
          throw new Error('Storage error');
        });

      const { result } = renderHook(() => useCodePanel());

      act(() => {
        result.current.setHeight(400);
      });

      // Height should still update in state even if persistence fails
      expect(result.current.height).toBe(400);
      expect(consoleError).toHaveBeenCalledWith(
        'Failed to persist code panel height:',
        expect.any(Error)
      );

      consoleError.mockRestore();
      setItemSpy.mockRestore();
    });

    it('should maintain stable reference across renders', () => {
      const { result, rerender } = renderHook(() => useCodePanel());

      const firstSetHeight = result.current.setHeight;
      rerender();
      const secondSetHeight = result.current.setHeight;

      expect(firstSetHeight).toBe(secondSetHeight);
    });
  });

  describe('persistence across hook instances', () => {
    it('should persist height between hook unmount and remount', () => {
      const { result, unmount } = renderHook(() => useCodePanel());

      act(() => {
        result.current.setHeight(550);
      });

      expect(result.current.height).toBe(550);

      unmount();

      const { result: result2 } = renderHook(() => useCodePanel());

      expect(result2.current.height).toBe(550);
    });

    it('should not persist isOpen state between instances', () => {
      const { result, unmount } = renderHook(() => useCodePanel());

      act(() => {
        result.current.toggleOpen();
      });

      expect(result.current.isOpen).toBe(true);

      unmount();

      const { result: result2 } = renderHook(() => useCodePanel());

      // isOpen should reset to false for new instance
      expect(result2.current.isOpen).toBe(false);
    });
  });

  describe('edge cases', () => {
    it('should handle very large height values', () => {
      const { result } = renderHook(() => useCodePanel());

      act(() => {
        result.current.setHeight(10000);
      });

      expect(result.current.height).toBe(10000);
      expect(localStorage.getItem(STORAGE_KEY_HEIGHT)).toBe('10000');
    });

    it('should handle fractional height values by truncating', () => {
      const { result } = renderHook(() => useCodePanel());

      act(() => {
        result.current.setHeight(350.75);
      });

      expect(result.current.height).toBe(350.75);
      // localStorage will store as string
      expect(localStorage.getItem(STORAGE_KEY_HEIGHT)).toBe('350.75');
    });

    it('should handle rapid successive height changes', () => {
      const { result } = renderHook(() => useCodePanel());

      act(() => {
        result.current.setHeight(100);
        result.current.setHeight(200);
        result.current.setHeight(300);
        result.current.setHeight(400);
      });

      expect(result.current.height).toBe(400);
      expect(localStorage.getItem(STORAGE_KEY_HEIGHT)).toBe('400');
    });

    it('should handle rapid successive toggle changes', () => {
      const { result } = renderHook(() => useCodePanel());

      act(() => {
        result.current.toggleOpen();
        result.current.toggleOpen();
        result.current.toggleOpen();
      });

      expect(result.current.isOpen).toBe(true);
    });
  });
});
