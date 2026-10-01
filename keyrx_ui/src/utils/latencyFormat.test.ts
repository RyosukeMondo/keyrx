import { describe, it, expect } from 'vitest';
import {
  formatLatencyUs,
  formatLatencyMs,
  niceLatencyTicks,
  niceTimeTicks,
  pickLatencyUnit,
  unitScale,
} from './latencyFormat';

describe('formatLatencyUs', () => {
  it('shows microseconds below 1 ms instead of 0.00ms', () => {
    expect(formatLatencyUs(4)).toBe('4µs');
    expect(formatLatencyUs(850)).toBe('850µs');
    expect(formatLatencyUs(0.4)).toBe('<1µs');
    expect(formatLatencyUs(0)).toBe('0µs');
  });

  it('switches to ms and s as values grow', () => {
    expect(formatLatencyUs(1230)).toBe('1.23ms');
    expect(formatLatencyUs(999.7)).toBe('1.00ms');
    expect(formatLatencyUs(2_500_000)).toBe('2.50s');
  });

  it('handles non-finite input', () => {
    expect(formatLatencyUs(NaN)).toBe('—');
  });
});

describe('ms helpers', () => {
  it('formatLatencyMs converts', () => {
    expect(formatLatencyMs(0.004)).toBe('4µs');
    expect(formatLatencyMs(1.5)).toBe('1.50ms');
  });

  it('chooses a readable axis unit', () => {
    expect(pickLatencyUnit(0.05)).toBe('µs');
    expect(pickLatencyUnit(2)).toBe('ms');
    expect(pickLatencyUnit(0)).toBe('ms');
    expect(unitScale('µs')).toBe(1000);
    expect(unitScale('ms')).toBe(1);
  });
});

describe('axis ticks', () => {
  it('latency ticks are whole units covering the max', () => {
    for (const max of [0.4, 3, 12.5, 37, 480, 1200]) {
      const ticks = niceLatencyTicks(max);
      expect(ticks[0]).toBe(0);
      expect(ticks[ticks.length - 1]).toBeGreaterThanOrEqual(max);
      if (max >= 3) ticks.forEach((t) => expect(Number.isInteger(t)).toBe(true));
    }
    expect(niceLatencyTicks(0)).toEqual([0, 1]);
  });

  it('time ticks are whole seconds with distinct HH:MM:SS labels', () => {
    const start = Date.UTC(2026, 0, 1, 12, 0, 0, 250);
    const ticks = niceTimeTicks(start, start + 60_000);
    expect(ticks.length).toBeLessThanOrEqual(6);
    ticks.forEach((t) => expect(t % 1000).toBe(0));
    const labels = ticks.map((t) => new Date(t).toISOString().slice(11, 19));
    expect(new Set(labels).size).toBe(labels.length);
    // sub-second span: still one tick, never repeated labels
    expect(niceTimeTicks(start, start + 400)).toHaveLength(1);
  });
});
