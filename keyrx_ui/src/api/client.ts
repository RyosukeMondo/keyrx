/**
 * Base API client utilities
 * Handles common error handling and request/response processing
 */

import { buildApiUrl } from '../config/constants';

export class ApiError extends Error {
  constructor(
    message: string,
    public statusCode: number,
    public errorCode?: string
  ) {
    super(message);
    this.name = 'ApiError';
  }
}

interface ApiResponse<T> {
  data?: T;
  error?: string | { code: string; message: string };
  errorCode?: string;
}

/**
 * Base fetch wrapper with error handling
 */
async function apiFetch<T>(
  endpoint: string,
  options?: RequestInit
): Promise<T> {
  const url = buildApiUrl(endpoint);

  try {
    const response = await fetch(url, {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        ...options?.headers,
      },
    });

    // Handle non-JSON responses (e.g., 204 No Content)
    if (response.status === 204) {
      return {} as T;
    }

    const data: ApiResponse<T> = await response.json();

    if (!response.ok) {
      // Extract error message from object or string
      let errorMessage: string;
      let errorCode: string | undefined;

      if (typeof data.error === 'object' && data.error !== null) {
        // Backend error object format: { code: "...", message: "..." }
        const message = data.error.message;

        // Ensure message is a string (handle case where message itself is an object)
        errorMessage =
          typeof message === 'string' ? message : JSON.stringify(message);

        errorCode = data.error.code;
      } else if (typeof data.error === 'string') {
        // Simple string error
        errorMessage = data.error;
        errorCode = data.errorCode;
      } else {
        // Fallback
        errorMessage = `Request failed: ${response.statusText}`;
        errorCode = data.errorCode;
      }

      throw new ApiError(errorMessage, response.status, errorCode);
    }

    // Return the data directly if it's wrapped, otherwise return the whole response
    return (data.data !== undefined ? data.data : data) as T;
  } catch (error) {
    if (error instanceof ApiError) {
      throw error;
    }

    // Network error or JSON parse error
    throw new ApiError(
      error instanceof Error ? error.message : 'Network error',
      0
    );
  }
}

export const apiClient = {
  get: <T>(endpoint: string) => apiFetch<T>(endpoint, { method: 'GET' }),

  post: <T>(endpoint: string, body?: unknown) =>
    apiFetch<T>(endpoint, {
      method: 'POST',
      body: body ? JSON.stringify(body) : undefined,
    }),

  put: <T>(endpoint: string, body?: unknown) =>
    apiFetch<T>(endpoint, {
      method: 'PUT',
      body: body ? JSON.stringify(body) : undefined,
    }),

  patch: <T>(endpoint: string, body?: unknown) =>
    apiFetch<T>(endpoint, {
      method: 'PATCH',
      body: body ? JSON.stringify(body) : undefined,
    }),

  delete: <T>(endpoint: string, body?: unknown) =>
    apiFetch<T>(endpoint, {
      method: 'DELETE',
      body: body ? JSON.stringify(body) : undefined,
    }),
};
