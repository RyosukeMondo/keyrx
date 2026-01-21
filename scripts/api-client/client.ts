/**
 * Type-safe API client for KeyRx daemon REST API
 *
 * Provides strongly-typed methods for all REST endpoints with:
 * - Zod schema validation for responses
 * - Automatic retry with exponential backoff
 * - Request/response timeout handling
 * - Comprehensive error handling
 */

import { z } from 'zod';
import {
  StatusResponseSchema,
  DeviceListResponseSchema,
  ProfileListResponseSchema,
  ProfileConfigResponseSchema,
  UpdateDeviceConfigResponseSchema,
  ActivationRpcResultSchema,
  DeviceRpcInfoSchema,
  ProfileRpcInfoSchema,
  validateApiResponse,
} from '../../keyrx_ui/src/api/schemas.ts';

/**
 * API client configuration options
 */
export interface ApiClientConfig {
  baseUrl: string;
  timeout?: number;
  maxRetries?: number;
  retryDelayMs?: number;
}

/**
 * HTTP request options
 */
interface RequestOptions {
  method: 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE';
  path: string;
  body?: unknown;
  timeout?: number;
  retries?: number;
}

/**
 * Validated API response wrapper
 */
interface ApiResponse<T> {
  status: number;
  headers: Headers;
  data: T;
}

/**
 * API client error types
 */
export class ApiClientError extends Error {
  constructor(
    message: string,
    public readonly statusCode?: number,
    public readonly response?: unknown
  ) {
    super(message);
    this.name = 'ApiClientError';
  }
}

export class NetworkError extends ApiClientError {
  constructor(message: string, cause?: Error) {
    super(message);
    this.name = 'NetworkError';
    this.cause = cause;
  }
}

export class ValidationError extends ApiClientError {
  constructor(message: string, public readonly validationErrors: unknown) {
    super(message);
    this.name = 'ValidationError';
  }
}

export class TimeoutError extends ApiClientError {
  constructor(message: string) {
    super(message);
    this.name = 'TimeoutError';
  }
}

/**
 * Sleep utility for retry delays
 */
const sleep = (ms: number): Promise<void> =>
  new Promise((resolve) => setTimeout(resolve, ms));

/**
 * Type-safe API client for KeyRx daemon
 */
export class ApiClient {
  private readonly baseUrl: string;
  private readonly timeout: number;
  private readonly maxRetries: number;
  private readonly retryDelayMs: number;

  constructor(config: ApiClientConfig) {
    this.baseUrl = config.baseUrl.replace(/\/$/, ''); // Remove trailing slash
    this.timeout = config.timeout ?? 5000; // 5 second default
    this.maxRetries = config.maxRetries ?? 3;
    this.retryDelayMs = config.retryDelayMs ?? 100;
  }

  /**
   * Make HTTP request with retry logic and timeout
   */
  private async request<T>(
    options: RequestOptions,
    schema: z.ZodSchema<T>,
    endpoint: string
  ): Promise<ApiResponse<T>> {
    const retries = options.retries ?? this.maxRetries;
    const timeout = options.timeout ?? this.timeout;
    const url = `${this.baseUrl}${options.path}`;

    for (let attempt = 0; attempt <= retries; attempt++) {
      try {
        const controller = new AbortController();
        const timeoutId = setTimeout(() => controller.abort(), timeout);

        const response = await fetch(url, {
          method: options.method,
          headers: {
            'Content-Type': 'application/json',
          },
          body: options.body ? JSON.stringify(options.body) : undefined,
          signal: controller.signal,
        });

        clearTimeout(timeoutId);

        // Parse response body
        let responseData: unknown;
        const contentType = response.headers.get('content-type');
        if (contentType?.includes('application/json')) {
          responseData = await response.json();
        } else {
          responseData = await response.text();
        }

        // Handle HTTP errors
        if (!response.ok) {
          throw new ApiClientError(
            `HTTP ${response.status}: ${JSON.stringify(responseData)}`,
            response.status,
            responseData
          );
        }

        // Validate response against schema
        const validatedData = validateApiResponse(schema, responseData, endpoint);

        return {
          status: response.status,
          headers: response.headers,
          data: validatedData,
        };
      } catch (error) {
        const isLastAttempt = attempt === retries;

        // Handle abort (timeout)
        if (error instanceof Error && error.name === 'AbortError') {
          if (isLastAttempt) {
            throw new TimeoutError(`Request timeout after ${timeout}ms: ${endpoint}`);
          }
        }
        // Handle network errors (retry)
        else if (error instanceof TypeError || error instanceof Error && error.message.includes('fetch')) {
          if (isLastAttempt) {
            throw new NetworkError(`Network error: ${error.message}`, error as Error);
          }
        }
        // Handle API client errors (don't retry)
        else if (error instanceof ApiClientError) {
          throw error;
        }
        // Unknown errors (don't retry)
        else {
          throw error;
        }

        // Exponential backoff: 100ms, 200ms, 400ms, ...
        if (!isLastAttempt) {
          const delay = this.retryDelayMs * Math.pow(2, attempt);
          await sleep(delay);
        }
      }
    }

    // Should never reach here due to throw in last attempt
    throw new NetworkError(`Failed after ${retries + 1} attempts`);
  }

