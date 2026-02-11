import { describe, it, expect } from 'vitest';
import {
  KeyRxError,
  ApiError,
  ValidationError,
  NetworkError,
  ProfileError,
  ConfigError,
  DeviceError,
  WebSocketError,
  parseError,
  ERROR_MESSAGES,
} from './errors';

describe('KeyRxError', () => {
  it('should create error with code and message', () => {
    const error = new KeyRxError('TEST_CODE', 'Test message');
    expect(error.code).toBe('TEST_CODE');
    expect(error.message).toBe('Test message');
    expect(error.name).toBe('KeyRxError');
  });

  it('should include context', () => {
    const context = { userId: '123', action: 'delete' };
    const error = new KeyRxError('TEST_CODE', 'Test message', context);
    expect(error.context).toEqual(context);
  });

  it('should serialize to JSON', () => {
    const error = new KeyRxError('TEST_CODE', 'Test message', { key: 'value' });
    const json = error.toJSON();
    expect(json).toMatchObject({
      name: 'KeyRxError',
      code: 'TEST_CODE',
      message: 'Test message',
      context: { key: 'value' },
    });
    expect(json).toHaveProperty('stack');
  });

  it('should return user message', () => {
    const error = new KeyRxError('TEST_CODE', 'User friendly message');
    expect(error.getUserMessage()).toBe('User friendly message');
  });
});

describe('ApiError', () => {
  it('should create API error with status code', () => {
    const error = new ApiError('Not found', 404);
    expect(error.name).toBe('ApiError');
    expect(error.code).toBe('API_ERROR');
    expect(error.statusCode).toBe(404);
  });

  it('should create not found error', () => {
    const error = ApiError.notFound('Profile');
    expect(error.message).toBe('Profile not found');
    expect(error.statusCode).toBe(404);
    expect(error.context).toEqual({ resource: 'Profile' });
  });

  it('should create unauthorized error', () => {
    const error = ApiError.unauthorized();
    expect(error.statusCode).toBe(401);
  });

  it('should create server error', () => {
    const error = ApiError.serverError();
    expect(error.statusCode).toBe(500);
  });

  it('should create timeout error', () => {
    const error = ApiError.timeout();
    expect(error.statusCode).toBe(408);
  });
});

describe('ValidationError', () => {
  it('should create validation error with field', () => {
    const error = new ValidationError('Invalid email', 'email');
    expect(error.name).toBe('ValidationError');
    expect(error.field).toBe('email');
    expect(error.context).toHaveProperty('field', 'email');
  });

  it('should create invalid field error', () => {
    const error = ValidationError.invalidField('username', 'contains spaces');
    expect(error.message).toBe('Invalid username: contains spaces');
    expect(error.field).toBe('username');
  });

  it('should create required field error', () => {
    const error = ValidationError.required('password');
    expect(error.message).toBe('password is required');
  });

  it('should create too long error', () => {
    const error = ValidationError.tooLong('name', 64);
    expect(error.message).toBe('name exceeds maximum length of 64');
    expect(error.context).toHaveProperty('maxLength', 64);
  });

  it('should create invalid format error', () => {
    const error = ValidationError.invalidFormat('email', 'email address');
    expect(error.message).toBe('email must be a valid email address');
  });
});

describe('NetworkError', () => {
  it('should create network error with cause', () => {
    const cause = new Error('Connection refused');
    const error = new NetworkError('Network failed', cause);
    expect(error.name).toBe('NetworkError');
    expect(error.cause).toBe(cause);
  });

  it('should create offline error', () => {
    const error = NetworkError.offline();
    expect(error.message).toBe('No network connection');
  });

  it('should create timeout error', () => {
    const error = NetworkError.timeout();
    expect(error.message).toBe('Network request timeout');
  });

  it('should create connection failed error', () => {
    const error = NetworkError.connectionFailed('http://example.com');
    expect(error.context).toHaveProperty('url', 'http://example.com');
  });
});

