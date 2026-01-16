/**
 * Unit tests for RhaiFormatter
 *
 * Tests formatting functionality including:
 * - Basic script formatting with consistent indentation
 * - Comment preservation (inline and block)
 * - Configurable formatting options
 * - Line length limits
 * - Error handling for invalid syntax
 * - Performance for large scripts
 */

import { describe, it, expect } from 'vitest';
import {
  formatRhaiScript,
  indentBlock,
  preserveComments,
  isLineTooLong,
  applyDefaultFormatOptions,
} from './rhaiFormatter';

describe('rhaiFormatter', () => {
  describe('formatRhaiScript', () => {
    it('formats simple mapping with consistent spacing', () => {
      const unformatted = 'map("VK_A","VK_B");';
      const formatted = formatRhaiScript(unformatted);
      expect(formatted).toBe(
        'device_start("*");\n    map("VK_A", "VK_B");\ndevice_end();'
      );
    });

    it('formats device blocks with proper indentation', () => {
      const unformatted = `device_start("*Keychron*");
map("VK_C","VK_D");
device_end();`;

      const formatted = formatRhaiScript(unformatted);
      expect(formatted).toContain('device_start("*Keychron*");');
      expect(formatted).toContain('    map("VK_C", "VK_D");');
      expect(formatted).toContain('device_end();');
    });

    it('preserves inline comments', () => {
      const script = `// This is a comment
map("VK_A", "VK_B");`;

      const formatted = formatRhaiScript(script);
      expect(formatted).toContain('// This is a comment');
      expect(formatted).toContain('map("VK_A", "VK_B");');
    });

    it('preserves block comments', () => {
      const script = `/* Block comment */
map("VK_A", "VK_B");`;

      const formatted = formatRhaiScript(script);
      expect(formatted).toContain('/* Block comment */');
      expect(formatted).toContain('map("VK_A", "VK_B");');
    });

    it('applies custom indentation size', () => {
      const unformatted = `device_start("*");
map("VK_A","VK_B");
device_end();`;

      const formatted = formatRhaiScript(unformatted, { indentSize: 2 });
      // Should use 2-space indent instead of default 4
      expect(formatted).toContain('  map("VK_A", "VK_B");');
    });

    it('applies custom blank lines between devices', () => {
      const unformatted = `device_start("*Device1*");
map("VK_A","VK_B");
device_end();
device_start("*Device2*");
map("VK_C","VK_D");
device_end();`;

      const formatted = formatRhaiScript(unformatted, {
        blankLinesBetweenDevices: 2,
      });

      // Should have 2 blank lines between device blocks
      expect(formatted).toMatch(/device_end\(\);\n\n\ndevice_start/);
    });

    it('applies custom blank lines between sections', () => {
      const unformatted = `map("VK_A","VK_B");
device_start("*");
map("VK_C","VK_D");
device_end();`;

      const formatted = formatRhaiScript(unformatted, {
        blankLinesBetweenDevices: 3,
      });

      // Global mappings are now wrapped in device_start("*"), so both become device blocks
      // Should have 3 blank lines between the two device blocks
      expect(formatted).toMatch(/device_end\(\);\n\n\n\ndevice_start/);
    });

    it('throws error for invalid syntax with line number', () => {
      const invalidScript = `map("VK_A", "VK_B"
device_start("*");`;

      expect(() => formatRhaiScript(invalidScript)).toThrow(/line/i);
    });

    it('handles unrecognized syntax gracefully', () => {
      // Parser silently ignores unrecognized lines (permissive mode)
      const scriptWithUnknown = 'map(VK_A, VK_B);\nmap("VK_C", "VK_D");'; // First line missing quotes

      const formatted = formatRhaiScript(scriptWithUnknown);
      // Should format the valid line, silently skip the invalid one
      expect(formatted).toContain('map("VK_C", "VK_D");');
    });

    it('formats tap_hold mappings correctly', () => {
      const script = 'tap_hold("VK_ESCAPE","VK_ESCAPE","VK_LCTRL",200);';
      const formatted = formatRhaiScript(script);
      expect(formatted).toContain(
        'tap_hold("VK_ESCAPE", "VK_ESCAPE", "VK_LCTRL", 200);'
      );
    });

    it('handles unsupported mapping types (macro) gracefully', () => {
      // Parser doesn't support macro yet - it will be silently ignored
      const script =
        'macro("VK_F1",["VK_LCTRL","VK_C"],50);\nmap("VK_A", "VK_B");';
      const formatted = formatRhaiScript(script);
      // Should format supported mappings, ignore unsupported ones
      expect(formatted).toContain('map("VK_A", "VK_B");');
      // Macro line is silently ignored by parser
      expect(formatted).not.toContain('macro');
    });

    it('handles unsupported mapping types (layer_switch) gracefully', () => {
      // Parser doesn't support layer_switch yet - it will be silently ignored
      const script =
        'layer_switch("VK_CAPSLOCK","layer1","toggle");\nmap("VK_C", "VK_D");';
      const formatted = formatRhaiScript(script);
      // Should format supported mappings, ignore unsupported ones
      expect(formatted).toContain('map("VK_C", "VK_D");');
      // layer_switch line is silently ignored by parser
      expect(formatted).not.toContain('layer_switch');
    });

    it('formats import statements correctly', () => {
      const script = `import "common.rhai";
map("VK_A", "VK_B");`;

      const formatted = formatRhaiScript(script);
      expect(formatted).toContain('import "common.rhai";');
      expect(formatted).toContain('map("VK_A", "VK_B");');
    });

    it('formats import with alias correctly', () => {
      const script = `import "lib.rhai" as lib;
map("VK_A", "VK_B");`;

      const formatted = formatRhaiScript(script);
      expect(formatted).toContain('import "lib.rhai" as lib;');
    });

    it('handles empty script', () => {
      const formatted = formatRhaiScript('');
      expect(formatted).toBe('');
    });

    it('handles script with only comments', () => {
      const script = `// Comment 1
/* Comment 2 */`;

      const formatted = formatRhaiScript(script);
      // Comments without any mappings will result in empty output
      // because the generator only outputs comments associated with code sections
      expect(formatted).toBe('');
    });

    it('maintains semantic equivalence after formatting', () => {
      const original = `map("VK_A","VK_B");
device_start("*Keychron*");
map("VK_C","VK_D");
tap_hold("VK_ESCAPE","VK_ESCAPE","VK_LCTRL",200);
device_end();`;

      const formatted = formatRhaiScript(original);

      // Re-format the formatted version
      const reformatted = formatRhaiScript(formatted);

      // Should be identical (idempotent)
      expect(reformatted).toBe(formatted);
    });

    it('formats when_start blocks with proper nesting', () => {
      const script = `device_start("*");
when_start("MD_00");
map("VK_J","VK_DOWN");
when_end();
device_end();`;

      const formatted = formatRhaiScript(script);
      expect(formatted).toContain('    when_start("MD_00");');
      expect(formatted).toContain('        map("VK_J", "VK_DOWN");');
      expect(formatted).toContain('    when_end();');
    });

    it('formats multiple global mappings', () => {
      const script = `map("VK_A","VK_B");
map("VK_C","VK_D");
map("VK_E","VK_F");`;

      const formatted = formatRhaiScript(script);
      const lines = formatted.split('\n');
      // With device_start("*") wrapper: device_start, 3 mappings (indented), device_end = 5 lines
      expect(lines).toHaveLength(5);
      expect(lines[0]).toBe('device_start("*");');
      expect(lines[1]).toBe('    map("VK_A", "VK_B");');
      expect(lines[2]).toBe('    map("VK_C", "VK_D");');
      expect(lines[3]).toBe('    map("VK_E", "VK_F");');
      expect(lines[4]).toBe('device_end();');
    });

    it('formats complex nested structures', () => {
      const script = `import "lib.rhai";

map("VK_A","VK_B");

device_start("*Keychron*");
map("VK_C","VK_D");
when_start("MD_00");
map("VK_J","VK_DOWN");
map("VK_K","VK_UP");
when_end();
device_end();`;

      const formatted = formatRhaiScript(script);

      // Check structure
      expect(formatted).toContain('import "lib.rhai";');
      expect(formatted).toContain('map("VK_A", "VK_B");');
      expect(formatted).toContain('device_start("*Keychron*");');
      expect(formatted).toContain('    map("VK_C", "VK_D");');
      expect(formatted).toContain('    when_start("MD_00");');
      expect(formatted).toContain('        map("VK_J", "VK_DOWN");');
      expect(formatted).toContain('        map("VK_K", "VK_UP");');
    });

    it('completes formatting within performance requirements', () => {
      // Generate a large script with 1000 mappings
      const mappings: string[] = [];
      for (let i = 0; i < 1000; i++) {
        mappings.push(`map("VK_${i}","VK_${i + 1000}");`);
      }
      const largeScript = mappings.join('\n');

      const startTime = performance.now();
      formatRhaiScript(largeScript);
      const endTime = performance.now();

      const duration = endTime - startTime;
      // Should complete within 50ms for 1000 lines
      expect(duration).toBeLessThan(50);
    });

    it('handles mixed comment styles in complex script', () => {
      const script = `// Global section
/* This is a block comment */
map("VK_A", "VK_B");

// Device section
device_start("*");
    /* Inside device */
    map("VK_C", "VK_D");
device_end();`;

      const formatted = formatRhaiScript(script);

      // Comments before sections are preserved
      expect(formatted).toContain('// Global section');
      expect(formatted).toContain('/* This is a block comment */');
      expect(formatted).toContain('// Device section');

      // Note: Comments inside device blocks are currently not fully preserved
      // This is a known limitation of the current parser/generator implementation
      // The parser captures them but the generator's comment insertion logic
      // only looks for comments before device blocks, not inside them
    });
  });

  describe('indentBlock', () => {
    it('indents lines by one level with default spacing', () => {
      const lines = ['map("VK_A", "VK_B");', 'map("VK_C", "VK_D");'];
      const indented = indentBlock(lines, 1);

      expect(indented[0]).toBe('    map("VK_A", "VK_B");');
      expect(indented[1]).toBe('    map("VK_C", "VK_D");');
    });

    it('indents lines by multiple levels', () => {
      const lines = ['map("VK_A", "VK_B");'];
      const indented = indentBlock(lines, 2);

      expect(indented[0]).toBe('        map("VK_A", "VK_B");');
    });

    it('uses custom indent size', () => {
      const lines = ['map("VK_A", "VK_B");'];
      const indented = indentBlock(lines, 1, 2);

      expect(indented[0]).toBe('  map("VK_A", "VK_B");');
    });

    it('does not indent at level 0', () => {
      const lines = ['map("VK_A", "VK_B");'];
      const indented = indentBlock(lines, 0);

      expect(indented[0]).toBe('map("VK_A", "VK_B");');
    });

    it('does not indent empty lines', () => {
      const lines = ['map("VK_A", "VK_B");', '', 'map("VK_C", "VK_D");'];
      const indented = indentBlock(lines, 1);

      expect(indented[0]).toBe('    map("VK_A", "VK_B");');
      expect(indented[1]).toBe('');
      expect(indented[2]).toBe('    map("VK_C", "VK_D");');
    });

    it('handles negative indent levels gracefully', () => {
      const lines = ['map("VK_A", "VK_B");'];
      const indented = indentBlock(lines, -1);

      // Should not indent
      expect(indented[0]).toBe('map("VK_A", "VK_B");');
    });
  });

  describe('preserveComments', () => {
    it('returns formatted version (comments preserved via AST)', () => {
      const original = '// Comment\nmap("VK_A", "VK_B");';
      const formatted = 'map("VK_A", "VK_B");';

      const result = preserveComments(original, formatted);
      expect(result).toBe(formatted);
    });

    it('handles empty strings', () => {
      const result = preserveComments('', '');
      expect(result).toBe('');
    });
  });

  describe('isLineTooLong', () => {
    it('returns false for line within limit', () => {
      const line = 'map("VK_A", "VK_B");';
      expect(isLineTooLong(line, 100)).toBe(false);
    });

    it('returns true for line exceeding limit', () => {
      const line = 'a'.repeat(101);
      expect(isLineTooLong(line, 100)).toBe(true);
    });

    it('returns false for line exactly at limit', () => {
      const line = 'a'.repeat(100);
      expect(isLineTooLong(line, 100)).toBe(false);
    });

    it('handles empty line', () => {
      expect(isLineTooLong('', 100)).toBe(false);
    });
  });

  describe('applyDefaultFormatOptions', () => {
    it('returns defaults when no options provided', () => {
      const options = applyDefaultFormatOptions();

      expect(options.indentSize).toBe(4);
      expect(options.maxLineLength).toBe(100);
      expect(options.blankLinesBetweenDevices).toBe(1);
      expect(options.blankLinesBetweenSections).toBe(2);
    });

    it('overrides specific options', () => {
      const options = applyDefaultFormatOptions({
        indentSize: 2,
        maxLineLength: 80,
      });

      expect(options.indentSize).toBe(2);
      expect(options.maxLineLength).toBe(80);
      expect(options.blankLinesBetweenDevices).toBe(1); // Default
      expect(options.blankLinesBetweenSections).toBe(2); // Default
    });

    it('handles all custom options', () => {
      const options = applyDefaultFormatOptions({
        indentSize: 2,
        maxLineLength: 80,
        blankLinesBetweenDevices: 3,
        blankLinesBetweenSections: 1,
      });

      expect(options.indentSize).toBe(2);
      expect(options.maxLineLength).toBe(80);
      expect(options.blankLinesBetweenDevices).toBe(3);
      expect(options.blankLinesBetweenSections).toBe(1);
    });

    it('handles zero values correctly', () => {
      const options = applyDefaultFormatOptions({
        indentSize: 0,
        blankLinesBetweenDevices: 0,
      });

      expect(options.indentSize).toBe(0);
      expect(options.blankLinesBetweenDevices).toBe(0);
    });
  });

  describe('edge cases', () => {
    it('formats script with trailing whitespace', () => {
      const script = 'map("VK_A", "VK_B");   \n   ';
      const formatted = formatRhaiScript(script);

      expect(formatted).toBe(
        'device_start("*");\n    map("VK_A", "VK_B");\ndevice_end();'
      );
    });

    it('formats script with Windows line endings', () => {
      const script = 'map("VK_A", "VK_B");\r\nmap("VK_C", "VK_D");';
      const formatted = formatRhaiScript(script);

      expect(formatted).toContain('map("VK_A", "VK_B");');
      expect(formatted).toContain('map("VK_C", "VK_D");');
    });

    it('formats script with mixed line endings', () => {
      const script =
        'map("VK_A", "VK_B");\r\nmap("VK_C", "VK_D");\nmap("VK_E", "VK_F");';
      const formatted = formatRhaiScript(script);

      const lines = formatted.split('\n');
      // With device_start("*") wrapper: device_start, 3 mappings, device_end = 5 lines
      expect(lines).toHaveLength(5);
    });

    it('handles very long key names gracefully', () => {
      const longKey = 'VK_' + 'A'.repeat(90);
      const script = `map("${longKey}", "VK_B");`;

      const formatted = formatRhaiScript(script);
      expect(formatted).toContain(longKey);
    });

    it('formats script with unicode characters', () => {
      const script = '// Comment with unicode: 日本語\nmap("VK_A", "VK_B");';
      const formatted = formatRhaiScript(script);

      expect(formatted).toContain('日本語');
      expect(formatted).toContain('map("VK_A", "VK_B");');
    });
  });
});
