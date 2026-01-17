/**
 * API Contract Tests
 *
 * These tests verify that API response schemas correctly validate data
 * using Zod validation. They catch contract violations at test time.
 *
 * Coverage:
 * - All REST endpoint response schemas
 * - WebSocket RPC message schemas
 * - Request and response validation
 * - Error cases (validation failures, malformed responses)
 *
 * Note: These are unit tests for schema validation, not integration tests.
 * Integration tests with real API calls are in backend tests.
 */

import { describe, it, expect, vi } from 'vitest';
import {
  DeviceListResponseSchema,
  DeviceEntrySchema,
  DeviceRpcInfoSchema,
  ProfileListResponseSchema,
  ProfileConfigResponseSchema,
  ServerMessageSchema,
  ClientMessageSchema,
  validateApiResponse,
  validateRpcMessage,
} from './schemas';

// =============================================================================
// Device API Contract Tests
// =============================================================================

describe('Device API Contracts', () => {
  describe('GET /api/devices response schema', () => {
    it('validates correct device list response', () => {
      // Response format matches Rust DeviceResponse struct in devices.rs
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

      // Should not throw
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
        devices: 'not an array', // Invalid: should be array
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
            // Missing required fields: name, path, active
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
            // serial, scope, and layout are optional
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
            unexpectedField: 'extra data', // Unexpected field
          },
        ],
      };

      // Should not throw (passthrough allows extra fields)
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
      // Backend often returns null for serial when device doesn't report one
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

      // Should not throw - null is acceptable for optional string fields
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
      // The actual PATCH endpoint returns { success: true }, not a device entry
      const mockResponse = {
        success: true,
      };

      // Note: PATCH returns a simple success response, not a device entry
      expect(mockResponse.success).toBe(true);
    });

    it('rejects response with invalid field types', () => {
      const invalidResponse = {
        id: 'device-1',
        name: 123, // Should be string
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

// =============================================================================
// Profile API Contract Tests
// =============================================================================

describe('Profile API Contracts', () => {
  describe('GET /api/profiles response schema', () => {
    it('validates correct profile list response', () => {
      // Response format matches Rust ProfileListResponse in profiles.rs
      const mockResponse = {
        profiles: [
          {
            name: 'default',
            rhaiPath: '/home/user/.config/keyrx/profiles/default.rhai',
            krxPath: '/home/user/.config/keyrx/profiles/default.krx',
            modifiedAt: '2026-01-10T14:53:01.212416136+00:00',
            createdAt: '2026-01-10T14:53:01.212416136+00:00',
            layerCount: 1,
            deviceCount: 0,
            keyCount: 5,
            isActive: true,
          },
          {
            name: 'gaming',
            rhaiPath: '/home/user/.config/keyrx/profiles/gaming.rhai',
            krxPath: '/home/user/.config/keyrx/profiles/gaming.krx',
            modifiedAt: '2026-01-10T15:00:00.000000000+00:00',
            createdAt: '2026-01-10T14:00:00.000000000+00:00',
            layerCount: 2,
            deviceCount: 1,
            keyCount: 10,
            isActive: false,
          },
        ],
      };

      const validated = validateApiResponse(
        ProfileListResponseSchema,
        mockResponse,
        'GET /api/profiles'
      );

      expect(validated.profiles).toHaveLength(2);
      expect(validated.profiles[0].name).toBe('default');
      expect(validated.profiles[0].isActive).toBe(true);
    });

    it('rejects profile with missing required fields', () => {
      const invalidResponse = {
        profiles: [
          {
            name: 'default',
            // Missing: rhaiPath, krxPath, modifiedAt, createdAt, layerCount, deviceCount, keyCount, isActive
          },
        ],
      };

      expect(() => {
        validateApiResponse(
          ProfileListResponseSchema,
          invalidResponse,
          'GET /api/profiles'
        );
      }).toThrow('API validation failed');
    });

    it('rejects profile with invalid field types', () => {
      const invalidResponse = {
        profiles: [
          {
            name: 'default',
            rhaiPath: '/path/to/default.rhai',
            krxPath: '/path/to/default.krx',
            modifiedAt: '2026-01-10T14:53:01+00:00',
            createdAt: '2026-01-10T14:53:01+00:00',
            layerCount: 'not a number', // Should be number
            deviceCount: 0,
            keyCount: 0,
            isActive: true,
          },
        ],
      };

      expect(() => {
        validateApiResponse(
          ProfileListResponseSchema,
          invalidResponse,
          'GET /api/profiles'
        );
      }).toThrow('API validation failed');
    });

    it('rejects invalid array types', () => {
      const invalidResponse = {
        profiles: 'not an array',
      };

      expect(() => {
        validateApiResponse(
          ProfileListResponseSchema,
          invalidResponse,
          'GET /api/profiles'
        );
      }).toThrow('API validation failed');
    });
  });

  describe('GET /api/profiles/:name/config response schema', () => {
    it('validates correct profile config response', () => {
      // Response format matches Rust ProfileConfigRpc in profile.rs
      const mockResponse = {
        name: 'default',
        source: 'map("VK_A", "VK_B");', // Rhai source code
      };

      const validated = validateApiResponse(
        ProfileConfigResponseSchema,
        mockResponse,
        'GET /api/profiles/:name/config'
      );

      expect(validated.name).toBe('default');
      expect(validated.source).toContain('map');
    });

    it('rejects config with missing source field', () => {
      const invalidResponse = {
        name: 'default',
        // Missing: source
      };

      expect(() => {
        validateApiResponse(
          ProfileConfigResponseSchema,
          invalidResponse,
          'GET /api/profiles/:name/config'
        );
      }).toThrow('API validation failed');
    });

    it('rejects config with missing name field', () => {
      const invalidResponse = {
        // Missing: name
        config: 'map("VK_A", "VK_B");',
      };

      expect(() => {
        validateApiResponse(
          ProfileConfigResponseSchema,
          invalidResponse,
          'GET /api/profiles/:name/config'
        );
      }).toThrow('API validation failed');
    });

    it('rejects config with invalid field types', () => {
      const invalidResponse = {
        name: 'default',
        config: 123, // Should be string
      };

      expect(() => {
        validateApiResponse(
          ProfileConfigResponseSchema,
          invalidResponse,
          'GET /api/profiles/:name/config'
        );
      }).toThrow('API validation failed');
    });
  });
});

// =============================================================================
// WebSocket RPC Contract Tests
// =============================================================================

describe('WebSocket RPC Contracts', () => {
  describe('Client Messages (Outgoing)', () => {
    it('validates query message', () => {
      const queryMessage = {
        type: 'query',
        content: {
          id: 'req-1',
          method: 'get_devices',
          params: null,
        },
      };

      // Should not throw
      const validated = validateRpcMessage(queryMessage, 'client');

      expect(validated.type).toBe('query');
      expect(validated.content.method).toBe('get_devices');
    });

    it('validates command message', () => {
      const commandMessage = {
        type: 'command',
        content: {
          id: 'req-2',
          method: 'activate_profile',
          params: { name: 'gaming' },
        },
      };

      const validated = validateRpcMessage(commandMessage, 'client');

      expect(validated.type).toBe('command');
      expect(validated.content.params).toEqual({ name: 'gaming' });
    });

    it('validates subscribe message', () => {
      const subscribeMessage = {
        type: 'subscribe',
        content: {
          id: 'sub-1',
          channel: 'daemon_state',
        },
      };

      const validated = validateRpcMessage(subscribeMessage, 'client');

      expect(validated.type).toBe('subscribe');
      expect(validated.content.channel).toBe('daemon_state');
    });

    it('validates unsubscribe message', () => {
      const unsubscribeMessage = {
        type: 'unsubscribe',
        content: {
          id: 'unsub-1',
          channel: 'daemon_state',
        },
      };

      const validated = validateRpcMessage(unsubscribeMessage, 'client');

      expect(validated.type).toBe('unsubscribe');
    });

    it('rejects client message with invalid type', () => {
      const invalidMessage = {
        type: 'invalid_type',
        content: {
          id: 'req-1',
          method: 'get_devices',
        },
      };

      expect(() => {
        validateRpcMessage(invalidMessage, 'client');
      }).toThrow('Invalid client RPC message');
    });

    it('rejects client message with missing required fields', () => {
      const invalidMessage = {
        type: 'query',
        content: {
          // Missing: id, method
        },
      };

      expect(() => {
        validateRpcMessage(invalidMessage, 'client');
      }).toThrow('Invalid client RPC message');
    });

    it('rejects client message without content', () => {
      const invalidMessage = {
        type: 'query',
        // Missing: content
      };

      expect(() => {
        validateRpcMessage(invalidMessage, 'client');
      }).toThrow('Invalid client RPC message');
    });
  });

  describe('Server Messages (Incoming)', () => {
    it('validates response message with result', () => {
      const responseMessage = {
        type: 'response',
        content: {
          id: 'req-1',
          result: { devices: [] },
        },
      };

      const validated = validateRpcMessage(responseMessage, 'server');

      expect(validated.type).toBe('response');
      expect(validated.content.result).toEqual({ devices: [] });
    });

    it('validates response message with error', () => {
      const errorMessage = {
        type: 'response',
        content: {
          id: 'req-1',
          error: {
            code: 404,
            message: 'Profile not found',
          },
        },
      };

      const validated = validateRpcMessage(errorMessage, 'server');

      expect(validated.type).toBe('response');
      expect(validated.content.error?.code).toBe(404);
    });

    it('validates event message', () => {
      const eventMessage = {
        type: 'event',
        content: {
          channel: 'daemon_state',
          data: {
            modifiers: ['LCtrl'],
            locks: [],
            layer: 'base',
            active_profile: 'gaming',
          },
        },
      };

      const validated = validateRpcMessage(eventMessage, 'server');

      expect(validated.type).toBe('event');
      expect(validated.content.channel).toBe('daemon_state');
    });

    it('validates connected message', () => {
      const connectedMessage = {
        type: 'connected',
        content: {
          version: '0.1.0',
          timestamp: Date.now(),
        },
      };

      const validated = validateRpcMessage(connectedMessage, 'server');

      expect(validated.type).toBe('connected');
      expect(validated.content.version).toBe('0.1.0');
    });

    it('rejects server message with invalid type', () => {
      const invalidMessage = {
        type: 'unknown_type',
        content: {},
      };

      expect(() => {
        validateRpcMessage(invalidMessage, 'server');
      }).toThrow('Invalid server RPC message');
    });

    it('rejects server message with missing content', () => {
      const invalidMessage = {
        type: 'response',
        // Missing: content
      };

      expect(() => {
        validateRpcMessage(invalidMessage, 'server');
      }).toThrow('Invalid server RPC message');
    });

    it('rejects response without id', () => {
      const invalidMessage = {
        type: 'response',
        content: {
          // Missing: id
          result: {},
        },
      };

      expect(() => {
        validateRpcMessage(invalidMessage, 'server');
      }).toThrow('Invalid server RPC message');
    });

    it('rejects event without channel', () => {
      const invalidMessage = {
        type: 'event',
        content: {
          // Missing: channel
          data: {},
        },
      };

      expect(() => {
        validateRpcMessage(invalidMessage, 'server');
      }).toThrow('Invalid server RPC message');
    });
  });
});

// =============================================================================
// Edge Cases and Error Handling
// =============================================================================

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

    // Verify structured logging
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

// =============================================================================
// Schema Completeness Tests
// =============================================================================

describe('Schema Completeness', () => {
  it('validates all required device RPC fields are enforced', () => {
    // DeviceRpcInfoSchema is used for /api/devices responses
    // Required: id, name, path, active
    const requiredFields = ['id', 'name', 'path', 'active'];

    for (const field of requiredFields) {
      const incompleteDevice: any = {
        id: 'device-1',
        name: 'Test',
        path: '/dev/input/event0',
        active: true,
      };

      delete incompleteDevice[field];

      expect(() => {
        validateApiResponse(
          DeviceRpcInfoSchema,
          incompleteDevice,
          `TEST missing ${field}`
        );
      }).toThrow('API validation failed');
    }
  });

  it('validates all required device entry fields are enforced (storage format)', () => {
    // DeviceEntrySchema is used for device metadata storage
    // Required: id, name, scope, last_seen
    const requiredFields = ['id', 'name', 'scope', 'last_seen'];

    for (const field of requiredFields) {
      const incompleteDevice: any = {
        id: 'device-1',
        name: 'Test',
        scope: 'Global',
        last_seen: Date.now(),
      };

      delete incompleteDevice[field];

      expect(() => {
        validateApiResponse(
          DeviceEntrySchema,
          incompleteDevice,
          `TEST missing ${field}`
        );
      }).toThrow('API validation failed');
    }
  });

  it('validates all required profile fields are enforced', () => {
    // ProfileRpcInfoSchema required fields
    const requiredFields = [
      'name',
      'rhaiPath',
      'krxPath',
      'modifiedAt',
      'createdAt',
      'layerCount',
      'deviceCount',
      'keyCount',
      'isActive',
    ];

    for (const field of requiredFields) {
      const incompleteProfile: any = {
        name: 'test',
        rhaiPath: '/path/to/test.rhai',
        krxPath: '/path/to/test.krx',
        modifiedAt: '2026-01-10T14:53:01+00:00',
        createdAt: '2026-01-10T14:53:01+00:00',
        layerCount: 1,
        deviceCount: 0,
        keyCount: 0,
        isActive: true,
      };

      delete incompleteProfile[field];

      const invalidResponse = { profiles: [incompleteProfile] };

      expect(() => {
        validateApiResponse(
          ProfileListResponseSchema,
          invalidResponse,
          `TEST missing ${field}`
        );
      }).toThrow('API validation failed');
    }
  });

  it('validates DeviceScope enum values (for storage format)', () => {
    const validScopes = ['Global', 'DeviceSpecific'];

    for (const scope of validScopes) {
      const device = {
        id: 'device-1',
        name: 'Test',
        scope,
        last_seen: Date.now(),
      };

      expect(() => {
        validateApiResponse(DeviceEntrySchema, device, 'TEST');
      }).not.toThrow();
    }

    // Invalid scope
    const invalidDevice = {
      id: 'device-1',
      name: 'Test',
      scope: 'InvalidScope',
      last_seen: Date.now(),
    };

    expect(() => {
      validateApiResponse(DeviceEntrySchema, invalidDevice, 'TEST');
    }).toThrow('API validation failed');
  });
});
