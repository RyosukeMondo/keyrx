/**
 * Unit tests for Rhai parser
 */

import { describe, it, expect } from 'vitest';
import { parseRhaiCode } from './rhaiParser';

describe('rhaiParser', () => {
  describe('parseRhaiCode', () => {
    it('should parse empty device block', () => {
      const rhai = `
device_start("*");
device_end();
      `.trim();

      const result = parseRhaiCode(rhai);

      expect(result.errors).toHaveLength(0);
      expect(result.config.layers).toHaveLength(1);
      expect(result.config.layers[0].isBase).toBe(true);
      expect(result.config.layers[0].mappings).toHaveLength(0);
      expect(result.config.modifiers).toHaveLength(0);
      expect(result.config.locks).toHaveLength(0);
    });

    it('should parse simple key mappings', () => {
      const rhai = `
device_start("*");
    map("A", "VK_B");
    map("C", "VK_D");
device_end();
      `.trim();

      const result = parseRhaiCode(rhai);

      expect(result.errors).toHaveLength(0);
      expect(result.config.layers).toHaveLength(1);
      expect(result.config.layers[0].mappings).toHaveLength(2);

      const [mapping1, mapping2] = result.config.layers[0].mappings;
      expect(mapping1.sourceKey).toBe('KEY_A');
      expect(mapping1.targetKey).toBe('KEY_B');
      expect(mapping1.type).toBe('simple');

      expect(mapping2.sourceKey).toBe('KEY_C');
      expect(mapping2.targetKey).toBe('KEY_D');
      expect(mapping2.type).toBe('simple');
    });

    it('should parse custom modifiers', () => {
      const rhai = `
device_start("*");
    // Custom Modifiers
    map("CAPSLOCK", "MD_00");
    map("TAB", "MD_01");
device_end();
      `.trim();

      const result = parseRhaiCode(rhai);

      expect(result.errors).toHaveLength(0);
      expect(result.config.modifiers).toHaveLength(2);

      const [mod1, mod2] = result.config.modifiers;
      expect(mod1.triggerKey).toBe('KEY_CAPSLOCK');
      expect(mod1.name).toBe('Modifier 0');

      expect(mod2.triggerKey).toBe('KEY_TAB');
      expect(mod2.name).toBe('Modifier 1');
    });

    it('should parse custom locks', () => {
      const rhai = `
device_start("*");
    // Custom Locks
    map("SCROLLLOCK", "LK_00");
    map("NUMLOCK", "LK_01");
device_end();
      `.trim();

      const result = parseRhaiCode(rhai);

      expect(result.errors).toHaveLength(0);
      expect(result.config.locks).toHaveLength(2);

      const [lock1, lock2] = result.config.locks;
      expect(lock1.triggerKey).toBe('KEY_SCROLLLOCK');
      expect(lock1.name).toBe('Lock 0');

      expect(lock2.triggerKey).toBe('KEY_NUMLOCK');
      expect(lock2.name).toBe('Lock 1');
    });

    it('should parse base layer mappings', () => {
      const rhai = `
device_start("*");
    // Base Layer Mappings
    map("Q", "VK_A");
    map("W", "VK_S");
    map("E", "VK_D");
device_end();
      `.trim();

      const result = parseRhaiCode(rhai);

      expect(result.errors).toHaveLength(0);
      expect(result.config.layers).toHaveLength(1);
      expect(result.config.layers[0].isBase).toBe(true);
      expect(result.config.layers[0].mappings).toHaveLength(3);

      const mappings = result.config.layers[0].mappings;
      expect(mappings[0].sourceKey).toBe('KEY_Q');
      expect(mappings[0].targetKey).toBe('KEY_A');
      expect(mappings[1].sourceKey).toBe('KEY_W');
      expect(mappings[1].targetKey).toBe('KEY_S');
      expect(mappings[2].sourceKey).toBe('KEY_E');
      expect(mappings[2].targetKey).toBe('KEY_D');
    });

    it('should parse conditional layer with when() block', () => {
      const rhai = `
device_start("*");
    map("CAPSLOCK", "MD_00");

    when("MD_00", [
        map("Q", "VK_1"),
        map("W", "VK_2"),
        map("E", "VK_3")
    ]);
device_end();
      `.trim();

      const result = parseRhaiCode(rhai);

      expect(result.errors).toHaveLength(0);
      expect(result.config.modifiers).toHaveLength(1);
      expect(result.config.layers).toHaveLength(2); // Base + conditional

      const baseLayer = result.config.layers.find(l => l.isBase);
      expect(baseLayer).toBeDefined();
      expect(baseLayer!.mappings).toHaveLength(0);

      const conditionalLayer = result.config.layers.find(l => !l.isBase);
      expect(conditionalLayer).toBeDefined();
      expect(conditionalLayer!.mappings).toHaveLength(3);

      const mappings = conditionalLayer!.mappings;
      expect(mappings[0].sourceKey).toBe('KEY_Q');
      expect(mappings[0].targetKey).toBe('KEY_1');
      expect(mappings[1].sourceKey).toBe('KEY_W');
      expect(mappings[1].targetKey).toBe('KEY_2');
      expect(mappings[2].sourceKey).toBe('KEY_E');
      expect(mappings[2].targetKey).toBe('KEY_3');
    });

    it('should parse multiple conditional layers', () => {
      const rhai = `
device_start("*");
    map("CAPSLOCK", "MD_00");
    map("TAB", "MD_01");

    when("MD_00", [
        map("Q", "VK_1")
    ]);

    when("MD_01", [
        map("A", "VK_Z")
    ]);
device_end();
      `.trim();

      const result = parseRhaiCode(rhai);

      expect(result.errors).toHaveLength(0);
      expect(result.config.modifiers).toHaveLength(2);
      expect(result.config.layers).toHaveLength(3); // Base + 2 conditional

      const baseLayer = result.config.layers.find(l => l.isBase);
      expect(baseLayer).toBeDefined();

      const conditionalLayers = result.config.layers.filter(l => !l.isBase);
      expect(conditionalLayers).toHaveLength(2);

      expect(conditionalLayers[0].mappings).toHaveLength(1);
      expect(conditionalLayers[0].mappings[0].targetKey).toBe('KEY_1');

      expect(conditionalLayers[1].mappings).toHaveLength(1);
      expect(conditionalLayers[1].mappings[0].targetKey).toBe('KEY_Z');
    });

    it('should parse complete configuration', () => {
      const rhai = `
// KeyRx2 Configuration
// Generated by Visual Config Builder

device_start("*");

    // Custom Modifiers
    map("CAPSLOCK", "MD_00");

    // Custom Locks
    map("SCROLLLOCK", "LK_00");

    // Base Layer Mappings
    map("Q", "VK_W");
    map("W", "VK_E");

    // Layer 1 (active when MD_00 is held)
    when("MD_00", [
        map("Q", "VK_1"),
        map("W", "VK_2")
    ]);

device_end();
      `.trim();

      const result = parseRhaiCode(rhai);

      expect(result.errors).toHaveLength(0);
      expect(result.warnings).toHaveLength(0);

      expect(result.config.modifiers).toHaveLength(1);
      expect(result.config.modifiers[0].triggerKey).toBe('KEY_CAPSLOCK');

      expect(result.config.locks).toHaveLength(1);
      expect(result.config.locks[0].triggerKey).toBe('KEY_SCROLLLOCK');

      expect(result.config.layers).toHaveLength(2);

      const baseLayer = result.config.layers.find(l => l.isBase);
      expect(baseLayer).toBeDefined();
      expect(baseLayer!.mappings).toHaveLength(2);

      const conditionalLayer = result.config.layers.find(l => !l.isBase);
      expect(conditionalLayer).toBeDefined();
      expect(conditionalLayer!.mappings).toHaveLength(2);
    });

    it('should handle comments correctly', () => {
      const rhai = `
device_start("*");
    // This is a comment
    map("A", "VK_B"); // Inline comment
    // Another comment
device_end();
      `.trim();

      const result = parseRhaiCode(rhai);

      expect(result.errors).toHaveLength(0);
      expect(result.config.layers[0].mappings).toHaveLength(1);
    });

    it('should handle empty lines', () => {
      const rhai = `
device_start("*");

    map("A", "VK_B");

    map("C", "VK_D");

device_end();
      `.trim();

      const result = parseRhaiCode(rhai);

      expect(result.errors).toHaveLength(0);
      expect(result.config.layers[0].mappings).toHaveLength(2);
    });

    it('should normalize key codes with KEY_ prefix', () => {
      const rhai = `
device_start("*");
    map("A", "VK_B");
    map("KEY_C", "KEY_D");
device_end();
      `.trim();

      const result = parseRhaiCode(rhai);

      expect(result.errors).toHaveLength(0);
      const mappings = result.config.layers[0].mappings;

      expect(mappings[0].sourceKey).toBe('KEY_A');
      expect(mappings[0].targetKey).toBe('KEY_B');

      expect(mappings[1].sourceKey).toBe('KEY_C');
      expect(mappings[1].targetKey).toBe('KEY_D');
    });

    it('should warn on unsupported features', () => {
      const rhai = `
device_start("*");
    map("A", "VK_B");
    tap_hold("C", "VK_D", "MD_00", 200);
    macro("E", ["VK_H", "VK_I"]);
device_end();
      `.trim();

      const result = parseRhaiCode(rhai);

      expect(result.warnings.length).toBeGreaterThan(0);
      expect(result.warnings.some(w => w.includes('tap_hold'))).toBe(true);
      expect(result.warnings.some(w => w.includes('macro'))).toBe(true);
    });

    it('should handle parse errors gracefully', () => {
      const rhai = 'invalid syntax here';

      const result = parseRhaiCode(rhai);

      expect(result.errors).toHaveLength(0); // No errors for malformed input
      expect(result.config.layers).toHaveLength(1);
      expect(result.config.layers[0].isBase).toBe(true);
    });

    it('should handle map statements with semicolons', () => {
      const rhai = `
device_start("*");
    map("A", "VK_B");
    map("C", "VK_D");
device_end();
      `.trim();

      const result = parseRhaiCode(rhai);

      expect(result.errors).toHaveLength(0);
      expect(result.config.layers[0].mappings).toHaveLength(2);
    });

    it('should handle map statements with commas in when blocks', () => {
      const rhai = `
device_start("*");
    when("MD_00", [
        map("A", "VK_B"),
        map("C", "VK_D")
    ]);
device_end();
      `.trim();

      const result = parseRhaiCode(rhai);

      expect(result.errors).toHaveLength(0);
      const conditionalLayer = result.config.layers.find(l => !l.isBase);
      expect(conditionalLayer).toBeDefined();
      expect(conditionalLayer!.mappings).toHaveLength(2);
    });
  });
});
