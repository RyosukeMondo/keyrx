/**
 * Device management API client tests
 *
 * Tests for fetchDevices, renameDevice, setDeviceScope, and forgetDevice.
 * Covers network failures, invalid device IDs, and schema validation errors.
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { apiClient, ApiError } from './client';
import {
  fetchDevices,
  renameDevice,
  setDeviceScope,
  forgetDevice,
} from './devices';

// Mock the deviceStorage utility to avoid localStorage issues in tests
vi.mock('../utils/deviceStorage', () => ({
  setDeviceEnabled: vi.fn(),
}));

describe('Device API Client', () => {
  beforeEach(() => {
    global.fetch = vi.fn();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe('fetchDevices', () => {
    it('should fetch and transform devices correctly', async () => {
      (global.fetch as any).mockResolvedValueOnce({
        ok: true,
        status: 200,
        json: async () => ({
          devices: [
            {
              id: 'device-1',
              name: 'AT Translated Set 2 keyboard',
              path: '/dev/input/event3',
              serial: '12345',
              active: true,
              scope: 'Global',
              layout: 'ANSI_104',
            },
            {
              id: 'device-2',
              name: 'keyrx-virtual',
              path: '/dev/input/keyrx',
              serial: null,
              active: true,
              scope: 'DeviceSpecific',
              layout: null,
            },
          ],
        }),
      });

      const devices = await fetchDevices();

      expect(devices).toHaveLength(2);
      expect(devices[0]).toMatchObject({
        id: 'device-1',
        name: 'AT Translated Set 2 keyboard',
        path: '/dev/input/event3',
        serial: '12345',
        active: true,
        scope: 'global',
        layout: 'ANSI_104',
        isVirtual: false,
        enabled: true,
      });
      expect(devices[1]).toMatchObject({
        id: 'device-2',
        name: 'keyrx-virtual',
        scope: 'device-specific',
        isVirtual: true,
      });
    });

    it('should default scope to global when unset', async () => {
      (global.fetch as any).mockResolvedValueOnce({
        ok: true,
        status: 200,
        json: async () => ({
          devices: [
            {
              id: 'device-1',
              name: 'Keyboard',
              path: '/dev/input/event3',
              serial: null,
              active: true,
              scope: null,
              layout: null,
            },
          ],
        }),
      });

      const devices = await fetchDevices();
      expect(devices[0].scope).toBe('global');
    });

    it('should handle network failure', async () => {
      (global.fetch as any).mockRejectedValueOnce(new Error('Network timeout'));

      try {
        await fetchDevices();
        expect.fail('Should have thrown');
      } catch (error) {
        expect(error).toBeInstanceOf(ApiError);
        expect((error as ApiError).message).toBe('Network timeout');
        expect((error as ApiError).statusCode).toBe(0);
      }
    });

    it('should handle 404 not found', async () => {
      (global.fetch as any).mockResolvedValueOnce({
        ok: false,
        status: 404,
        statusText: 'Not Found',
        json: async () => ({
          success: false,
          error: 'No devices found',
        }),
      });

      try {
        await fetchDevices();
        expect.fail('Should have thrown');
      } catch (error) {
        expect(error).toBeInstanceOf(ApiError);
        expect((error as ApiError).statusCode).toBe(404);
      }
    });

    it('should handle server error', async () => {
      (global.fetch as any).mockResolvedValueOnce({
        ok: false,
        status: 500,
        json: async () => ({
          success: false,
          error: {
            code: 'INTERNAL_ERROR',
            message: 'Failed to enumerate devices',
          },
        }),
      });

      await expect(fetchDevices()).rejects.toThrow('Failed to enumerate devices');
    });

    it('should handle invalid response schema', async () => {
      (global.fetch as any).mockResolvedValueOnce({
        ok: true,
        status: 200,
        json: async () => ({
          devices: 'not-an-array', // Invalid: should be array
        }),
      });

      await expect(fetchDevices()).rejects.toThrow(/API validation failed/);
    });
  });

  describe('renameDevice', () => {
    it('should rename device successfully', async () => {
      (global.fetch as any).mockResolvedValueOnce({
        ok: true,
        status: 200,
        json: async () => ({
          success: true,
        }),
      });

      const result = await renameDevice('device-1', 'My Keyboard');

      expect(result).toEqual({ success: true });
      expect(global.fetch).toHaveBeenCalledWith(
        expect.stringContaining('/api/devices/device-1/name'),
        expect.objectContaining({
          method: 'PUT',
          body: JSON.stringify({ name: 'My Keyboard' }),
        })
      );
    });

    it('should handle invalid device ID (404)', async () => {
      (global.fetch as any).mockResolvedValueOnce({
        ok: false,
        status: 404,
        json: async () => ({
          success: false,
          error: {
            code: 'NOT_FOUND',
            message: 'Device not found',
          },
        }),
      });

      await expect(renameDevice('nonexistent-id', 'New Name')).rejects.toThrow(
        'Device not found'
      );
    });

    it('should handle network failure during rename', async () => {
      (global.fetch as any).mockRejectedValueOnce(new Error('Connection reset'));

      await expect(
        renameDevice('device-1', 'New Name')
      ).rejects.toThrow('Connection reset');
    });

    it('should handle invalid device ID format (400)', async () => {
      (global.fetch as any).mockResolvedValueOnce({
        ok: false,
        status: 400,
        json: async () => ({
          success: false,
          error: {
            code: 'INVALID_INPUT',
            message: 'Invalid device ID format',
          },
        }),
      });

      await expect(
        renameDevice('invalid-id-with-special-chars-!@#', 'Name')
      ).rejects.toThrow('Invalid device ID format');
    });

    it('should handle empty name validation error', async () => {
      (global.fetch as any).mockResolvedValueOnce({
        ok: false,
        status: 400,
        json: async () => ({
          success: false,
          error: 'Name cannot be empty',
        }),
      });

      await expect(renameDevice('device-1', '')).rejects.toThrow(
        'Name cannot be empty'
      );
    });
  });

  describe('setDeviceScope', () => {
    it('should set device scope to global', async () => {
      (global.fetch as any).mockResolvedValueOnce({
        ok: true,
        status: 200,
        json: async () => ({
          success: true,
        }),
      });

      const result = await setDeviceScope('device-1', 'global');

      expect(result).toEqual({ success: true });
      expect(global.fetch).toHaveBeenCalledWith(
        expect.stringContaining('/api/devices/device-1/scope'),
        expect.objectContaining({
          method: 'PUT',
          body: JSON.stringify({ scope: 'global' }),
        })
      );
    });

    it('should set device scope to device-specific', async () => {
      (global.fetch as any).mockResolvedValueOnce({
        ok: true,
        status: 200,
        json: async () => ({
          success: true,
        }),
      });

      const result = await setDeviceScope('device-1', 'device-specific');

      expect(result).toEqual({ success: true });
    });

    it('should handle invalid device ID (404)', async () => {
      (global.fetch as any).mockResolvedValueOnce({
        ok: false,
        status: 404,
        json: async () => ({
          success: false,
          error: {
            code: 'NOT_FOUND',
            message: 'Device not found',
          },
        }),
      });

      await expect(
        setDeviceScope('nonexistent-id', 'global')
      ).rejects.toThrow('Device not found');
    });

    it('should handle network failure during scope change', async () => {
      (global.fetch as any).mockRejectedValueOnce(
        new Error('Request timeout')
      );

      await expect(
        setDeviceScope('device-1', 'global')
      ).rejects.toThrow('Request timeout');
    });

    it('should handle invalid scope value (400)', async () => {
      (global.fetch as any).mockResolvedValueOnce({
        ok: false,
        status: 400,
        json: async () => ({
          success: false,
          error: 'Invalid scope value. Must be "global" or "device-specific"',
        }),
      });

      await expect(
        setDeviceScope('device-1', 'invalid-scope' as any)
      ).rejects.toThrow(
        'Invalid scope value. Must be "global" or "device-specific"'
      );
    });
  });

  describe('forgetDevice', () => {
    it('should forget device successfully', async () => {
      (global.fetch as any).mockResolvedValueOnce({
        ok: true,
        status: 200,
        json: async () => ({
          success: true,
        }),
      });

      const result = await forgetDevice('device-1');

      expect(result).toEqual({ success: true });
      expect(global.fetch).toHaveBeenCalledWith(
        expect.stringContaining('/api/devices/device-1'),
        expect.objectContaining({
          method: 'DELETE',
        })
      );
    });

    it('should handle invalid device ID (404)', async () => {
      (global.fetch as any).mockResolvedValueOnce({
        ok: false,
        status: 404,
        json: async () => ({
          success: false,
          error: {
            code: 'NOT_FOUND',
            message: 'Device not found',
          },
        }),
      });

      await expect(forgetDevice('nonexistent-id')).rejects.toThrow(
        'Device not found'
      );
    });

    it('should handle network failure during forget', async () => {
      (global.fetch as any).mockRejectedValueOnce(new Error('Network unreachable'));

      await expect(forgetDevice('device-1')).rejects.toThrow(
        'Network unreachable'
      );
    });

    it('should handle device in use error (409)', async () => {
      (global.fetch as any).mockResolvedValueOnce({
        ok: false,
        status: 409,
        json: async () => ({
          success: false,
          error: {
            code: 'DEVICE_IN_USE',
            message: 'Cannot forget device while it is active',
          },
        }),
      });

      await expect(forgetDevice('active-device')).rejects.toThrow(
        'Cannot forget device while it is active'
      );
    });

    it('should handle server error during forget (500)', async () => {
      (global.fetch as any).mockResolvedValueOnce({
        ok: false,
        status: 500,
        json: async () => ({
          success: false,
          error: {
            code: 'INTERNAL_ERROR',
            message: 'Failed to remove device',
          },
        }),
      });

      await expect(forgetDevice('device-1')).rejects.toThrow(
        'Failed to remove device'
      );
    });
  });
});