/**
 * Standalone test for parseRhaiScript against examples/user_layout.rhai
 *
 * Run: node keyrx_ui/test_parser.mjs
 *
 * This is a plain JavaScript port of src/utils/rhaiParser.ts parseRhaiScript()
 * to test the parser logic without TypeScript compilation.
 */

import { readFileSync } from 'fs';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));

// ============================================================
// Port of parseRhaiScript from src/utils/rhaiParser.ts
// ============================================================
function parseRhaiScript(script) {
  try {
    const lines = script.split('\n');
    const ast = {
      imports: [],
      globalMappings: [],
      deviceBlocks: [],
      comments: [],
    };

    let currentDeviceBlock = null;
    let currentModifierLayer = null;
    let lineNumber = 0;
    /** @type {string[]} */
    const skippedLines = []; // Track lines that were silently ignored

    for (const line of lines) {
      lineNumber++;
      const trimmed = line.trim();

      // Skip empty lines
      if (!trimmed) continue;

      // Extract comments
      const commentMatch = trimmed.match(/^\/\/(.*)$/);
      if (commentMatch) {
        ast.comments.push({
          text: commentMatch[1].trim(),
          line: lineNumber,
          type: 'line',
        });
        continue;
      }

      // Extract block comments (simplified - assumes single-line /* */)
      const blockCommentMatch = trimmed.match(/^\/\*(.*)\*\/$/);
      if (blockCommentMatch) {
        ast.comments.push({
          text: blockCommentMatch[1].trim(),
          line: lineNumber,
          type: 'block',
        });
        continue;
      }

      // Parse import statements
      const importMatch = trimmed.match(
        /^import\s+"([^"]+)"(?:\s+as\s+(\w+))?/
      );
      if (importMatch) {
        ast.imports.push({
          path: importMatch[1],
          alias: importMatch[2],
          line: lineNumber,
        });
        continue;
      }

      // Parse device_start
      const deviceStartMatch = trimmed.match(
        /^device_start\s*\(\s*"([^"]+)"\s*\)/
      );
      if (deviceStartMatch) {
        if (currentDeviceBlock) {
          return {
            success: false,
            error: {
              message: 'Nested device blocks are not allowed',
              line: lineNumber,
              column: 0,
              suggestion:
                'Close previous device block with device_end() before starting a new one',
            },
          };
        }
        currentDeviceBlock = {
          pattern: deviceStartMatch[1],
          mappings: [],
          layers: [],
          startLine: lineNumber,
          endLine: -1,
        };
        continue;
      }

      // Parse device_end
      if (trimmed.match(/^device_end\s*\(\s*\)/)) {
        if (!currentDeviceBlock) {
          return {
            success: false,
            error: {
              message: 'device_end() without matching device_start()',
              line: lineNumber,
              column: 0,
              suggestion: 'Add device_start("pattern") before device_end()',
            },
          };
        }
        currentDeviceBlock.endLine = lineNumber;
        ast.deviceBlocks.push(currentDeviceBlock);
        currentDeviceBlock = null;
        continue;
      }

      // Parse when_start (modifier layers)
      const whenStartMatch = trimmed.match(
        /^when_start\s*\(\s*(?:"([^"]+)"|\[([^\]]+)\])\s*\)/
      );
      if (whenStartMatch) {
        if (currentModifierLayer) {
          return {
            success: false,
            error: {
              message: 'Nested when blocks are not allowed',
              line: lineNumber,
              column: 0,
              suggestion:
                'Close previous when block with when_end() before starting a new one',
            },
          };
        }

        // Parse modifiers - either single string or array
        let modifiers;
        if (whenStartMatch[1]) {
          modifiers = whenStartMatch[1];
        } else {
          modifiers = whenStartMatch[2]
            .split(',')
            .map((m) => m.trim().replace(/["']/g, ''));
        }

        currentModifierLayer = {
          modifiers,
          mappings: [],
          startLine: lineNumber,
          endLine: -1,
        };
        continue;
      }

      // Parse when_end
      if (trimmed.match(/^when_end\s*\(\s*\)/)) {
        if (!currentModifierLayer) {
          return {
            success: false,
            error: {
              message: 'when_end() without matching when_start()',
              line: lineNumber,
              column: 0,
              suggestion: 'Add when_start("modifier") before when_end()',
            },
          };
        }
        currentModifierLayer.endLine = lineNumber;

        if (currentDeviceBlock) {
          currentDeviceBlock.layers.push(currentModifierLayer);
        }
        currentModifierLayer = null;
        continue;
      }

      // Parse map() - simple mapping
      const mapMatch = trimmed.match(
        /^map\s*\(\s*"([^"]+)"\s*,\s*"?([^")]+)"?\s*\)/
      );
      if (mapMatch) {
        const mapping = {
          type: 'simple',
          sourceKey: mapMatch[1],
          targetKey: mapMatch[2].replace(/"/g, ''),
          line: lineNumber,
        };

        if (currentModifierLayer) {
          currentModifierLayer.mappings.push(mapping);
        } else if (currentDeviceBlock) {
          currentDeviceBlock.mappings.push(mapping);
        } else {
          ast.globalMappings.push(mapping);
        }
        continue;
      }

      // Parse tap_hold()
      const tapHoldMatch = trimmed.match(
        /^tap_hold\s*\(\s*"([^"]+)"\s*,\s*"([^"]+)"\s*,\s*"([^"]+)"\s*,\s*(\d+)\s*\)/
      );
      if (tapHoldMatch) {
        const mapping = {
          type: 'tap_hold',
          sourceKey: tapHoldMatch[1],
          line: lineNumber,
          tapHold: {
            tapAction: tapHoldMatch[2],
            holdAction: tapHoldMatch[3],
            thresholdMs: parseInt(tapHoldMatch[4], 10),
          },
        };

        if (currentModifierLayer) {
          currentModifierLayer.mappings.push(mapping);
        } else if (currentDeviceBlock) {
          currentDeviceBlock.mappings.push(mapping);
        } else {
          ast.globalMappings.push(mapping);
        }
        continue;
      }

      // Parse helper functions like with_ctrl(), with_shift(), etc.
      const withModMatch = trimmed.match(
        /^map\s*\(\s*"([^"]+)"\s*,\s*(with_\w+\([^)]+\))\s*\)/
      );
      if (withModMatch) {
        const mapping = {
          type: 'simple',
          sourceKey: withModMatch[1],
          targetKey: withModMatch[2],
          line: lineNumber,
        };

        if (currentModifierLayer) {
          currentModifierLayer.mappings.push(mapping);
        } else if (currentDeviceBlock) {
          currentDeviceBlock.mappings.push(mapping);
        } else {
          ast.globalMappings.push(mapping);
        }
        continue;
      }

      // If we get here, line was silently ignored
      skippedLines.push(`  L${lineNumber}: ${trimmed}`);
    }

    // Check for unclosed blocks
    if (currentDeviceBlock) {
      return {
        success: false,
        error: {
          message: 'Unclosed device block',
          line: currentDeviceBlock.startLine,
          column: 0,
          suggestion: 'Add device_end() to close the device block',
        },
      };
    }

    if (currentModifierLayer) {
      return {
        success: false,
        error: {
          message: 'Unclosed when block',
          line: currentModifierLayer.startLine,
          column: 0,
          suggestion: 'Add when_end() to close the when block',
        },
      };
    }

    return {
      success: true,
      ast,
      skippedLines, // Extra: not in original, for debugging
    };
  } catch (error) {
    return {
      success: false,
      error: {
        message: error instanceof Error ? error.message : 'Unknown parse error',
        line: 0,
        column: 0,
        suggestion: 'Check script syntax and try again',
      },
    };
  }
}

