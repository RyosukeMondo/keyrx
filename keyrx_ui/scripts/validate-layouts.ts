#!/usr/bin/env tsx
/**
 * Layout Validation Script
 *
 * Validates that all keyboard layout files have correct key codes
 * that match the system's KeyCode enum.
 *
 * Run: npx tsx scripts/validate-layouts.ts
 */

import * as fs from 'fs';
import * as path from 'path';
import { fileURLToPath } from 'url';
import { dirname } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Expected key codes based on keyrx_core KeyCode enum
// Reference: keyrx_core/src/config/keys.rs
const VALID_KEY_CODES = {
  // Letters A-Z
  letters: Array.from({ length: 26 }, (_, i) => `KC_${String.fromCharCode(65 + i)}`),

  // Top row numbers 0-9
  numbers: Array.from({ length: 10 }, (_, i) => `KC_${i}`),

  // Function keys F1-F24
  functionKeys: [
    ...Array.from({ length: 12 }, (_, i) => `KC_F${i + 1}`),
    ...Array.from({ length: 12 }, (_, i) => `KC_F${i + 13}`),
  ],

  // Modifiers
  modifiers: [
    'KC_LSFT', 'KC_RSFT', 'KC_LCTL', 'KC_RCTL',
    'KC_LALT', 'KC_RALT', 'KC_LGUI', 'KC_RGUI',
  ],

  // Special keys
  special: [
    'KC_ESC', 'KC_ENT', 'KC_BSPC', 'KC_TAB', 'KC_SPC',
    'KC_CAPS', 'KC_NLCK', 'KC_SLCK', 'KC_PSCR', 'KC_PAUS',
    'KC_INS', 'KC_DEL', 'KC_HOME', 'KC_END',
    'KC_PGUP', 'KC_PGDN',
  ],

  // Arrow keys
  arrows: ['KC_LEFT', 'KC_RGHT', 'KC_UP', 'KC_DOWN'],

  // Punctuation
  punctuation: [
    'KC_LBRC', 'KC_RBRC', 'KC_BSLS', 'KC_SCLN', 'KC_QUOT',
    'KC_COMM', 'KC_DOT', 'KC_SLSH', 'KC_GRV', 'KC_MINS', 'KC_EQL',
  ],

  // Numpad
  numpad: [
    ...Array.from({ length: 10 }, (_, i) => `KC_P${i}`),
    'KC_PSLS', 'KC_PAST', 'KC_PMNS', 'KC_PPLS', 'KC_PENT', 'KC_PDOT',
    'KC_NLCK',
  ],

  // Media keys
  media: [
    'KC_MUTE', 'KC_VOLD', 'KC_VOLU',
    'KC_MPLY', 'KC_MSTP', 'KC_MPRV', 'KC_MNXT',
  ],

  // System keys
  system: ['KC_PWR', 'KC_SLEP', 'KC_WAKE'],

  // Browser keys
  browser: [
    'KC_WBAK', 'KC_WFWD', 'KC_WREF', 'KC_WSTP',
    'KC_WSCH', 'KC_WFAV', 'KC_WHOM',
  ],

  // Application keys
  app: ['KC_MAIL', 'KC_CALC', 'KC_MYCM'],

  // Other
  other: [
    'KC_APP', 'KC_HELP', 'KC_SLCT', 'KC_EXEC',
    'KC_UNDO', 'KC_REDO', 'KC_CUT', 'KC_COPY', 'KC_PSTE', 'KC_FIND',
  ],

  // JIS keys
  jis: [
    'KC_ZKHK', 'KC_KANA', 'KC_HIRA', 'KC_HENK',
    'KC_MHEN', 'KC_YEN', 'KC_RO', 'KC_KANA', 'KC_JYEN',
  ],

  // Korean keys
  korean: ['KC_LANG1', 'KC_LANG2'],

  // ISO keys
  iso: [
    'KC_NUBS', // Extra key between left shift and Z
    'KC_NUHS', // ISO hash/number sign key (next to Enter)
  ],
};

// Flatten all valid key codes
const ALL_VALID_CODES = Object.values(VALID_KEY_CODES).flat();

