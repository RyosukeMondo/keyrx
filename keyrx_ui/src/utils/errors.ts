/**
 * Custom error hierarchy for KeyRx
 * SSOT for error handling - no duplicate error classes allowed
 */

export interface ErrorContext {
  [key: string]: unknown;
}

/**
 * Base error class for all KeyRx errors
 */
export class KeyRxError extends Error {
  constructor(
    public readonly code: string,
    message: string,
    public readonly context?: ErrorContext
  ) {
    super(message);
    this.name = 'KeyRxError';
    Object.setPrototypeOf(this, KeyRxError.prototype);
  }

  /**
   * Serialize error for logging/transmission
   */
  toJSON(): object {
    return {
      name: this.name,
      code: this.code,
      message: this.message,
      context: this.context,
      stack: this.stack,
    };
  }

  /**
   * Get user-friendly message (for toast notifications)
   */
  getUserMessage(): string {
    return this.message;
  }
}

/**
 * API-related errors (network, HTTP status codes)
 */
export class ApiError extends KeyRxError {
  constructor(
    message: string,
    public readonly statusCode?: number,
    context?: ErrorContext
  ) {
    super('API_ERROR', message, context);
    this.name = 'ApiError';
    Object.setPrototypeOf(this, ApiError.prototype);
  }

  static notFound(resource: string): ApiError {
    return new ApiError(`${resource} not found`, 404, { resource });
  }

  static unauthorized(message = 'Unauthorized'): ApiError {
    return new ApiError(message, 401);
  }

  static serverError(message = 'Internal server error'): ApiError {
    return new ApiError(message, 500);
  }

  static timeout(message = 'Request timeout'): ApiError {
    return new ApiError(message, 408);
  }
}

/**
 * Validation errors (input validation, config validation)
 */
export class ValidationError extends KeyRxError {
  constructor(
    message: string,
    public readonly field?: string,
    context?: ErrorContext
  ) {
    super('VALIDATION_ERROR', message, { ...context, field });
    this.name = 'ValidationError';
    Object.setPrototypeOf(this, ValidationError.prototype);
  }

  static invalidField(field: string, reason: string): ValidationError {
    return new ValidationError(`Invalid ${field}: ${reason}`, field);
  }

  static required(field: string): ValidationError {
    return new ValidationError(`${field} is required`, field);
  }

  static tooLong(field: string, maxLength: number): ValidationError {
    return new ValidationError(
      `${field} exceeds maximum length of ${maxLength}`,
      field,
      { maxLength }
    );
  }

  static invalidFormat(field: string, format: string): ValidationError {
    return new ValidationError(
      `${field} must be a valid ${format}`,
      field,
      { format }
    );
  }
}

/**
 * Network-related errors (connection, timeout, CORS)
 */
export class NetworkError extends KeyRxError {
  constructor(
    message: string,
    public readonly cause?: Error,
    context?: ErrorContext
  ) {
    super('NETWORK_ERROR', message, context);
    this.name = 'NetworkError';
    Object.setPrototypeOf(this, NetworkError.prototype);
  }

  static offline(): NetworkError {
    return new NetworkError('No network connection');
  }

  static timeout(): NetworkError {
    return new NetworkError('Network request timeout');
  }

  static connectionFailed(url: string): NetworkError {
    return new NetworkError('Connection failed', undefined, { url });
  }
}

/**
 * Profile-related errors
 */
export class ProfileError extends KeyRxError {
  constructor(message: string, context?: ErrorContext) {
    super('PROFILE_ERROR', message, context);
    this.name = 'ProfileError';
    Object.setPrototypeOf(this, ProfileError.prototype);
  }

  static notFound(name: string): ProfileError {
    return new ProfileError(`Profile not found: ${name}`, { name });
  }

  static compilationFailed(name: string, reason: string): ProfileError {
    return new ProfileError(
      `Failed to compile profile ${name}: ${reason}`,
      { name, reason }
    );
  }

  static invalidName(name: string): ProfileError {
    return new ProfileError(`Invalid profile name: ${name}`, { name });
  }

  static activationFailed(name: string, reason: string): ProfileError {
    return new ProfileError(
      `Failed to activate profile ${name}: ${reason}`,
      { name, reason }
    );
  }
}

/**
 * Configuration errors
 */
export class ConfigError extends KeyRxError {
  constructor(message: string, context?: ErrorContext) {
    super('CONFIG_ERROR', message, context);
    this.name = 'ConfigError';
    Object.setPrototypeOf(this, ConfigError.prototype);
  }

  static loadFailed(reason: string): ConfigError {
    return new ConfigError(`Failed to load configuration: ${reason}`, { reason });
  }

  static saveFailed(reason: string): ConfigError {
    return new ConfigError(`Failed to save configuration: ${reason}`, { reason });
  }

  static invalidFormat(format: string): ConfigError {
    return new ConfigError(`Invalid configuration format: ${format}`, { format });
  }
}

/**
 * Device-related errors
 */
export class DeviceError extends KeyRxError {
  constructor(message: string, context?: ErrorContext) {
    super('DEVICE_ERROR', message, context);
    this.name = 'DeviceError';
    Object.setPrototypeOf(this, DeviceError.prototype);
  }

  static notFound(deviceId: string): DeviceError {
    return new DeviceError(`Device not found: ${deviceId}`, { deviceId });
  }

  static notConnected(deviceId: string): DeviceError {
    return new DeviceError(`Device not connected: ${deviceId}`, { deviceId });
  }

  static accessDenied(deviceId: string): DeviceError {
    return new DeviceError(`Access denied to device: ${deviceId}`, { deviceId });
  }
}

/**
 * WebSocket errors
 */
export class WebSocketError extends KeyRxError {
  constructor(message: string, context?: ErrorContext) {
    super('WEBSOCKET_ERROR', message, context);
    this.name = 'WebSocketError';
    Object.setPrototypeOf(this, WebSocketError.prototype);
  }

  static connectionFailed(url: string): WebSocketError {
    return new WebSocketError('WebSocket connection failed', { url });
  }

  static disconnected(reason?: string): WebSocketError {
    return new WebSocketError('WebSocket disconnected', { reason });
  }

  static messageError(error: string): WebSocketError {
    return new WebSocketError('WebSocket message error', { error });
  }
}

/**
 * Parse unknown error into KeyRxError
 */
export function parseError(error: unknown): KeyRxError {
  if (error instanceof KeyRxError) {
    return error;
  }

  if (error instanceof Error) {
    // Check for specific error types
    if (error.message.includes('fetch') || error.message.includes('network')) {
      return new NetworkError(error.message, error);
    }
    return new KeyRxError('UNKNOWN_ERROR', error.message, { originalError: error });
  }

  if (typeof error === 'string') {
    return new KeyRxError('UNKNOWN_ERROR', error);
  }

  return new KeyRxError('UNKNOWN_ERROR', 'An unknown error occurred', {
    originalError: error,
  });
}

/**
 * Error code to user-friendly message mapping (for i18n)
 */
export const ERROR_MESSAGES: Record<string, string> = {
  API_ERROR: 'Failed to communicate with server',
  VALIDATION_ERROR: 'Invalid input provided',
  NETWORK_ERROR: 'Network connection error',
  PROFILE_ERROR: 'Profile operation failed',
  CONFIG_ERROR: 'Configuration error',
  DEVICE_ERROR: 'Device operation failed',
  WEBSOCKET_ERROR: 'Real-time connection error',
  UNKNOWN_ERROR: 'An unexpected error occurred',
};