// ============================================================
// Simulate SVGKeyboard's normalizeKeyCode (KC_ -> VK_)
// ============================================================
function normalizeKeyCodeLayout(code) {
  if (!code) return code;
  if (code.startsWith('VK_')) return code;

  if (code.match(/^KC_[0-9]$/)) {
    const digit = code.charAt(code.length - 1);
    return `VK_Num${digit}`;
  }

  if (code.match(/^KC_P[0-9]$/)) {
    const digit = code.charAt(code.length - 1);
    return `VK_Numpad${digit}`;
  }

  const specialKeyMap = {
    KC_LBRC: 'VK_LeftBracket',
    KC_RBRC: 'VK_RightBracket',
    KC_BSLS: 'VK_Backslash',
    KC_SCLN: 'VK_Semicolon',
    KC_QUOT: 'VK_Quote',
    KC_COMM: 'VK_Comma',
    KC_DOT: 'VK_Period',
    KC_SLSH: 'VK_Slash',
    KC_GRV: 'VK_Grave',
    KC_MINS: 'VK_Minus',
    KC_EQL: 'VK_Equal',
    KC_ESC: 'VK_Escape',
    KC_TAB: 'VK_Tab',
    KC_CAPS: 'VK_CapsLock',
    KC_SPC: 'VK_Space',
    KC_ENT: 'VK_Enter',
    KC_BSPC: 'VK_Backspace',
    KC_DEL: 'VK_Delete',
    KC_INS: 'VK_Insert',
    KC_HOME: 'VK_Home',
    KC_END: 'VK_End',
    KC_PGUP: 'VK_PageUp',
    KC_PGDN: 'VK_PageDown',
    KC_UP: 'VK_Up',
    KC_DOWN: 'VK_Down',
    KC_LEFT: 'VK_Left',
    KC_RGHT: 'VK_Right',
    KC_PSCR: 'VK_PrintScreen',
    KC_SCRL: 'VK_ScrollLock',
    KC_PAUS: 'VK_Pause',
    KC_LSFT: 'VK_LShift',
    KC_RSFT: 'VK_RShift',
    KC_LCTL: 'VK_LCtrl',
    KC_RCTL: 'VK_RCtrl',
    KC_LALT: 'VK_LAlt',
    KC_RALT: 'VK_RAlt',
    KC_LGUI: 'VK_LMeta',
    KC_RGUI: 'VK_RMeta',
    KC_F1: 'VK_F1', KC_F2: 'VK_F2', KC_F3: 'VK_F3', KC_F4: 'VK_F4',
    KC_F5: 'VK_F5', KC_F6: 'VK_F6', KC_F7: 'VK_F7', KC_F8: 'VK_F8',
    KC_F9: 'VK_F9', KC_F10: 'VK_F10', KC_F11: 'VK_F11', KC_F12: 'VK_F12',
    KC_NLCK: 'VK_NumLock',
    KC_PSLS: 'VK_NumpadDivide',
    KC_PAST: 'VK_NumpadMultiply',
    KC_PMNS: 'VK_NumpadSubtract',
    KC_PPLS: 'VK_NumpadAdd',
    KC_PENT: 'VK_NumpadEnter',
    KC_PDOT: 'VK_NumpadDecimal',
  };

  if (specialKeyMap[code]) return specialKeyMap[code];
  if (code.startsWith('KC_')) return code.replace(/^KC_/, 'VK_');
  return `VK_${code}`;
}

