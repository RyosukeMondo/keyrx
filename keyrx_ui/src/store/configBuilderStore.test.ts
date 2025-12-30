/**
 * Unit tests for configBuilderStore error handling
 */

import { renderHook, act } from '@testing-library/react';
import { useConfigBuilderStore } from './configBuilderStore';

describe('configBuilderStore - Error Management', () => {
  beforeEach(() => {
    // Reset store to initial state before each test
    const { result } = renderHook(() => useConfigBuilderStore());
    act(() => {
      result.current.resetConfig();
    });
  });

  describe('Error state initialization', () => {
    it('should initialize with no error', () => {
      const { result } = renderHook(() => useConfigBuilderStore());
      expect(result.current.lastError).toBeNull();
    });
  });

  describe('setError action', () => {
    it('should set error message', () => {
      const { result } = renderHook(() => useConfigBuilderStore());

      act(() => {
        result.current.setError('Test error message');
      });

      expect(result.current.lastError).toBe('Test error message');
    });

    it('should allow setting null error', () => {
      const { result } = renderHook(() => useConfigBuilderStore());

      act(() => {
        result.current.setError('Error');
        result.current.setError(null);
      });

      expect(result.current.lastError).toBeNull();
    });

    it('should overwrite previous error', () => {
      const { result } = renderHook(() => useConfigBuilderStore());

      act(() => {
        result.current.setError('First error');
        result.current.setError('Second error');
      });

      expect(result.current.lastError).toBe('Second error');
    });
  });

  describe('clearError action', () => {
    it('should clear error message', () => {
      const { result } = renderHook(() => useConfigBuilderStore());

      act(() => {
        result.current.setError('Test error');
        result.current.clearError();
      });

      expect(result.current.lastError).toBeNull();
    });

    it('should be idempotent when no error exists', () => {
      const { result } = renderHook(() => useConfigBuilderStore());

      act(() => {
        result.current.clearError();
        result.current.clearError();
      });

      expect(result.current.lastError).toBeNull();
    });
  });

  describe('removeLayer action - Error propagation', () => {
    it('should set error when trying to remove base layer', () => {
      const { result } = renderHook(() => useConfigBuilderStore());
      const baseLayer = result.current.layers.find((l) => l.isBase);

      act(() => {
        result.current.removeLayer(baseLayer!.id);
      });

      expect(result.current.lastError).toBe('Cannot remove the base layer');
      expect(result.current.layers).toHaveLength(1); // Base layer still exists
    });

    it('should clear error when successfully removing non-base layer', () => {
      const { result } = renderHook(() => useConfigBuilderStore());

      // Add a non-base layer
      act(() => {
        result.current.addLayer('test-layer');
      });

      const addedLayer = result.current.layers.find((l) => !l.isBase);

      // Set an error first
      act(() => {
        result.current.setError('Previous error');
      });

      // Remove the non-base layer
      act(() => {
        result.current.removeLayer(addedLayer!.id);
      });

      expect(result.current.lastError).toBeNull();
      expect(result.current.layers).toHaveLength(1); // Only base layer remains
    });

    it('should not modify layers when trying to remove base layer', () => {
      const { result } = renderHook(() => useConfigBuilderStore());
      const initialLayers = result.current.layers;
      const baseLayer = initialLayers.find((l) => l.isBase);

      act(() => {
        result.current.removeLayer(baseLayer!.id);
      });

      expect(result.current.layers).toEqual(initialLayers);
    });

    it('should preserve other state when setting error', () => {
      const { result } = renderHook(() => useConfigBuilderStore());
      const baseLayer = result.current.layers.find((l) => l.isBase);

      // Set some state
      act(() => {
        result.current.markDirty();
        result.current.addModifier('test-mod', 'KEY_A');
      });

      const isDirtyBefore = result.current.isDirty;
      const modifiersBefore = result.current.modifiers;

      act(() => {
        result.current.removeLayer(baseLayer!.id);
      });

      expect(result.current.isDirty).toBe(isDirtyBefore);
      expect(result.current.modifiers).toEqual(modifiersBefore);
    });
  });

  describe('resetConfig action', () => {
    it('should clear error on reset', () => {
      const { result } = renderHook(() => useConfigBuilderStore());

      act(() => {
        result.current.setError('Test error');
        result.current.resetConfig();
      });

      expect(result.current.lastError).toBeNull();
    });
  });

  describe('setConfig action', () => {
    it('should preserve error state when config includes it', () => {
      const { result } = renderHook(() => useConfigBuilderStore());

      const newConfig = {
        layers: [
          {
            id: 'base',
            name: 'base',
            mappings: [],
            isBase: true,
          },
        ],
        modifiers: [],
        locks: [],
        currentLayerId: 'base',
        isDirty: false,
        lastError: 'Config error',
      };

      act(() => {
        result.current.setConfig(newConfig);
      });

      expect(result.current.lastError).toBe('Config error');
    });

    it('should clear error when config explicitly sets it to null', () => {
      const { result } = renderHook(() => useConfigBuilderStore());

      const newConfig = {
        layers: [
          {
            id: 'base',
            name: 'base',
            mappings: [],
            isBase: true,
          },
        ],
        modifiers: [],
        locks: [],
        currentLayerId: 'base',
        isDirty: false,
        lastError: null,
      };

      act(() => {
        result.current.setError('Previous error');
        result.current.setConfig(newConfig);
      });

      // Error should be cleared since newConfig explicitly sets lastError to null
      expect(result.current.lastError).toBeNull();
    });
  });
});
