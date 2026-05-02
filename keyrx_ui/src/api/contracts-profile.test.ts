/**
 * Profile API Contract Tests
 */

import { describe, it, expect } from 'vitest';
import {
  ProfileListResponseSchema,
  ProfileConfigResponseSchema,
  validateApiResponse,
} from './schemas';

describe('Profile API Contracts', () => {
  describe('GET /api/profiles response schema', () => {
    it('validates correct profile list response', () => {
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
            layerCount: 'not a number',
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
      const mockResponse = {
        name: 'default',
        source: 'map("VK_A", "VK_B");',
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
        config: 123,
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