// ============================================================
// Simulate useASTSync's normalizeKeyCode (VK_ passthrough)
// ============================================================
function normalizeKeyCodeAST(key) {
  if (!key) return key;
  if (key.startsWith('VK_')) return key;
  if (key.startsWith('KC_')) return key.replace(/^KC_/, 'VK_');
  if (/^[A-Z0-9]$/i.test(key)) return `VK_${key.toUpperCase()}`;

  const knownKeys = [
    'ESCAPE', 'ENTER', 'SPACE', 'TAB', 'BACKSPACE', 'DELETE',
    'INSERT', 'HOME', 'END', 'PAGEUP', 'PAGEDOWN',
    'UP', 'DOWN', 'LEFT', 'RIGHT',
    'CAPSLOCK', 'NUMLOCK', 'SCROLLLOCK',
    'LEFTSHIFT', 'RIGHTSHIFT', 'LEFTCONTROL', 'RIGHTCONTROL',
    'LEFTALT', 'RIGHTALT', 'LEFTMETA', 'RIGHTMETA',
  ];
  if (knownKeys.includes(key.toUpperCase())) return `VK_${key.toUpperCase()}`;
  return key;
}

// ============================================================
// Main test
// ============================================================

const rhaiPath = resolve(__dirname, '..', 'examples', 'user_layout.rhai');
const jisLayoutPath = resolve(__dirname, 'src', 'data', 'layouts', 'JIS_109.json');

