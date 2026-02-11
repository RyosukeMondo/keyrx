/**
 * Structured logging utility for KeyRx
 * JSON format: {timestamp, level, service, event, context}
 *
 * IMPORTANT: Never log secrets, PII, or sensitive data
 */

export enum LogLevel {
  DEBUG = 'DEBUG',
  INFO = 'INFO',
  WARN = 'WARN',
  ERROR = 'ERROR',
}

export interface LogContext {
  [key: string]: unknown;
}

export interface LogEntry {
  timestamp: string;
  level: LogLevel;
  service: string;
  event: string;
  context?: LogContext;
  error?: {
    name: string;
    message: string;
    code?: string;
    stack?: string;
  };
}

/**
 * List of sensitive keys that should never be logged
 */
const SENSITIVE_KEYS = new Set([
  'password',
  'secret',
  'token',
  'apikey',
  'apiKey',
  'api_key',
  'authorization',
  'auth',
  'privatekey',
  'privateKey',
  'private_key',
  'sessionid',
  'sessionId',
  'session_id',
  'cookie',
  'credentials',
]);

/**
 * Sanitize context to remove sensitive data
 */
function sanitizeContext(context?: LogContext): LogContext | undefined {
  if (!context) return undefined;

  const sanitized: LogContext = {};
  for (const [key, value] of Object.entries(context)) {
    // Check both original key and lowercase version
    const keyLower = key.toLowerCase();
    const isSensitive = SENSITIVE_KEYS.has(key) || SENSITIVE_KEYS.has(keyLower);

    if (isSensitive) {
      sanitized[key] = '[REDACTED]';
    } else if (Array.isArray(value)) {
      // Handle arrays
      sanitized[key] = value.map(item =>
        typeof item === 'object' && item !== null
          ? sanitizeContext(item as LogContext)
          : item
      );
    } else if (typeof value === 'object' && value !== null) {
      sanitized[key] = sanitizeContext(value as LogContext);
    } else {
      sanitized[key] = value;
    }
  }
  return sanitized;
}

/**
 * Format log entry as JSON string
 */
function formatLogEntry(entry: LogEntry): string {
  return JSON.stringify(entry);
}

/**
 * Core logging function
 */
function log(level: LogLevel, event: string, context?: LogContext, error?: Error): void {
  const entry: LogEntry = {
    timestamp: new Date().toISOString(),
    level,
    service: 'keyrx-ui',
    event,
    context: sanitizeContext(context),
  };

  if (error) {
    entry.error = {
      name: error.name,
      message: error.message,
      code: 'code' in error ? (error as { code: string }).code : undefined,
      stack: error.stack,
    };
  }

  const formatted = formatLogEntry(entry);

  // Use appropriate console method based on level
  switch (level) {
    case LogLevel.ERROR:
      console.error(formatted);
      break;
    case LogLevel.WARN:
      console.warn(formatted);
      break;
    case LogLevel.INFO:
      console.info(formatted);
      break;
    case LogLevel.DEBUG:
      console.debug(formatted);
      break;
  }
}

/**
 * Structured logger with level-specific methods
 */
export const logger = {
  /**
   * Log debug information (development only)
   */
  debug(event: string, context?: LogContext): void {
    if (import.meta.env.DEV) {
      log(LogLevel.DEBUG, event, context);
    }
  },

  /**
   * Log informational messages
   */
  info(event: string, context?: LogContext): void {
    log(LogLevel.INFO, event, context);
  },

  /**
   * Log warnings
   */
  warn(event: string, context?: LogContext): void {
    log(LogLevel.WARN, event, context);
  },

  /**
   * Log errors
   */
  error(event: string, error?: Error, context?: LogContext): void {
    log(LogLevel.ERROR, event, context, error);
  },

  /**
   * Create scoped logger for specific module
   */
  scope(scope: string) {
    return {
      debug(event: string, context?: LogContext): void {
        logger.debug(event, { ...context, scope });
      },
      info(event: string, context?: LogContext): void {
        logger.info(event, { ...context, scope });
      },
      warn(event: string, context?: LogContext): void {
        logger.warn(event, { ...context, scope });
      },
      error(event: string, error?: Error, context?: LogContext): void {
        logger.error(event, error, { ...context, scope });
      },
    };
  },
};

/**
 * Performance measurement utility
 */
export class PerformanceLogger {
  private startTime: number;
  private marks: Map<string, number> = new Map();

  constructor(private operation: string) {
    this.startTime = performance.now();
    logger.debug(`${operation}_started`, { startTime: this.startTime });
  }

  mark(label: string): void {
    const time = performance.now();
    this.marks.set(label, time);
    logger.debug(`${this.operation}_mark_${label}`, {
      elapsed: time - this.startTime,
    });
  }

  end(context?: LogContext): void {
    const endTime = performance.now();
    const duration = endTime - this.startTime;

    const marks: Record<string, number> = {};
    for (const [label, time] of this.marks.entries()) {
      marks[label] = time - this.startTime;
    }

    logger.info(`${this.operation}_completed`, {
      ...context,
      duration,
      marks: Object.keys(marks).length > 0 ? marks : undefined,
    });
  }

  endWithError(error: Error, context?: LogContext): void {
    const endTime = performance.now();
    const duration = endTime - this.startTime;

    logger.error(`${this.operation}_failed`, error, {
      ...context,
      duration,
    });
  }
}

/**
 * Helper to measure async operations
 */
export async function measureAsync<T>(
  operation: string,
  fn: () => Promise<T>,
  context?: LogContext
): Promise<T> {
  const perf = new PerformanceLogger(operation);
  try {
    const result = await fn();
    perf.end(context);
    return result;
  } catch (error) {
    perf.endWithError(error as Error, context);
    throw error;
  }
}

/**
 * Helper to measure sync operations
 */
export function measureSync<T>(
  operation: string,
  fn: () => T,
  context?: LogContext
): T {
  const perf = new PerformanceLogger(operation);
  try {
    const result = fn();
    perf.end(context);
    return result;
  } catch (error) {
    perf.endWithError(error as Error, context);
    throw error;
  }
}
