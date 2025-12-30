/**
 * Tests for test utilities module.
 *
 * Verifies that shared test helpers work correctly.
 */

import { describe, it, expect, vi } from 'vitest';
import { screen } from '@testing-library/react';
import {
  renderWithProviders,
  createMockStore,
  waitForAsync,
  mockProfiles,
  mockKeyboardEvents,
} from './testUtils';
import { createElement } from 'react';

describe('testUtils', () => {
  describe('renderWithProviders', () => {
    it('should render a basic component', () => {
      const TestComponent = () => <div>Test Content</div>;
      renderWithProviders(<TestComponent />);

      expect(screen.getByText('Test Content')).toBeInTheDocument();
    });

    it('should render with DndContext when withDnd is true', () => {
      const TestComponent = () => <div data-testid="test-component">DnD Component</div>;
      const { container } = renderWithProviders(<TestComponent />, { withDnd: true });

      expect(screen.getByTestId('test-component')).toBeInTheDocument();
      expect(screen.getByText('DnD Component')).toBeInTheDocument();
    });

    it('should render without DndContext when withDnd is false', () => {
      const TestComponent = () => <div>Regular Component</div>;
      renderWithProviders(<TestComponent />, { withDnd: false });

      expect(screen.getByText('Regular Component')).toBeInTheDocument();
    });

    it('should render without DndContext by default', () => {
      const TestComponent = () => <div>Default Component</div>;
      renderWithProviders(<TestComponent />);

      expect(screen.getByText('Default Component')).toBeInTheDocument();
    });

    it('should warn when initialConfigState is provided', () => {
      const consoleSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});
      const TestComponent = () => <div>Component</div>;

      renderWithProviders(<TestComponent />, {
        initialConfigState: { isDirty: true },
      });

      expect(consoleSpy).toHaveBeenCalledWith(
        expect.stringContaining('initialConfigState provided but not applied')
      );

      consoleSpy.mockRestore();
    });

    it('should pass through custom render options', () => {
      const TestComponent = () => <div>Custom Options</div>;
      const { container } = renderWithProviders(<TestComponent />, {
        container: document.body,
      });

      expect(container).toBe(document.body);
    });
  });

  describe('createMockStore', () => {
    it('should create a default mock store state', () => {
      const mockStore = createMockStore();

      expect(mockStore).toEqual({
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
      });
    });

    it('should merge initial state with defaults', () => {
      const mockStore = createMockStore({
        isDirty: true,
        currentLayerId: 'custom-layer',
      });

      expect(mockStore.isDirty).toBe(true);
      expect(mockStore.currentLayerId).toBe('custom-layer');
      expect(mockStore.layers).toEqual([
        {
          id: 'base',
          name: 'base',
          mappings: [],
          isBase: true,
        },
      ]);
    });

    it('should override layers when provided', () => {
      const customLayers = [
        { id: 'layer1', name: 'symbols', mappings: [], isBase: false },
      ];
      const mockStore = createMockStore({
        layers: customLayers,
      });

      expect(mockStore.layers).toEqual(customLayers);
    });

    it('should override modifiers when provided', () => {
      const customModifiers = [
        { id: 'mod1', name: 'super', triggerKey: 'KEY_LEFTMETA' },
      ];
      const mockStore = createMockStore({
        modifiers: customModifiers,
      });

      expect(mockStore.modifiers).toEqual(customModifiers);
    });

    it('should override locks when provided', () => {
      const customLocks = [
        { id: 'lock1', name: 'caps', triggerKey: 'KEY_CAPSLOCK' },
      ];
      const mockStore = createMockStore({
        locks: customLocks,
      });

      expect(mockStore.locks).toEqual(customLocks);
    });
  });

  describe('waitForAsync', () => {
    it('should resolve when callback completes', async () => {
      let completed = false;
      await waitForAsync(async () => {
        completed = true;
      });

      expect(completed).toBe(true);
    });

    it('should resolve for synchronous callbacks', async () => {
      let value = 0;
      await waitForAsync(() => {
        value = 42;
      });

      expect(value).toBe(42);
    });

    it('should timeout when callback takes too long', async () => {
      const slowCallback = async () => {
        await new Promise((resolve) => setTimeout(resolve, 2000));
      };

      await expect(waitForAsync(slowCallback, 100)).rejects.toThrow(
        'waitForAsync timed out after 100ms'
      );
    });

    it('should use default timeout of 1000ms', async () => {
      const slowCallback = async () => {
        await new Promise((resolve) => setTimeout(resolve, 1500));
      };

      await expect(waitForAsync(slowCallback)).rejects.toThrow(
        'waitForAsync timed out after 1000ms'
      );
    });

    it('should reject when callback throws an error', async () => {
      const errorCallback = async () => {
        throw new Error('Test error');
      };

      await expect(waitForAsync(errorCallback)).rejects.toThrow('Test error');
    });

    it('should handle fast async operations', async () => {
      let result = '';
      await waitForAsync(async () => {
        await Promise.resolve();
        result = 'completed';
      }, 5000);

      expect(result).toBe('completed');
    });

    it('should clear timeout on successful completion', async () => {
      const clearTimeoutSpy = vi.spyOn(global, 'clearTimeout');
      await waitForAsync(() => {}, 1000);

      expect(clearTimeoutSpy).toHaveBeenCalled();
      clearTimeoutSpy.mockRestore();
    });

    it('should clear timeout on error', async () => {
      const clearTimeoutSpy = vi.spyOn(global, 'clearTimeout');
      try {
        await waitForAsync(async () => {
          throw new Error('Test');
        }, 1000);
      } catch (e) {
        // Expected error
      }

      expect(clearTimeoutSpy).toHaveBeenCalled();
      clearTimeoutSpy.mockRestore();
    });
  });

  describe('mockProfiles', () => {
    it('should export mock profile data', () => {
      expect(mockProfiles).toHaveLength(3);
      expect(mockProfiles[0]).toEqual({
        name: 'default',
        rhai_path: '/path/to/default.rhai',
        krx_path: '/path/to/default.krx',
        modified_at: 1234567890,
        layer_count: 2,
        is_active: true,
      });
    });

    it('should have one active profile', () => {
      const activeProfiles = mockProfiles.filter((p) => p.is_active);
      expect(activeProfiles).toHaveLength(1);
      expect(activeProfiles[0].name).toBe('default');
    });

    it('should have valid profile structure', () => {
      mockProfiles.forEach((profile) => {
        expect(profile).toHaveProperty('name');
        expect(profile).toHaveProperty('rhai_path');
        expect(profile).toHaveProperty('krx_path');
        expect(profile).toHaveProperty('modified_at');
        expect(profile).toHaveProperty('layer_count');
        expect(profile).toHaveProperty('is_active');
      });
    });
  });

  describe('mockKeyboardEvents', () => {
    it('should export mock keyboard event data', () => {
      expect(mockKeyboardEvents).toHaveLength(3);
      expect(mockKeyboardEvents[0]).toEqual({
        timestamp: 1234567890000,
        event_type: 'KeyPress',
        key_code: 65,
        description: 'Key A pressed',
      });
    });

    it('should have valid event structure', () => {
      mockKeyboardEvents.forEach((event) => {
        expect(event).toHaveProperty('timestamp');
        expect(event).toHaveProperty('event_type');
        expect(event).toHaveProperty('key_code');
        expect(event).toHaveProperty('description');
      });
    });

    it('should include different event types', () => {
      const eventTypes = mockKeyboardEvents.map((e) => e.event_type);
      expect(eventTypes).toContain('KeyPress');
      expect(eventTypes).toContain('KeyRelease');
      expect(eventTypes).toContain('Remapped');
    });
  });
});
