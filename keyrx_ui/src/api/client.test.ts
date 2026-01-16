/**
 * API client error handling tests
 *
 * These tests ensure that backend error responses are properly
 * extracted and converted to user-friendly messages.
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { apiClient, ApiError } from './client';

describe('API Client Error Handling', () => {
  beforeEach(() => {
    global.fetch = vi.fn();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe('Backend error object format', () => {
    it('should extract message from backend error object', async () => {
      // Mock backend returning error object format
      (global.fetch as any).mockResolvedValueOnce({
        ok: false,
        status: 500,
        json: async () => ({
          success: false,
          error: {
            code: 'INTERNAL_ERROR',
            message: 'Profile not found',
          },
        }),
      });

      try {
        await apiClient.get('/api/profiles');
        expect.fail('Should have thrown an error');
      } catch (error) {
        expect(error).toBeInstanceOf(ApiError);
        expect((error as ApiError).message).toBe('Profile not found');
        expect((error as ApiError).errorCode).toBe('INTERNAL_ERROR');
        expect((error as ApiError).statusCode).toBe(500);
      }
    });

    it('should handle string error format', async () => {
      // Mock backend returning simple string error
      (global.fetch as any).mockResolvedValueOnce({
        ok: false,
        status: 400,
        json: async () => ({
          success: false,
          error: 'Invalid request',
        }),
      });

      try {
        await apiClient.post('/api/profiles', { name: '' });
        expect.fail('Should have thrown an error');
      } catch (error) {
        expect(error).toBeInstanceOf(ApiError);
        expect((error as ApiError).message).toBe('Invalid request');
      }
    });

    it('should use fallback message when no error provided', async () => {
      // Mock backend returning no error message
      (global.fetch as any).mockResolvedValueOnce({
        ok: false,
        status: 404,
        statusText: 'Not Found',
        json: async () => ({
          success: false,
        }),
      });

      try {
        await apiClient.get('/api/profiles/nonexistent');
        expect.fail('Should have thrown an error');
      } catch (error) {
        expect(error).toBeInstanceOf(ApiError);
        expect((error as ApiError).message).toBe('Request failed: Not Found');
      }
    });
  });

  describe('Profile activation error scenarios', () => {
    it('should handle activation failure with descriptive message', async () => {
      (global.fetch as any).mockResolvedValueOnce({
        ok: false,
        status: 500,
        json: async () => ({
          success: false,
          error: {
            code: 'COMPILATION_ERROR',
            message:
              'Failed to compile profile: syntax error in layer definition',
          },
        }),
      });

      try {
        await apiClient.post('/api/profiles/gaming/activate');
        expect.fail('Should have thrown an error');
      } catch (error) {
        expect(error).toBeInstanceOf(ApiError);
        expect((error as ApiError).message).toContain(
          'Failed to compile profile'
        );
        expect((error as ApiError).message).toContain('syntax error');
        // This should NOT be [object Object]
        expect((error as ApiError).message).not.toBe('[object Object]');
      }
    });

    it('should handle network errors', async () => {
      (global.fetch as any).mockRejectedValueOnce(new Error('Network timeout'));

      try {
        await apiClient.get('/api/profiles');
        expect.fail('Should have thrown an error');
      } catch (error) {
        expect(error).toBeInstanceOf(ApiError);
        expect((error as ApiError).message).toBe('Network timeout');
        expect((error as ApiError).statusCode).toBe(0);
      }
    });

    it('should handle JSON parse errors', async () => {
      (global.fetch as any).mockResolvedValueOnce({
        ok: false,
        status: 500,
        statusText: 'Internal Server Error',
        json: async () => {
          throw new Error('Invalid JSON');
        },
      });

      try {
        await apiClient.get('/api/profiles');
        expect.fail('Should have thrown an error');
      } catch (error) {
        expect(error).toBeInstanceOf(ApiError);
        expect((error as ApiError).message).toBe('Invalid JSON');
      }
    });
  });

  describe('Success responses', () => {
    it('should return data from successful response', async () => {
      (global.fetch as any).mockResolvedValueOnce({
        ok: true,
        status: 200,
        json: async () => ({
          profiles: [
            { name: 'default', isActive: true },
            { name: 'gaming', isActive: false },
          ],
        }),
      });

      const result = await apiClient.get<{ profiles: any[] }>('/api/profiles');
      expect(result.profiles).toHaveLength(2);
      expect(result.profiles[0].name).toBe('default');
    });

    it('should handle 204 No Content', async () => {
      (global.fetch as any).mockResolvedValueOnce({
        ok: true,
        status: 204,
      });

      const result = await apiClient.delete('/api/profiles/test');
      expect(result).toEqual({});
    });
  });

  describe('Regression tests for [object Object] bug', () => {
    it('should never display [object Object] in error messages', async () => {
      const errorFormats = [
        // Backend error object
        {
          error: {
            code: 'ERROR',
            message: 'Test error',
          },
        },
        // Nested error
        {
          error: {
            code: 'ERROR',
            message: {
              detail: 'Complex error',
            },
          },
        },
      ];

      for (const errorFormat of errorFormats) {
        (global.fetch as any).mockResolvedValueOnce({
          ok: false,
          status: 500,
          json: async () => errorFormat,
        });

        try {
          await apiClient.get('/api/test');
          expect.fail('Should have thrown');
        } catch (error) {
          expect(error).toBeInstanceOf(ApiError);
          const message = (error as ApiError).message;
          expect(message).not.toBe('[object Object]');
          expect(message).not.toContain('[object Object]');
        }
      }
    });
  });
});
