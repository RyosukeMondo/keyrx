import { describe, it, expect } from 'vitest';
import {
  formatLatencyUs,
  formatLatencyMs,
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
