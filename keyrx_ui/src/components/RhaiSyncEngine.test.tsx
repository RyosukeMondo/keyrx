/**
 * Tests for RhaiSyncEngine component
 *
 * These tests verify the bidirectional synchronization engine between
 * visual editor and code editor, including:
 * - Visual to code sync (immediate)
 * - Code to visual sync (debounced 500ms)
 * - State machine transitions
 * - Error handling and recovery
 * - LocalStorage persistence
 * - Debouncing behavior
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook, act, waitFor } from '@testing-library/react';
import { useRhaiSyncEngine } from './RhaiSyncEngine';
import type { RhaiAST } from '../utils/rhaiParser';

// Mock the parser and code generator
vi.mock('../utils/rhaiParser', () => ({
  parseRhaiScript: vi.fn((code: string) => {
    // Mock successful parse for valid code
    if (code.includes('map("VK_A", "VK_B")')) {
      return {
        success: true,
        ast: {
          imports: [],
          globalMappings: [
            {
              type: 'simple',
              sourceKey: 'VK_A',
              targetKey: 'VK_B',
              line: 1,
            },
          ],
          deviceBlocks: [],
          comments: [],
        },
      };
    }
    // Mock parse error for invalid code
    if (code.includes('invalid')) {
      return {
        success: false,
        error: {
          line: 1,
          column: 1,
          message: 'Unexpected token',
          suggestion: 'Check syntax',
        },
      };
    }
    // Empty script
    return {
      success: true,
      ast: {
        imports: [],
        globalMappings: [],
        deviceBlocks: [],
        comments: [],
      },
    };
  }),
}));

vi.mock('../utils/rhaiCodeGen', () => ({
  generateRhaiScript: vi.fn((ast: RhaiAST) => {
    if (ast.globalMappings.length > 0) {
      const mapping = ast.globalMappings[0];
      if (mapping.type === 'simple' && mapping.targetKey) {
        return `map("${mapping.sourceKey}", "${mapping.targetKey}");`;
      }
    }
    return '';
  }),
}));

describe('useRhaiSyncEngine', () => {
  let localStorageMock: { [key: string]: string } = {};

  beforeEach(() => {
    // Reset localStorage mock
    localStorageMock = {};
    vi.spyOn(Storage.prototype, 'getItem').mockImplementation(
      (key: string) => localStorageMock[key] || null
    );
    vi.spyOn(Storage.prototype, 'setItem').mockImplementation(
      (key: string, value: string) => {
        localStorageMock[key] = value;
      }
    );
    vi.spyOn(Storage.prototype, 'removeItem').mockImplementation(
      (key: string) => {
        delete localStorageMock[key];
      }
    );

    // Use fake timers for debounce testing
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.restoreAllMocks();
    vi.useRealTimers();
  });

  describe('initialization', () => {
    it('should initialize with idle state', () => {
      const { result } = renderHook(() =>
        useRhaiSyncEngine({ storageKey: 'test-profile' })
      );

      expect(result.current.state).toBe('idle');
      expect(result.current.direction).toBe('none');
      expect(result.current.error).toBeNull();
      expect(result.current.lastValidAST).toBeNull();
    });

    it('should restore from localStorage if data exists and is fresh', () => {
      const savedData = {
        code: 'map("VK_A", "VK_B");',
        timestamp: Date.now() - 1000, // 1 second ago
      };
      localStorageMock['rhai-sync-test-profile'] = JSON.stringify(savedData);

      const { result } = renderHook(() =>
        useRhaiSyncEngine({ storageKey: 'test-profile' })
      );

      expect(result.current.getCode()).toBe('map("VK_A", "VK_B");');
      expect(result.current.lastValidAST).not.toBeNull();
    });

    it('should not restore from localStorage if data is stale (>24 hours)', () => {
      const savedData = {
        code: 'map("VK_A", "VK_B");',
        timestamp: Date.now() - 25 * 60 * 60 * 1000, // 25 hours ago
      };
      localStorageMock['rhai-sync-test-profile'] = JSON.stringify(savedData);

      const { result } = renderHook(() =>
        useRhaiSyncEngine({ storageKey: 'test-profile' })
      );

      expect(result.current.getCode()).toBe('');
      expect(localStorageMock['rhai-sync-test-profile']).toBeUndefined();
    });

    it('should handle localStorage errors gracefully', () => {
      vi.spyOn(Storage.prototype, 'getItem').mockImplementation(() => {
        throw new Error('localStorage error');
      });
      const consoleWarnSpy = vi
        .spyOn(console, 'warn')
        .mockImplementation(() => {});

      const { result } = renderHook(() =>
        useRhaiSyncEngine({ storageKey: 'test-profile' })
      );

      expect(result.current.state).toBe('idle');
      expect(consoleWarnSpy).toHaveBeenCalled();
    });
  });

  describe('visual to code sync', () => {
    it('should generate code immediately when visual changes', async () => {
      const { result } = renderHook(() =>
        useRhaiSyncEngine({ storageKey: 'test-profile' })
      );

      const ast: RhaiAST = {
        imports: [],
        globalMappings: [
          {
            type: 'simple',
            sourceKey: 'VK_A',
            targetKey: 'VK_B',
            line: 1,
          },
        ],
        deviceBlocks: [],
        comments: [],
      };

      act(() => {
        result.current.onVisualChange(ast);
      });

      // State should transition to generating
      expect(result.current.state).toBe('generating');

      // Wait for async operations
      await act(async () => {
        await vi.advanceTimersByTimeAsync(20);
      });

      // Should return to idle
      expect(result.current.state).toBe('idle');
      expect(result.current.direction).toBe('none');
      expect(result.current.getCode()).toBe('map("VK_A", "VK_B");');
      expect(result.current.lastValidAST).toEqual(ast);
    });

    it('should persist code to localStorage on visual change', async () => {
      const { result } = renderHook(() =>
        useRhaiSyncEngine({ storageKey: 'test-profile' })
      );

      const ast: RhaiAST = {
        imports: [],
        globalMappings: [
          {
            type: 'simple',
            sourceKey: 'VK_A',
            targetKey: 'VK_B',
            line: 1,
          },
        ],
        deviceBlocks: [],
        comments: [],
      };

      act(() => {
        result.current.onVisualChange(ast);
      });

      await act(async () => {
        await vi.advanceTimersByTimeAsync(20);
      });

      const saved = JSON.parse(localStorageMock['rhai-sync-test-profile']);
      expect(saved.code).toBe('map("VK_A", "VK_B");');
      expect(saved.timestamp).toBeGreaterThan(0);
    });

    it('should handle code generation errors', async () => {
      const onError = vi.fn();
      const { result } = renderHook(() =>
        useRhaiSyncEngine({ storageKey: 'test-profile', onError })
      );

      // Mock code generator to throw error
      const { generateRhaiScript } = await import('../utils/rhaiCodeGen');
      vi.mocked(generateRhaiScript).mockImplementationOnce(() => {
        throw new Error('Generation failed');
      });

      const ast: RhaiAST = {
        imports: [],
        globalMappings: [],
        deviceBlocks: [],
        comments: [],
      };

      act(() => {
        result.current.onVisualChange(ast);
      });

      await act(async () => {
        await vi.advanceTimersByTimeAsync(20);
      });

      expect(result.current.state).toBe('error');
      expect(result.current.error).not.toBeNull();
      expect(result.current.error?.message).toBe('Generation failed');
      expect(onError).toHaveBeenCalledWith(
        expect.objectContaining({ message: 'Generation failed' }),
        'visual-to-code'
      );
    });
  });

  describe('code to visual sync', () => {
    it('should parse code with debouncing (500ms)', async () => {
      const { result } = renderHook(() =>
        useRhaiSyncEngine({ storageKey: 'test-profile', debounceMs: 500 })
      );

      act(() => {
        result.current.onCodeChange('map("VK_A", "VK_B");');
      });

      // Should not parse immediately
      expect(result.current.state).toBe('idle');

      // Advance time by 400ms (not enough)
      await act(async () => {
        await vi.advanceTimersByTimeAsync(400);
      });
      expect(result.current.state).toBe('idle');

      // Advance time by another 100ms (total 500ms)
      await act(async () => {
        await vi.advanceTimersByTimeAsync(100);
      });

      // Should start parsing
      expect(result.current.state).toBe('parsing');

      // Wait for parsing to complete
      await act(async () => {
        await vi.advanceTimersByTimeAsync(20);
      });

      expect(result.current.state).toBe('idle');
      expect(result.current.getAST()).not.toBeNull();
      expect(result.current.getAST()?.globalMappings).toHaveLength(1);
    });

    it('should debounce multiple rapid code changes', async () => {
      const { result } = renderHook(() =>
        useRhaiSyncEngine({ storageKey: 'test-profile', debounceMs: 500 })
      );

      // Rapid changes
      act(() => {
        result.current.onCodeChange('map("VK_A", "VK_B");');
      });

      await act(async () => {
        await vi.advanceTimersByTimeAsync(200);
      });

      act(() => {
        result.current.onCodeChange('map("VK_A", "VK_C");');
      });

      await act(async () => {
        await vi.advanceTimersByTimeAsync(200);
      });

      act(() => {
        result.current.onCodeChange('map("VK_A", "VK_D");');
      });

      // Should only parse once after full debounce period
      await act(async () => {
        await vi.advanceTimersByTimeAsync(500);
      });

      expect(result.current.state).toBe('parsing');

      await act(async () => {
        await vi.advanceTimersByTimeAsync(20);
      });

      expect(result.current.state).toBe('idle');
      // Only the last code should be parsed
      expect(result.current.getCode()).toBe('map("VK_A", "VK_D");');
    });

    it('should persist code to localStorage immediately on change', () => {
      const { result } = renderHook(() =>
        useRhaiSyncEngine({ storageKey: 'test-profile' })
      );

      act(() => {
        result.current.onCodeChange('map("VK_A", "VK_B");');
      });

      const saved = JSON.parse(localStorageMock['rhai-sync-test-profile']);
      expect(saved.code).toBe('map("VK_A", "VK_B");');
    });

    it('should handle parse errors gracefully', async () => {
      const onError = vi.fn();
      const { result } = renderHook(() =>
        useRhaiSyncEngine({ storageKey: 'test-profile', onError })
      );

      act(() => {
        result.current.onCodeChange('invalid syntax');
      });

      await act(async () => {
        await vi.advanceTimersByTimeAsync(500);
      });

      expect(result.current.state).toBe('parsing');

      await act(async () => {
        await vi.advanceTimersByTimeAsync(20);
      });

      expect(result.current.state).toBe('error');
      expect(result.current.error).not.toBeNull();
      expect(result.current.error?.message).toBe('Unexpected token');
      expect(onError).toHaveBeenCalledWith(
        expect.objectContaining({ message: 'Unexpected token' }),
        'code-to-visual'
      );
    });

    it('should preserve last valid AST on parse error', async () => {
      const { result } = renderHook(() =>
        useRhaiSyncEngine({ storageKey: 'test-profile' })
      );

      // First, set valid code
      act(() => {
        result.current.onCodeChange('map("VK_A", "VK_B");');
      });

      await act(async () => {
        await vi.advanceTimersByTimeAsync(520);
      });

      const validAST = result.current.getAST();
      expect(validAST).not.toBeNull();

      // Now set invalid code
      act(() => {
        result.current.onCodeChange('invalid syntax');
      });

      await act(async () => {
        await vi.advanceTimersByTimeAsync(520);
      });

      // Last valid AST should be preserved
      expect(result.current.state).toBe('error');
      expect(result.current.lastValidAST).toEqual(validAST);
    });
  });

  describe('state machine', () => {
    it('should transition through states: idle → parsing → idle', async () => {
      const onStateChange = vi.fn();
      const { result } = renderHook(() =>
        useRhaiSyncEngine({ storageKey: 'test-profile', onStateChange })
      );

      expect(result.current.state).toBe('idle');

      act(() => {
        result.current.onCodeChange('map("VK_A", "VK_B");');
      });

      await act(async () => {
        await vi.advanceTimersByTimeAsync(500);
      });

      expect(result.current.state).toBe('parsing');
      expect(onStateChange).toHaveBeenCalledWith('parsing');

      await act(async () => {
        await vi.advanceTimersByTimeAsync(20);
      });

      expect(result.current.state).toBe('idle');
      expect(onStateChange).toHaveBeenCalledWith('idle');
    });

    it('should transition through states: idle → generating → idle', async () => {
      const onStateChange = vi.fn();
      const { result } = renderHook(() =>
        useRhaiSyncEngine({ storageKey: 'test-profile', onStateChange })
      );

      const ast: RhaiAST = {
        imports: [],
        globalMappings: [
          { type: 'simple', sourceKey: 'VK_A', targetKey: 'VK_B', line: 1 },
        ],
        deviceBlocks: [],
        comments: [],
      };

      act(() => {
        result.current.onVisualChange(ast);
      });

      expect(result.current.state).toBe('generating');

      await act(async () => {
        await vi.advanceTimersByTimeAsync(20);
      });

      expect(result.current.state).toBe('idle');
    });

    it('should transition to error state on parse failure', async () => {
      const { result } = renderHook(() =>
        useRhaiSyncEngine({ storageKey: 'test-profile' })
      );

      act(() => {
        result.current.onCodeChange('invalid syntax');
      });

      await act(async () => {
        await vi.advanceTimersByTimeAsync(520);
      });

      expect(result.current.state).toBe('error');
      expect(result.current.error).not.toBeNull();
    });

    it('should clear error state', async () => {
      const { result } = renderHook(() =>
        useRhaiSyncEngine({ storageKey: 'test-profile' })
      );

      act(() => {
        result.current.onCodeChange('invalid syntax');
      });

      await act(async () => {
        await vi.advanceTimersByTimeAsync(520);
      });

      expect(result.current.state).toBe('error');

      act(() => {
        result.current.clearError();
      });

      expect(result.current.state).toBe('idle');
      expect(result.current.error).toBeNull();
    });
  });

  describe('sync lock (prevent infinite loops)', () => {
    it('should not allow sync while another sync is in progress', async () => {
      const { result } = renderHook(() =>
        useRhaiSyncEngine({ storageKey: 'test-profile' })
      );

      const ast: RhaiAST = {
        imports: [],
        globalMappings: [
          { type: 'simple', sourceKey: 'VK_A', targetKey: 'VK_B', line: 1 },
        ],
        deviceBlocks: [],
        comments: [],
      };

      // Start visual change (sync lock engaged)
      act(() => {
        result.current.onVisualChange(ast);
      });

      expect(result.current.state).toBe('generating');

      // Try to start code change (should be blocked by sync lock)
      act(() => {
        result.current.onCodeChange('map("VK_X", "VK_Y");');
      });

      // Should still be in generating state
      expect(result.current.state).toBe('generating');

      // Complete first sync
      await act(async () => {
        await vi.advanceTimersByTimeAsync(20);
      });

      expect(result.current.state).toBe('idle');
    });
  });

  describe('force sync', () => {
    it('should force sync code-to-visual', async () => {
      const { result } = renderHook(() =>
        useRhaiSyncEngine({ storageKey: 'test-profile' })
      );

      // Set code without triggering auto sync
      act(() => {
        result.current.onCodeChange('map("VK_A", "VK_B");');
      });

      // Don't wait for debounce, force sync instead
      act(() => {
        result.current.forceSync('code-to-visual');
      });

      expect(result.current.state).toBe('parsing');

      await act(async () => {
        await vi.advanceTimersByTimeAsync(20);
      });

      expect(result.current.state).toBe('idle');
      expect(result.current.getAST()).not.toBeNull();
    });

    it('should force sync visual-to-code', async () => {
      const { result } = renderHook(() =>
        useRhaiSyncEngine({ storageKey: 'test-profile' })
      );

      const ast: RhaiAST = {
        imports: [],
        globalMappings: [
          { type: 'simple', sourceKey: 'VK_A', targetKey: 'VK_B', line: 1 },
        ],
        deviceBlocks: [],
        comments: [],
      };

      act(() => {
        result.current.onVisualChange(ast);
      });

      await act(async () => {
        await vi.advanceTimersByTimeAsync(20);
      });

      // Force regenerate
      act(() => {
        result.current.forceSync('visual-to-code');
      });

      expect(result.current.state).toBe('generating');

      await act(async () => {
        await vi.advanceTimersByTimeAsync(20);
      });

      expect(result.current.state).toBe('idle');
    });
  });

  describe('persistence options', () => {
    it('should not persist when enablePersistence is false', async () => {
      const { result } = renderHook(() =>
        useRhaiSyncEngine({
          storageKey: 'test-profile',
          enablePersistence: false,
        })
      );

      act(() => {
        result.current.onCodeChange('map("VK_A", "VK_B");');
      });

      expect(localStorageMock['rhai-sync-test-profile']).toBeUndefined();
    });
  });

  describe('getters', () => {
    it('should get current code', () => {
      const { result } = renderHook(() =>
        useRhaiSyncEngine({ storageKey: 'test-profile' })
      );

      act(() => {
        result.current.onCodeChange('map("VK_A", "VK_B");');
      });

      expect(result.current.getCode()).toBe('map("VK_A", "VK_B");');
    });

    it('should get current AST', async () => {
      const { result } = renderHook(() =>
        useRhaiSyncEngine({ storageKey: 'test-profile' })
      );

      act(() => {
        result.current.onCodeChange('map("VK_A", "VK_B");');
      });

      await act(async () => {
        await vi.advanceTimersByTimeAsync(520);
      });

      const ast = result.current.getAST();
      expect(ast).not.toBeNull();
      expect(ast?.globalMappings).toHaveLength(1);
    });
  });
});