  /**
   * GET /api/health - Health check endpoint
   */
  async getHealth(): Promise<ApiResponse<{ status: string; version: string }>> {
    const schema = z.object({
      status: z.string(),
      version: z.string(),
    });

    return this.request(
      { method: 'GET', path: '/api/health' },
      schema,
      'GET /api/health'
    );
  }

  /**
   * GET /api/version - Get daemon version info
   */
  async getVersion(): Promise<ApiResponse<{ version: string; build_time: string; platform: string }>> {
    const schema = z.object({
      version: z.string(),
      build_time: z.string(),
      platform: z.string(),
      git_hash: z.string().optional(),
    });

    return this.request(
      { method: 'GET', path: '/api/version' },
      schema,
      'GET /api/version'
    );
  }

  /**
   * GET /api/status - Get daemon status
   */
  async getStatus(): Promise<ApiResponse<z.infer<typeof StatusResponseSchema>>> {
    return this.request(
      { method: 'GET', path: '/api/status' },
      StatusResponseSchema,
      'GET /api/status'
    );
  }

  /**
   * GET /api/devices - List all devices
   */
  async getDevices(): Promise<ApiResponse<z.infer<typeof DeviceListResponseSchema>>> {
    return this.request(
      { method: 'GET', path: '/api/devices' },
      DeviceListResponseSchema,
      'GET /api/devices'
    );
  }

  /**
   * PATCH /api/devices/:id - Update device configuration
   */
  async patchDevice(
    id: string,
    updates: {
      enabled?: boolean;
      layout?: string;
      scope?: string;
    }
  ): Promise<ApiResponse<z.infer<typeof UpdateDeviceConfigResponseSchema>>> {
    return this.request(
      {
        method: 'PATCH',
        path: `/api/devices/${encodeURIComponent(id)}`,
        body: updates,
      },
      UpdateDeviceConfigResponseSchema,
      `PATCH /api/devices/${id}`
    );
  }

  /**
   * GET /api/profiles - List all profiles
   */
  async getProfiles(): Promise<ApiResponse<z.infer<typeof ProfileListResponseSchema>>> {
    return this.request(
      { method: 'GET', path: '/api/profiles' },
      ProfileListResponseSchema,
      'GET /api/profiles'
    );
  }

  /**
   * GET /api/profiles/active - Get active profile
   */
  async getActiveProfile(): Promise<ApiResponse<{ active_profile: string | null }>> {
    const schema = z.object({
      active_profile: z.string().nullable(),
    });

    return this.request(
      { method: 'GET', path: '/api/profiles/active' },
      schema,
      'GET /api/profiles/active'
    );
  }

  /**
   * GET /api/profiles/:name - Get profile configuration
   */
  async getProfileConfig(name: string): Promise<ApiResponse<z.infer<typeof ProfileConfigResponseSchema>>> {
    return this.request(
      { method: 'GET', path: `/api/profiles/${encodeURIComponent(name)}` },
      ProfileConfigResponseSchema,
      `GET /api/profiles/${name}`
    );
  }

  /**
   * POST /api/profiles - Create new profile
   */
  async createProfile(
    name: string,
    template?: string
  ): Promise<ApiResponse<{ message: string; profile: { name: string } }>> {
    const schema = z.object({
      message: z.string(),
      profile: z.object({
        name: z.string(),
      }),
    });

    return this.request(
      {
        method: 'POST',
        path: '/api/profiles',
        body: { name, template },
      },
      schema,
      'POST /api/profiles'
    );
  }

