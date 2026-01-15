import { describe, it, expect } from 'vitest';
import ANSI_104 from '../ANSI_104.json';
import ANSI_87 from '../ANSI_87.json';
import ISO_105 from '../ISO_105.json';
import ISO_88 from '../ISO_88.json';
import JIS_109 from '../JIS_109.json';
import COMPACT_60 from '../COMPACT_60.json';
import COMPACT_65 from '../COMPACT_65.json';
import COMPACT_75 from '../COMPACT_75.json';
import COMPACT_96 from '../COMPACT_96.json';
import HHKB from '../HHKB.json';
import NUMPAD from '../NUMPAD.json';

interface LayoutKey {
  code: string;
  label: string;
  x: number;
  y: number;
  w?: number;
  h?: number;
}

interface Layout {
  name: string;
  keys: LayoutKey[];
}

describe('Keyboard Layout Data Validation', () => {
  const layouts = [
    { data: ANSI_104, expectedName: 'ANSI 104', expectedKeyCount: 104 },
    { data: ANSI_87, expectedName: 'ANSI 87 (TKL)', expectedKeyCount: 87 },
    { data: ISO_105, expectedName: 'ISO 105', expectedKeyCount: 105 },
    { data: ISO_88, expectedName: 'ISO 88 (TKL)', expectedKeyCount: 88 },
    { data: JIS_109, expectedName: 'JIS 109', expectedKeyCount: 109 },
    { data: COMPACT_60, expectedName: '60% Compact', expectedKeyCount: 61 },
    { data: COMPACT_65, expectedName: '65% Compact', expectedKeyCount: 67 },
    { data: COMPACT_75, expectedName: '75% Compact', expectedKeyCount: 82 },
    { data: COMPACT_96, expectedName: '96% Compact', expectedKeyCount: 100 },
    { data: HHKB, expectedName: 'HHKB', expectedKeyCount: 60 },
    { data: NUMPAD, expectedName: 'Numpad', expectedKeyCount: 17 },
  ];

  describe.each(layouts)('$expectedName Layout', ({ data, expectedName, expectedKeyCount }) => {
    const layout = data as Layout;

    it('should have required name field', () => {
      expect(layout).toHaveProperty('name');
      expect(layout.name).toBe(expectedName);
      expect(typeof layout.name).toBe('string');
      expect(layout.name.length).toBeGreaterThan(0);
    });

    it('should have required keys array', () => {
      expect(layout).toHaveProperty('keys');
      expect(Array.isArray(layout.keys)).toBe(true);
    });

    it(`should have exactly ${expectedKeyCount} keys`, () => {
      expect(layout.keys).toHaveLength(expectedKeyCount);
    });

    it('should have all keys with required fields', () => {
      layout.keys.forEach((key, index) => {
        expect(key, `Key at index ${index} should have 'code' field`).toHaveProperty('code');
        expect(key, `Key at index ${index} should have 'label' field`).toHaveProperty('label');
        expect(key, `Key at index ${index} should have 'x' field`).toHaveProperty('x');
        expect(key, `Key at index ${index} should have 'y' field`).toHaveProperty('y');

        expect(typeof key.code, `Key '${key.code}' code should be string`).toBe('string');
        expect(typeof key.label, `Key '${key.code}' label should be string`).toBe('string');
        expect(typeof key.x, `Key '${key.code}' x should be number`).toBe('number');
        expect(typeof key.y, `Key '${key.code}' y should be number`).toBe('number');

        expect(key.code.length, `Key '${key.code}' code should not be empty`).toBeGreaterThan(0);
        expect(key.label.length, `Key '${key.code}' label should not be empty`).toBeGreaterThan(0);
      });
    });

    it('should have valid optional width field when present', () => {
      layout.keys.forEach((key) => {
        if ('w' in key) {
          expect(typeof key.w, `Key '${key.code}' width should be number`).toBe('number');
          expect(key.w, `Key '${key.code}' width should be positive`).toBeGreaterThan(0);
        }
      });
    });

    it('should have valid optional height field when present', () => {
      layout.keys.forEach((key) => {
        if ('h' in key) {
          expect(typeof key.h, `Key '${key.code}' height should be number`).toBe('number');
          expect(key.h, `Key '${key.code}' height should be positive`).toBeGreaterThan(0);
        }
      });
    });

    it('should have non-negative coordinates', () => {
      layout.keys.forEach((key) => {
        expect(key.x, `Key '${key.code}' x coordinate should be non-negative`).toBeGreaterThanOrEqual(0);
        expect(key.y, `Key '${key.code}' y coordinate should be non-negative`).toBeGreaterThanOrEqual(0);
      });
    });

    it('should have unique key codes', () => {
      const keyCodes = layout.keys.map((key) => key.code);
      const uniqueKeyCodes = new Set(keyCodes);
      expect(uniqueKeyCodes.size, 'All key codes should be unique').toBe(keyCodes.length);
    });
  });

  describe('Layout-specific validations', () => {
    it('ANSI_104 should have numpad keys', () => {
      const ansiLayout = ANSI_104 as Layout;
      const hasNumpadKeys = ansiLayout.keys.some((key) =>
        ['KC_NLCK', 'KC_PSLS', 'KC_PAST', 'KC_PMNS', 'KC_PPLS', 'KC_PENT', 'KC_PDOT'].includes(key.code) ||
        /^KC_P[0-9]$/.test(key.code)
      );
      expect(hasNumpadKeys, 'ANSI_104 should contain numpad keys (full-size layout)').toBe(true);
    });

    it('ANSI_87 should not have numpad keys', () => {
      const tkl87Layout = ANSI_87 as Layout;
      const hasNumpadKeys = tkl87Layout.keys.some((key) =>
        ['KC_NLCK', 'KC_PSLS', 'KC_PAST', 'KC_PMNS', 'KC_PPLS', 'KC_PENT', 'KC_PDOT'].includes(key.code) ||
        /^KC_P[0-9]$/.test(key.code)
      );
      expect(hasNumpadKeys, 'ANSI_87 should not contain numpad keys').toBe(false);
    });

    it('ISO_105 should have ISO-specific key KC_NUBS', () => {
      const isoLayout = ISO_105 as Layout;
      const hasNubs = isoLayout.keys.some((key) => key.code === 'KC_NUBS');
      expect(hasNubs, 'ISO_105 should contain KC_NUBS key').toBe(true);
    });

    it('ISO_88 should have ISO-specific key KC_NUBS', () => {
      const iso88Layout = ISO_88 as Layout;
      const hasNubs = iso88Layout.keys.some((key) => key.code === 'KC_NUBS');
      expect(hasNubs, 'ISO_88 should contain KC_NUBS key').toBe(true);
    });

    it('JIS_109 should have Japanese-specific keys', () => {
      const jisLayout = JIS_109 as Layout;
      const japaneseKeys = ['KC_JYEN', 'KC_RO', 'KC_HENK', 'KC_MHEN', 'KC_KANA'];

      japaneseKeys.forEach((keyCode) => {
        const hasKey = jisLayout.keys.some((key) => key.code === keyCode);
        expect(hasKey, `JIS_109 should contain ${keyCode}`).toBe(true);
      });
    });

    it('COMPACT_60 should not have F-row keys', () => {
      const compact60Layout = COMPACT_60 as Layout;
      const hasFKeys = compact60Layout.keys.some((key) => /^KC_F\d+$/.test(key.code));
      expect(hasFKeys, '60% layout should not have F-row keys').toBe(false);
    });

    it('COMPACT_75 should have F-row keys', () => {
      const compact75Layout = COMPACT_75 as Layout;
      const hasFKeys = compact75Layout.keys.some((key) => /^KC_F\d+$/.test(key.code));
      expect(hasFKeys, '75% layout should have F-row keys').toBe(true);
    });

    it('COMPACT_96 should have numpad keys', () => {
      const compact96Layout = COMPACT_96 as Layout;
      const hasNumpadKeys = compact96Layout.keys.some((key) =>
        ['KC_NLCK', 'KC_PSLS', 'KC_PAST', 'KC_PMNS'].includes(key.code)
      );
      expect(hasNumpadKeys, '96% layout should have numpad keys').toBe(true);
    });

    it('NUMPAD should only have numpad-related keys', () => {
      const numpadLayout = NUMPAD as Layout;
      const allKeysAreNumpad = numpadLayout.keys.every((key) =>
        ['KC_NLCK', 'KC_PSLS', 'KC_PAST', 'KC_PMNS', 'KC_PPLS', 'KC_PENT', 'KC_PDOT'].includes(key.code) ||
        /^KC_P[0-9]$/.test(key.code)
      );
      expect(allKeysAreNumpad, 'NUMPAD should only contain numpad keys').toBe(true);
    });

    it('HHKB should have Control in Caps Lock position', () => {
      const hhkbLayout = HHKB as Layout;
      const capsKey = hhkbLayout.keys.find((key) => key.code === 'KC_CAPS');
      const ctrlKey = hhkbLayout.keys.find((key) => key.code === 'KC_LCTL');

      // HHKB doesn't have KC_CAPS, instead has Control where Caps would be
      expect(capsKey, 'HHKB should not have Caps Lock key').toBeUndefined();
      expect(ctrlKey, 'HHKB should have left Control key').toBeDefined();
    });
  });
});
