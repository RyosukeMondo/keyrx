/**
 * Which devices are offered as targets in the config page.
 *
 * keyrx's own output device and other tools' virtual keyboards are never
 * something to remap, so they are hidden from the device chips. The daemon
 * says so with `is_keyrx_output` / `is_virtual` (see api/devices.ts); older
 * daemons omit the fields and the name `keyrx*` is the stand-in.
 */
import type { DeviceEntry } from '@/types';

type Visibility = Pick<DeviceEntry, 'name'> &
  Partial<Pick<DeviceEntry, 'isVirtual' | 'isKeyrxOutput'>>;

export function isRemappableDevice(device: Visibility): boolean {
  if (device.isKeyrxOutput === true || device.isVirtual === true) return false;
  return !/^keyrx/i.test(device.name);
}

export function remappableDevices<T extends Visibility>(
  devices: readonly T[] | undefined
): T[] {
  return (devices ?? []).filter(isRemappableDevice);
}
