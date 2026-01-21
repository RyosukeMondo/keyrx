/**
 * API Client Library
 *
 * Type-safe, validated API client for testing the KeyRx daemon REST API.
 * Uses Zod schemas for runtime validation and automatic retry on network errors.
 */

import { z } from 'zod';
import {
  DeviceListResponseSchema,
  ProfileListResponseSchema,
  ProfileConfigResponseSchema,
  StatusResponseSchema,
  UpdateDeviceConfigResponseSchema,
  LatencyRpcStatsSchema,
} from '../../keyrx_ui/src/api/schemas';

export interface ApiClientConfig {
  baseUrl: string;
  timeoutMs?: number;
  maxRetries?: number;
  retryDelayMs?: number;
}

export interface ApiError {
  code: number;
  message: string;
  data?: unknown;
}

export class ApiClientError extends Error {
  constructor(
    message: string,
    public statusCode: number,
    public apiError?: ApiError
  ) {
    super(message);
    this.name = 'ApiClientError';
  }
}

/**
 * Type-safe API client for KeyRx daemon REST API
 */
export class ApiClient {
  private config: Required<ApiClientConfig>;

  constructor(config: ApiClientConfig) {
    this.config = {
      baseUrl: config.baseUrl,
      timeoutMs: config.timeoutMs ?? 5000,
      maxRetries: config.maxRetries ?? 3,
      retryDelayMs: config.retryDelayMs ?? 1000,
    };
  }

  /**
   * Make HTTP request with retry logic and timeout
   */
  private async request<T>(
    method: string,
    endpoint: string,
    schema: z.ZodSchema<T>,
    options?: {
      body?: unknown;
      headers?: Record<string, string>;
      skipRetry?: boolean;
    }
  ): Promise<T> {
    const url = `${this.config.baseUrl}${endpoint}`;
    let lastError: Error | null = null;
    const maxAttempts = options?.skipRetry ? 1 : this.config.maxRetries;

    for (let attempt = 0; attempt < maxAttempts; attempt++) {
      try {
        const controller = new AbortController();
        const timeoutId = setTimeout(() => controller.abort(), this.config.timeoutMs);

        const response = await fetch(url, {
          method,
          headers: {
            'Content-Type': 'application/json',
            ...options?.headers,
          },
          body: options?.body ? JSON.stringify(options.body) : undefined,
          signal: controller.signal,
        });

        clearTimeout(timeoutId);

        // Handle HTTP errors
        if (!response.ok) {
          const errorBody = await response.json().catch(() => ({}));

          // Try to extract API error structure
          const apiError: ApiError | undefined = errorBody.error
            ? {
                code: errorBody.error.code ?? response.status,
                message: errorBody.error.message ?? response.statusText,
                data: errorBody.error.data,
              }
            : undefined;

          throw new ApiClientError(
            `HTTP ${response.status}: ${response.statusText}`,
            response.status,
            apiError
          );
        }

        // Parse and validate response
        const data = await response.json();
        const result = schema.safeParse(data);

        if (!result.success) {
          throw new Error(`Response validation failed: ${result.error.message}`);
        }

        return result.data;
      } catch (error) {
        lastError = error instanceof Error ? error : new Error(String(error));

        // Don't retry on validation errors or client errors (4xx)
        if (
          error instanceof ApiClientError &&
          error.statusCode >= 400 &&
          error.statusCode < 500
        ) {
          throw error;
        }

        // Don't retry validation errors
        if (error instanceof Error && error.message.includes('validation failed')) {
          throw error;
        }

        // Retry on network errors (ECONNREFUSED, timeout, etc.)
        if (attempt < maxAttempts - 1) {
          const delay = this.config.retryDelayMs * Math.pow(2, attempt); // Exponential backoff
          await new Promise(resolve => setTimeout(resolve, delay));
          continue;
        }

        // Max retries reached
        throw lastError;
      }
    }

    throw lastError || new Error('Request failed after retries');
  }

