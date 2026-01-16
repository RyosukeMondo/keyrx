/**
 * Tests for RpcClient
 *
 * Tests the type-safe RPC client wrapper that provides typed methods for all
 * daemon communication operations. Covers profile, device, config, metrics,
 * simulation, and subscription methods.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { RpcClient } from './rpc';
import type { UseUnifiedApiReturn } from '../hooks/useUnifiedApi';
import type { DaemonState, KeyEvent, LatencyMetrics } from '../types/rpc';

// Create a mock UnifiedApi implementation
const createMockApi = (): UseUnifiedApiReturn => ({
  query: vi.fn(),
  command: vi.fn(),
  subscribe: vi.fn(),
  isConnected: true,
  readyState: 1, // OPEN
  lastError: null,
});

describe('RpcClient', () => {
  let mockApi: UseUnifiedApiReturn;
  let client: RpcClient;

  beforeEach(() => {
    mockApi = createMockApi();
    client = new RpcClient(mockApi);
  });

  describe('Constructor', () => {
    it('creates client with UnifiedApi instance', () => {
      expect(client).toBeInstanceOf(RpcClient);
      expect(client.isConnected).toBe(true);
    });
  });

  describe('Profile Methods', () => {
    it('getProfiles calls query with correct method', async () => {
      const mockProfiles = [
        {
          name: 'default',
          rhaiPath: '/path/default.rhai',
          krxPath: '/path/default.krx',
          modifiedAt: '2024-01-01',
          createdAt: '2024-01-01',
          layerCount: 1,
          modifierCount: 0,
          activeDeviceCount: 0,
        },
      ];
      vi.mocked(mockApi.query).mockResolvedValue(mockProfiles);

      const result = await client.getProfiles();

      expect(mockApi.query).toHaveBeenCalledWith('get_profiles');
      expect(result).toEqual(mockProfiles);
    });

    it('createProfile calls command with name and template', async () => {
      vi.mocked(mockApi.command).mockResolvedValue(undefined);

      await client.createProfile('gaming', 'default');

      expect(mockApi.command).toHaveBeenCalledWith('create_profile', {
        name: 'gaming',
        template: 'default',
      });
    });

    it('createProfile without template', async () => {
      vi.mocked(mockApi.command).mockResolvedValue(undefined);

      await client.createProfile('gaming');

      expect(mockApi.command).toHaveBeenCalledWith('create_profile', {
        name: 'gaming',
        template: undefined,
      });
    });

    it('activateProfile calls command with name', async () => {
      vi.mocked(mockApi.command).mockResolvedValue(undefined);

      await client.activateProfile('gaming');

      expect(mockApi.command).toHaveBeenCalledWith('activate_profile', {
        name: 'gaming',
      });
    });

    it('deleteProfile calls command with name', async () => {
      vi.mocked(mockApi.command).mockResolvedValue(undefined);

      await client.deleteProfile('gaming');

      expect(mockApi.command).toHaveBeenCalledWith('delete_profile', {
        name: 'gaming',
      });
    });

    it('duplicateProfile calls command with source and new name', async () => {
      vi.mocked(mockApi.command).mockResolvedValue(undefined);

      await client.duplicateProfile('default', 'gaming');

      expect(mockApi.command).toHaveBeenCalledWith('duplicate_profile', {
        source_name: 'default',
        new_name: 'gaming',
      });
    });

    it('renameProfile calls command with old and new name', async () => {
      vi.mocked(mockApi.command).mockResolvedValue(undefined);

      await client.renameProfile('default', 'custom');

      expect(mockApi.command).toHaveBeenCalledWith('rename_profile', {
        old_name: 'default',
        new_name: 'custom',
      });
    });

    it('getProfileConfig calls query and validates response', async () => {
      const mockConfig = { name: 'default', source: 'map("A", "B");' };
      vi.mocked(mockApi.query).mockResolvedValue(mockConfig);

      const result = await client.getProfileConfig('default');

      expect(mockApi.query).toHaveBeenCalledWith('get_profile_config', {
        name: 'default',
      });
      expect(result).toEqual({ name: 'default', source: 'map("A", "B");' });
    });

    it('setProfileConfig calls command with name and source', async () => {
      vi.mocked(mockApi.command).mockResolvedValue({});

      await client.setProfileConfig('default', 'map("X", "Y");');

      expect(mockApi.command).toHaveBeenCalledWith('set_profile_config', {
        name: 'default',
        source: 'map("X", "Y");',
      });
    });

    it('getActiveProfile calls query', async () => {
      vi.mocked(mockApi.query).mockResolvedValue('gaming');

      const result = await client.getActiveProfile();

      expect(mockApi.query).toHaveBeenCalledWith('get_active_profile');
      expect(result).toBe('gaming');
    });

    it('getActiveProfile returns null when no profile active', async () => {
      vi.mocked(mockApi.query).mockResolvedValue(null);

      const result = await client.getActiveProfile();

      expect(result).toBe(null);
    });
  });

  describe('Device Methods', () => {
    it('getDevices calls query', async () => {
      const mockDevices = [
        {
          id: 'dev1',
          name: 'Keyboard',
          path: '/dev/input/event0',
          serial: 'ABC123',
          active: true,
          scope: null,
          layout: null,
        },
      ];
      vi.mocked(mockApi.query).mockResolvedValue(mockDevices);

      const result = await client.getDevices();

      expect(mockApi.query).toHaveBeenCalledWith('get_devices');
      expect(result).toEqual(mockDevices);
    });

    it('renameDevice calls command with serial and new name', async () => {
      vi.mocked(mockApi.command).mockResolvedValue(undefined);

      await client.renameDevice('ABC123', 'Gaming Keyboard');

      expect(mockApi.command).toHaveBeenCalledWith('rename_device', {
        serial: 'ABC123',
        new_name: 'Gaming Keyboard',
      });
    });

    it('setScopeDevice calls command with serial and scope', async () => {
      vi.mocked(mockApi.command).mockResolvedValue(undefined);

      await client.setScopeDevice('ABC123', 'global');

      expect(mockApi.command).toHaveBeenCalledWith('set_scope_device', {
        serial: 'ABC123',
        scope: 'global',
      });
    });

    it('forgetDevice calls command with serial', async () => {
      vi.mocked(mockApi.command).mockResolvedValue(undefined);

      await client.forgetDevice('ABC123');

      expect(mockApi.command).toHaveBeenCalledWith('forget_device', {
        serial: 'ABC123',
      });
    });

    it('setDeviceLayout calls command with serial and layout', async () => {
      vi.mocked(mockApi.command).mockResolvedValue(undefined);

      await client.setDeviceLayout('ABC123', 'ansi');

      expect(mockApi.command).toHaveBeenCalledWith('set_device_layout', {
        serial: 'ABC123',
        layout: 'ansi',
      });
    });
  });

  describe('Configuration Methods', () => {
    it('getConfig calls query', async () => {
      const mockConfig = { code: 'map("A", "B");', hash: 'abc123' };
      vi.mocked(mockApi.query).mockResolvedValue(mockConfig);

      const result = await client.getConfig();

      expect(mockApi.query).toHaveBeenCalledWith('get_config');
      expect(result).toEqual(mockConfig);
    });

    it('updateConfig calls command with code', async () => {
      vi.mocked(mockApi.command).mockResolvedValue(undefined);

      await client.updateConfig('map("X", "Y");');

      expect(mockApi.command).toHaveBeenCalledWith('update_config', {
        code: 'map("X", "Y");',
      });
    });

    it('setKeyMapping calls command with layer, key code, and mapping', async () => {
      vi.mocked(mockApi.command).mockResolvedValue(undefined);

      await client.setKeyMapping('base', 'A', 'B');

      expect(mockApi.command).toHaveBeenCalledWith('set_key_mapping', {
        layer: 'base',
        key_code: 'A',
        mapping: 'B',
      });
    });

    it('deleteKeyMapping calls command with layer and key code', async () => {
      vi.mocked(mockApi.command).mockResolvedValue(undefined);

      await client.deleteKeyMapping('base', 'A');

      expect(mockApi.command).toHaveBeenCalledWith('delete_key_mapping', {
        layer: 'base',
        key_code: 'A',
      });
    });

    it('getLayers calls query', async () => {
      const mockLayers = [
        { name: 'base', keys: {} },
        { name: 'function', keys: {} },
      ];
      vi.mocked(mockApi.query).mockResolvedValue(mockLayers);

      const result = await client.getLayers();

      expect(mockApi.query).toHaveBeenCalledWith('get_layers');
      expect(result).toEqual(mockLayers);
    });
  });

  describe('Metrics Methods', () => {
    it('getLatency calls query', async () => {
      const mockLatency: LatencyMetrics = {
        min: 100,
        avg: 150,
        max: 200,
        p95: 180,
        p99: 190,
        timestamp: 1234567890,
      };
      vi.mocked(mockApi.query).mockResolvedValue(mockLatency);

      const result = await client.getLatency();

      expect(mockApi.query).toHaveBeenCalledWith('get_latency');
      expect(result).toEqual(mockLatency);
    });

    it('getEvents calls query with default pagination', async () => {
      const mockEvents = {
        events: [],
        total: 0,
        limit: 100,
        offset: 0,
      };
      vi.mocked(mockApi.query).mockResolvedValue(mockEvents);

      const result = await client.getEvents();

      expect(mockApi.query).toHaveBeenCalledWith('get_events', {
        limit: undefined,
        offset: undefined,
      });
      expect(result).toEqual(mockEvents);
    });

    it('getEvents calls query with custom pagination', async () => {
      const mockEvents = {
        events: [],
        total: 1000,
        limit: 50,
        offset: 100,
      };
      vi.mocked(mockApi.query).mockResolvedValue(mockEvents);

      const result = await client.getEvents(50, 100);

      expect(mockApi.query).toHaveBeenCalledWith('get_events', {
        limit: 50,
        offset: 100,
      });
      expect(result).toEqual(mockEvents);
    });

    it('clearEvents calls command', async () => {
      vi.mocked(mockApi.command).mockResolvedValue(undefined);

      await client.clearEvents();

      expect(mockApi.command).toHaveBeenCalledWith('clear_events');
    });
  });

  describe('Simulation Methods', () => {
    it('simulate calls command with input', async () => {
      const mockInput = [
        {
          events: [
            { keycode: 'A', event_type: 'press' as const, timestamp_us: 1000 },
          ],
        },
      ];
      const mockResult = [
        {
          states: [],
          outputs: [
            { keycode: 'B', event_type: 'press' as const, timestamp_us: 1500 },
          ],
          latency: [500],
          final_state: {
            active_modifiers: [],
            active_locks: [],
            active_layer: null,
          },
        },
      ];
      vi.mocked(mockApi.command).mockResolvedValue(mockResult);

      const result = await client.simulate(mockInput);

      expect(mockApi.command).toHaveBeenCalledWith('simulate', {
        input: mockInput,
      });
      expect(result).toEqual(mockResult);
    });

    it('resetSimulator calls command', async () => {
      vi.mocked(mockApi.command).mockResolvedValue(undefined);

      await client.resetSimulator();

      expect(mockApi.command).toHaveBeenCalledWith('reset_simulator');
    });
  });

  describe('Subscription Methods', () => {
    it('onDaemonState subscribes to daemon-state', () => {
      const mockUnsubscribe = vi.fn();
      vi.mocked(mockApi.subscribe).mockReturnValue(mockUnsubscribe);
      const handler = vi.fn();

      const unsubscribe = client.onDaemonState(handler);

      expect(mockApi.subscribe).toHaveBeenCalledWith(
        'daemon-state',
        expect.any(Function)
      );
      expect(unsubscribe).toBe(mockUnsubscribe);
    });

    it('onKeyEvent subscribes to events', () => {
      const mockUnsubscribe = vi.fn();
      vi.mocked(mockApi.subscribe).mockReturnValue(mockUnsubscribe);
      const handler = vi.fn();

      const unsubscribe = client.onKeyEvent(handler);

      expect(mockApi.subscribe).toHaveBeenCalledWith(
        'events',
        expect.any(Function)
      );
      expect(unsubscribe).toBe(mockUnsubscribe);
    });

    it('onLatencyUpdate subscribes to latency', () => {
      const mockUnsubscribe = vi.fn();
      vi.mocked(mockApi.subscribe).mockReturnValue(mockUnsubscribe);
      const handler = vi.fn();

      const unsubscribe = client.onLatencyUpdate(handler);

      expect(mockApi.subscribe).toHaveBeenCalledWith(
        'latency',
        expect.any(Function)
      );
      expect(unsubscribe).toBe(mockUnsubscribe);
    });

    it('unsubscribe function works correctly', () => {
      const mockUnsubscribe = vi.fn();
      vi.mocked(mockApi.subscribe).mockReturnValue(mockUnsubscribe);

      const unsubscribe = client.onDaemonState(vi.fn());
      unsubscribe();

      expect(mockUnsubscribe).toHaveBeenCalled();
    });
  });

  describe('Connection State', () => {
    it('isConnected returns API connection state', () => {
      expect(client.isConnected).toBe(true);

      mockApi.isConnected = false;
      const client2 = new RpcClient(mockApi);
      expect(client2.isConnected).toBe(false);
    });

    it('readyState returns API ready state', () => {
      expect(client.readyState).toBe(1);

      mockApi.readyState = 0;
      const client2 = new RpcClient(mockApi);
      expect(client2.readyState).toBe(0);
    });

    it('lastError returns API last error', () => {
      expect(client.lastError).toBe(null);

      const error = new Error('Connection failed');
      mockApi.lastError = error;
      const client2 = new RpcClient(mockApi);
      expect(client2.lastError).toBe(error);
    });
  });

  describe('Error Handling', () => {
    it('propagates query errors', async () => {
      const error = new Error('Query failed');
      vi.mocked(mockApi.query).mockRejectedValue(error);

      await expect(client.getProfiles()).rejects.toThrow('Query failed');
    });

    it('propagates command errors', async () => {
      const error = new Error('Command failed');
      vi.mocked(mockApi.command).mockRejectedValue(error);

      await expect(client.createProfile('test')).rejects.toThrow(
        'Command failed'
      );
    });

    it('handles validation errors in getProfileConfig', async () => {
      // Return invalid response (missing required field)
      vi.mocked(mockApi.query).mockResolvedValue({ name: 'default' });

      await expect(client.getProfileConfig('default')).rejects.toThrow();
    });
  });

  describe('Edge Cases', () => {
    it('handles empty profile name', async () => {
      vi.mocked(mockApi.command).mockResolvedValue(undefined);

      await client.createProfile('');

      expect(mockApi.command).toHaveBeenCalledWith('create_profile', {
        name: '',
        template: undefined,
      });
    });

    it('handles special characters in profile names', async () => {
      vi.mocked(mockApi.command).mockResolvedValue(undefined);

      await client.activateProfile('my-profile_123');

      expect(mockApi.command).toHaveBeenCalledWith('activate_profile', {
        name: 'my-profile_123',
      });
    });

    it('handles empty device serial', async () => {
      vi.mocked(mockApi.command).mockResolvedValue(undefined);

      await client.renameDevice('', 'New Name');

      expect(mockApi.command).toHaveBeenCalledWith('rename_device', {
        serial: '',
        new_name: 'New Name',
      });
    });

    it('handles empty configuration code', async () => {
      vi.mocked(mockApi.command).mockResolvedValue(undefined);

      await client.updateConfig('');

      expect(mockApi.command).toHaveBeenCalledWith('update_config', {
        code: '',
      });
    });

    it('handles large pagination values', async () => {
      vi.mocked(mockApi.query).mockResolvedValue({
        events: [],
        total: 0,
        limit: 10000,
        offset: 50000,
      });

      await client.getEvents(10000, 50000);

      expect(mockApi.query).toHaveBeenCalledWith('get_events', {
        limit: 10000,
        offset: 50000,
      });
    });

    it('handles zero pagination values', async () => {
      vi.mocked(mockApi.query).mockResolvedValue({
        events: [],
        total: 0,
        limit: 0,
        offset: 0,
      });

      await client.getEvents(0, 0);

      expect(mockApi.query).toHaveBeenCalledWith('get_events', {
        limit: 0,
        offset: 0,
      });
    });
  });

  describe('Type Safety', () => {
    it('returns correctly typed profile data', async () => {
      const mockProfile = {
        name: 'test',
        rhaiPath: '/path/test.rhai',
        krxPath: '/path/test.krx',
        modifiedAt: '2024-01-01',
        createdAt: '2024-01-01',
        layerCount: 1,
        modifierCount: 0,
        activeDeviceCount: 0,
      };
      vi.mocked(mockApi.query).mockResolvedValue([mockProfile]);

      const result = await client.getProfiles();

      // Type assertion check
      expect(result[0].name).toBe('test');
      expect(result[0].layerCount).toBe(1);
    });

    it('returns correctly typed device data', async () => {
      const mockDevice = {
        id: 'dev1',
        name: 'Keyboard',
        path: '/dev/input/event0',
        serial: 'ABC123',
        active: true,
        scope: null,
        layout: null,
      };
      vi.mocked(mockApi.query).mockResolvedValue([mockDevice]);

      const result = await client.getDevices();

      // Type assertion check
      expect(result[0].id).toBe('dev1');
      expect(result[0].active).toBe(true);
    });

    it('returns correctly typed latency metrics', async () => {
      const mockMetrics: LatencyMetrics = {
        min: 100,
        avg: 150,
        max: 200,
        p95: 180,
        p99: 190,
        timestamp: 1234567890,
      };
      vi.mocked(mockApi.query).mockResolvedValue(mockMetrics);

      const result = await client.getLatency();

      // Type assertion check
      expect(result.min).toBe(100);
      expect(result.p95).toBe(180);
    });
  });
});
