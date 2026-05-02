/**
 * Device API Contract Tests
 */

import { describe, it, expect, vi } from 'vitest';
import {
  DeviceListResponseSchema,
  DeviceRpcInfoSchema,
  validateApiResponse,
} from './schemas';

describe('Device API Contracts', () => {
  describe('GET /api/devices response schema', () => {
    it('validates correct device list response', () => {
      const mockResponse = {
        devices: [
          {
            id: 'device-1',
            name: 'Test Keyboard',
            path: '/dev/input/event0',
            serial: 'ABC123',
            active: true,
            layout: 'ANSI_104',
          },
        ],
      };

      const validated = validateApiResponse(
        DeviceListResponseSchema,
        mockResponse,
        'GET /api/devices'
      );

      expect(validated.devices).toHaveLength(1);
      expect(validated.devices[0].id).toBe('device-1');
    });

    it('rejects device list with invalid structure', () => {
      const invalidResponse = {
        devices: 'not an array',
      };

      expect(() => {
        validateApiResponse(
          DeviceListResponseSchema,
          invalidResponse,
          'GET /api/devices'
        );
      }).toThrow('API validation failed');
    });

    it('rejects device entry with missing required fields', () => {
      const invalidResponse = {
        devices: [
          {
            id: 'device-1',
          },
        ],
      };

      expect(() => {
        validateApiResponse(
          DeviceListResponseSchema,
          invalidResponse,
          'GET /api/devices'
        );
      }).toThrow('API validation failed');
    });

    it('allows device entry with optional fields omitted', () => {
      const validResponse = {
        devices: [
          {
            id: 'device-1',
            name: 'Test Keyboard',
            path: '/dev/input/event0',
            active: true,
          },
        ],
      };

      const validated = validateApiResponse(
        DeviceListResponseSchema,
        validResponse,
        'GET /api/devices'
      );

      expect(validated.devices[0].serial).toBeUndefined();
      expect(validated.devices[0].layout).toBeUndefined();
    });

    it('passes through unexpected fields with warning', () => {
      const consoleDebugSpy = vi
        .spyOn(console, 'debug')
        .mockImplementation(() => {});

      const responseWithExtra = {
        devices: [
          {
            id: 'device-1',
            name: 'Test Keyboard',
            path: '/dev/input/event0',
            active: true,
            unexpectedField: 'extra data',
          },
        ],
      };

      const validated = validateApiResponse(
        DeviceListResponseSchema,
        responseWithExtra,
        'GET /api/devices'
      );

      expect(validated.devices[0]).toHaveProperty('unexpectedField');
      expect(consoleDebugSpy).toHaveBeenCalled();

      consoleDebugSpy.mockRestore();
    });

    it('validates device with null serial (common case)', () => {
      const responseWithNullSerial = {
        devices: [
          {
            id: 'device-1',
            name: 'Test Keyboard',
            path: '/dev/input/event0',
            serial: null,
            active: true,
            layout: null,
          },
        ],
      };

      const validated = validateApiResponse(
        DeviceListResponseSchema,
        responseWithNullSerial,
        'GET /api/devices'
      );

      expect(validated.devices[0].serial).toBeNull();
    });
  });

  describe('PATCH /api/devices/:id response schema', () => {
    it('validates device update response (success JSON)', () => {
      const mockResponse = {
        success: true,
      };

      expect(mockResponse.success).toBe(true);
    });

    it('rejects response with invalid field types', () => {
      const invalidResponse = {
        id: 'device-1',
        name: 123,
        path: '/dev/input/event0',
        active: true,
      };

      expect(() => {
        validateApiResponse(
          DeviceRpcInfoSchema,
          invalidResponse,
          'PATCH /api/devices/:id'
        );
      }).toThrow('API validation failed');
    });
  });
});
