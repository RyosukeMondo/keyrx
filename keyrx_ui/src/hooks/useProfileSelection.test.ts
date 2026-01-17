import { renderHook, act } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { useProfileSelection } from './useProfileSelection';
import { useActiveProfileQuery } from './useProfiles';
import { useSearchParams, useParams } from 'react-router-dom';

// Mock dependencies
vi.mock('./useProfiles');
vi.mock('react-router-dom', () => ({
  useSearchParams: vi.fn(),
  useParams: vi.fn(),
}));

const mockUseActiveProfileQuery = vi.mocked(useActiveProfileQuery);
const mockUseSearchParams = vi.mocked(useSearchParams);
const mockUseParams = vi.mocked(useParams);

describe('useProfileSelection', () => {
  beforeEach(() => {
    vi.clearAllMocks();

    // Default mocks
    mockUseSearchParams.mockReturnValue([new URLSearchParams(), vi.fn()]);
    mockUseParams.mockReturnValue({});
    mockUseActiveProfileQuery.mockReturnValue({
      data: undefined,
      isLoading: false,
      error: null,
    } as any);
  });

  describe('Fallback priority', () => {
    it('should use manual selection when set', () => {
      const { result } = renderHook(() => useProfileSelection());

      expect(result.current.selectedProfileName).toBe('Default');

      // Set manual selection
      act(() => {
        result.current.setSelectedProfileName('ManualProfile');
      });

      expect(result.current.selectedProfileName).toBe('ManualProfile');
    });

    it('should prefer manual selection over all other sources', () => {
      mockUseParams.mockReturnValue({ name: 'RouteProfile' });
      mockUseSearchParams.mockReturnValue([
        new URLSearchParams('profile=QueryProfile'),
        vi.fn(),
      ]);
      mockUseActiveProfileQuery.mockReturnValue({
        data: 'ActiveProfile',
        isLoading: false,
        error: null,
      } as any);

      const { result } = renderHook(() => useProfileSelection('PropProfile'));

      // Initially should use prop
      expect(result.current.selectedProfileName).toBe('PropProfile');

      // Manual selection should override everything
      act(() => {
        result.current.setSelectedProfileName('ManualProfile');
      });
      expect(result.current.selectedProfileName).toBe('ManualProfile');
    });

    it('should use prop when no manual selection', () => {
      const { result } = renderHook(() => useProfileSelection('PropProfile'));

      expect(result.current.selectedProfileName).toBe('PropProfile');
    });

    it('should use route parameter when no manual selection or prop', () => {
      mockUseParams.mockReturnValue({ name: 'RouteProfile' });

      const { result } = renderHook(() => useProfileSelection());

      expect(result.current.selectedProfileName).toBe('RouteProfile');
    });

    it('should use query parameter when no manual selection, prop, or route', () => {
      mockUseSearchParams.mockReturnValue([
        new URLSearchParams('profile=QueryProfile'),
        vi.fn(),
      ]);

      const { result } = renderHook(() => useProfileSelection());

      expect(result.current.selectedProfileName).toBe('QueryProfile');
    });

    it('should use active profile when no other sources', () => {
      mockUseActiveProfileQuery.mockReturnValue({
        data: 'ActiveProfile',
        isLoading: false,
        error: null,
      } as any);

      const { result } = renderHook(() => useProfileSelection());

      expect(result.current.selectedProfileName).toBe('ActiveProfile');
    });

    it('should default to "Default" when all sources are null', () => {
      const { result } = renderHook(() => useProfileSelection());

      expect(result.current.selectedProfileName).toBe('Default');
    });
  });

  describe('Priority order verification', () => {
    it('should prioritize prop over route parameter', () => {
      mockUseParams.mockReturnValue({ name: 'RouteProfile' });

      const { result } = renderHook(() => useProfileSelection('PropProfile'));

      expect(result.current.selectedProfileName).toBe('PropProfile');
    });

    it('should prioritize route parameter over query parameter', () => {
      mockUseParams.mockReturnValue({ name: 'RouteProfile' });
      mockUseSearchParams.mockReturnValue([
        new URLSearchParams('profile=QueryProfile'),
        vi.fn(),
      ]);

      const { result } = renderHook(() => useProfileSelection());

      expect(result.current.selectedProfileName).toBe('RouteProfile');
    });

    it('should prioritize query parameter over active profile', () => {
      mockUseSearchParams.mockReturnValue([
        new URLSearchParams('profile=QueryProfile'),
        vi.fn(),
      ]);
      mockUseActiveProfileQuery.mockReturnValue({
        data: 'ActiveProfile',
        isLoading: false,
        error: null,
      } as any);

      const { result } = renderHook(() => useProfileSelection());

      expect(result.current.selectedProfileName).toBe('QueryProfile');
    });

    it('should prioritize active profile over default', () => {
      mockUseActiveProfileQuery.mockReturnValue({
        data: 'ActiveProfile',
        isLoading: false,
        error: null,
      } as any);

      const { result } = renderHook(() => useProfileSelection());

      expect(result.current.selectedProfileName).toBe('ActiveProfile');
    });
  });

  describe('Manual selection state', () => {
    it('should maintain manual selection across re-renders', () => {
      const { result, rerender } = renderHook(() => useProfileSelection());

      act(() => {
        result.current.setSelectedProfileName('ManualProfile');
      });
      expect(result.current.selectedProfileName).toBe('ManualProfile');

      rerender();
      expect(result.current.selectedProfileName).toBe('ManualProfile');
    });

    it('should allow changing manual selection', () => {
      const { result } = renderHook(() => useProfileSelection());

      act(() => {
        result.current.setSelectedProfileName('Profile1');
      });
      expect(result.current.selectedProfileName).toBe('Profile1');

      act(() => {
        result.current.setSelectedProfileName('Profile2');
      });
      expect(result.current.selectedProfileName).toBe('Profile2');
    });
  });

  describe('Edge cases', () => {
    it('should handle undefined prop', () => {
      const { result } = renderHook(() => useProfileSelection(undefined));

      expect(result.current.selectedProfileName).toBe('Default');
    });

    it('should handle empty string prop', () => {
      const { result } = renderHook(() => useProfileSelection(''));

      // Empty string is falsy, should fallback to Default
      expect(result.current.selectedProfileName).toBe('Default');
    });

    it('should handle null from all sources', () => {
      mockUseParams.mockReturnValue({});
      mockUseSearchParams.mockReturnValue([new URLSearchParams(), vi.fn()]);
      mockUseActiveProfileQuery.mockReturnValue({
        data: null,
        isLoading: false,
        error: null,
      } as any);

      const { result } = renderHook(() => useProfileSelection());

      expect(result.current.selectedProfileName).toBe('Default');
    });

    it('should handle loading state from active profile query', () => {
      mockUseActiveProfileQuery.mockReturnValue({
        data: undefined,
        isLoading: true,
        error: null,
      } as any);

      const { result } = renderHook(() => useProfileSelection());

      // Should fallback to Default while loading
      expect(result.current.selectedProfileName).toBe('Default');
    });
  });
});
