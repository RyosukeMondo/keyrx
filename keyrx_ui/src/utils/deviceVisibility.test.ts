import { describe, expect, it } from 'vitest';
import { isRemappableDevice, remappableDevices } from './deviceVisibility';

describe('isRemappableDevice', () => {
  it('hides what the API flags, and keyrx* by name when it says nothing', () => {
    expect(isRemappableDevice({ name: 'Anything', isKeyrxOutput: true })).toBe(false);
    expect(isRemappableDevice({ name: 'Other tool', isVirtual: true })).toBe(false);
    expect(isRemappableDevice({ name: 'keyrx' })).toBe(false);
    expect(isRemappableDevice({ name: 'KeyRx output' })).toBe(false);
  });

  it('keeps real keyboards, including when the fields are absent', () => {
    expect(isRemappableDevice({ name: 'HHKB' })).toBe(true);
    expect(
      isRemappableDevice({ name: 'HHKB', isVirtual: false, isKeyrxOutput: false })
    ).toBe(true);
  });

  it('remappableDevices tolerates undefined', () => {
    expect(remappableDevices(undefined)).toEqual([]);
    expect(
      remappableDevices([{ name: 'keyrx' }, { name: 'A' }]).map((d) => d.name)
    ).toEqual(['A']);
  });
});