console.log('='.repeat(70));
console.log('KeyRx Parser Test: parseRhaiScript() vs user_layout.rhai');
console.log('='.repeat(70));

// Read the rhai source
let rhaiSource;
try {
  rhaiSource = readFileSync(rhaiPath, 'utf8');
  console.log(`\nLoaded: ${rhaiPath}`);
  console.log(`  Lines: ${rhaiSource.split('\n').length}`);
} catch (err) {
  console.error(`FATAL: Cannot read ${rhaiPath}: ${err.message}`);
  process.exit(1);
}

// Parse
const result = parseRhaiScript(rhaiSource);

console.log('\n--- Parse Result ---');
console.log(`  Success: ${result.success}`);

if (!result.success) {
  console.log(`  ERROR: ${result.error.message}`);
  console.log(`    Line: ${result.error.line}, Column: ${result.error.column}`);
  console.log(`    Suggestion: ${result.error.suggestion}`);
  process.exit(1);
}

const ast = result.ast;

console.log(`  Imports: ${ast.imports.length}`);
console.log(`  Global mappings: ${ast.globalMappings.length}`);
console.log(`  Device blocks: ${ast.deviceBlocks.length}`);
console.log(`  Comments: ${ast.comments.length}`);

for (const block of ast.deviceBlocks) {
  console.log(`\n  Device block: pattern="${block.pattern}" (lines ${block.startLine}-${block.endLine})`);
  console.log(`    Base mappings: ${block.mappings.length}`);
  console.log(`    Layers: ${block.layers.length}`);

  // Count mapping types
  const types = {};
  for (const m of block.mappings) {
    types[m.type] = (types[m.type] || 0) + 1;
  }
  console.log(`    Base mapping types: ${JSON.stringify(types)}`);

  for (const layer of block.layers) {
    const mods = Array.isArray(layer.modifiers) ? layer.modifiers.join(', ') : layer.modifiers;
    const ltypes = {};
    for (const m of layer.mappings) {
      ltypes[m.type] = (ltypes[m.type] || 0) + 1;
    }
    console.log(`    Layer "${mods}" (lines ${layer.startLine}-${layer.endLine}): ${layer.mappings.length} mappings ${JSON.stringify(ltypes)}`);
  }
}

// Show skipped lines (lines that matched no regex)
if (result.skippedLines && result.skippedLines.length > 0) {
  console.log(`\n--- WARNING: ${result.skippedLines.length} lines silently skipped ---`);
  for (const line of result.skippedLines) {
    console.log(line);
  }
} else {
  console.log('\n--- No lines were silently skipped ---');
}

// ============================================================
// Test 2: Key code matching (layout vs parser)
// ============================================================
console.log('\n' + '='.repeat(70));
console.log('Key Code Matching Test: JIS_109 layout codes vs parsed AST keys');
console.log('='.repeat(70));

let jisLayout;
try {
  jisLayout = JSON.parse(readFileSync(jisLayoutPath, 'utf8'));
  console.log(`\nLoaded: ${jisLayoutPath}`);
  console.log(`  Layout keys: ${jisLayout.keys.length}`);
} catch (err) {
  console.error(`FATAL: Cannot read ${jisLayoutPath}: ${err.message}`);
  process.exit(1);
}