  /**
   * GET /api/status - Get daemon status
   */
  async getStatus(): Promise<z.infer<typeof StatusResponseSchema>> {
    return this.request('GET', '/api/status', StatusResponseSchema);
  }

  /**
   * GET /api/devices - List all devices
   */
  async getDevices(): Promise<z.infer<typeof DeviceListResponseSchema>> {
    return this.request('GET', '/api/devices', DeviceListResponseSchema);
  }

  /**
   * GET /api/profiles - List all profiles
   */
  async getProfiles(): Promise<z.infer<typeof ProfileListResponseSchema>> {
    return this.request('GET', '/api/profiles', ProfileListResponseSchema);
  }

  /**
   * GET /api/profiles/:name/config - Get profile configuration
   */
  async getProfileConfig(name: string): Promise<z.infer<typeof ProfileConfigResponseSchema>> {
    return this.request(
      'GET',
      `/api/profiles/${encodeURIComponent(name)}/config`,
      ProfileConfigResponseSchema
    );
  }

  /**
   * POST /api/profiles - Create new profile
   */
  async createProfile(name: string): Promise<{ success: boolean; name: string }> {
    const schema = z.object({
      success: z.boolean(),
      name: z.string(),
    });

    return this.request('POST', '/api/profiles', schema, {
      body: { name },
    });
  }

  /**
   * POST /api/profiles/:name/activate - Activate profile
   */
  async activateProfile(name: string): Promise<{
    success: boolean;
    compile_time_ms?: number;
    reload_time_ms?: number;
    error?: string;
  }> {
    const schema = z.object({
      success: z.boolean(),
      compile_time_ms: z.number().optional(),
      reload_time_ms: z.number().optional(),
      error: z.string().optional(),
    });

    return this.request(
      'POST',
      `/api/profiles/${encodeURIComponent(name)}/activate`,
      schema
    );
  }

  /**
   * DELETE /api/profiles/:name - Delete profile
   */
  async deleteProfile(name: string): Promise<{ success: boolean }> {
    const schema = z.object({
      success: z.boolean(),
    });

    return this.request('DELETE', `/api/profiles/${encodeURIComponent(name)}`, schema);
  }

  /**
   * PATCH /api/devices/:id - Update device configuration
   */
  async patchDevice(
    deviceId: string,
    updates: {
      name?: string;
      scope?: string;
      layout?: string;
    }
  ): Promise<z.infer<typeof UpdateDeviceConfigResponseSchema>> {
    return this.request(
      'PATCH',
      `/api/devices/${encodeURIComponent(deviceId)}`,
      UpdateDeviceConfigResponseSchema,
      {
        body: updates,
      }
    );
  }

  /**
   * GET /api/metrics/latency - Get latency statistics
   */
  async getMetrics(): Promise<z.infer<typeof LatencyRpcStatsSchema>> {
    return this.request('GET', '/api/metrics/latency', LatencyRpcStatsSchema);
  }

  /**
   * GET /api/layouts - Get available keyboard layouts
   */
  async getLayouts(): Promise<{
    layouts: Array<{
      id: string;
      name: string;
      description?: string;
    }>;
  }> {
    const schema = z.object({
      layouts: z.array(
        z.object({
          id: z.string(),
          name: z.string(),
          description: z.string().optional(),
        })
      ),
    });

    return this.request('GET', '/api/layouts', schema);
  }

  /**
   * POST /api/profiles/:name/config - Set profile configuration
   */
  async setProfileConfig(
    name: string,
    source: string
  ): Promise<{ success: boolean }> {
    const schema = z.object({
      success: z.boolean(),
    });

    return this.request(
      'POST',
      `/api/profiles/${encodeURIComponent(name)}/config`,
      schema,
      {
        body: { source },
      }
    );
  }

  /**
   * Get base URL
   */
  getBaseUrl(): string {
    return this.config.baseUrl;
  }
}