// VK_ to KC_ mapping for validation
const VK_TO_KC_MAP: Record<string, string> = {
  // Letters
  ...Object.fromEntries(Array.from({ length: 26 }, (_, i) => {
    const letter = String.fromCharCode(65 + i);
    return [`VK_${letter}`, `KC_${letter}`];
  })),

  // Top row numbers (VK_Num0-9 maps to KC_0-9)
  ...Object.fromEntries(Array.from({ length: 10 }, (_, i) =>
    [`VK_Num${i}`, `KC_${i}`]
  )),

  // Numpad (VK_Numpad0-9 maps to KC_P0-9)
  ...Object.fromEntries(Array.from({ length: 10 }, (_, i) =>
    [`VK_Numpad${i}`, `KC_P${i}`]
  )),

  // Function keys
  ...Object.fromEntries(Array.from({ length: 24 }, (_, i) =>
    [`VK_F${i + 1}`, `KC_F${i + 1}`]
  )),

  // Modifiers
  'VK_LShift': 'KC_LSFT',
  'VK_RShift': 'KC_RSFT',
  'VK_LCtrl': 'KC_LCTL',
  'VK_RCtrl': 'KC_RCTL',
  'VK_LAlt': 'KC_LALT',
  'VK_RAlt': 'KC_RALT',
  'VK_LMeta': 'KC_LGUI',
  'VK_RMeta': 'KC_RGUI',

  // Special keys
  'VK_Escape': 'KC_ESC',
  'VK_Enter': 'KC_ENT',
  'VK_Backspace': 'KC_BSPC',
  'VK_Tab': 'KC_TAB',
  'VK_Space': 'KC_SPC',
  'VK_CapsLock': 'KC_CAPS',
  'VK_NumLock': 'KC_NLCK',
  'VK_ScrollLock': 'KC_SLCK',
  'VK_PrintScreen': 'KC_PSCR',
  'VK_Pause': 'KC_PAUS',
  'VK_Insert': 'KC_INS',
  'VK_Delete': 'KC_DEL',
  'VK_Home': 'KC_HOME',
  'VK_End': 'KC_END',
  'VK_PageUp': 'KC_PGUP',
  'VK_PageDown': 'KC_PGDN',

  // Arrows
  'VK_Left': 'KC_LEFT',
  'VK_Right': 'KC_RGHT',
  'VK_Up': 'KC_UP',
  'VK_Down': 'KC_DOWN',

  // Punctuation
  'VK_LeftBracket': 'KC_LBRC',
  'VK_RightBracket': 'KC_RBRC',
  'VK_Backslash': 'KC_BSLS',
  'VK_Semicolon': 'KC_SCLN',
  'VK_Quote': 'KC_QUOT',
  'VK_Comma': 'KC_COMM',
  'VK_Period': 'KC_DOT',
  'VK_Slash': 'KC_SLSH',
  'VK_Grave': 'KC_GRV',
  'VK_Minus': 'KC_MINS',
  'VK_Equal': 'KC_EQL',

  // Numpad special
  'VK_NumpadDivide': 'KC_PSLS',
  'VK_NumpadMultiply': 'KC_PAST',
  'VK_NumpadSubtract': 'KC_PMNS',
  'VK_NumpadAdd': 'KC_PPLS',
  'VK_NumpadEnter': 'KC_PENT',
  'VK_NumpadDecimal': 'KC_PDOT',

  // Other
  'VK_Menu': 'KC_APP',
};

interface LayoutKey {
  code: string;
  label: string;
  x: number;
  y: number;
  w?: number;
  h?: number;
  shape?: string;
}

interface Layout {
  name: string;
  keys: LayoutKey[];
}

function validateLayout(layoutPath: string): {
  valid: boolean;
  errors: string[];
  warnings: string[];
} {
  const errors: string[] = [];
  const warnings: string[] = [];

  try {
    const content = fs.readFileSync(layoutPath, 'utf-8');
    const layout: Layout = JSON.parse(content);

    if (!layout.name) {
      errors.push('Missing layout name');
    }

    if (!Array.isArray(layout.keys)) {
      errors.push('Missing or invalid keys array');
      return { valid: false, errors, warnings };
    }

    const seenCodes = new Set<string>();

    layout.keys.forEach((key, index) => {
      // Check required fields
      if (!key.code) {
        errors.push(`Key at index ${index} missing 'code' field`);
        return;
      }

      if (!key.label) {
        warnings.push(`Key ${key.code} missing 'label' field`);
      }

      if (key.x === undefined || key.y === undefined) {
        errors.push(`Key ${key.code} missing position (x or y)`);
      }

      // Check for duplicate codes
      if (seenCodes.has(key.code)) {
        errors.push(`Duplicate key code: ${key.code}`);
      }
      seenCodes.add(key.code);

      // Validate key code format
      if (!key.code.startsWith('KC_')) {
        errors.push(`Invalid key code format: ${key.code} (must start with KC_)`);
      } else if (!ALL_VALID_CODES.includes(key.code)) {
        warnings.push(`Unknown key code: ${key.code} (not in KeyCode enum)`);
      }
    });

    return {
      valid: errors.length === 0,
      errors,
      warnings,
    };
  } catch (err) {
    return {
      valid: false,
      errors: [`Failed to parse layout: ${err instanceof Error ? err.message : String(err)}`],
      warnings,
    };
  }
}

function main() {
  const layoutsDir = path.join(__dirname, '../src/data/layouts');

  console.log('🔍 Validating keyboard layouts...\n');

  const files = fs.readdirSync(layoutsDir).filter(f => f.endsWith('.json'));

  let totalErrors = 0;
  let totalWarnings = 0;

  files.forEach(file => {
    const layoutPath = path.join(layoutsDir, file);
    const result = validateLayout(layoutPath);

    console.log(`📄 ${file}`);

    if (result.valid) {
      console.log('  ✅ Valid');
    } else {
      console.log('  ❌ Invalid');
    }

    if (result.errors.length > 0) {
      console.log('  Errors:');
      result.errors.forEach(err => console.log(`    ❌ ${err}`));
      totalErrors += result.errors.length;
    }

    if (result.warnings.length > 0) {
      console.log('  Warnings:');
      result.warnings.forEach(warn => console.log(`    ⚠️  ${warn}`));
      totalWarnings += result.warnings.length;
    }

    console.log('');
  });

  console.log('─'.repeat(60));
  console.log(`\n📊 Summary:`);
  console.log(`  Files validated: ${files.length}`);
  console.log(`  Total errors: ${totalErrors}`);
  console.log(`  Total warnings: ${totalWarnings}`);

  if (totalErrors > 0) {
    console.log('\n❌ Validation failed');
    process.exit(1);
  } else if (totalWarnings > 0) {
    console.log('\n⚠️  Validation passed with warnings');
    process.exit(0);
  } else {
    console.log('\n✅ All layouts valid!');
    process.exit(0);
  }
}

main();
