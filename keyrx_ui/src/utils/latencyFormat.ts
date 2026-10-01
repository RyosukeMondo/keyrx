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

/**
 * Axis ticks on whole `unit`s (never "12.5µs"): about `count` evenly spaced
 * multiples of a 1/2/5 step covering [0, max].
 */
export function niceLatencyTicks(max: number, count = 5): number[] {
  if (!Number.isFinite(max) || max <= 0) return [0, 1];
  const raw = max / count;
  const pow = 10 ** Math.floor(Math.log10(raw));
  const step = [1, 2, 5, 10].map((m) => m * pow).find((s) => s >= raw) ?? raw;
  const ticks: number[] = [];
  for (let v = 0; ; v += step) {
    ticks.push(Number(v.toFixed(6)));
    if (v >= max) break;
  }
  return ticks;
}

/**
 * Time-axis ticks on whole seconds whose HH:MM:SS labels are all different:
 * at most `count` ticks, spaced by a whole number of seconds.
 */
export function niceTimeTicks(
  minMs: number,
  maxMs: number,
  count = 5
): number[] {
  if (!(maxMs > minMs)) return [Math.floor(minMs / 1000) * 1000];
  const spanSec = Math.max(1, Math.ceil((maxMs - minMs) / 1000));
  const step = Math.max(1, Math.ceil(spanSec / count));
  const first = Math.ceil(minMs / 1000) * 1000;
  const ticks: number[] = [];
  for (let t = first; t <= maxMs; t += step * 1000) ticks.push(t);
  return ticks.length > 0 ? ticks : [first];
}
