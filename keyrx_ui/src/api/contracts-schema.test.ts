/**
 * Schema Completeness Tests
 */

import { describe, it, expect } from 'vitest';
import {
  DeviceEntrySchema,
  DeviceRpcInfoSchema,
  ProfileListResponseSchema,
  validateApiResponse,
} from './schemas';

describe('Schema Completeness', () => {
  it('validates all required device RPC fields are enforced', () => {
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
