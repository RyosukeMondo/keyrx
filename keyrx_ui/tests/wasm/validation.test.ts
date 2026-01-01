/**
 * WASM Validation Integration Tests
 *
 * Tests the validate_config WASM function to ensure:
 * - Validation detects syntax errors with correct line/column
 * - Valid configurations return empty error array
 * - Validation results are deterministic (same input = same output)
 * - Size limits are enforced
 *
 * @requirements REQ-5 (AC2, AC3, AC4, AC5, AC6)
 */

import { describe, it, expect, beforeAll } from 'vitest';

describe('WASM Validation Integration Tests', () => {
  let wasm: typeof import('../../src/wasm/pkg/keyrx_core');

  beforeAll(async () => {
    // Import and initialize WASM module
    wasm = await import('../../src/wasm/pkg/keyrx_core');
    wasm.wasm_init();
  });

  describe('Valid Configurations', () => {
    it('should return empty array for valid simple config', () => {
      const validConfig = `
        device("*") {
          map("A", "B");
        }
      `;

      const errors = wasm.validate_config(validConfig);
      expect(errors).toEqual([]);
    });

    it('should return empty array for valid multi-device config', () => {
      const validConfig = `
        device("keyboard-1") {
          map("A", "B");
        }
        device("keyboard-2") {
          map("VK_A", "VK_B");
        }
      `;

      const errors = wasm.validate_config(validConfig);
      expect(errors).toEqual([]);
    });

    it('should return empty array for config with multiple mappings', () => {
      const validConfig = `
        device("*") {
          map("A", "B");
          map("B", "A");
          map("VK_A", "VK_B");
        }
      `;

      const errors = wasm.validate_config(validConfig);
      expect(errors).toEqual([]);
    });
  });

  describe('Invalid Configurations', () => {
    it('should detect missing closing brace with correct line/column', () => {
      const invalidConfig = `
        device("*") {
          map("A", "B");
      `;

      const errors = wasm.validate_config(invalidConfig);
      expect(errors).not.toEqual([]);
      expect(errors.length).toBeGreaterThan(0);
      expect(errors[0]).toHaveProperty('line');
      expect(errors[0]).toHaveProperty('column');
      expect(errors[0]).toHaveProperty('message');
      expect(errors[0].line).toBeGreaterThan(0);
      expect(errors[0].column).toBeGreaterThan(0);
    });

    it('should detect invalid function call', () => {
      const invalidConfig = `
        device("*") {
          invalid_function("A", "B");
        }
      `;

      const errors = wasm.validate_config(invalidConfig);
      expect(errors).not.toEqual([]);
      expect(errors.length).toBeGreaterThan(0);
      expect(errors[0].message).toBeTruthy();
    });

    it('should detect syntax error in device pattern', () => {
      const invalidConfig = `
        device(*) {
          map("A", "B");
        }
      `;

      const errors = wasm.validate_config(invalidConfig);
      expect(errors).not.toEqual([]);
      expect(errors.length).toBeGreaterThan(0);
    });

    it('should detect missing semicolon', () => {
      const invalidConfig = `
        device("*") {
          map("A", "B")
          map("B", "A");
        }
      `;

      const errors = wasm.validate_config(invalidConfig);
      expect(errors).not.toEqual([]);
      expect(errors.length).toBeGreaterThan(0);
    });
  });

  describe('Size Limits', () => {
    it('should enforce 1MB size limit', () => {
      // Create config larger than 1MB
      const largeConfig = 'x'.repeat(2 * 1024 * 1024); // 2MB

      const errors = wasm.validate_config(largeConfig);
      expect(errors).not.toEqual([]);
      expect(errors.length).toBeGreaterThan(0);
      expect(errors[0].message).toContain('too large');
    });

    it('should accept config under 1MB', () => {
      // Create config under 1MB with repeated valid mappings
      const lines = [];
      lines.push('device("*") {');
      for (let i = 0; i < 100; i++) {
        lines.push('  map("A", "B");');
      }
      lines.push('}');
      const config = lines.join('\n');

      expect(config.length).toBeLessThan(1024 * 1024);
      const errors = wasm.validate_config(config);
      expect(errors).toEqual([]);
    });
  });

  describe('Determinism', () => {
    it('should return identical results for same valid config', () => {
      const config = `
        device("*") {
          map("A", "B");
          map("B", "A");
        }
      `;

      const errors1 = wasm.validate_config(config);
      const errors2 = wasm.validate_config(config);
      const errors3 = wasm.validate_config(config);

      expect(errors1).toEqual(errors2);
      expect(errors2).toEqual(errors3);
      expect(errors1).toEqual([]);
    });

    it('should return identical errors for same invalid config', () => {
      const config = `
        device("*") {
          map("A", "B")
      `;

      const errors1 = wasm.validate_config(config);
      const errors2 = wasm.validate_config(config);
      const errors3 = wasm.validate_config(config);

      // Results should be identical
      expect(errors1.length).toBe(errors2.length);
      expect(errors2.length).toBe(errors3.length);

      if (errors1.length > 0) {
        expect(errors1[0].line).toBe(errors2[0].line);
        expect(errors1[0].column).toBe(errors2[0].column);
        expect(errors1[0].message).toBe(errors2[0].message);

        expect(errors2[0].line).toBe(errors3[0].line);
        expect(errors2[0].column).toBe(errors3[0].column);
        expect(errors2[0].message).toBe(errors3[0].message);
      }
    });

    it('should be deterministic across multiple different configs', () => {
      const configs = [
        'device("*") { map("A", "B"); }',
        'device("kbd") { map("VK_A", "VK_B"); }',
        'device("*") {\n  map("A", "B");\n  map("B", "A");\n}',
      ];

      configs.forEach((config) => {
        const result1 = wasm.validate_config(config);
        const result2 = wasm.validate_config(config);
        expect(result1).toEqual(result2);
      });
    });
  });

  describe('Error Format', () => {
    it('should return errors with all required fields', () => {
      const invalidConfig = `
        device("*") {
          map("A", "B")
      `;

      const errors = wasm.validate_config(invalidConfig);
      expect(errors).not.toEqual([]);

      errors.forEach((error: any) => {
        expect(error).toHaveProperty('line');
        expect(error).toHaveProperty('column');
        expect(error).toHaveProperty('length');
        expect(error).toHaveProperty('message');
        expect(typeof error.line).toBe('number');
        expect(typeof error.column).toBe('number');
        expect(typeof error.length).toBe('number');
        expect(typeof error.message).toBe('string');
      });
    });

    it('should return line and column >= 1', () => {
      const invalidConfig = `
        device("*") {
          map("A", "B")
      `;

      const errors = wasm.validate_config(invalidConfig);
      expect(errors).not.toEqual([]);

      errors.forEach((error: any) => {
        expect(error.line).toBeGreaterThanOrEqual(1);
        expect(error.column).toBeGreaterThanOrEqual(1);
      });
    });
  });

  describe('Edge Cases', () => {
    it('should handle empty string', () => {
      const errors = wasm.validate_config('');
      // Empty config may be valid or invalid depending on parser
      // Just verify it doesn't crash
      expect(Array.isArray(errors)).toBe(true);
    });

    it('should handle whitespace-only config', () => {
      const errors = wasm.validate_config('   \n\n\t  ');
      expect(Array.isArray(errors)).toBe(true);
    });

    it('should handle config with unicode characters', () => {
      const config = `
        device("🎹-keyboard") {
          map("A", "B");
        }
      `;
      const errors = wasm.validate_config(config);
      expect(Array.isArray(errors)).toBe(true);
    });

    it('should handle config with special characters in strings', () => {
      const config = `
        device("key\\nboard") {
          map("A", "B");
        }
      `;
      const errors = wasm.validate_config(config);
      expect(Array.isArray(errors)).toBe(true);
    });

    it('should handle deeply nested structures', () => {
      const config = `
        device("*") {
          map("A", "B");
        }
        device("kbd-1") {
          map("B", "A");
        }
        device("kbd-2") {
          map("VK_A", "VK_B");
        }
      `;
      const errors = wasm.validate_config(config);
      expect(errors).toEqual([]);
    });
  });
});
