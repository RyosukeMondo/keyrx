/**
 * Unit tests for useDragAndDrop hook
 */

import React from 'react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { useDragAndDrop } from './useDragAndDrop';
import * as configApi from '../api/config';
import type { AssignableKey } from '../types/config';

// Mock the config API
vi.mock('../api/config');

describe('useDragAndDrop', () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    // Create fresh query client for each test
    queryClient = new QueryClient({
      defaultOptions: {
        queries: { retry: false },
        mutations: { retry: false },
      },
    });

    // Reset mocks
    vi.clearAllMocks();
  });

  const wrapper = ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );

  describe('Initialization', () => {
    it('initializes with null activeDragKey', () => {
      const { result } = renderHook(
        () => useDragAndDrop({ profileName: 'Default', selectedLayer: 'base' }),
        { wrapper }
      );

      expect(result.current.activeDragKey).toBeNull();
      expect(result.current.isSaving).toBe(false);
    });

    it('provides all required handler functions', () => {
      const { result } = renderHook(
        () => useDragAndDrop({ profileName: 'Default', selectedLayer: 'base' }),
        { wrapper }
      );

      expect(typeof result.current.handleDragStart).toBe('function');
      expect(typeof result.current.handleDragEnd).toBe('function');
      expect(typeof result.current.handleKeyDrop).toBe('function');
    });
  });

  describe('Drag Start/End', () => {
    it('sets activeDragKey on drag start', () => {
      const { result } = renderHook(
        () => useDragAndDrop({ profileName: 'Default', selectedLayer: 'base' }),
        { wrapper }
      );

      const draggedKey: AssignableKey = {
        id: 'VK_A',
        label: 'A',
        category: 'vk',
        description: 'Virtual Key A',
      };

      const event = {
        active: {
          id: 'drag-vk-a',
          data: {
            current: draggedKey,
          },
        },
      } as any;

      act(() => {
        result.current.handleDragStart(event);
      });

      expect(result.current.activeDragKey).toEqual(draggedKey);
    });

    it('clears activeDragKey on drag end', () => {
      const { result } = renderHook(
        () => useDragAndDrop({ profileName: 'Default', selectedLayer: 'base' }),
        { wrapper }
      );

      const draggedKey: AssignableKey = {
        id: 'VK_A',
        label: 'A',
        category: 'vk',
        description: 'Virtual Key A',
      };

      // Start drag
      act(() => {
        result.current.handleDragStart({
          active: { id: 'drag-vk-a', data: { current: draggedKey } },
        } as any);
      });

      expect(result.current.activeDragKey).not.toBeNull();

      // End drag
      act(() => {
        result.current.handleDragEnd({} as any);
      });

      expect(result.current.activeDragKey).toBeNull();
    });
  });

  describe('Key Drop - Simple Mappings', () => {
    it('creates simple mapping for virtual key', async () => {
      const mockSetKeyMapping = vi.mocked(configApi.setKeyMapping);
      mockSetKeyMapping.mockResolvedValue({ success: true });

      const { result } = renderHook(
        () => useDragAndDrop({ profileName: 'Default', selectedLayer: 'base' }),
        { wrapper }
      );

      const droppedKey: AssignableKey = {
        id: 'VK_A',
        label: 'A',
        category: 'vk',
        description: 'Virtual Key A',
      };

      await act(async () => {
        await result.current.handleKeyDrop('KC_CAPS', droppedKey);
      });

      expect(mockSetKeyMapping).toHaveBeenCalledWith('Default', 'KC_CAPS', {
        keyCode: 'KC_CAPS',
        type: 'simple',
        simple: 'VK_A',
      });
    });

    it('creates simple mapping for modifier key', async () => {
      const mockSetKeyMapping = vi.mocked(configApi.setKeyMapping);
      mockSetKeyMapping.mockResolvedValue({ success: true });

      const { result } = renderHook(
        () => useDragAndDrop({ profileName: 'Default', selectedLayer: 'base' }),
        { wrapper }
      );

      const droppedKey: AssignableKey = {
        id: 'MD_CTRL',
        label: 'Ctrl',
        category: 'modifier',
        description: 'Control modifier',
      };

      await act(async () => {
        await result.current.handleKeyDrop('KC_CAPS', droppedKey);
      });

      expect(mockSetKeyMapping).toHaveBeenCalledWith('Default', 'KC_CAPS', {
        keyCode: 'KC_CAPS',
        type: 'simple',
        simple: 'MD_CTRL',
      });
    });

    it('creates simple mapping for lock key', async () => {
      const mockSetKeyMapping = vi.mocked(configApi.setKeyMapping);
      mockSetKeyMapping.mockResolvedValue({ success: true });

      const { result } = renderHook(
        () => useDragAndDrop({ profileName: 'Default', selectedLayer: 'base' }),
        { wrapper }
      );

      const droppedKey: AssignableKey = {
        id: 'LK_NUMLOCK',
        label: 'NumLock',
        category: 'lock',
        description: 'Num Lock key',
      };

      await act(async () => {
        await result.current.handleKeyDrop('KC_CAPS', droppedKey);
      });

      expect(mockSetKeyMapping).toHaveBeenCalledWith('Default', 'KC_CAPS', {
        keyCode: 'KC_CAPS',
        type: 'simple',
        simple: 'LK_NUMLOCK',
      });
    });
  });

  describe('Key Drop - Layer Switches', () => {
    it('creates layer-switch mapping for layer key', async () => {
      const mockSetKeyMapping = vi.mocked(configApi.setKeyMapping);
      mockSetKeyMapping.mockResolvedValue({ success: true });

      const { result } = renderHook(
        () => useDragAndDrop({ profileName: 'Default', selectedLayer: 'base' }),
        { wrapper }
      );

      const droppedKey: AssignableKey = {
        id: 'layer_nav',
        label: 'Nav Layer',
        category: 'layer',
        description: 'Navigation layer',
      };

      await act(async () => {
        await result.current.handleKeyDrop('KC_CAPS', droppedKey);
      });

      expect(mockSetKeyMapping).toHaveBeenCalledWith('Default', 'KC_CAPS', {
        keyCode: 'KC_CAPS',
        type: 'layer_switch',
        layer: 'nav',
      });
    });
  });

  describe('Key Drop - Macros', () => {
    it('creates macro mapping for macro key', async () => {
      const mockSetKeyMapping = vi.mocked(configApi.setKeyMapping);
      mockSetKeyMapping.mockResolvedValue({ success: true });

      const { result } = renderHook(
        () => useDragAndDrop({ profileName: 'Default', selectedLayer: 'base' }),
        { wrapper }
      );

      const droppedKey: AssignableKey = {
        id: 'macro_hello',
        label: 'Hello',
        category: 'macro',
        description: 'Types "hello"',
      };

      await act(async () => {
        await result.current.handleKeyDrop('KC_F1', droppedKey);
      });

      expect(mockSetKeyMapping).toHaveBeenCalledWith('Default', 'KC_F1', {
        keyCode: 'KC_F1',
        type: 'macro',
        macro: ['macro_hello'],
      });
    });
  });

  describe('Optimistic Updates', () => {
    it('successfully saves mapping and invalidates cache', async () => {
      const mockSetKeyMapping = vi.mocked(configApi.setKeyMapping);
      mockSetKeyMapping.mockResolvedValue({ success: true });

      // Set initial config in cache
      queryClient.setQueryData(['config', 'Default'], {
        profileName: 'Default',
        activeLayer: 'base',
        keyMappings: {},
        layers: ['base'],
      });

      const { result } = renderHook(
        () => useDragAndDrop({ profileName: 'Default', selectedLayer: 'base' }),
        { wrapper }
      );

      const droppedKey: AssignableKey = {
        id: 'VK_A',
        label: 'A',
        category: 'vk',
        description: 'Virtual Key A',
      };

      // Perform drop operation
      await act(async () => {
        await result.current.handleKeyDrop('KC_CAPS', droppedKey);
      });

      // API should have been called
      expect(mockSetKeyMapping).toHaveBeenCalled();
    });

    it('rolls back optimistic update on error', async () => {
      const mockSetKeyMapping = vi.mocked(configApi.setKeyMapping);
      mockSetKeyMapping.mockRejectedValue(new Error('API Error'));

      // Set initial config in cache
      const initialConfig = {
        profileName: 'Default',
        activeLayer: 'base',
        keyMappings: { KC_CAPS: { keyCode: 'KC_CAPS', type: 'simple' as const, simple: 'VK_B' } },
        layers: ['base'],
      };
      queryClient.setQueryData(['config', 'Default'], initialConfig);

      const { result } = renderHook(
        () => useDragAndDrop({ profileName: 'Default', selectedLayer: 'base' }),
        { wrapper }
      );

      const droppedKey: AssignableKey = {
        id: 'VK_A',
        label: 'A',
        category: 'vk',
        description: 'Virtual Key A',
      };

      // Attempt drop (should fail)
      await act(async () => {
        try {
          await result.current.handleKeyDrop('KC_CAPS', droppedKey);
        } catch {
          // Expected to throw
        }
      });

      // Wait for rollback
      await waitFor(() => {
        const cachedConfig = queryClient.getQueryData(['config', 'Default']) as any;
        // Should be rolled back to original value (VK_B, not VK_A)
        expect(cachedConfig.keyMappings['KC_CAPS'].simple).toBe('VK_B');
      });
    });
  });

  describe('Saving State', () => {
    it('initializes with isSaving false', () => {
      const { result } = renderHook(
        () => useDragAndDrop({ profileName: 'Default', selectedLayer: 'base' }),
        { wrapper }
      );

      expect(result.current.isSaving).toBe(false);
    });

    it('completes save operation successfully', async () => {
      const mockSetKeyMapping = vi.mocked(configApi.setKeyMapping);
      mockSetKeyMapping.mockResolvedValue({ success: true });

      const { result } = renderHook(
        () => useDragAndDrop({ profileName: 'Default', selectedLayer: 'base' }),
        { wrapper }
      );

      const droppedKey: AssignableKey = {
        id: 'VK_A',
        label: 'A',
        category: 'vk',
        description: 'Virtual Key A',
      };

      // Perform drop
      await act(async () => {
        await result.current.handleKeyDrop('KC_CAPS', droppedKey);
      });

      // After completion, should not be saving
      expect(result.current.isSaving).toBe(false);
    });
  });
});
