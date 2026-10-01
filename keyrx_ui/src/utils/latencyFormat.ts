/**
 * Latency formatting with auto-scaled units.
 *
 * The daemon measures in microseconds and typical remap latency is tens of
 * microseconds, so a fixed "0.00ms" hides everything. One formatter for every
 * Monitor surface (stat cards, chart, event log, CSV export).
 */

/** Format microseconds: "850µs", "1.23ms", "2.50s". */
export function formatLatencyUs(us: number): string {
  if (!Number.isFinite(us)) return '—';
  if (us <= 0) return '0µs';
  if (us < 1) return '<1µs';
  const rounded = Math.round(us);
  if (rounded < 1000) return `${rounded}µs`;
  const ms = us / 1000;
  if (ms < 1000) return `${ms.toFixed(2)}ms`;
  return `${(ms / 1000).toFixed(2)}s`;
}

/** Same as {@link formatLatencyUs} for values already in milliseconds. */
export const formatLatencyMs = (ms: number): string =>
  formatLatencyUs(ms * 1000);

export type LatencyUnit = 'µs' | 'ms';

/** Unit that keeps the largest value readable (µs below 1 ms, otherwise ms). */
export function pickLatencyUnit(maxMs: number): LatencyUnit {
  return maxMs > 0 && maxMs < 1 ? 'µs' : 'ms';
}

/** Multiplier from milliseconds to the chosen axis unit. */
export const unitScale = (unit: LatencyUnit): number =>
  unit === 'µs' ? 1000 : 1;
