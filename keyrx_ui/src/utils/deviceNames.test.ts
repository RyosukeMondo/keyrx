import { describe, expect, it } from 'vitest';
import { resolveDeviceName } from './deviceNames';

const devices = [{ id: 'path-/dev/input/event25', name: 'Areac Board' }];

describe('resolveDeviceName', () => {
  it('replaces a path-style name with the device-list name', () => {
    expect(
      resolveDeviceName(
        {
          deviceId: 'path-/dev/input/event25',
          deviceName: 'path-/dev/input/event25',
        },
        devices
      )
    ).toBe('Areac Board');
  });

  it('keeps a real name and falls back to the node name for gone devices', () => {
    expect(resolveDeviceName({ deviceName: 'Kinesis' }, devices)).toBe('Kinesis');
    expect(
      resolveDeviceName({ deviceId: 'path-/dev/input/event9' }, devices)
    ).toBe('event9');
    expect(resolveDeviceName({}, devices)).toBeUndefined();
  });
});