// Build set of ALL source keys from the AST (all device blocks + global)
const allASTKeys = new Set();
for (const m of ast.globalMappings) {
  allASTKeys.add(normalizeKeyCodeAST(m.sourceKey));
}
for (const block of ast.deviceBlocks) {
  for (const m of block.mappings) {
    allASTKeys.add(normalizeKeyCodeAST(m.sourceKey));
  }
  for (const layer of block.layers) {
    for (const m of layer.mappings) {
      allASTKeys.add(normalizeKeyCodeAST(m.sourceKey));
    }
  }
}

console.log(`  Unique source keys in AST: ${allASTKeys.size}`);

// Build set of normalized layout codes
const layoutVKCodes = new Set();
const layoutCodeMap = new Map(); // VK code -> original KC code
for (const key of jisLayout.keys) {
  const normalized = normalizeKeyCodeLayout(key.code);
  layoutVKCodes.add(normalized);
  layoutCodeMap.set(normalized, key.code);
}

console.log(`  Unique VK codes in layout: ${layoutVKCodes.size}`);

// Find mismatches
const matchedKeys = [];
const unmatchedASTKeys = [];
const unmatchedLayoutKeys = [];

for (const vk of allASTKeys) {
  if (layoutVKCodes.has(vk)) {
    matchedKeys.push(vk);
  } else {
    unmatchedASTKeys.push(vk);
  }
}

for (const vk of layoutVKCodes) {
  if (!allASTKeys.has(vk)) {
    unmatchedLayoutKeys.push(`${vk} (from ${layoutCodeMap.get(vk)})`);
  }
}

console.log(`\n  Matched (AST key exists in layout): ${matchedKeys.length}`);
console.log(`  AST keys NOT in layout: ${unmatchedASTKeys.length}`);
if (unmatchedASTKeys.length > 0) {
  console.log('    These AST keys have no matching layout key (won\'t be visualized):');
  for (const k of unmatchedASTKeys.sort()) {
    console.log(`      ${k}`);
  }
}
console.log(`  Layout keys NOT in AST: ${unmatchedLayoutKeys.length}`);
if (unmatchedLayoutKeys.length > 0) {
  console.log('    These layout keys have no mapping (expected - not all keys are remapped):');
  for (const k of unmatchedLayoutKeys.sort()) {
    console.log(`      ${k}`);
  }
}

// ============================================================
// Test 3: Simulate useASTSync with globalSelected=true
// ============================================================
console.log('\n' + '='.repeat(70));
console.log('useASTSync Simulation: globalSelected=true, selectedDevices=[]');
console.log('='.repeat(70));

const layerMappings = new Map();
layerMappings.set('base', new Map());

const baseMap = layerMappings.get('base');

// Process global mappings
ast.globalMappings.forEach((m) => {
  baseMap.set(normalizeKeyCodeAST(m.sourceKey), { type: m.type, tapAction: m.targetKey });
});

// Process wildcard device block
const wildcardBlock = ast.deviceBlocks.find(b => b.pattern === '*');
if (wildcardBlock) {
  console.log(`\n  Found wildcard block: ${wildcardBlock.mappings.length} base mappings, ${wildcardBlock.layers.length} layers`);

  wildcardBlock.mappings.forEach((m) => {
    const key = normalizeKeyCodeAST(m.sourceKey);
    if (m.type === 'simple') {
      baseMap.set(key, { type: 'simple', tapAction: m.targetKey });
    } else if (m.type === 'tap_hold') {
      baseMap.set(key, {
        type: 'tap_hold',
        tapAction: m.tapHold.tapAction,
        holdAction: m.tapHold.holdAction,
        threshold: m.tapHold.thresholdMs,
      });
    }
  });

  wildcardBlock.layers.forEach((layer) => {
    const mods = Array.isArray(layer.modifiers) ? layer.modifiers : [layer.modifiers];
    mods.forEach((mod) => {
      const layerId = mod.toLowerCase().replace('_', '-');
      if (!layerMappings.has(layerId)) {
        layerMappings.set(layerId, new Map());
      }
      const layerMap = layerMappings.get(layerId);
      layer.mappings.forEach((m) => {
        layerMap.set(normalizeKeyCodeAST(m.sourceKey), { type: m.type, tapAction: m.targetKey });
      });
    });
  });
} else {
  console.log('\n  WARNING: No wildcard (*) device block found!');
}

