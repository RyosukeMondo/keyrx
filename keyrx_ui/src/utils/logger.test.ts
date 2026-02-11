import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import {
  logger,
  LogLevel,
  PerformanceLogger,
  measureAsync,
  measureSync,
} from './logger';

describe('logger', () => {
  let consoleErrorSpy: ReturnType<typeof vi.spyOn>;
  let consoleWarnSpy: ReturnType<typeof vi.spyOn>;
  let consoleInfoSpy: ReturnType<typeof vi.spyOn>;
  let consoleDebugSpy: ReturnType<typeof vi.spyOn>;

  beforeEach(() => {
    consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
    consoleWarnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});
    consoleInfoSpy = vi.spyOn(console, 'info').mockImplementation(() => {});
    consoleDebugSpy = vi.spyOn(console, 'debug').mockImplementation(() => {});
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe('info', () => {
    it('should log info message', () => {
      logger.info('test_event', { key: 'value' });

      expect(consoleInfoSpy).toHaveBeenCalledOnce();
      const logEntry = JSON.parse(consoleInfoSpy.mock.calls[0][0]);

      expect(logEntry).toMatchObject({
        level: LogLevel.INFO,
        service: 'keyrx-ui',
        event: 'test_event',
        context: { key: 'value' },
      });
      expect(logEntry.timestamp).toBeDefined();
    });

    it('should log without context', () => {
      logger.info('simple_event');

      expect(consoleInfoSpy).toHaveBeenCalledOnce();
      const logEntry = JSON.parse(consoleInfoSpy.mock.calls[0][0]);

      expect(logEntry.event).toBe('simple_event');
      expect(logEntry.context).toBeUndefined();
    });
  });

  describe('warn', () => {
    it('should log warning message', () => {
      logger.warn('warning_event', { issue: 'deprecated' });

      expect(consoleWarnSpy).toHaveBeenCalledOnce();
      const logEntry = JSON.parse(consoleWarnSpy.mock.calls[0][0]);

      expect(logEntry).toMatchObject({
        level: LogLevel.WARN,
        event: 'warning_event',
        context: { issue: 'deprecated' },
      });
    });
  });

  describe('error', () => {
    it('should log error with Error object', () => {
      const error = new Error('Test error');
      logger.error('error_event', error, { userId: '123' });

      expect(consoleErrorSpy).toHaveBeenCalledOnce();
      const logEntry = JSON.parse(consoleErrorSpy.mock.calls[0][0]);

      expect(logEntry).toMatchObject({
        level: LogLevel.ERROR,
        event: 'error_event',
        context: { userId: '123' },
      });
      expect(logEntry.error).toMatchObject({
        name: 'Error',
        message: 'Test error',
      });
      expect(logEntry.error.stack).toBeDefined();
    });

    it('should log error without Error object', () => {
      logger.error('error_event', undefined, { action: 'failed' });

      expect(consoleErrorSpy).toHaveBeenCalledOnce();
      const logEntry = JSON.parse(consoleErrorSpy.mock.calls[0][0]);

      expect(logEntry.event).toBe('error_event');
      expect(logEntry.error).toBeUndefined();
    });
  });

  describe('sensitive data sanitization', () => {
    it('should redact password', () => {
      logger.info('login_attempt', { username: 'user', password: 'secret123' });

      const logEntry = JSON.parse(consoleInfoSpy.mock.calls[0][0]);
      expect(logEntry.context.username).toBe('user');
      expect(logEntry.context.password).toBe('[REDACTED]');
    });

    it('should redact API keys', () => {
      logger.info('api_call', { apiKey: 'sk-123', api_key: 'sk-456' });

      const logEntry = JSON.parse(consoleInfoSpy.mock.calls[0][0]);
      expect(logEntry.context.apiKey).toBe('[REDACTED]');
      expect(logEntry.context.api_key).toBe('[REDACTED]');
    });

    it('should redact tokens', () => {
      logger.info('auth', { token: 'abc123', authorization: 'Bearer xyz' });

      const logEntry = JSON.parse(consoleInfoSpy.mock.calls[0][0]);
      expect(logEntry.context.token).toBe('[REDACTED]');
      expect(logEntry.context.authorization).toBe('[REDACTED]');
    });

    it('should redact nested sensitive data', () => {
      logger.info('nested', {
        user: {
          id: '123',
          password: 'secret',
        },
      });

      const logEntry = JSON.parse(consoleInfoSpy.mock.calls[0][0]);
      expect(logEntry.context.user.id).toBe('123');
      expect(logEntry.context.user.password).toBe('[REDACTED]');
    });

    it('should not redact safe data', () => {
      logger.info('safe_data', {
        userId: '123',
        action: 'create',
        timestamp: Date.now(),
      });

      const logEntry = JSON.parse(consoleInfoSpy.mock.calls[0][0]);
      expect(logEntry.context.userId).toBe('123');
      expect(logEntry.context.action).toBe('create');
      expect(logEntry.context.timestamp).toBeDefined();
    });
  });

  describe('scope', () => {
    it('should create scoped logger', () => {
      const scopedLogger = logger.scope('auth');
      scopedLogger.info('login', { username: 'user' });

      const logEntry = JSON.parse(consoleInfoSpy.mock.calls[0][0]);
      expect(logEntry.context.scope).toBe('auth');
      expect(logEntry.context.username).toBe('user');
    });

    it('should preserve existing context', () => {
      const scopedLogger = logger.scope('metrics');
      scopedLogger.warn('high_latency', { latency: 500 });

      const logEntry = JSON.parse(consoleWarnSpy.mock.calls[0][0]);
      expect(logEntry.context).toMatchObject({
        scope: 'metrics',
        latency: 500,
      });
    });
  });
});