describe('ProfileError', () => {
  it('should create profile not found error', () => {
    const error = ProfileError.notFound('default');
    expect(error.message).toBe('Profile not found: default');
    expect(error.context).toEqual({ name: 'default' });
  });

  it('should create compilation failed error', () => {
    const error = ProfileError.compilationFailed('default', 'syntax error');
    expect(error.message).toContain('Failed to compile profile default');
    expect(error.context).toMatchObject({ name: 'default', reason: 'syntax error' });
  });

  it('should create invalid name error', () => {
    const error = ProfileError.invalidName('invalid@name');
    expect(error.message).toBe('Invalid profile name: invalid@name');
  });

  it('should create activation failed error', () => {
    const error = ProfileError.activationFailed('default', 'device not found');
    expect(error.message).toContain('Failed to activate profile default');
  });
});

describe('ConfigError', () => {
  it('should create load failed error', () => {
    const error = ConfigError.loadFailed('file not found');
    expect(error.message).toContain('Failed to load configuration');
  });

  it('should create save failed error', () => {
    const error = ConfigError.saveFailed('permission denied');
    expect(error.message).toContain('Failed to save configuration');
  });

  it('should create invalid format error', () => {
    const error = ConfigError.invalidFormat('JSON');
    expect(error.message).toContain('Invalid configuration format: JSON');
  });
});

describe('DeviceError', () => {
  it('should create device not found error', () => {
    const error = DeviceError.notFound('device-123');
    expect(error.message).toBe('Device not found: device-123');
  });

  it('should create not connected error', () => {
    const error = DeviceError.notConnected('device-123');
    expect(error.message).toBe('Device not connected: device-123');
  });

  it('should create access denied error', () => {
    const error = DeviceError.accessDenied('device-123');
    expect(error.message).toBe('Access denied to device: device-123');
  });
});

describe('WebSocketError', () => {
  it('should create connection failed error', () => {
    const error = WebSocketError.connectionFailed('ws://localhost:9867');
    expect(error.message).toBe('WebSocket connection failed');
    expect(error.context).toHaveProperty('url');
  });

  it('should create disconnected error', () => {
    const error = WebSocketError.disconnected('server closed');
    expect(error.message).toBe('WebSocket disconnected');
  });

  it('should create message error', () => {
    const error = WebSocketError.messageError('invalid JSON');
    expect(error.message).toBe('WebSocket message error');
  });
});

describe('parseError', () => {
  it('should return KeyRxError as-is', () => {
    const original = new KeyRxError('TEST', 'Test');
    const parsed = parseError(original);
    expect(parsed).toBe(original);
  });

  it('should convert Error to KeyRxError', () => {
    const original = new Error('Test error');
    const parsed = parseError(original);
    expect(parsed).toBeInstanceOf(KeyRxError);
    expect(parsed.message).toBe('Test error');
  });

  it('should convert network Error to NetworkError', () => {
    const original = new Error('fetch failed');
    const parsed = parseError(original);
    expect(parsed).toBeInstanceOf(NetworkError);
  });

  it('should convert string to KeyRxError', () => {
    const parsed = parseError('Error message');
    expect(parsed).toBeInstanceOf(KeyRxError);
    expect(parsed.message).toBe('Error message');
  });

  it('should handle unknown error types', () => {
    const parsed = parseError({ weird: 'object' });
    expect(parsed).toBeInstanceOf(KeyRxError);
    expect(parsed.code).toBe('UNKNOWN_ERROR');
  });
});

describe('ERROR_MESSAGES', () => {
  it('should have message for all error codes', () => {
    expect(ERROR_MESSAGES.API_ERROR).toBeDefined();
    expect(ERROR_MESSAGES.VALIDATION_ERROR).toBeDefined();
    expect(ERROR_MESSAGES.NETWORK_ERROR).toBeDefined();
    expect(ERROR_MESSAGES.PROFILE_ERROR).toBeDefined();
    expect(ERROR_MESSAGES.CONFIG_ERROR).toBeDefined();
    expect(ERROR_MESSAGES.DEVICE_ERROR).toBeDefined();
    expect(ERROR_MESSAGES.WEBSOCKET_ERROR).toBeDefined();
    expect(ERROR_MESSAGES.UNKNOWN_ERROR).toBeDefined();
  });
});
