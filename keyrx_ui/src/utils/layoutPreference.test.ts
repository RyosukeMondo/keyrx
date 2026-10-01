import { describe, it, expect } from 'vitest';
import {
  detectLayout,
  devicesForScopes,
  isLayoutType,
  resolveStoredLayout,
} from './layoutPreference';
import type { DeviceEntry } from '@/types';
import { buildScopes } from './deviceScopes';

const dev = (over: Partial<DeviceEntry>): DeviceEntry => ({
  id: 'path-/dev/input/event3',
  name: 'USB Keyboard',
  path: '/dev/input/event3',
  serial: null,
  active: true,
  scope: 'global',
  layout: null,
  isVirtual: false,
  ...over,
});

describe('resolveStoredLayout', () => {
  it('prefers the selected device layout over the global default', () => {
    expect(resolveStoredLayout([dev({ layout: 'JIS_109' })], 'ISO_105')).toBe(
      'JIS_109'
    );
  });

  it('falls back to the global layout, then null (never invents one)', () => {
    expect(resolveStoredLayout([dev({})], 'ISO_105')).toBe('ISO_105');
    expect(resolveStoredLayout([dev({})], null)).toBeNull();
    expect(resolveStoredLayout([], undefined)).toBeNull();
  });

  it('ignores layout names the UI cannot draw', () => {
    expect(
      resolveStoredLayout([dev({ layout: 'KLE_custom' })], 'NOPE')
    ).toBeNull();
    expect(isLayoutType('JIS_109')).toBe(true);
    expect(isLayoutType('nope')).toBe(false);
  });
});

describe('detectLayout', () => {
  it('detects JIS from profile source keys', () => {
    expect(detectLayout('map("VK_Muhenkan", "VK_A");', [])).toBe('JIS_109');
    expect(detectLayout('map("VK_A", "VK_B");', [])).toBe('ANSI_104');
  });

  it('detects a JIS board from the device name', () => {
    expect(
      detectLayout('map("VK_A", "VK_B");', [dev({ name: 'Acme JIS Keyboard' })])
    ).toBe('JIS_109');
    expect(detectLayout('map("VK_A", "VK_B");', [dev({})])).toBe('ANSI_104');
  });
});

describe('devicesForScopes', () => {
  it('resolves pattern scopes to the devices they match', () => {
    const pad = dev({
      id: 'p2',
      name: 'A11y Numpad',
      path: '/dev/input/event9',
    });
    const all = [dev({}), pad];
    const scopes = buildScopes(['a11y*'], all).filter(
      (s) => s.pattern !== undefined
    );
    expect(devicesForScopes(scopes, all).map((d) => d.id)).toEqual(['p2']);
  });
});
