import { describe, it, expect } from 'vitest';
import { matchesPattern, matchesAnyIdentity } from './devicePattern';

// Vectors copied from keyrx_core/src/runtime/device_pattern.rs tests.
describe('devicePattern (port of keyrx_core::runtime::device_pattern)', () => {
  it('matches globs like the daemon', () => {
    expect(matchesPattern('anything', '*')).toBe(true);
    expect(matchesPattern('USB Keyboard', 'usb*')).toBe(true);
    expect(matchesPattern('USB Keyboard', '*KEYBOARD')).toBe(true);
    expect(matchesPattern('my numpad 2', '*NumPad*')).toBe(true);
    expect(matchesPattern('usb-num-kbd', 'usb*num*kbd')).toBe(true);
    expect(matchesPattern('Exact', 'exact')).toBe(true);
    expect(matchesPattern('USB Keyboard', '*mouse*')).toBe(false);
    expect(matchesPattern('abc', 'abcd')).toBe(false);
  });

  it('matches if any identity matches', () => {
    expect(
      matchesAnyIdentity(['path-/dev/input/event7', 'USB NumPad'], '*numpad*')
    ).toBe(true);
    expect(matchesAnyIdentity([], '*')).toBe(false);
    expect(matchesAnyIdentity([null, undefined, 'x'], 'y')).toBe(false);
  });
});