  /**
   * PUT /api/profiles/:name - Update profile configuration
   */
  async setProfileConfig(
    name: string,
    config: { source: string }
  ): Promise<ApiResponse<{ success: boolean }>> {
    const schema = z.object({
      success: z.boolean(),
    });

    return this.request(
      {
        method: 'PUT',
        path: `/api/profiles/${encodeURIComponent(name)}`,
        body: config,
      },
      schema,
      `PUT /api/profiles/${name}`
    );
  }

  /**
   * POST /api/profiles/:name/activate - Activate profile
   */
  async activateProfile(name: string): Promise<ApiResponse<z.infer<typeof ActivationRpcResultSchema>>> {
    return this.request(
      {
        method: 'POST',
        path: `/api/profiles/${encodeURIComponent(name)}/activate`,
      },
      ActivationRpcResultSchema,
      `POST /api/profiles/${name}/activate`
    );
  }

  /**
   * DELETE /api/profiles/:name - Delete profile
   */
  async deleteProfile(name: string): Promise<ApiResponse<{ success: boolean }>> {
    const schema = z.object({
      success: z.boolean(),
      message: z.string().optional(),
    });

    return this.request(
      {
        method: 'DELETE',
        path: `/api/profiles/${encodeURIComponent(name)}`,
      },
      schema,
      `DELETE /api/profiles/${name}`
    );
  }

  /**
   * GET /api/metrics/latency - Get latency metrics
   */
  async getLatencyMetrics(): Promise<ApiResponse<{
    min_us: number;
    avg_us: number;
    max_us: number;
    p50_us: number;
    p95_us: number;
    p99_us: number;
    count: number;
  }>> {
    const schema = z.object({
      min_us: z.number(),
      avg_us: z.number(),
      max_us: z.number(),
      p50_us: z.number(),
      p95_us: z.number(),
      p99_us: z.number(),
      count: z.number(),
    });

    return this.request(
      { method: 'GET', path: '/api/metrics/latency' },
      schema,
      'GET /api/metrics/latency'
    );
  }

  /**
   * GET /api/layouts - Get available keyboard layouts
   */
  async getLayouts(): Promise<ApiResponse<{ layouts: string[] }>> {
    const schema = z.object({
      layouts: z.array(z.string()),
    });

    return this.request(
      { method: 'GET', path: '/api/layouts' },
      schema,
      'GET /api/layouts'
    );
  }

  /**
   * Generic request method for custom endpoints
   * Useful for testing endpoints not yet in the client
   */
  async customRequest<T>(
    method: 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE',
    path: string,
    schema: z.ZodSchema<T>,
    body?: unknown
  ): Promise<ApiResponse<T>> {
    return this.request(
      { method, path, body },
      schema,
      `${method} ${path}`
    );
  }

  /**
   * Convenience method for POST requests
   * Returns raw response data without schema validation
   *
   * @param path - API endpoint path
   * @param body - Request body
   * @returns Promise with response data
   *
   * @example
   * const response = await client.post('/profiles/test/duplicate', { new_name: 'test-copy' });
   */
  async post(path: string, body?: unknown): Promise<any> {
    const response = await this.customRequest(
      'POST',
      path,
      z.any(),
      body
    );
    return response.data;
  }

  /**
   * Convenience method for PUT requests
   * Returns raw response data without schema validation
   *
   * @param path - API endpoint path
   * @param body - Request body
   * @returns Promise with response data
   *
   * @example
   * const response = await client.put('/profiles/test/rename', { new_name: 'test-renamed' });
   */
  async put(path: string, body?: unknown): Promise<any> {
    const response = await this.customRequest(
      'PUT',
      path,
      z.any(),
      body
    );
    return response.data;
  }

  /**
   * Convenience method for DELETE requests
   * Returns raw response data without schema validation
   *
   * @param path - API endpoint path
   * @returns Promise with response data
   *
   * @example
   * const response = await client.delete('/profiles/test');
   */
  async delete(path: string): Promise<any> {
    const response = await this.customRequest(
      'DELETE',
      path,
      z.any()
    );
    return response.data;
  }
}

/**
 * Create API client instance
 *
 * @example
 * const client = createApiClient({ baseUrl: 'http://localhost:9867' });
 * const status = await client.getStatus();
 */
export function createApiClient(config: ApiClientConfig): ApiClient {
  return new ApiClient(config);
}
