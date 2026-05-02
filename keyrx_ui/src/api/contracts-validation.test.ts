/**
 * Validation Error Handling Tests
 */

import { describe, it, expect, vi } from 'vitest';
import {
  DeviceListResponseSchema,
  DeviceEntrySchema,
  validateApiResponse,
} from './schemas';

describe('Validation Error Handling', () => {
  it('includes endpoint in error message', () => {
    const invalidData = { invalid: 'structure' };

    try {
      validateApiResponse(
        DeviceListResponseSchema,
        invalidData,
        'GET /api/devices'
      );
      expect.fail('Should have thrown');
    } catch (error) {
      expect((error as Error).message).toContain('GET /api/devices');
      expect((error as Error).message).toContain('validation failed');
    }
  });

  it('logs structured error on validation failure', () => {
    const consoleErrorSpy = vi
      .spyOn(console, 'error')
      .mockImplementation(() => {});

    const invalidData = { devices: 'not an array' };

    try {
      validateApiResponse(
        DeviceListResponseSchema,
        invalidData,
        'GET /api/devices'
      );
    } catch {
      // Expected to throw
    }

    expect(consoleErrorSpy).toHaveBeenCalled();

    const logCall = consoleErrorSpy.mock.calls[0][0];
    const logData = JSON.parse(logCall);

    expect(logData).toHaveProperty('timestamp');
    expect(logData).toHaveProperty('level', 'error');
    expect(logData).toHaveProperty('service', 'API Validation');
    expect(logData).toHaveProperty('event', 'validation_failed');
    expect(logData.context).toHaveProperty('endpoint', 'GET /api/devices');

    consoleErrorSpy.mockRestore();
  });

  it('handles null and undefined data gracefully', () => {
    expect(() => {
      validateApiResponse(DeviceListResponseSchema, null, 'TEST');
    }).toThrow('API validation failed');

    expect(() => {
      validateApiResponse(DeviceListResponseSchema, undefined, 'TEST');
    }).toThrow('API validation failed');
  });

  it('handles non-object data gracefully', () => {
    expect(() => {
      validateApiResponse(DeviceListResponseSchema, 'string', 'TEST');
    }).toThrow('API validation failed');

    expect(() => {
      validateApiResponse(DeviceListResponseSchema, 123, 'TEST');
    }).toThrow('API validation failed');

    expect(() => {
      validateApiResponse(DeviceListResponseSchema, true, 'TEST');
    }).toThrow('API validation failed');
  });

  it('handles arrays as invalid input for object schemas', () => {
    expect(() => {
      validateApiResponse(DeviceEntrySchema, [], 'TEST');
    }).toThrow('API validation failed');
  });
});
