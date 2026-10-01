/**
 * Human device names for events.
 *
 * The daemon currently reports `deviceName` equal to the device id for
 * path-identified devices (`path-/dev/input/event25`), which is useless in the
 * event log. Resolve the real name from the device list by id; when the device
 * is gone (unplugged since the event) fall back to the node name (`event25`).
 */

interface NamedDevice {
  id: string;
  name: string;
}

/** True for strings that are ids/paths rather than names. */
export function looksLikeDeviceId(value: string | undefined): boolean {
  return !!value && (value.startsWith('path-') || value.startsWith('/dev/'));
}

export function resolveDeviceName(
  event: { deviceId?: string; deviceName?: string },
  devices: readonly NamedDevice[]
): string | undefined {
  const { deviceId, deviceName } = event;
  if (deviceName && !looksLikeDeviceId(deviceName)) return deviceName;
  const id = deviceId ?? deviceName;
  if (!id) return undefined;
  const known = devices.find((d) => d.id === id);
  if (known?.name) return known.name;
  return id.replace(/^path-/, '').split('/').pop() || undefined;
}