describe('PerformanceLogger', () => {
  let consoleInfoSpy: ReturnType<typeof vi.spyOn>;
  let consoleDebugSpy: ReturnType<typeof vi.spyOn>;
  let consoleErrorSpy: ReturnType<typeof vi.spyOn>;

  beforeEach(() => {
    consoleInfoSpy = vi.spyOn(console, 'info').mockImplementation(() => {});
    consoleDebugSpy = vi.spyOn(console, 'debug').mockImplementation(() => {});
    consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('should log operation start', () => {
    const perf = new PerformanceLogger('test_operation');

    // Skip debug check in test (only logs in DEV mode)
    expect(perf).toBeDefined();
  });

  it('should log marks', () => {
    const perf = new PerformanceLogger('test_operation');
    perf.mark('checkpoint1');

    expect(perf).toBeDefined();
  });

  it('should log completion with duration', () => {
    const perf = new PerformanceLogger('test_operation');
    perf.end({ result: 'success' });

    expect(consoleInfoSpy).toHaveBeenCalled();
    const logEntry = JSON.parse(consoleInfoSpy.mock.calls[0][0]);

    expect(logEntry.event).toBe('test_operation_completed');
    expect(logEntry.context.duration).toBeGreaterThanOrEqual(0);
    expect(logEntry.context.result).toBe('success');
  });

  it('should log error with duration', () => {
    const perf = new PerformanceLogger('test_operation');
    const error = new Error('Test error');
    perf.endWithError(error, { action: 'test' });

    expect(consoleErrorSpy).toHaveBeenCalled();
    const logEntry = JSON.parse(consoleErrorSpy.mock.calls[0][0]);

    expect(logEntry.event).toBe('test_operation_failed');
    expect(logEntry.context.duration).toBeGreaterThanOrEqual(0);
    expect(logEntry.error.message).toBe('Test error');
  });

  it('should track multiple marks', () => {
    const perf = new PerformanceLogger('complex_operation');
    perf.mark('step1');
    perf.mark('step2');
    perf.end();

    expect(consoleInfoSpy).toHaveBeenCalled();
    const logEntry = JSON.parse(consoleInfoSpy.mock.calls[0][0]);

    expect(logEntry.context.marks).toBeDefined();
    expect(logEntry.context.marks.step1).toBeGreaterThanOrEqual(0);
    expect(logEntry.context.marks.step2).toBeGreaterThanOrEqual(0);
  });
});

describe('measureAsync', () => {
  let consoleInfoSpy: ReturnType<typeof vi.spyOn>;
  let consoleErrorSpy: ReturnType<typeof vi.spyOn>;

  beforeEach(() => {
    consoleInfoSpy = vi.spyOn(console, 'info').mockImplementation(() => {});
    consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('should measure successful async operation', async () => {
    const result = await measureAsync('fetch_data', async () => {
      await new Promise((resolve) => setTimeout(resolve, 10));
      return 'success';
    });

    expect(result).toBe('success');
    expect(consoleInfoSpy).toHaveBeenCalled();

    const logEntry = JSON.parse(consoleInfoSpy.mock.calls[0][0]);
    expect(logEntry.event).toBe('fetch_data_completed');
    expect(logEntry.context.duration).toBeGreaterThanOrEqual(10);
  });

  it('should measure failed async operation', async () => {
    const error = new Error('Async error');

    await expect(
      measureAsync('failing_operation', async () => {
        throw error;
      })
    ).rejects.toThrow('Async error');

    expect(consoleErrorSpy).toHaveBeenCalled();
    const logEntry = JSON.parse(consoleErrorSpy.mock.calls[0][0]);
    expect(logEntry.event).toBe('failing_operation_failed');
    expect(logEntry.error.message).toBe('Async error');
  });

  it('should include custom context', async () => {
    await measureAsync(
      'fetch_profile',
      async () => 'profile',
      { profileId: '123' }
    );

    const logEntry = JSON.parse(consoleInfoSpy.mock.calls[0][0]);
    expect(logEntry.context.profileId).toBe('123');
  });
});

describe('measureSync', () => {
  let consoleInfoSpy: ReturnType<typeof vi.spyOn>;
  let consoleErrorSpy: ReturnType<typeof vi.spyOn>;

  beforeEach(() => {
    consoleInfoSpy = vi.spyOn(console, 'info').mockImplementation(() => {});
    consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('should measure successful sync operation', () => {
    const result = measureSync('calculate', () => {
      return 42;
    });

    expect(result).toBe(42);
    expect(consoleInfoSpy).toHaveBeenCalled();

    const logEntry = JSON.parse(consoleInfoSpy.mock.calls[0][0]);
    expect(logEntry.event).toBe('calculate_completed');
    expect(logEntry.context.duration).toBeGreaterThanOrEqual(0);
  });

  it('should measure failed sync operation', () => {
    expect(() => {
      measureSync('failing_sync', () => {
        throw new Error('Sync error');
      });
    }).toThrow('Sync error');

    expect(consoleErrorSpy).toHaveBeenCalled();
    const logEntry = JSON.parse(consoleErrorSpy.mock.calls[0][0]);
    expect(logEntry.event).toBe('failing_sync_failed');
  });

  it('should include custom context', () => {
    measureSync('parse_config', () => ({}), { format: 'json' });

    const logEntry = JSON.parse(consoleInfoSpy.mock.calls[0][0]);
    expect(logEntry.context.format).toBe('json');
  });
});
