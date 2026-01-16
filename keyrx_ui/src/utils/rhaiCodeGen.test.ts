/**
 * Unit tests for Rhai code generator
 *
 * Tests code generation for all mapping types, device blocks, formatting,
 * and round-trip compatibility with the parser.
 */

import { describe, it, expect } from 'vitest';
import {
  generateRhaiScript,
  generateDeviceBlock,
  generateKeyMapping,
} from './rhaiCodeGen';
import { parseRhaiScript } from './rhaiParser';
import type {
  RhaiAST,
  DeviceBlock,
  KeyMapping,
  ModifierLayer,
  ImportStatement,
  Comment,
} from './rhaiParser';

describe('rhaiCodeGen', () => {
  describe('generateKeyMapping', () => {
    it('should generate simple mapping', () => {
      const mapping: KeyMapping = {
        type: 'simple',
        sourceKey: 'VK_A',
        targetKey: 'VK_B',
        line: 1,
      };

      const result = generateKeyMapping(mapping);
      expect(result).toBe('map("VK_A", "VK_B");');
    });

    it('should generate simple mapping with helper function', () => {
      const mapping: KeyMapping = {
        type: 'simple',
        sourceKey: 'VK_A',
        targetKey: 'with_ctrl("VK_B")',
        line: 1,
      };

      const result = generateKeyMapping(mapping);
      expect(result).toBe('map("VK_A", with_ctrl("VK_B"));');
    });

    it('should generate tap-hold mapping', () => {
      const mapping: KeyMapping = {
        type: 'tap_hold',
        sourceKey: 'VK_CAPSLOCK',
        line: 1,
        tapHold: {
          tapAction: 'VK_ESCAPE',
          holdAction: 'VK_LCTRL',
          thresholdMs: 200,
        },
      };

      const result = generateKeyMapping(mapping);
      expect(result).toBe('tap_hold("VK_CAPSLOCK", "VK_ESCAPE", "VK_LCTRL", 200);');
    });

    it('should generate macro mapping without delay', () => {
      const mapping: KeyMapping = {
        type: 'macro',
        sourceKey: 'VK_F1',
        line: 1,
        macro: {
          keys: ['VK_H', 'VK_E', 'VK_L', 'VK_L', 'VK_O'],
        },
      };

      const result = generateKeyMapping(mapping);
      expect(result).toBe('macro("VK_F1", ["VK_H", "VK_E", "VK_L", "VK_L", "VK_O"]);');
    });

    it('should generate macro mapping with delay', () => {
      const mapping: KeyMapping = {
        type: 'macro',
        sourceKey: 'VK_F1',
        line: 1,
        macro: {
          keys: ['VK_H', 'VK_E', 'VK_L', 'VK_L', 'VK_O'],
          delayMs: 50,
        },
      };

      const result = generateKeyMapping(mapping);
      expect(result).toBe('macro("VK_F1", ["VK_H", "VK_E", "VK_L", "VK_L", "VK_O"], 50);');
    });

    it('should generate layer switch mapping without mode', () => {
      const mapping: KeyMapping = {
        type: 'layer_switch',
        sourceKey: 'VK_F2',
        line: 1,
        layerSwitch: {
          layerId: 'layer_1',
        },
      };

      const result = generateKeyMapping(mapping);
      expect(result).toBe('layer_switch("VK_F2", "layer_1");');
    });

    it('should generate layer switch mapping with toggle mode', () => {
      const mapping: KeyMapping = {
        type: 'layer_switch',
        sourceKey: 'VK_F2',
        line: 1,
        layerSwitch: {
          layerId: 'layer_1',
          mode: 'toggle',
        },
      };

      const result = generateKeyMapping(mapping);
      expect(result).toBe('layer_switch("VK_F2", "layer_1", "toggle");');
    });

    it('should generate layer switch mapping with momentary mode', () => {
      const mapping: KeyMapping = {
        type: 'layer_switch',
        sourceKey: 'VK_F2',
        line: 1,
        layerSwitch: {
          layerId: 'layer_1',
          mode: 'momentary',
        },
      };

      const result = generateKeyMapping(mapping);
      expect(result).toBe('layer_switch("VK_F2", "layer_1", "momentary");');
    });

    it('should throw error for simple mapping without targetKey', () => {
      const mapping: KeyMapping = {
        type: 'simple',
        sourceKey: 'VK_A',
        line: 1,
      };

      expect(() => generateKeyMapping(mapping)).toThrow('Simple mapping missing targetKey');
    });

    it('should throw error for tap-hold mapping without config', () => {
      const mapping: KeyMapping = {
        type: 'tap_hold',
        sourceKey: 'VK_A',
        line: 1,
      };

      expect(() => generateKeyMapping(mapping)).toThrow('Tap-hold mapping missing tapHold config');
    });

    it('should throw error for macro mapping without config', () => {
      const mapping: KeyMapping = {
        type: 'macro',
        sourceKey: 'VK_A',
        line: 1,
      };

      expect(() => generateKeyMapping(mapping)).toThrow('Macro mapping missing macro config');
    });

    it('should throw error for layer switch mapping without config', () => {
      const mapping: KeyMapping = {
        type: 'layer_switch',
        sourceKey: 'VK_A',
        line: 1,
      };

      expect(() => generateKeyMapping(mapping)).toThrow('Layer switch mapping missing layerSwitch config');
    });
  });

  describe('generateDeviceBlock', () => {
    it('should generate device block with simple mappings', () => {
      const deviceBlock: DeviceBlock = {
        pattern: '*Keychron*',
        mappings: [
          { type: 'simple', sourceKey: 'VK_A', targetKey: 'VK_B', line: 2 },
          { type: 'simple', sourceKey: 'VK_C', targetKey: 'VK_D', line: 3 },
        ],
        layers: [],
        startLine: 1,
        endLine: 4,
      };

      const result = generateDeviceBlock(deviceBlock);
      expect(result).toEqual([
        'device_start("*Keychron*");',
        '    map("VK_A", "VK_B");',
        '    map("VK_C", "VK_D");',
        'device_end();',
      ]);
    });

    it('should generate device block with wildcard pattern', () => {
      const deviceBlock: DeviceBlock = {
        pattern: '*',
        mappings: [
          { type: 'simple', sourceKey: 'VK_A', targetKey: 'VK_B', line: 2 },
        ],
        layers: [],
        startLine: 1,
        endLine: 3,
      };

      const result = generateDeviceBlock(deviceBlock);
      expect(result).toEqual([
        'device_start("*");',
        '    map("VK_A", "VK_B");',
        'device_end();',
      ]);
    });

    it('should generate device block with modifier layers', () => {
      const layer: ModifierLayer = {
        modifiers: 'MD_00',
        mappings: [
          { type: 'simple', sourceKey: 'VK_H', targetKey: 'VK_LEFT', line: 4 },
          { type: 'simple', sourceKey: 'VK_L', targetKey: 'VK_RIGHT', line: 5 },
        ],
        startLine: 3,
        endLine: 6,
      };

      const deviceBlock: DeviceBlock = {
        pattern: '*',
        mappings: [
          { type: 'simple', sourceKey: 'VK_A', targetKey: 'VK_B', line: 2 },
        ],
        layers: [layer],
        startLine: 1,
        endLine: 7,
      };

      const result = generateDeviceBlock(deviceBlock);
      expect(result).toEqual([
        'device_start("*");',
        '    map("VK_A", "VK_B");',
        '    when_start("MD_00");',
        '        map("VK_H", "VK_LEFT");',
        '        map("VK_L", "VK_RIGHT");',
        '    when_end();',
        'device_end();',
      ]);
    });

    it('should generate device block with multiple modifier layers', () => {
      const layer1: ModifierLayer = {
        modifiers: 'MD_00',
        mappings: [
          { type: 'simple', sourceKey: 'VK_H', targetKey: 'VK_LEFT', line: 4 },
        ],
        startLine: 3,
        endLine: 5,
      };

      const layer2: ModifierLayer = {
        modifiers: ['MD_01', 'MD_02'],
        mappings: [
          { type: 'simple', sourceKey: 'VK_J', targetKey: 'VK_DOWN', line: 7 },
        ],
        startLine: 6,
        endLine: 8,
      };

      const deviceBlock: DeviceBlock = {
        pattern: '*',
        mappings: [],
        layers: [layer1, layer2],
        startLine: 1,
        endLine: 9,
      };

      const result = generateDeviceBlock(deviceBlock);
      expect(result).toEqual([
        'device_start("*");',
        '    when_start("MD_00");',
        '        map("VK_H", "VK_LEFT");',
        '    when_end();',
        '    when_start(["MD_01", "MD_02"]);',
        '        map("VK_J", "VK_DOWN");',
        '    when_end();',
        'device_end();',
      ]);
    });

    it('should generate empty device block', () => {
      const deviceBlock: DeviceBlock = {
        pattern: '*',
        mappings: [],
        layers: [],
        startLine: 1,
        endLine: 2,
      };

      const result = generateDeviceBlock(deviceBlock);
      expect(result).toEqual([
        'device_start("*");',
        'device_end();',
      ]);
    });
  });

  describe('generateRhaiScript', () => {
    it('should generate script with only global mappings', () => {
      const ast: RhaiAST = {
        imports: [],
        globalMappings: [
          { type: 'simple', sourceKey: 'VK_A', targetKey: 'VK_B', line: 1 },
          { type: 'simple', sourceKey: 'VK_C', targetKey: 'VK_D', line: 2 },
        ],
        deviceBlocks: [],
        comments: [],
      };

      const result = generateRhaiScript(ast);
      expect(result).toBe('device_start("*");\n    map("VK_A", "VK_B");\n    map("VK_C", "VK_D");\ndevice_end();');
    });

    it('should generate script with import statements', () => {
      const ast: RhaiAST = {
        imports: [
          { path: 'common.rhai', line: 1 },
          { path: 'helpers.rhai', alias: 'h', line: 2 },
        ],
        globalMappings: [
          { type: 'simple', sourceKey: 'VK_A', targetKey: 'VK_B', line: 4 },
        ],
        deviceBlocks: [],
        comments: [],
      };

      const result = generateRhaiScript(ast);
      const lines = result.split('\n');
      expect(lines[0]).toBe('import "common.rhai";');
      expect(lines[1]).toBe('import "helpers.rhai" as h;');
      expect(lines[2]).toBe('');
      expect(lines[3]).toBe('');
      expect(lines[4]).toBe('device_start("*");');
      expect(lines[5]).toBe('    map("VK_A", "VK_B");');
      expect(lines[6]).toBe('device_end();');
    });

    it('should generate script with device blocks', () => {
      const ast: RhaiAST = {
        imports: [],
        globalMappings: [],
        deviceBlocks: [
          {
            pattern: '*Keychron*',
            mappings: [
              { type: 'simple', sourceKey: 'VK_A', targetKey: 'VK_B', line: 2 },
            ],
            layers: [],
            startLine: 1,
            endLine: 3,
          },
          {
            pattern: '*HHKB*',
            mappings: [
              { type: 'simple', sourceKey: 'VK_C', targetKey: 'VK_D', line: 5 },
            ],
            layers: [],
            startLine: 4,
            endLine: 6,
          },
        ],
        comments: [],
      };

      const result = generateRhaiScript(ast);
      const lines = result.split('\n');
      expect(lines[0]).toBe('device_start("*Keychron*");');
      expect(lines[1]).toBe('    map("VK_A", "VK_B");');
      expect(lines[2]).toBe('device_end();');
      expect(lines[3]).toBe('');
      expect(lines[4]).toBe('device_start("*HHKB*");');
      expect(lines[5]).toBe('    map("VK_C", "VK_D");');
      expect(lines[6]).toBe('device_end();');
    });

    it('should generate script with global and device mappings', () => {
      const ast: RhaiAST = {
        imports: [],
        globalMappings: [
          { type: 'simple', sourceKey: 'VK_A', targetKey: 'VK_B', line: 1 },
        ],
        deviceBlocks: [
          {
            pattern: '*',
            mappings: [
              { type: 'simple', sourceKey: 'VK_C', targetKey: 'VK_D', line: 4 },
            ],
            layers: [],
            startLine: 3,
            endLine: 5,
          },
        ],
        comments: [],
      };

      const result = generateRhaiScript(ast);
      const lines = result.split('\n');
      expect(lines[0]).toBe('device_start("*");');
      expect(lines[1]).toBe('    map("VK_A", "VK_B");');
      expect(lines[2]).toBe('device_end();');
      expect(lines[3]).toBe(''); // Only 1 blank line (blankLinesBetweenDevices default)
      expect(lines[4]).toBe('device_start("*");');
      expect(lines[5]).toBe('    map("VK_C", "VK_D");');
      expect(lines[6]).toBe('device_end();');
    });

    it('should generate script with comments', () => {
      const ast: RhaiAST = {
        imports: [],
        globalMappings: [
          { type: 'simple', sourceKey: 'VK_A', targetKey: 'VK_B', line: 2 },
        ],
        deviceBlocks: [],
        comments: [
          { text: 'Global mappings', line: 1, type: 'line' },
        ],
      };

      const result = generateRhaiScript(ast);
      const lines = result.split('\n');
      expect(lines[0]).toBe('// Global mappings');
      expect(lines[1]).toBe('device_start("*");');
      expect(lines[2]).toBe('    map("VK_A", "VK_B");');
      expect(lines[3]).toBe('device_end();');
    });

    it('should generate script with block comments', () => {
      const ast: RhaiAST = {
        imports: [],
        globalMappings: [
          { type: 'simple', sourceKey: 'VK_A', targetKey: 'VK_B', line: 2 },
        ],
        deviceBlocks: [],
        comments: [
          { text: 'Important note', line: 1, type: 'block' },
        ],
      };

      const result = generateRhaiScript(ast);
      const lines = result.split('\n');
      expect(lines[0]).toBe('/* Important note */');
      expect(lines[1]).toBe('device_start("*");');
      expect(lines[2]).toBe('    map("VK_A", "VK_B");');
      expect(lines[3]).toBe('device_end();');
    });

    it('should generate empty script for empty AST', () => {
      const ast: RhaiAST = {
        imports: [],
        globalMappings: [],
        deviceBlocks: [],
        comments: [],
      };

      const result = generateRhaiScript(ast);
      expect(result).toBe('');
    });

    it('should respect custom formatting options', () => {
      const ast: RhaiAST = {
        imports: [],
        globalMappings: [],
        deviceBlocks: [
          {
            pattern: '*',
            mappings: [
              { type: 'simple', sourceKey: 'VK_A', targetKey: 'VK_B', line: 2 },
            ],
            layers: [],
            startLine: 1,
            endLine: 3,
          },
          {
            pattern: '*Keychron*',
            mappings: [
              { type: 'simple', sourceKey: 'VK_C', targetKey: 'VK_D', line: 5 },
            ],
            layers: [],
            startLine: 4,
            endLine: 6,
          },
        ],
        comments: [],
      };

      const result = generateRhaiScript(ast, {
        indentSize: 2,
        blankLinesBetweenDevices: 0,
      });

      const lines = result.split('\n');
      expect(lines[0]).toBe('device_start("*");');
      expect(lines[1]).toBe('  map("VK_A", "VK_B");'); // 2 spaces
      expect(lines[2]).toBe('device_end();');
      expect(lines[3]).toBe('device_start("*Keychron*");'); // No blank line
      expect(lines[4]).toBe('  map("VK_C", "VK_D");');
      expect(lines[5]).toBe('device_end();');
    });
  });

  describe('round-trip compatibility', () => {
    it('should preserve simple mappings through parse->generate->parse', () => {
      const original = 'map("VK_A", "VK_B");\nmap("VK_C", "VK_D");';

      const parseResult1 = parseRhaiScript(original);
      expect(parseResult1.success).toBe(true);

      const generated = generateRhaiScript(parseResult1.ast!);
      const parseResult2 = parseRhaiScript(generated);
      expect(parseResult2.success).toBe(true);

      // After generation, global mappings are wrapped in device_start("*") block
      // So they become device block mappings instead of global mappings
      expect(parseResult2.ast!.deviceBlocks).toHaveLength(1);
      expect(parseResult2.ast!.deviceBlocks[0].pattern).toBe('*');
      expect(parseResult2.ast!.deviceBlocks[0].mappings).toHaveLength(2);
      expect(parseResult2.ast!.deviceBlocks[0].mappings[0].sourceKey).toBe('VK_A');
      expect(parseResult2.ast!.deviceBlocks[0].mappings[0].targetKey).toBe('VK_B');
      expect(parseResult2.ast!.deviceBlocks[0].mappings[1].sourceKey).toBe('VK_C');
      expect(parseResult2.ast!.deviceBlocks[0].mappings[1].targetKey).toBe('VK_D');
    });

    it('should preserve tap-hold mappings through round-trip', () => {
      const original = 'tap_hold("VK_CAPSLOCK", "VK_ESCAPE", "VK_LCTRL", 200);';

      const parseResult1 = parseRhaiScript(original);
      expect(parseResult1.success).toBe(true);

      const generated = generateRhaiScript(parseResult1.ast!);
      const parseResult2 = parseRhaiScript(generated);
      expect(parseResult2.success).toBe(true);

      // After generation, global mappings are wrapped in device_start("*") block
      expect(parseResult2.ast!.deviceBlocks).toHaveLength(1);
      expect(parseResult2.ast!.deviceBlocks[0].pattern).toBe('*');
      expect(parseResult2.ast!.deviceBlocks[0].mappings).toHaveLength(1);
      const mapping = parseResult2.ast!.deviceBlocks[0].mappings[0];
      expect(mapping.type).toBe('tap_hold');
      expect(mapping.tapHold?.tapAction).toBe('VK_ESCAPE');
      expect(mapping.tapHold?.holdAction).toBe('VK_LCTRL');
      expect(mapping.tapHold?.thresholdMs).toBe(200);
    });

    it('should preserve device blocks through round-trip', () => {
      const original = `device_start("*Keychron*");
    map("VK_A", "VK_B");
device_end();`;

      const parseResult1 = parseRhaiScript(original);
      expect(parseResult1.success).toBe(true);

      const generated = generateRhaiScript(parseResult1.ast!);
      const parseResult2 = parseRhaiScript(generated);
      expect(parseResult2.success).toBe(true);

      expect(parseResult2.ast!.deviceBlocks).toHaveLength(1);
      expect(parseResult2.ast!.deviceBlocks[0].pattern).toBe('*Keychron*');
      expect(parseResult2.ast!.deviceBlocks[0].mappings).toHaveLength(1);
    });

    it('should preserve modifier layers through round-trip', () => {
      const original = `device_start("*");
    when_start("MD_00");
        map("VK_H", "VK_LEFT");
        map("VK_L", "VK_RIGHT");
    when_end();
device_end();`;

      const parseResult1 = parseRhaiScript(original);
      expect(parseResult1.success).toBe(true);

      const generated = generateRhaiScript(parseResult1.ast!);
      const parseResult2 = parseRhaiScript(generated);
      expect(parseResult2.success).toBe(true);

      expect(parseResult2.ast!.deviceBlocks).toHaveLength(1);
      expect(parseResult2.ast!.deviceBlocks[0].layers).toHaveLength(1);
      expect(parseResult2.ast!.deviceBlocks[0].layers[0].modifiers).toBe('MD_00');
      expect(parseResult2.ast!.deviceBlocks[0].layers[0].mappings).toHaveLength(2);
    });

    it('should preserve complex script with all features through round-trip', () => {
      const original = `import "common.rhai";

// Global mappings
map("VK_A", "VK_B");
tap_hold("VK_CAPSLOCK", "VK_ESCAPE", "VK_LCTRL", 200);

device_start("*Keychron*");
    map("VK_C", "VK_D");
    when_start("MD_00");
        map("VK_H", "VK_LEFT");
    when_end();
device_end();`;

      const parseResult1 = parseRhaiScript(original);
      expect(parseResult1.success).toBe(true);

      const generated = generateRhaiScript(parseResult1.ast!);
      const parseResult2 = parseRhaiScript(generated);
      expect(parseResult2.success).toBe(true);

      // Verify structure is preserved
      // After generation, global mappings are wrapped in device_start("*") block
      expect(parseResult2.ast!.imports).toHaveLength(1);
      expect(parseResult2.ast!.deviceBlocks).toHaveLength(2); // Now 2 device blocks: "*" and "*Keychron*"
      expect(parseResult2.ast!.deviceBlocks[0].pattern).toBe('*');
      expect(parseResult2.ast!.deviceBlocks[0].mappings).toHaveLength(2); // The 2 global mappings
      expect(parseResult2.ast!.deviceBlocks[1].pattern).toBe('*Keychron*');
      expect(parseResult2.ast!.deviceBlocks[1].mappings).toHaveLength(1);
      expect(parseResult2.ast!.deviceBlocks[1].layers).toHaveLength(1);
    });

    it('should handle edge case: 1000 mappings within 50ms', () => {
      // Create AST with 1000 mappings
      const mappings: KeyMapping[] = [];
      for (let i = 0; i < 1000; i++) {
        mappings.push({
          type: 'simple',
          sourceKey: `VK_${i}`,
          targetKey: `VK_${i + 1000}`,
          line: i + 1,
        });
      }

      const ast: RhaiAST = {
        imports: [],
        globalMappings: mappings,
        deviceBlocks: [],
        comments: [],
      };

      const startTime = performance.now();
      const generated = generateRhaiScript(ast);
      const endTime = performance.now();

      const duration = endTime - startTime;
      expect(duration).toBeLessThan(50); // Must complete within 50ms
      // With device_start("*") wrapper, we have 1000 mappings + device_start + device_end = 1002 lines
      expect(generated.split('\n')).toHaveLength(1002);
    });

    it('should preserve helper functions through round-trip', () => {
      const original = 'map("VK_A", with_ctrl("VK_B"));';

      const parseResult1 = parseRhaiScript(original);
      expect(parseResult1.success).toBe(true);

      const generated = generateRhaiScript(parseResult1.ast!);
      expect(generated).toBe('device_start("*");\n    map("VK_A", with_ctrl("VK_B"));\ndevice_end();');

      const parseResult2 = parseRhaiScript(generated);
      expect(parseResult2.success).toBe(true);
      // After generation, global mappings are wrapped in device_start("*") block
      expect(parseResult2.ast!.deviceBlocks).toHaveLength(1);
      expect(parseResult2.ast!.deviceBlocks[0].mappings[0].targetKey).toBe('with_ctrl("VK_B")');
    });
  });

  describe('formatting rules', () => {
    it('should use 4-space indentation by default', () => {
      const ast: RhaiAST = {
        imports: [],
        globalMappings: [],
        deviceBlocks: [
          {
            pattern: '*',
            mappings: [
              { type: 'simple', sourceKey: 'VK_A', targetKey: 'VK_B', line: 2 },
            ],
            layers: [],
            startLine: 1,
            endLine: 3,
          },
        ],
        comments: [],
      };

      const result = generateRhaiScript(ast);
      const lines = result.split('\n');
      // Check that the mapping line starts with exactly 4 spaces
      expect(lines[1]).toMatch(/^    map/);
    });

    it('should add 1 blank line between device blocks by default', () => {
      const ast: RhaiAST = {
        imports: [],
        globalMappings: [],
        deviceBlocks: [
          {
            pattern: '*',
            mappings: [],
            layers: [],
            startLine: 1,
            endLine: 2,
          },
          {
            pattern: '*Keychron*',
            mappings: [],
            layers: [],
            startLine: 3,
            endLine: 4,
          },
        ],
        comments: [],
      };

      const result = generateRhaiScript(ast);
      const lines = result.split('\n');
      // Should be: device_start, device_end, blank line, device_start, device_end
      expect(lines[2]).toBe(''); // Blank line between blocks
    });

    it('should add 2 blank lines between sections by default', () => {
      const ast: RhaiAST = {
        imports: [
          { path: 'common.rhai', line: 1 },
        ],
        globalMappings: [
          { type: 'simple', sourceKey: 'VK_A', targetKey: 'VK_B', line: 3 },
        ],
        deviceBlocks: [],
        comments: [],
      };

      const result = generateRhaiScript(ast);
      const lines = result.split('\n');
      // Should be: import, blank, blank, device_start, mapping, device_end
      expect(lines[1]).toBe('');
      expect(lines[2]).toBe('');
      expect(lines[3]).toBe('device_start("*");');
      expect(lines[4]).toBe('    map("VK_A", "VK_B");');
      expect(lines[5]).toBe('device_end();');
    });
  });
});
