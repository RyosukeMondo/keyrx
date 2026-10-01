import { describe, it, expect } from 'vitest';
import { diffMappings } from './mappingDiff';
import { friendlyKeyName } from './keyNames';

const base = `device_start("*");
  map("VK_A", "VK_B");
device_end();
`;

describe('friendlyKeyName', () => {
  it('speaks plain language', () => {
    expect(friendlyKeyName('VK_CapsLock')).toBe('Caps Lock');
    expect(friendlyKeyName('VK_LCtrl')).toBe('Ctrl');
    expect(friendlyKeyName('VK_RShift')).toBe('Right Shift');
    expect(friendlyKeyName('CapsLock')).toBe('Caps Lock');
    expect(friendlyKeyName('VK_A')).toBe('A');
    expect(friendlyKeyName('VK_Num3')).toBe('3');
    expect(friendlyKeyName('VK_PageUp')).toBe('Page Up');
    expect(friendlyKeyName('VK_F13')).toBe('F13');
    expect(friendlyKeyName('with_shift(VK_A)')).toBe('Shift+A');
    expect(friendlyKeyName('MD_00')).toBe('MD_00');
  });
});

describe('diffMappings', () => {
  it('reports added, changed and removed mappings', () => {
    const modified = `device_start("*");
  map("VK_A", "VK_C");
  map("VK_CapsLock", "VK_LCtrl");
device_end();
`;
    const changes = diffMappings(base, modified)!;
    expect(changes).toEqual([
      expect.objectContaining({ kind: 'changed', from: 'A', to: 'C' }),
      expect.objectContaining({
        kind: 'added',
        from: 'Caps Lock',
        to: 'Ctrl',
      }),
    ]);
    const removed = diffMappings(base, 'device_start("*");\ndevice_end();\n')!;
    expect(removed).toEqual([
      expect.objectContaining({ kind: 'removed', from: 'A', to: '' }),
    ]);
  });

  it('reports a swap as two changes', () => {
    const swapped = `device_start("*");
  map("VK_CapsLock", "VK_LCtrl");
  map("VK_LCtrl", "VK_CapsLock");
device_end();
`;
    const changes = diffMappings(
      'device_start("*");\ndevice_end();\n',
      swapped
    )!;
    expect(changes.map((c) => `${c.from}>${c.to}`)).toEqual([
      'Caps Lock>Ctrl',
      'Ctrl>Caps Lock',
    ]);
  });

  it('scopes changes to layers and device patterns', () => {
    const modified = `device_start("usb*");
  when_start("MD_00");
    map("VK_H", "VK_Left");
  when_end();
device_end();
`;
    const [c] = diffMappings('', modified)!;
    expect(c).toMatchObject({ layer: 'MD_00', device: 'usb*', from: 'H' });
  });

  it('describes tap/hold and ignores unchanged mappings', () => {
    const modified = base.replace(
      'device_end();',
      '  tap_hold("VK_Space", "VK_Space", "MD_00", 200);\ndevice_end();'
    );
    const changes = diffMappings(base, modified)!;
    expect(changes).toHaveLength(1);
    expect(changes[0].tapHold).toEqual({ tap: 'Space', hold: 'MD_00' });
  });

  it('returns null when a side cannot be parsed', () => {
    expect(diffMappings(base, 'device_start("x");')).toBeNull();
  });
});
