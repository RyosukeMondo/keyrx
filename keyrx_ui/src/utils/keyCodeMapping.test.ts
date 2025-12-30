/**
 * Tests for keyCodeMapping utilities
 */

import {
  eventCodeToVK,
  vkToEventCode,
  vkToLabel,
  formatKeyCode,
  labelToVK,
  parseKeyCode,
  isKnownKeyCode,
  getAllEventCodes,
  getAllVKNames,
} from './keyCodeMapping';

describe('keyCodeMapping', () => {
  describe('eventCodeToVK', () => {
    it('should convert letter codes to VK names', () => {
      expect(eventCodeToVK(30)).toBe('VK_A');
      expect(eventCodeToVK(44)).toBe('VK_Z');
      expect(eventCodeToVK(16)).toBe('VK_Q');
    });

    it('should convert number codes to VK names', () => {
      expect(eventCodeToVK(2)).toBe('VK_Num1');
      expect(eventCodeToVK(11)).toBe('VK_Num0');
    });

    it('should convert function key codes to VK names', () => {
      expect(eventCodeToVK(59)).toBe('VK_F1');
      expect(eventCodeToVK(88)).toBe('VK_F12');
      expect(eventCodeToVK(183)).toBe('VK_F13');
    });

    it('should convert special key codes to VK names', () => {
      expect(eventCodeToVK(1)).toBe('VK_Escape');
      expect(eventCodeToVK(28)).toBe('VK_Enter');
      expect(eventCodeToVK(57)).toBe('VK_Space');
    });

    it('should convert modifier codes to VK names', () => {
      expect(eventCodeToVK(29)).toBe('VK_LeftCtrl');
      expect(eventCodeToVK(42)).toBe('VK_LeftShift');
      expect(eventCodeToVK(56)).toBe('VK_LeftAlt');
    });

    it('should convert navigation codes to VK names', () => {
      expect(eventCodeToVK(102)).toBe('VK_Home');
      expect(eventCodeToVK(103)).toBe('VK_Up');
      expect(eventCodeToVK(111)).toBe('VK_Delete');
    });

    it('should convert symbol codes to VK names', () => {
      expect(eventCodeToVK(12)).toBe('VK_Minus');
      expect(eventCodeToVK(51)).toBe('VK_Comma');
    });

    it('should handle unknown codes with VK_Unknown format', () => {
      expect(eventCodeToVK(999)).toBe('VK_Unknown999');
      expect(eventCodeToVK(0)).toBe('VK_Unknown0');
      expect(eventCodeToVK(500)).toBe('VK_Unknown500');
    });

    it('should handle edge cases', () => {
      expect(eventCodeToVK(-1)).toBe('VK_Unknown-1');
      expect(eventCodeToVK(255)).toBe('VK_Unknown255');
    });
  });

  describe('vkToEventCode', () => {
    it('should convert VK names to event codes', () => {
      expect(vkToEventCode('VK_A')).toBe(30);
      expect(vkToEventCode('VK_Enter')).toBe(28);
      expect(vkToEventCode('VK_Space')).toBe(57);
    });

    it('should return null for unknown VK names', () => {
      expect(vkToEventCode('VK_Invalid')).toBeNull();
      expect(vkToEventCode('Invalid')).toBeNull();
      expect(vkToEventCode('')).toBeNull();
    });

    it('should be inverse of eventCodeToVK for known codes', () => {
      const knownCodes = [30, 28, 57, 1, 42, 102];
      for (const code of knownCodes) {
        const vk = eventCodeToVK(code);
        expect(vkToEventCode(vk)).toBe(code);
      }
    });
  });

  describe('vkToLabel', () => {
    it('should convert letter VK names to single letters', () => {
      expect(vkToLabel('VK_A')).toBe('A');
      expect(vkToLabel('VK_Z')).toBe('Z');
    });

    it('should convert number VK names to digits', () => {
      expect(vkToLabel('VK_Num1')).toBe('1');
      expect(vkToLabel('VK_Num0')).toBe('0');
    });

    it('should convert function keys to F1-F20 format', () => {
      expect(vkToLabel('VK_F1')).toBe('F1');
      expect(vkToLabel('VK_F12')).toBe('F12');
      expect(vkToLabel('VK_F13')).toBe('F13');
    });

    it('should use common abbreviations for special keys', () => {
      expect(vkToLabel('VK_Escape')).toBe('ESC');
      expect(vkToLabel('VK_Enter')).toBe('ENTER');
      expect(vkToLabel('VK_Backspace')).toBe('BACKSPACE');
    });

    it('should use standard modifier abbreviations', () => {
      expect(vkToLabel('VK_LeftCtrl')).toBe('LCTRL');
      expect(vkToLabel('VK_LeftShift')).toBe('LSHIFT');
      expect(vkToLabel('VK_LeftAlt')).toBe('LALT');
    });

    it('should convert navigation keys to abbreviations', () => {
      expect(vkToLabel('VK_PageUp')).toBe('PGUP');
      expect(vkToLabel('VK_PageDown')).toBe('PGDN');
      expect(vkToLabel('VK_Insert')).toBe('INS');
      expect(vkToLabel('VK_Delete')).toBe('DEL');
    });

    it('should show actual symbols for symbol keys', () => {
      expect(vkToLabel('VK_Minus')).toBe('-');
      expect(vkToLabel('VK_Equal')).toBe('=');
      expect(vkToLabel('VK_Comma')).toBe(',');
    });

    it('should handle VK_Unknown format', () => {
      expect(vkToLabel('VK_Unknown123')).toBe('KEY_123');
      expect(vkToLabel('VK_Unknown999')).toBe('KEY_999');
    });

    it('should strip VK_ prefix for unmapped keys', () => {
      expect(vkToLabel('VK_CustomKey')).toBe('CustomKey');
    });
  });

  describe('formatKeyCode', () => {
    it('should convert code to human-readable label', () => {
      expect(formatKeyCode(30)).toBe('A');
      expect(formatKeyCode(28)).toBe('ENTER');
      expect(formatKeyCode(57)).toBe('SPACE');
    });

    it('should handle unknown codes', () => {
      expect(formatKeyCode(999)).toBe('KEY_999');
      expect(formatKeyCode(0)).toBe('KEY_0');
    });

    it('should format all common key types correctly', () => {
      expect(formatKeyCode(1)).toBe('ESC');
      expect(formatKeyCode(42)).toBe('LSHIFT');
      expect(formatKeyCode(102)).toBe('HOME');
      expect(formatKeyCode(59)).toBe('F1');
      expect(formatKeyCode(2)).toBe('1');
    });
  });

  describe('labelToVK', () => {
    it('should convert labels to VK names', () => {
      expect(labelToVK('A')).toBe('VK_A');
      expect(labelToVK('ENTER')).toBe('VK_Enter');
      expect(labelToVK('ESC')).toBe('VK_Escape');
    });

    it('should be case-insensitive', () => {
      expect(labelToVK('a')).toBe('VK_A');
      expect(labelToVK('enter')).toBe('VK_Enter');
      expect(labelToVK('LCTRL')).toBe('VK_LeftCtrl');
      expect(labelToVK('lctrl')).toBe('VK_LeftCtrl');
    });

    it('should handle modifier abbreviations', () => {
      expect(labelToVK('LCTRL')).toBe('VK_LeftCtrl');
      expect(labelToVK('LSHIFT')).toBe('VK_LeftShift');
      expect(labelToVK('LALT')).toBe('VK_LeftAlt');
    });

    it('should return null for unknown labels', () => {
      expect(labelToVK('INVALID')).toBeNull();
      expect(labelToVK('')).toBeNull();
      expect(labelToVK('XYZ123')).toBeNull();
    });

    it('should be inverse of vkToLabel for known keys', () => {
      const vkNames = ['VK_A', 'VK_Enter', 'VK_Space', 'VK_LeftCtrl', 'VK_F1'];
      for (const vk of vkNames) {
        const label = vkToLabel(vk);
        expect(labelToVK(label)).toBe(vk);
      }
    });
  });

  describe('parseKeyCode', () => {
    it('should convert labels to event codes', () => {
      expect(parseKeyCode('A')).toBe(30);
      expect(parseKeyCode('ENTER')).toBe(28);
      expect(parseKeyCode('SPACE')).toBe(57);
    });

    it('should be case-insensitive', () => {
      expect(parseKeyCode('a')).toBe(30);
      expect(parseKeyCode('enter')).toBe(28);
    });

    it('should return null for unknown labels', () => {
      expect(parseKeyCode('INVALID')).toBeNull();
      expect(parseKeyCode('')).toBeNull();
    });

    it('should be inverse of formatKeyCode for known codes', () => {
      const codes = [30, 28, 57, 1, 42, 102];
      for (const code of codes) {
        const label = formatKeyCode(code);
        expect(parseKeyCode(label)).toBe(code);
      }
    });
  });

  describe('isKnownKeyCode', () => {
    it('should return true for known codes', () => {
      expect(isKnownKeyCode(30)).toBe(true); // A
      expect(isKnownKeyCode(28)).toBe(true); // Enter
      expect(isKnownKeyCode(57)).toBe(true); // Space
    });

    it('should return false for unknown codes', () => {
      expect(isKnownKeyCode(999)).toBe(false);
      expect(isKnownKeyCode(0)).toBe(false);
      expect(isKnownKeyCode(-1)).toBe(false);
    });

    it('should cover all standard key ranges', () => {
      // At least some keys from each category should be known
      expect(isKnownKeyCode(1)).toBe(true); // Escape
      expect(isKnownKeyCode(30)).toBe(true); // Letter
      expect(isKnownKeyCode(2)).toBe(true); // Number
      expect(isKnownKeyCode(59)).toBe(true); // Function
      expect(isKnownKeyCode(42)).toBe(true); // Modifier
      expect(isKnownKeyCode(102)).toBe(true); // Navigation
      expect(isKnownKeyCode(51)).toBe(true); // Symbol
    });
  });

  describe('getAllEventCodes', () => {
    it('should return an array of numbers', () => {
      const codes = getAllEventCodes();
      expect(Array.isArray(codes)).toBe(true);
      expect(codes.every((c) => typeof c === 'number')).toBe(true);
    });

    it('should include common key codes', () => {
      const codes = getAllEventCodes();
      expect(codes).toContain(30); // A
      expect(codes).toContain(28); // Enter
      expect(codes).toContain(57); // Space
    });

    it('should not contain duplicates', () => {
      const codes = getAllEventCodes();
      const unique = [...new Set(codes)];
      expect(codes.length).toBe(unique.length);
    });

    it('should have reasonable size (60+ keys minimum)', () => {
      const codes = getAllEventCodes();
      expect(codes.length).toBeGreaterThan(60);
    });
  });

  describe('getAllVKNames', () => {
    it('should return an array of strings', () => {
      const names = getAllVKNames();
      expect(Array.isArray(names)).toBe(true);
      expect(names.every((n) => typeof n === 'string')).toBe(true);
    });

    it('should include common VK names', () => {
      const names = getAllVKNames();
      expect(names).toContain('VK_A');
      expect(names).toContain('VK_Enter');
      expect(names).toContain('VK_Space');
    });

    it('should have all names start with VK_', () => {
      const names = getAllVKNames();
      expect(names.every((n) => n.startsWith('VK_'))).toBe(true);
    });

    it('should match the count of event codes', () => {
      const codes = getAllEventCodes();
      const names = getAllVKNames();
      expect(names.length).toBe(codes.length);
    });
  });

  describe('round-trip conversions', () => {
    it('should support code -> VK -> code round trip', () => {
      const testCodes = [30, 28, 57, 1, 42, 102, 59];
      for (const code of testCodes) {
        const vk = eventCodeToVK(code);
        const backToCode = vkToEventCode(vk);
        expect(backToCode).toBe(code);
      }
    });

    it('should support code -> label -> code round trip', () => {
      const testCodes = [30, 28, 57, 1, 42, 102, 59];
      for (const code of testCodes) {
        const label = formatKeyCode(code);
        const backToCode = parseKeyCode(label);
        expect(backToCode).toBe(code);
      }
    });

    it('should support VK -> label -> VK round trip', () => {
      const testVKs = ['VK_A', 'VK_Enter', 'VK_Space', 'VK_LeftCtrl', 'VK_F1'];
      for (const vk of testVKs) {
        const label = vkToLabel(vk);
        const backToVK = labelToVK(label);
        expect(backToVK).toBe(vk);
      }
    });
  });

  describe('edge cases', () => {
    it('should handle negative codes', () => {
      expect(eventCodeToVK(-1)).toBe('VK_Unknown-1');
      expect(isKnownKeyCode(-1)).toBe(false);
    });

    it('should handle very large codes', () => {
      expect(eventCodeToVK(99999)).toBe('VK_Unknown99999');
      expect(isKnownKeyCode(99999)).toBe(false);
    });

    it('should handle empty strings', () => {
      expect(labelToVK('')).toBeNull();
      expect(parseKeyCode('')).toBeNull();
    });

    it('should handle whitespace in labels', () => {
      expect(labelToVK('  ')).toBeNull();
      expect(parseKeyCode('  ')).toBeNull();
    });
  });

  describe('special characters in labels', () => {
    it('should handle symbol characters as labels', () => {
      expect(labelToVK('-')).toBe('VK_Minus');
      expect(labelToVK('=')).toBe('VK_Equal');
      expect(labelToVK(',')).toBe('VK_Comma');
      expect(labelToVK('.')).toBe('VK_Period');
      expect(labelToVK('/')).toBe('VK_Slash');
    });

    it('should convert symbol codes correctly', () => {
      expect(formatKeyCode(12)).toBe('-');
      expect(formatKeyCode(13)).toBe('=');
      expect(formatKeyCode(51)).toBe(',');
    });
  });
});