console.log(`\n  Layer mappings produced:`);
for (const [layerId, map] of layerMappings.entries()) {
  console.log(`    ${layerId}: ${map.size} keys`);
}

// Check how many base-layer keys match layout keys
const baseLayerMatches = [];
const baseLayerMisses = [];
for (const [vk] of baseMap) {
  if (layoutVKCodes.has(vk)) {
    baseLayerMatches.push(vk);
  } else {
    baseLayerMisses.push(vk);
  }
}

console.log(`\n  Base layer keys matching JIS layout: ${baseLayerMatches.length}/${baseMap.size}`);
if (baseLayerMisses.length > 0) {
  console.log(`  Base layer keys NOT matching any JIS layout key:`);
  for (const k of baseLayerMisses.sort()) {
    console.log(`    ${k}`);
  }
}

// ============================================================
// Test 4: Specific regex edge cases
// ============================================================
console.log('\n' + '='.repeat(70));
console.log('Regex Edge Case Tests');
console.log('='.repeat(70));

const testCases = [
  {
    name: 'with_shift (single arg)',
    line: 'map("VK_Slash", with_shift("VK_Num2"));       // phys / -> @',
    expectMatch: 'withMod',
    expectSource: 'VK_Slash',
    expectTarget: 'with_shift("VK_Num2")',
  },
  {
    name: 'with_mods (5 args with commas)',
    line: 'map("VK_Num2", with_mods("VK_Z", false, true, false, false));  // 2 -> C-Z',
    expectMatch: 'withMod',
    expectSource: 'VK_Num2',
    expectTarget: 'with_mods("VK_Z", false, true, false, false)',
  },
  {
    name: 'simple map with semicolon comment',
    line: 'map("VK_Semicolon", "VK_V");  // ; -> V',
    expectMatch: 'simple',
    expectSource: 'VK_Semicolon',
    expectTarget: 'VK_V',
  },
  {
    name: 'Japanese key name',
    line: 'map("VK_無変換", "VK_Space");       // 無変換 (Muhenkan) -> Space',
    expectMatch: 'simple',
    expectSource: 'VK_無変換',
    expectTarget: 'VK_Space',
  },
  {
    name: 'Comment with parens in it',
    line: 'map("VK_Enter", "VK_Yen");          // Enter -> Yen (円キー)',
    expectMatch: 'simple',
    expectSource: 'VK_Enter',
    expectTarget: 'VK_Yen',
  },
  {
    name: 'tap_hold 4 args',
    line: 'tap_hold("VK_B", "VK_Enter", "MD_00", 200);       // B: tap=Enter, hold=MD_00',
    expectMatch: 'tapHold',
    expectSource: 'VK_B',
    expectTarget: undefined,
  },
  {
    name: 'with_shift with closing paren in comment',
    line: 'map("VK_LeftBracket", with_shift("VK_Period")); // [ -> >',
    expectMatch: 'withMod',
    expectSource: 'VK_LeftBracket',
    expectTarget: 'with_shift("VK_Period")',
  },
  {
    name: 'with_mods with Shift+Ctrl',
    line: 'map("VK_H", with_mods("VK_B", true, true, false, false));      // H -> C-S-B',
    expectMatch: 'withMod',
    expectSource: 'VK_H',
    expectTarget: 'with_mods("VK_B", true, true, false, false)',
  },
];

let passed = 0;
let failed = 0;

