import { describe, it, expect } from 'vitest';
import {
  parseRhaiScript,
  extractDevicePatterns,
  hasGlobalMappings,
  getMappingsForDevice,
  validateAST,
  type RhaiAST,
  type KeyMapping,
} from './rhaiParser';

describe('rhaiParser', () => {
  describe('parseRhaiScript', () => {
    describe('simple mappings', () => {
      it('should parse simple map() statement', () => {
        const script = `
          device_start("*");
            map("VK_A", "VK_B");
          device_end();
        `;

        const result = parseRhaiScript(script);

        expect(result.success).toBe(true);
        expect(result.ast).toBeDefined();
        expect(result.ast!.deviceBlocks).toHaveLength(1);
        expect(result.ast!.deviceBlocks[0].mappings).toHaveLength(1);

        const mapping = result.ast!.deviceBlocks[0].mappings[0];
        expect(mapping.type).toBe('simple');
        expect(mapping.sourceKey).toBe('VK_A');
        expect(mapping.targetKey).toBe('VK_B');
      });

      it('should parse multiple simple mappings', () => {
        const script = `
          device_start("*");
            map("VK_A", "VK_B");
            map("VK_Q", "VK_W");
            map("VK_Num1", "VK_Num2");
          device_end();
        `;

        const result = parseRhaiScript(script);

        expect(result.success).toBe(true);
        expect(result.ast!.deviceBlocks[0].mappings).toHaveLength(3);
      });

      it('should parse mappings with modifier helpers', () => {
        const script = `
          device_start("*");
            map("VK_C", with_ctrl("VK_C"));
            map("VK_V", with_shift("VK_V"));
          device_end();
        `;

        const result = parseRhaiScript(script);

        expect(result.success).toBe(true);
        expect(result.ast!.deviceBlocks[0].mappings).toHaveLength(2);
        expect(result.ast!.deviceBlocks[0].mappings[0].targetKey).toBe(
          'with_ctrl("VK_C")'
        );
      });
    });

    describe('tap_hold mappings', () => {
      it('should parse tap_hold() statement', () => {
        const script = `
          device_start("*");
            tap_hold("VK_CapsLock", "VK_Escape", "MD_01", 200);
          device_end();
        `;

        const result = parseRhaiScript(script);

        expect(result.success).toBe(true);
        expect(result.ast!.deviceBlocks[0].mappings).toHaveLength(1);

        const mapping = result.ast!.deviceBlocks[0].mappings[0];
        expect(mapping.type).toBe('tap_hold');
        expect(mapping.sourceKey).toBe('VK_CapsLock');
        expect(mapping.tapHold).toBeDefined();
        expect(mapping.tapHold!.tapAction).toBe('VK_Escape');
        expect(mapping.tapHold!.holdAction).toBe('MD_01');
        expect(mapping.tapHold!.thresholdMs).toBe(200);
      });

      it('should parse multiple tap_hold mappings', () => {
        const script = `
          device_start("*");
            tap_hold("VK_CapsLock", "VK_Escape", "MD_01", 200);
            tap_hold("VK_Space", "VK_Space", "MD_00", 200);
            tap_hold("VK_Enter", "VK_Enter", "MD_02", 150);
          device_end();
        `;

        const result = parseRhaiScript(script);

        expect(result.success).toBe(true);
        expect(result.ast!.deviceBlocks[0].mappings).toHaveLength(3);
        expect(
          result.ast!.deviceBlocks[0].mappings[2].tapHold!.thresholdMs
        ).toBe(150);
      });
    });

    describe('device blocks', () => {
      it('should parse device block with wildcard pattern', () => {
        const script = `
          device_start("*");
            map("VK_A", "VK_B");
          device_end();
        `;

        const result = parseRhaiScript(script);

        expect(result.success).toBe(true);
        expect(result.ast!.deviceBlocks).toHaveLength(1);
        expect(result.ast!.deviceBlocks[0].pattern).toBe('*');
      });

      it('should parse device block with specific pattern', () => {
        const script = `
          device_start("*Keychron*");
            map("VK_A", "VK_B");
          device_end();
        `;

        const result = parseRhaiScript(script);

        expect(result.success).toBe(true);
        expect(result.ast!.deviceBlocks[0].pattern).toBe('*Keychron*');
      });

      it('should parse multiple device blocks', () => {
        const script = `
          device_start("*Laptop*");
            map("VK_A", "VK_B");
          device_end();

          device_start("*Keychron*");
            map("VK_C", "VK_D");
          device_end();

          device_start("*");
            map("VK_E", "VK_F");
          device_end();
        `;

        const result = parseRhaiScript(script);

        expect(result.success).toBe(true);
        expect(result.ast!.deviceBlocks).toHaveLength(3);
        expect(result.ast!.deviceBlocks[0].pattern).toBe('*Laptop*');
        expect(result.ast!.deviceBlocks[1].pattern).toBe('*Keychron*');
        expect(result.ast!.deviceBlocks[2].pattern).toBe('*');
      });

      it('should track device block line numbers', () => {
        const script = `device_start("*");
  map("VK_A", "VK_B");
device_end();`;

        const result = parseRhaiScript(script);

        expect(result.success).toBe(true);
        expect(result.ast!.deviceBlocks[0].startLine).toBe(1);
        expect(result.ast!.deviceBlocks[0].endLine).toBe(3);
      });
    });

    describe('modifier layers (when blocks)', () => {
      it('should parse when_start with single modifier', () => {
        const script = `
          device_start("*");
            when_start("MD_01");
              map("VK_H", "VK_Left");
              map("VK_J", "VK_Down");
            when_end();
          device_end();
        `;

        const result = parseRhaiScript(script);

        expect(result.success).toBe(true);
        expect(result.ast!.deviceBlocks[0].layers).toHaveLength(1);

        const layer = result.ast!.deviceBlocks[0].layers[0];
        expect(layer.modifiers).toBe('MD_01');
        expect(layer.mappings).toHaveLength(2);
      });

      it('should parse when_start with multiple modifiers', () => {
        const script = `
          device_start("*");
            when_start(["MD_00", "MD_01"]);
              map("VK_W", with_ctrl("VK_Right"));
            when_end();
          device_end();
        `;

        const result = parseRhaiScript(script);

        expect(result.success).toBe(true);
        expect(result.ast!.deviceBlocks[0].layers).toHaveLength(1);

        const layer = result.ast!.deviceBlocks[0].layers[0];
        expect(Array.isArray(layer.modifiers)).toBe(true);
        expect(layer.modifiers).toEqual(['MD_00', 'MD_01']);
      });

      it('should parse multiple when blocks', () => {
        const script = `
          device_start("*");
            when_start("MD_00");
              map("VK_H", "VK_Left");
            when_end();

            when_start("MD_01");
              map("VK_J", "VK_Down");
            when_end();
          device_end();
        `;

        const result = parseRhaiScript(script);

        expect(result.success).toBe(true);
        expect(result.ast!.deviceBlocks[0].layers).toHaveLength(2);
      });

      it('should track when block line numbers', () => {
        const script = `device_start("*");
  when_start("MD_00");
    map("VK_H", "VK_Left");
  when_end();
device_end();`;

        const result = parseRhaiScript(script);

        expect(result.success).toBe(true);
        const layer = result.ast!.deviceBlocks[0].layers[0];
        expect(layer.startLine).toBe(2);
        expect(layer.endLine).toBe(4);
      });
    });

    describe('global mappings', () => {
      it('should parse global mappings outside device blocks', () => {
        const script = `
          map("VK_A", "VK_B");
          map("VK_C", "VK_D");

          device_start("*");
            map("VK_E", "VK_F");
          device_end();
        `;

        const result = parseRhaiScript(script);

        expect(result.success).toBe(true);
        expect(result.ast!.globalMappings).toHaveLength(2);
        expect(result.ast!.deviceBlocks[0].mappings).toHaveLength(1);
      });
    });

    describe('import statements', () => {
      it('should parse import without alias', () => {
        const script = `
          import "stdlib/ctrl.rhai";

          device_start("*");
            map("VK_A", "VK_B");
          device_end();
        `;

        const result = parseRhaiScript(script);

        expect(result.success).toBe(true);
        expect(result.ast!.imports).toHaveLength(1);
        expect(result.ast!.imports[0].path).toBe('stdlib/ctrl.rhai');
        expect(result.ast!.imports[0].alias).toBeUndefined();
      });

      it('should parse import with alias', () => {
        const script = `
          import "utils/helpers.rhai" as helpers;

          device_start("*");
            map("VK_A", "VK_B");
          device_end();
        `;

        const result = parseRhaiScript(script);

        expect(result.success).toBe(true);
        expect(result.ast!.imports[0].alias).toBe('helpers');
      });
    });

    describe('comments', () => {
      it('should preserve line comments', () => {
        const script = `
          // This is a comment
          device_start("*");
            map("VK_A", "VK_B"); // Inline comment
          device_end();
        `;

        const result = parseRhaiScript(script);

        expect(result.success).toBe(true);
        expect(result.ast!.comments.length).toBeGreaterThan(0);
        expect(result.ast!.comments[0].type).toBe('line');
      });

      it('should preserve block comments', () => {
        const script = `
          /* Block comment */
          device_start("*");
            map("VK_A", "VK_B");
          device_end();
        `;

        const result = parseRhaiScript(script);

        expect(result.success).toBe(true);
        expect(result.ast!.comments.length).toBeGreaterThan(0);
        expect(result.ast!.comments[0].type).toBe('block');
      });

      it('should track comment line numbers', () => {
        const script = `// Line 1
device_start("*");
  // Line 3
  map("VK_A", "VK_B");
device_end();`;

        const result = parseRhaiScript(script);

        expect(result.success).toBe(true);
        expect(result.ast!.comments[0].line).toBe(1);
        expect(result.ast!.comments[1].line).toBe(3);
      });
    });

    describe('error handling', () => {
      it('should detect nested device blocks', () => {
        const script = `
          device_start("*");
            device_start("*Keychron*");
            device_end();
          device_end();
        `;

        const result = parseRhaiScript(script);

        expect(result.success).toBe(false);
        expect(result.error).toBeDefined();
        expect(result.error!.message).toContain('Nested device blocks');
        expect(result.error!.suggestion).toBeDefined();
      });

      it('should detect device_end without device_start', () => {
        const script = `
          map("VK_A", "VK_B");
          device_end();
        `;

        const result = parseRhaiScript(script);

        expect(result.success).toBe(false);
        expect(result.error!.message).toContain(
          'device_end() without matching device_start()'
        );
      });

      it('should detect unclosed device block', () => {
        const script = `
          device_start("*");
            map("VK_A", "VK_B");
        `;

        const result = parseRhaiScript(script);

        expect(result.success).toBe(false);
        expect(result.error!.message).toContain('Unclosed device block');
        expect(result.error!.line).toBeGreaterThan(0);
      });

      it('should detect nested when blocks', () => {
        const script = `
          device_start("*");
            when_start("MD_00");
              when_start("MD_01");
              when_end();
            when_end();
          device_end();
        `;

        const result = parseRhaiScript(script);

        expect(result.success).toBe(false);
        expect(result.error!.message).toContain('Nested when blocks');
      });

      it('should detect when_end without when_start', () => {
        const script = `
          device_start("*");
            map("VK_A", "VK_B");
            when_end();
          device_end();
        `;

        const result = parseRhaiScript(script);

        expect(result.success).toBe(false);
        expect(result.error!.message).toContain(
          'when_end() without matching when_start()'
        );
      });

      it('should detect unclosed when block', () => {
        const script = `
          device_start("*");
            when_start("MD_00");
              map("VK_A", "VK_B");
          device_end();
        `;

        const result = parseRhaiScript(script);

        expect(result.success).toBe(false);
        expect(result.error!.message).toContain('Unclosed when block');
      });
    });

    describe('edge cases', () => {
      it('should handle empty script', () => {
        const script = '';

        const result = parseRhaiScript(script);

        expect(result.success).toBe(true);
        expect(result.ast!.deviceBlocks).toHaveLength(0);
        expect(result.ast!.globalMappings).toHaveLength(0);
      });

      it('should handle comments-only script', () => {
        const script = `
          // Just comments
          /* Nothing else */
        `;

        const result = parseRhaiScript(script);

        expect(result.success).toBe(true);
        expect(result.ast!.comments.length).toBeGreaterThan(0);
        expect(result.ast!.deviceBlocks).toHaveLength(0);
      });

      it('should handle large script (10000+ lines)', () => {
        // Generate a large script
        const lines = ['device_start("*");'];
        for (let i = 0; i < 10000; i++) {
          lines.push(`  map("VK_${i % 100}", "VK_${(i + 1) % 100}");`);
        }
        lines.push('device_end();');
        const script = lines.join('\n');

        const startTime = Date.now();
        const result = parseRhaiScript(script);
        const duration = Date.now() - startTime;

        expect(result.success).toBe(true);
        expect(duration).toBeLessThan(100); // Should complete within 100ms
        expect(result.ast!.deviceBlocks[0].mappings).toHaveLength(10000);
      });

      it('should handle malformed syntax gracefully', () => {
        const script = `
          device_start("*");
            map("VK_A" "VK_B");  // Missing comma
          device_end();
        `;

        const result = parseRhaiScript(script);

        // Parser is permissive - should skip unrecognized lines
        expect(result.success).toBe(true);
        expect(result.ast!.deviceBlocks[0].mappings).toHaveLength(0);
      });
    });
  });

  describe('extractDevicePatterns', () => {
    it('should extract all device patterns', () => {
      const script = `
        device_start("*Laptop*");
          map("VK_A", "VK_B");
        device_end();

        device_start("*Keychron*");
          map("VK_C", "VK_D");
        device_end();
      `;

      const result = parseRhaiScript(script);
      const patterns = extractDevicePatterns(result.ast!);

      expect(patterns).toEqual(['*Laptop*', '*Keychron*']);
    });
  });

  describe('hasGlobalMappings', () => {
    it('should return true when global mappings exist', () => {
      const script = `
        map("VK_A", "VK_B");

        device_start("*");
          map("VK_C", "VK_D");
        device_end();
      `;

      const result = parseRhaiScript(script);
      expect(hasGlobalMappings(result.ast!)).toBe(true);
    });

    it('should return false when no global mappings', () => {
      const script = `
        device_start("*");
          map("VK_A", "VK_B");
        device_end();
      `;

      const result = parseRhaiScript(script);
      expect(hasGlobalMappings(result.ast!)).toBe(false);
    });
  });

  describe('getMappingsForDevice', () => {
    it('should get mappings for specific device', () => {
      const script = `
        device_start("*Keychron*");
          map("VK_A", "VK_B");
          map("VK_C", "VK_D");
        device_end();

        device_start("*");
          map("VK_E", "VK_F");
        device_end();
      `;

      const result = parseRhaiScript(script);
      const mappings = getMappingsForDevice(result.ast!, '*Keychron*');

      expect(mappings).toHaveLength(2);
      expect(mappings![0].sourceKey).toBe('VK_A');
    });

    it('should include layer mappings', () => {
      const script = `
        device_start("*");
          map("VK_A", "VK_B");
          when_start("MD_00");
            map("VK_H", "VK_Left");
          when_end();
        device_end();
      `;

      const result = parseRhaiScript(script);
      const mappings = getMappingsForDevice(result.ast!, '*');

      expect(mappings).toHaveLength(2);
    });

    it('should return undefined for non-existent device', () => {
      const script = `
        device_start("*");
          map("VK_A", "VK_B");
        device_end();
      `;

      const result = parseRhaiScript(script);
      const mappings = getMappingsForDevice(result.ast!, '*Keychron*');

      expect(mappings).toBeUndefined();
    });
  });

  describe('validateAST', () => {
    it('should detect duplicate device patterns', () => {
      const script = `
        device_start("*");
          map("VK_A", "VK_B");
        device_end();

        device_start("*");
          map("VK_C", "VK_D");
        device_end();
      `;

      const result = parseRhaiScript(script);
      const validation = validateAST(result.ast!);

      expect(validation.valid).toBe(false);
      expect(validation.errors[0]).toContain('Duplicate device pattern');
    });

    it('should detect tap_hold threshold too low', () => {
      const script = `
        device_start("*");
          tap_hold("VK_Space", "VK_Space", "MD_00", 25);
        device_end();
      `;

      const result = parseRhaiScript(script);
      const validation = validateAST(result.ast!);

      expect(validation.valid).toBe(false);
      expect(validation.errors[0]).toContain('threshold too low');
      expect(validation.errors[0]).toContain('25ms');
    });

    it('should detect tap_hold threshold too high', () => {
      const script = `
        device_start("*");
          tap_hold("VK_Space", "VK_Space", "MD_00", 1500);
        device_end();
      `;

      const result = parseRhaiScript(script);
      const validation = validateAST(result.ast!);

      expect(validation.valid).toBe(false);
      expect(validation.errors[0]).toContain('threshold too high');
      expect(validation.errors[0]).toContain('1500ms');
    });

    it('should pass validation for valid AST', () => {
      const script = `
        device_start("*Laptop*");
          map("VK_A", "VK_B");
        device_end();

        device_start("*Keychron*");
          tap_hold("VK_Space", "VK_Space", "MD_00", 200);
        device_end();
      `;

      const result = parseRhaiScript(script);
      const validation = validateAST(result.ast!);

      expect(validation.valid).toBe(true);
      expect(validation.errors).toHaveLength(0);
    });
  });
});
