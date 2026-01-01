/**
 * Base API client utilities
 * Handles common error handling and request/response processing
 */

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
  error?: string;
  errorCode?: string;
}

/**
 * Base fetch wrapper with error handling
 */
async function apiFetch<T>(
  endpoint: string,
  options?: RequestInit
): Promise<T> {
  const baseUrl = import.meta.env.VITE_API_URL || '';
  const url = `${baseUrl}${endpoint}`;

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
      throw new ApiError(
        data.error || `Request failed: ${response.statusText}`,
        response.status,
        data.errorCode
      );
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

  delete: <T>(endpoint: string, body?: unknown) =>
    apiFetch<T>(endpoint, {
      method: 'DELETE',
      body: body ? JSON.stringify(body) : undefined,
    }),
};
