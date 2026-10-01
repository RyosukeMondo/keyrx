import { afterEach, describe, expect, it } from 'vitest';
import {
  dslKeyName,
  formatKeyLabel,
  friendlyKeyName,
  keyNameParts,
} from './keyNames';
import { setLocale } from '@/i18n';

describe('Japanese keycap and key names', () => {
  afterEach(() => setLocale('en'));

  it('names JIS keys in Japanese with the DSL name as secondary text', () => {
    setLocale('ja');
    expect(keyNameParts('VK_Muhenkan')).toEqual({
      primary: '無変換',
      secondary: 'Muhenkan',
    });
    expect(keyNameParts('KC_HENK')).toEqual({
      primary: '変換',
      secondary: 'Henkan',
    });
    expect(keyNameParts('VK_Zenkaku').primary).toBe('半角/全角');
    expect(friendlyKeyName('VK_CapsLock')).toBe('Caps Lock（英数）');
    expect(formatKeyLabel('VK_Muhenkan')).toBe('無変換');
  });

  it('stays English by default and speaks real key names', () => {
    expect(keyNameParts('VK_Muhenkan')).toEqual({ primary: 'Muhenkan' });
    expect(friendlyKeyName('VK_Grave')).toBe('Backtick');
    expect(friendlyKeyName('KC_P3')).toBe('Numpad 3');
    expect(friendlyKeyName('KC_SPC')).toBe('Space');
    expect(dslKeyName('LK-00')).toBe('LK_00');
  });
});
