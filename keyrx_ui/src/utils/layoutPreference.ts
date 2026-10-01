/**
 * Which keyboard layout the config page should draw.
 *
 * Priority (first hit wins):
 *   1. the layout stored on a selected device (daemon `devices.json`)
 *   2. the global default layout, if one was ever set
 *   3. detection: JIS-only keys in the profile source, or a device whose name
 *      says it is a JIS/Japanese board
 *   4. ANSI_104
 *
 * The user's choice is persisted to (1) or (2) by the page, so it survives
 * reloads and profile switches instead of being re-detected every time.
 */
import type { LayoutType } from '@/components/KeyboardVisualizer';
import type { DeviceEntry } from '@/types';
import type { Device } from '@/components/DeviceSelector';
import { matchesAnyIdentity } from '@/utils/devicePattern';

export const LAYOUT_TYPES: readonly LayoutType[] = [
  'ANSI_104',
  'ANSI_87',
  'ISO_105',
  'ISO_88',
  'JIS_109',
  'COMPACT_60',
  'COMPACT_65',
  'COMPACT_75',
  'COMPACT_96',
  'HHKB',
  'NUMPAD',
];

export const isLayoutType = (value: unknown): value is LayoutType =>
  typeof value === 'string' &&
  (LAYOUT_TYPES as readonly string[]).includes(value);

const JIS_SOURCE_KEYS = [
  'VK_Zenkaku',
  'VK_全角',
  'VK_無変換',
  'VK_変換',
  'VK_ひらがな',
  'VK_カタカナ',
  'VK_Ro',
  'VK_Yen',
  'VK_Henkan',
  'VK_Muhenkan',
];

// Names that say JIS: "JIS", "Japanese", "日本語", "109" (the JIS key count), "JP".
const JIS_DEVICE_NAME = /\bjis\b|japan|\b109\b|\bjp\b|日本語/i;

/** Devices a selected scope stands for (a pattern scope may cover several). */
export function devicesForScopes(
  scopes: readonly Device[],
  devices: readonly DeviceEntry[]
): DeviceEntry[] {
  const found = new Map<string, DeviceEntry>();
  for (const scope of scopes) {
    const direct = devices.find((d) => d.id === scope.id);
    if (direct) found.set(direct.id, direct);
    else if (scope.pattern !== undefined) {
      devices
        .filter((d) =>
          matchesAnyIdentity([d.id, d.name, d.path, d.serial], scope.pattern!)
        )
        .forEach((d) => found.set(d.id, d));
    }
  }
  return [...found.values()];
}

/** JIS evidence from the profile source or the relevant devices' names. */
export function detectLayout(
  source: string | undefined,
  relevantDevices: readonly DeviceEntry[]
): LayoutType {
  if (source && JIS_SOURCE_KEYS.some((k) => source.includes(k))) {
    return 'JIS_109';
  }
  // `hasJisKeys` is the daemon's own evidence (the device reports JIS-only
  // keys such as Yen/Ro/Henkan); the name is the fallback for older daemons.
  if (
    relevantDevices.some(
      (d) => d.hasJisKeys === true || JIS_DEVICE_NAME.test(d.name)
    )
  ) {
    return 'JIS_109';
  }
  return source ? 'ANSI_104' : 'JIS_109';
}

/** Layout the user has already chosen (device first, then global), if any. */
export function resolveStoredLayout(
  selectedDevices: readonly DeviceEntry[],
  globalLayout: string | null | undefined
): LayoutType | null {
  const fromDevice = selectedDevices.find((d) => isLayoutType(d.layout));
  if (fromDevice && isLayoutType(fromDevice.layout)) return fromDevice.layout;
  return isLayoutType(globalLayout) ? globalLayout : null;
}