for (const tc of testCases) {
  const trimmed = tc.line.trim();

  // Try simple map regex
  const mapMatch = trimmed.match(
    /^map\s*\(\s*"([^"]+)"\s*,\s*"?([^")]+)"?\s*\)/
  );

  // Try tap_hold regex
  const tapHoldMatch = trimmed.match(
    /^tap_hold\s*\(\s*"([^"]+)"\s*,\s*"([^"]+)"\s*,\s*"([^"]+)"\s*,\s*(\d+)\s*\)/
  );

  // Try withMod regex
  const withModMatch = trimmed.match(
    /^map\s*\(\s*"([^"]+)"\s*,\s*(with_\w+\([^)]+\))\s*\)/
  );

  let actualMatch = 'none';
  let actualSource = null;
  let actualTarget = null;

  if (mapMatch) {
    actualMatch = 'simple';
    actualSource = mapMatch[1];
    actualTarget = mapMatch[2].replace(/"/g, '');
  }

  // tap_hold checked BEFORE withMod (same order as parser)
  if (!mapMatch && tapHoldMatch) {
    actualMatch = 'tapHold';
    actualSource = tapHoldMatch[1];
    actualTarget = undefined;
  }

  if (!mapMatch && !tapHoldMatch && withModMatch) {
    actualMatch = 'withMod';
    actualSource = withModMatch[1];
    actualTarget = withModMatch[2];
  }

  const matchOk = actualMatch === tc.expectMatch;
  const sourceOk = actualSource === tc.expectSource;
  const targetOk = actualTarget === tc.expectTarget;
  const allOk = matchOk && sourceOk && targetOk;

  if (allOk) {
    console.log(`  PASS: ${tc.name}`);
    passed++;
  } else {
    console.log(`  FAIL: ${tc.name}`);
    if (!matchOk) console.log(`    Match type: expected="${tc.expectMatch}" actual="${actualMatch}"`);
    if (!sourceOk) console.log(`    Source key: expected="${tc.expectSource}" actual="${actualSource}"`);
    if (!targetOk) console.log(`    Target key: expected="${tc.expectTarget}" actual="${actualTarget}"`);
    failed++;
  }
}

console.log(`\n  Results: ${passed} passed, ${failed} failed out of ${testCases.length}`);

// ============================================================
// Summary
// ============================================================
console.log('\n' + '='.repeat(70));
console.log('SUMMARY');
console.log('='.repeat(70));

const totalMappings = (wildcardBlock?.mappings.length || 0) +
  (wildcardBlock?.layers.reduce((sum, l) => sum + l.mappings.length, 0) || 0);

console.log(`
  Parser:
    Status:            ${result.success ? 'SUCCESS' : 'FAILURE'}
    Device blocks:     ${ast.deviceBlocks.length}
    Total mappings:    ${totalMappings} (in wildcard block)
    Layers:            ${wildcardBlock?.layers.length || 0}
    Skipped lines:     ${result.skippedLines?.length || 0}

  Key Code Matching:
    AST keys matched:  ${matchedKeys.length}/${allASTKeys.size} in JIS layout
    Unmatched AST:     ${unmatchedASTKeys.length} keys

  useASTSync sim:
    Base layer keys:   ${baseMap.size}
    Visible on layout: ${baseLayerMatches.length}/${baseMap.size}
    Missing from vis:  ${baseLayerMisses.length}

  Regex edge cases:    ${passed}/${testCases.length} passed
`);

if (unmatchedASTKeys.length > 0 || baseLayerMisses.length > 0) {
  console.log('  >>> KEY CODE MISMATCHES DETECTED <<<');
  console.log('  Some parsed keys use codes that don\'t match the JIS layout.');
  console.log('  The SVGKeyboard normalizeKeyCode() may not handle these codes.');
  console.log('  This would cause mappings to be invisible on those keys.');
}

if (result.skippedLines?.length > 0) {
  console.log('  >>> PARSER SILENTLY SKIPPED LINES <<<');
  console.log('  The parser could not match some non-empty, non-comment lines.');
  console.log('  These mappings are LOST and will not appear in the visual editor.');
}

process.exit(failed > 0 || !result.success ? 1 : 0);
