/**
 * Time formatting utilities for converting and displaying timestamps.
 *
 * These utilities handle conversion between microseconds and human-readable
 * formats, supporting both absolute and relative time displays.
 */

/**
 * Converts microseconds to human-readable format (ms/s).
 *
 * @param micros - Time in microseconds
 * @returns Formatted string (e.g., "1.23ms", "2.45s")
 *
 * @example
 * formatTimestampMs(1234) // "1.23ms"
 * formatTimestampMs(1234567) // "1.23s"
 */
export function formatTimestampMs(micros: number): string {
  if (micros < 1000) {
    return `${micros}μs`;
  } else if (micros < 1000000) {
    return `${(micros / 1000).toFixed(2)}ms`;
  } else {
    return `${(micros / 1000000).toFixed(2)}s`;
  }
}

/**
 * Formats timestamp as relative time ("2 hours ago").
 *
 * @param timestamp - Timestamp in milliseconds since UNIX epoch
 * @returns Formatted relative time string
 *
 * @example
 * formatTimestampRelative(Date.now() - 3600000) // "1 hour ago"
 * formatTimestampRelative(Date.now() - 120000) // "2 minutes ago"
 */
export function formatTimestampRelative(timestamp: number): string {
  const now = Date.now();
  const diffMs = now - timestamp;
  const diffSec = Math.floor(diffMs / 1000);

  if (diffSec < 1) {
    return "just now";
  } else if (diffSec < 60) {
    return `${diffSec}s ago`;
  } else if (diffSec < 3600) {
    const minutes = Math.floor(diffSec / 60);
    return `${minutes}m ago`;
  } else if (diffSec < 86400) {
    const hours = Math.floor(diffSec / 3600);
    return `${hours}h ago`;
  } else {
    const days = Math.floor(diffSec / 86400);
    return `${days}d ago`;
  }
}

/**
 * Formats duration in milliseconds.
 *
 * @param durationMs - Duration in milliseconds
 * @returns Formatted duration string
 *
 * @example
 * formatDuration(1234) // "1.23s"
 * formatDuration(123) // "123ms"
 */
export function formatDuration(durationMs: number): string {
  if (durationMs < 1000) {
    return `${durationMs}ms`;
  } else {
    return `${(durationMs / 1000).toFixed(2)}s`;
  }
}
