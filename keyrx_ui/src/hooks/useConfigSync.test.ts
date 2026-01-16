import { renderHook, act } from '@testing-library/react';
import { vi, describe, it, expect, beforeEach } from 'vitest';
import { useConfigSync } from './useConfigSync';
import type {
  RhaiSyncEngineResult,
  SyncState,
} from '@/components/RhaiSyncEngine';

// Mock the useRhaiSyncEngine hook
vi.mock('@/components/RhaiSyncEngine', () => ({
  useRhaiSyncEngine: vi.fn(),
}));

import { useRhaiSyncEngine } from '@/components/RhaiSyncEngine';
const mockUseRhaiSyncEngine = useRhaiSyncEngine as ReturnType<typeof vi.fn>;

describe('useConfigSync', () => {
  let mockSyncEngine: RhaiSyncEngineResult;
  let mockOnStateChange: ReturnType<typeof vi.fn>;
  let mockOnError: ReturnType<typeof vi.fn>;

  beforeEach(() => {
    // Reset mocks
    vi.clearAllMocks();

    // Create mock callbacks
    mockOnStateChange = vi.fn();
    mockOnError = vi.fn();

    // Create a complete mock sync engine
    mockSyncEngine = {
      state: 'idle' as SyncState,
      direction: 'none',
      error: null,
      lastValidAST: null,
      onCodeChange: vi.fn(),
      onVisualChange: vi.fn(),
      getCode: vi.fn(() => ''),
      getAST: vi.fn(() => null),
      clearError: vi.fn(),
      forceSync: vi.fn(),
    };

    // Setup mock implementation to capture callbacks
    mockUseRhaiSyncEngine.mockImplementation((options) => {
      if (options.onStateChange) mockOnStateChange = options.onStateChange;
      if (options.onError) mockOnError = options.onError;
      return mockSyncEngine;
    });
  });

  it('should initialize with default sync status "saved"', () => {
    const { result } = renderHook(() => useConfigSync('TestProfile'));

    expect(result.current.syncStatus).toBe('saved');
    expect(result.current.lastSaveTime).toBeNull();
    expect(result.current.syncEngine).toBeDefined();
  });

  it('should initialize RhaiSyncEngine with correct config', () => {
    renderHook(() => useConfigSync('TestProfile'));

    expect(mockUseRhaiSyncEngine).toHaveBeenCalledWith({
      storageKey: 'profile-TestProfile',
      debounceMs: 500,
      onStateChange: expect.any(Function),
      onError: expect.any(Function),
    });
  });

  it('should update storageKey when profileName changes', () => {
    const { rerender } = renderHook(
      ({ profileName }) => useConfigSync(profileName),
      { initialProps: { profileName: 'Profile1' } }
    );

    expect(mockUseRhaiSyncEngine).toHaveBeenCalledWith(
      expect.objectContaining({
        storageKey: 'profile-Profile1',
      })
    );

    rerender({ profileName: 'Profile2' });

    expect(mockUseRhaiSyncEngine).toHaveBeenLastCalledWith(
      expect.objectContaining({
        storageKey: 'profile-Profile2',
      })
    );
  });

  it('should reset sync status when profile changes', () => {
    const { result, rerender } = renderHook(
      ({ profileName }) => useConfigSync(profileName),
      { initialProps: { profileName: 'Profile1' } }
    );

    // Change sync status
    act(() => {
      result.current.setSyncStatus('unsaved');
      result.current.setLastSaveTime(new Date('2026-01-01'));
    });

    expect(result.current.syncStatus).toBe('unsaved');
    expect(result.current.lastSaveTime).not.toBeNull();

    // Change profile
    rerender({ profileName: 'Profile2' });

    // Status should reset
    expect(result.current.syncStatus).toBe('saved');
    expect(result.current.lastSaveTime).toBeNull();
  });

  it('should allow updating sync status', () => {
    const { result } = renderHook(() => useConfigSync('TestProfile'));

    expect(result.current.syncStatus).toBe('saved');

    act(() => {
      result.current.setSyncStatus('unsaved');
    });

    expect(result.current.syncStatus).toBe('unsaved');

    act(() => {
      result.current.setSyncStatus('saving');
    });

    expect(result.current.syncStatus).toBe('saving');

    act(() => {
      result.current.setSyncStatus('saved');
    });

    expect(result.current.syncStatus).toBe('saved');
  });

  it('should allow updating last save time', () => {
    const { result } = renderHook(() => useConfigSync('TestProfile'));

    expect(result.current.lastSaveTime).toBeNull();

    const saveTime = new Date('2026-01-16T12:00:00Z');
    act(() => {
      result.current.setLastSaveTime(saveTime);
    });

    expect(result.current.lastSaveTime).toBe(saveTime);

    act(() => {
      result.current.setLastSaveTime(null);
    });

    expect(result.current.lastSaveTime).toBeNull();
  });

  it('should provide access to syncEngine methods', () => {
    const { result } = renderHook(() => useConfigSync('TestProfile'));

    expect(result.current.syncEngine.onCodeChange).toBeDefined();
    expect(result.current.syncEngine.onVisualChange).toBeDefined();
    expect(result.current.syncEngine.getCode).toBeDefined();
    expect(result.current.syncEngine.getAST).toBeDefined();
    expect(result.current.syncEngine.clearError).toBeDefined();
    expect(result.current.syncEngine.forceSync).toBeDefined();
  });

  it('should pass through syncEngine state', () => {
    mockSyncEngine.state = 'parsing';
    mockSyncEngine.direction = 'code-to-visual';

    const { result } = renderHook(() => useConfigSync('TestProfile'));

    expect(result.current.syncEngine.state).toBe('parsing');
    expect(result.current.syncEngine.direction).toBe('code-to-visual');
  });

  it('should log state changes when onStateChange is called', () => {
    const consoleDebugSpy = vi
      .spyOn(console, 'debug')
      .mockImplementation(() => {});

    renderHook(() => useConfigSync('TestProfile'));

    // Simulate state change callback
    act(() => {
      mockOnStateChange('parsing' as SyncState);
    });

    expect(consoleDebugSpy).toHaveBeenCalledWith(
      'Sync state changed:',
      'parsing'
    );

    consoleDebugSpy.mockRestore();
  });

  it('should log errors when onError is called', () => {
    const consoleErrorSpy = vi
      .spyOn(console, 'error')
      .mockImplementation(() => {});

    renderHook(() => useConfigSync('TestProfile'));

    const mockError = { message: 'Parse error', line: 1, column: 5 };
    const direction = 'code-to-visual';

    // Simulate error callback
    act(() => {
      mockOnError(mockError, direction);
    });

    expect(consoleErrorSpy).toHaveBeenCalledWith('Sync error:', {
      error: mockError,
      direction,
    });

    consoleErrorSpy.mockRestore();
  });

  it('should maintain stable references for setters', () => {
    const { result, rerender } = renderHook(() => useConfigSync('TestProfile'));

    const firstSetSyncStatus = result.current.setSyncStatus;
    const firstSetLastSaveTime = result.current.setLastSaveTime;

    rerender();

    expect(result.current.setSyncStatus).toBe(firstSetSyncStatus);
    expect(result.current.setLastSaveTime).toBe(firstSetLastSaveTime);
  });

  it('should use debounceMs of 500', () => {
    renderHook(() => useConfigSync('TestProfile'));

    expect(mockUseRhaiSyncEngine).toHaveBeenCalledWith(
      expect.objectContaining({
        debounceMs: 500,
      })
    );
  });

  it('should handle multiple rapid profile changes', () => {
    const { result, rerender } = renderHook(
      ({ profileName }) => useConfigSync(profileName),
      { initialProps: { profileName: 'Profile1' } }
    );

    // Set status
    act(() => {
      result.current.setSyncStatus('unsaved');
    });

    // Rapid profile changes
    rerender({ profileName: 'Profile2' });
    rerender({ profileName: 'Profile3' });
    rerender({ profileName: 'Profile4' });

    // Status should be reset
    expect(result.current.syncStatus).toBe('saved');
    expect(result.current.lastSaveTime).toBeNull();

    // Should be initialized with last profile
    expect(mockUseRhaiSyncEngine).toHaveBeenLastCalledWith(
      expect.objectContaining({
        storageKey: 'profile-Profile4',
      })
    );
  });

  it('should handle empty profile name', () => {
    renderHook(() => useConfigSync(''));

    expect(mockUseRhaiSyncEngine).toHaveBeenCalledWith(
      expect.objectContaining({
        storageKey: 'profile-',
      })
    );
  });

  it('should handle profile names with special characters', () => {
    const specialProfileName = 'My-Profile_123 (Test)';
    renderHook(() => useConfigSync(specialProfileName));

    expect(mockUseRhaiSyncEngine).toHaveBeenCalledWith(
      expect.objectContaining({
        storageKey: 'profile-My-Profile_123 (Test)',
      })
    );
  });
});
