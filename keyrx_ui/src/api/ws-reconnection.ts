/**
 * Reconnection Scheduler with exponential backoff
 *
 * Handles scheduling reconnection attempts with configurable
 * backoff intervals and max retry limits.
 */
import { WS_RECONNECT_CONFIG } from '../config/constants';

export class ReconnectionScheduler {
  private attempts = 0;
  private timeoutId: number | null = null;
  private readonly intervals: readonly number[];
  private readonly maxInterval: number;
  private readonly maxRetries: number;
  private onSchedule: (delay: number, attempt: () => void) => void;

  constructor(
    onSchedule: (delay: number, attempt: () => void) => void,
    intervals: readonly number[] = [100, 200, 400, 800, 1600],
    maxInterval = 5000,
    maxRetries: number = WS_RECONNECT_CONFIG.maxRetries
  ) {
    this.onSchedule = onSchedule;
    this.intervals = intervals;
    this.maxInterval = maxInterval;
    this.maxRetries = maxRetries;
  }

  /**
   * Schedule next reconnection attempt with exponential backoff
   */
  schedule(attempt: () => void): boolean {
    if (this.attempts >= this.maxRetries) {
      return false;
    }

    const delay =
      this.attempts < this.intervals.length
        ? this.intervals[this.attempts]
        : this.maxInterval;

    this.timeoutId = window.setTimeout(() => {
      this.attempts++;
      attempt();
    }, delay);

    return true;
  }

  /**
   * Clear any pending reconnection timeout
   */
  clear(): void {
    if (this.timeoutId !== null) {
      window.clearTimeout(this.timeoutId);
      this.timeoutId = null;
    }
  }

  /**
   * Reset scheduler state (call on successful connect)
   */
  reset(): void {
    this.clear();
    this.attempts = 0;
  }

  /**
   * Check if max retries reached
   */
  isExhausted(): boolean {
    return this.attempts >= this.maxRetries;
  }

  get attemptCount(): number {
    return this.attempts;
  }
}
