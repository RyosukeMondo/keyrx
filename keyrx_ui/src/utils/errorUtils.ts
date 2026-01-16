/**
 * Error handling utilities
 *
 * Provides consistent error message extraction across the application.
 */

/**
 * Extract a human-readable error message from any error type
 *
 * Handles:
 * - Error instances
 * - Error-like objects with message property
 * - Objects with nested error structures
 * - Primitive values
 *
 * @param error - Any error value
 * @param fallback - Fallback message if extraction fails
 * @returns Human-readable error message
 *
 * @example
 * ```ts
 * try {
 *   await apiCall();
 * } catch (err) {
 *   const message = getErrorMessage(err, 'Operation failed');
 *   setError(message);
 * }
 * ```
 */
export function getErrorMessage(
  error: unknown,
  fallback = 'An error occurred'
): string {
  // Handle null/undefined, but allow false/0/"" as valid values
  if (error === null || error === undefined) {
    return fallback;
  }

  // Standard Error instance
  if (error instanceof Error) {
    return error.message;
  }

  // Error-like object with message
  if (typeof error === 'object' && error !== null && 'message' in error) {
    const message = (error as { message: unknown }).message;

    // If message is a string, return it
    if (typeof message === 'string') {
      return message;
    }

    // If message is an object, try to stringify it
    if (typeof message === 'object' && message !== null) {
      return JSON.stringify(message);
    }
  }

  // String error
  if (typeof error === 'string') {
    return error;
  }

  // Try to stringify objects
  if (typeof error === 'object') {
    try {
      return JSON.stringify(error);
    } catch {
      return fallback;
    }
  }

  // Fallback for primitives
  return String(error);
}

/**
 * Format error for user display
 *
 * Ensures error messages are user-friendly and safe to display
 *
 * @param error - Any error value
 * @param context - Optional context prefix (e.g., "Failed to save profile")
 * @returns Formatted error message
 */
export function formatErrorForDisplay(
  error: unknown,
  context?: string
): string {
  const message = getErrorMessage(error);

  if (context) {
    // Avoid duplicate context if message already starts with it
    if (message.toLowerCase().startsWith(context.toLowerCase())) {
      return message;
    }
    return `${context}: ${message}`;
  }

  return message;
}
