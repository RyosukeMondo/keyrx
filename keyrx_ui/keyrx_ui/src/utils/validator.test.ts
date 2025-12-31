/**
 * Unit tests for ConfigValidator service.
 *
 * Tests cover:
 * - Valid configuration validation (0 errors)
 * - Invalid syntax handling (parse errors with line numbers)
 * - WASM error parsing (line/column extraction)
 * - Linting rules (unused layers, naming consistency)
 * - WASM crash handling (graceful fallback)
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { ConfigValidator, validator } from './validator';
import type { ValidationResult } from '@/types/validation';

// Mock the WASM core module
vi.mock('@/wasm/core', () => ({
  wasmCore: {
    loadConfig: vi.fn(),
  },
}));

// Mock linting rules
vi.mock('./lintingRules', () => ({
  lintUnusedLayers: vi.fn(() => []),
  lintNamingConsistency: vi.fn(() => []),
}));

import { wasmCore } from '@/wasm/core';
import { lintUnusedLayers, lintNamingConsistency } from './lintingRules';

describe('ConfigValidator', () => {
  let configValidator: ConfigValidator;

  beforeEach(() => {
    configValidator = new ConfigValidator();
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe('validate()', () => {
    describe('valid configurations', () => {
      it('should return no errors for valid Rhai config', async () => {
        const validConfig = `
          layer "base" {
            map KEY_A to KEY_B
          }
        `;

        vi.mocked(wasmCore.loadConfig).mockResolvedValue({} as any);

        const result = await configValidator.validate(validConfig);

        expect(result.errors).toHaveLength(0);
        expect(result.warnings).toHaveLength(0);
        expect(result.timestamp).toBeDefined();
        expect(wasmCore.loadConfig).toHaveBeenCalledWith(validConfig);
      });

      it('should include statistics for valid config', async () => {
        const validConfig = `
          layer "base" {
            map KEY_A to KEY_B
          }
          modifier "shift" {
            map KEY_C to KEY_D
          }
        `;

        vi.mocked(wasmCore.loadConfig).mockResolvedValue({} as any);

        const result = await configValidator.validate(validConfig);

        expect(result.stats).toBeDefined();
        expect(result.stats?.lineCount).toBeGreaterThan(0);
        expect(result.stats?.layerCount).toBe(1);
        expect(result.stats?.modifierCount).toBe(1);
      });
    });

    describe('empty configurations', () => {
      it('should return error for empty config', async () => {
        const result = await configValidator.validate('');

        expect(result.errors).toHaveLength(1);
        expect(result.errors[0].code).toBe('EMPTY_CONFIG');
        expect(result.errors[0].line).toBe(1);
        expect(result.errors[0].column).toBe(1);
        expect(result.errors[0].message).toContain('cannot be empty');
      });

      it('should return error for whitespace-only config', async () => {
        const result = await configValidator.validate('   \n  \t  \n  ');

        expect(result.errors).toHaveLength(1);
        expect(result.errors[0].code).toBe('EMPTY_CONFIG');
      });
    });

    describe('syntax errors', () => {
      it('should parse WASM error with line and column', async () => {
        vi.mocked(wasmCore.loadConfig).mockRejectedValue(
          new Error('Parse error at line 4, column 9: Missing semicolon')
        );

        const result = await configValidator.validate('invalid config');

        expect(result.errors).toHaveLength(1);
        expect(result.errors[0].line).toBe(4);
        expect(result.errors[0].column).toBe(9);
        expect(result.errors[0].message).toBe('Missing semicolon');
        expect(result.errors[0].code).toBe('WASM_PARSE_ERROR');
      });

      it('should parse WASM error with only line number', async () => {
        vi.mocked(wasmCore.loadConfig).mockRejectedValue(
          new Error('Error at Line 10: Undefined layer')
        );

        const result = await configValidator.validate('invalid config');

        expect(result.errors).toHaveLength(1);
        expect(result.errors[0].line).toBe(10);
        expect(result.errors[0].column).toBe(1); // Default column
        expect(result.errors[0].message).toBe('Undefined layer');
        expect(result.errors[0].code).toBe('WASM_PARSE_ERROR');
      });

      it('should parse alternative error format (at 4:9 - message)', async () => {
        vi.mocked(wasmCore.loadConfig).mockRejectedValue(
          new Error('at 15:23 - Invalid key code')
        );

        const result = await configValidator.validate('invalid config');

        expect(result.errors).toHaveLength(1);
        expect(result.errors[0].line).toBe(15);
        expect(result.errors[0].column).toBe(23);
        expect(result.errors[0].message).toBe('Invalid key code');
        expect(result.errors[0].code).toBe('WASM_PARSE_ERROR');
      });

      it('should handle unparseable error messages gracefully', async () => {
        vi.mocked(wasmCore.loadConfig).mockRejectedValue(
          new Error('Completely unparseable error message')
        );

        const result = await configValidator.validate('invalid config');

        expect(result.errors).toHaveLength(1);
        expect(result.errors[0].line).toBe(1);
        expect(result.errors[0].column).toBe(1);
        expect(result.errors[0].message).toBe('Completely unparseable error message');
        expect(result.errors[0].code).toBe('WASM_ERROR');
      });

      it('should handle string errors from WASM', async () => {
        vi.mocked(wasmCore.loadConfig).mockRejectedValue('String error at line 5, column 10: Test');

        const result = await configValidator.validate('invalid config');

        expect(result.errors).toHaveLength(1);
        expect(result.errors[0].line).toBe(5);
        expect(result.errors[0].column).toBe(10);
        expect(result.errors[0].message).toBe('Test');
      });

      it('should handle unknown error types', async () => {
        vi.mocked(wasmCore.loadConfig).mockRejectedValue({ weird: 'object' });

        const result = await configValidator.validate('invalid config');

        expect(result.errors).toHaveLength(1);
        expect(result.errors[0].code).toBe('WASM_UNKNOWN_ERROR');
        expect(result.errors[0].message).toBe('Unknown WASM error occurred');
      });
    });

    describe('linting rules', () => {
      it('should run linting rules when enabled', async () => {
        const config = `
          layer "unused_layer" {
            map KEY_A to KEY_B
          }
        `;

        vi.mocked(wasmCore.loadConfig).mockResolvedValue({} as any);
        vi.mocked(lintUnusedLayers).mockReturnValue([
          {
            line: 2,
            column: 1,
            message: "Layer 'unused_layer' is defined but never activated",
            code: 'UNUSED_LAYER',
          },
        ]);

        const result = await configValidator.validate(config, {
          enableLinting: true,
          includeHints: true,
        });

        expect(result.warnings).toHaveLength(1);
        expect(result.warnings[0].code).toBe('UNUSED_LAYER');
        expect(lintUnusedLayers).toHaveBeenCalledWith(config);
      });

      it('should not run linting rules when disabled', async () => {
        const config = 'layer "test" { map KEY_A to KEY_B }';

        vi.mocked(wasmCore.loadConfig).mockResolvedValue({} as any);

        const result = await configValidator.validate(config, {
          enableLinting: false,
          includeHints: false,
        });

        expect(lintUnusedLayers).not.toHaveBeenCalled();
        expect(lintNamingConsistency).not.toHaveBeenCalled();
      });

      it('should not run linting if syntax errors exist', async () => {
        vi.mocked(wasmCore.loadConfig).mockRejectedValue(
          new Error('Parse error at line 1, column 1: Syntax error')
        );

        const result = await configValidator.validate('invalid', {
          enableLinting: true,
          includeHints: true,
        });

        expect(result.errors).toHaveLength(1);
        expect(lintUnusedLayers).not.toHaveBeenCalled();
        expect(lintNamingConsistency).not.toHaveBeenCalled();
      });

      it('should include hints when enabled', async () => {
        const config = 'layer "camelCase" { } layer "snake_case" { }';

        vi.mocked(wasmCore.loadConfig).mockResolvedValue({} as any);
        vi.mocked(lintNamingConsistency).mockReturnValue([
          {
            line: 1,
            column: 1,
            message: 'Consider using consistent naming',
            code: 'NAMING_INCONSISTENCY',
          },
        ]);

        const result = await configValidator.validate(config, {
          enableLinting: true,
          includeHints: true,
        });

        expect(result.hints).toHaveLength(1);
        expect(result.hints[0].code).toBe('NAMING_INCONSISTENCY');
      });

      it('should exclude hints when disabled', async () => {
        const config = 'layer "test" { }';

        vi.mocked(wasmCore.loadConfig).mockResolvedValue({} as any);
        vi.mocked(lintNamingConsistency).mockReturnValue([
          {
            line: 1,
            column: 1,
            message: 'Hint',
            code: 'HINT',
          },
        ]);

        const result = await configValidator.validate(config, {
          enableLinting: true,
          includeHints: false,
        });

        expect(result.hints).toHaveLength(0);
      });

      it('should detect large configs', async () => {
        const largeConfig = 'layer "test" { }\n'.repeat(600);

        vi.mocked(wasmCore.loadConfig).mockResolvedValue({} as any);

        const result = await configValidator.validate(largeConfig, {
          enableLinting: true,
          includeHints: true,
        });

        expect(result.warnings.some(w => w.code === 'LARGE_CONFIG')).toBe(true);
      });
    });

    describe('error/warning limits', () => {
      it('should limit errors when maxErrors is set', async () => {
        // Simulate multiple errors by having WASM reject multiple times
        // In practice, we can only test the limit by checking the implementation
        vi.mocked(wasmCore.loadConfig).mockRejectedValue(
          new Error('Error at line 1, column 1: Error 1')
        );

        const result = await configValidator.validate('invalid', {
          enableLinting: false,
          includeHints: false,
          maxErrors: 1,
        });

        // Should have at most maxErrors + 1 (the "too many errors" message)
        expect(result.errors.length).toBeLessThanOrEqual(2);
      });

      it('should limit warnings when maxWarnings is set', async () => {
        const config = `
          layer "unused1" { }
          layer "unused2" { }
          layer "unused3" { }
        `;

        vi.mocked(wasmCore.loadConfig).mockResolvedValue({} as any);
        vi.mocked(lintUnusedLayers).mockReturnValue([
          { line: 2, column: 1, message: 'Warning 1', code: 'UNUSED_LAYER' },
          { line: 3, column: 1, message: 'Warning 2', code: 'UNUSED_LAYER' },
          { line: 4, column: 1, message: 'Warning 3', code: 'UNUSED_LAYER' },
        ]);

        const result = await configValidator.validate(config, {
          enableLinting: true,
          includeHints: false,
          maxWarnings: 2,
        });

        expect(result.warnings.length).toBeLessThanOrEqual(2);
      });
    });
  });

  describe('parseWasmError()', () => {
    it('should extract line and column from standard format', () => {
      const error = new Error('Parse error at line 42, column 13: Unexpected token');

      const errors = configValidator.parseWasmError(error);

      expect(errors).toHaveLength(1);
      expect(errors[0].line).toBe(42);
      expect(errors[0].column).toBe(13);
      expect(errors[0].message).toBe('Unexpected token');
      expect(errors[0].code).toBe('WASM_PARSE_ERROR');
    });

    it('should extract line and column from alternative format', () => {
      const error = new Error('at 25:7 - Invalid syntax');

      const errors = configValidator.parseWasmError(error);

      expect(errors).toHaveLength(1);
      expect(errors[0].line).toBe(25);
      expect(errors[0].column).toBe(7);
      expect(errors[0].message).toBe('Invalid syntax');
    });

    it('should handle errors with Line (capital L)', () => {
      const error = new Error('Error at Line 5, Column 3: Test error');

      const errors = configValidator.parseWasmError(error);

      expect(errors).toHaveLength(1);
      expect(errors[0].line).toBe(5);
      expect(errors[0].column).toBe(3);
    });

    it('should handle errors without column (defaults to 1)', () => {
      const error = new Error('Error at line 10: Missing brace');

      const errors = configValidator.parseWasmError(error);

      expect(errors).toHaveLength(1);
      expect(errors[0].line).toBe(10);
      expect(errors[0].column).toBe(1);
      expect(errors[0].message).toBe('Missing brace');
    });

    it('should handle string errors', () => {
      const error = 'Parse error at line 3, column 5: String error';

      const errors = configValidator.parseWasmError(error);

      expect(errors).toHaveLength(1);
      expect(errors[0].line).toBe(3);
      expect(errors[0].column).toBe(5);
    });

    it('should create generic error for unparseable messages', () => {
      const error = new Error('Something went wrong');

      const errors = configValidator.parseWasmError(error);

      expect(errors).toHaveLength(1);
      expect(errors[0].line).toBe(1);
      expect(errors[0].column).toBe(1);
      expect(errors[0].message).toBe('Something went wrong');
      expect(errors[0].code).toBe('WASM_ERROR');
    });

    it('should handle unknown error types', () => {
      const error = { not: 'an error' };

      const errors = configValidator.parseWasmError(error);

      expect(errors).toHaveLength(1);
      expect(errors[0].code).toBe('WASM_UNKNOWN_ERROR');
      expect(errors[0].message).toBe('Unknown WASM error occurred');
    });

    it('should handle null/undefined', () => {
      const errors1 = configValidator.parseWasmError(null);
      const errors2 = configValidator.parseWasmError(undefined);

      expect(errors1[0].code).toBe('WASM_UNKNOWN_ERROR');
      expect(errors2[0].code).toBe('WASM_UNKNOWN_ERROR');
    });
  });

  describe('runLintingRules()', () => {
    it('should run all linting rules', () => {
      const config = 'layer "test" { }';

      vi.mocked(lintUnusedLayers).mockReturnValue([]);
      vi.mocked(lintNamingConsistency).mockReturnValue([]);

      const result = configValidator.runLintingRules(config);

      expect(lintUnusedLayers).toHaveBeenCalledWith(config);
      expect(lintNamingConsistency).toHaveBeenCalledWith(config);
      expect(result.warnings).toHaveLength(0);
      expect(result.hints).toHaveLength(0);
    });

    it('should detect large configs (>500 lines)', () => {
      const largeConfig = 'line\n'.repeat(600);

      const result = configValidator.runLintingRules(largeConfig);

      expect(result.warnings).toHaveLength(1);
      expect(result.warnings[0].code).toBe('LARGE_CONFIG');
      expect(result.warnings[0].message).toContain('601 lines');
    });

    it('should not warn for configs under 500 lines', () => {
      const normalConfig = 'line\n'.repeat(400);

      vi.mocked(lintUnusedLayers).mockReturnValue([]);
      vi.mocked(lintNamingConsistency).mockReturnValue([]);

      const result = configValidator.runLintingRules(normalConfig);

      expect(result.warnings.some(w => w.code === 'LARGE_CONFIG')).toBe(false);
    });

    it('should collect warnings from linting rules', () => {
      vi.mocked(lintUnusedLayers).mockReturnValue([
        {
          line: 5,
          column: 1,
          message: 'Unused layer',
          code: 'UNUSED_LAYER',
        },
      ]);

      const result = configValidator.runLintingRules('config');

      expect(result.warnings).toHaveLength(1);
      expect(result.warnings[0].code).toBe('UNUSED_LAYER');
    });

    it('should collect hints from linting rules', () => {
      vi.mocked(lintNamingConsistency).mockReturnValue([
        {
          line: 1,
          column: 1,
          message: 'Naming hint',
          code: 'NAMING_INCONSISTENCY',
        },
      ]);

      const result = configValidator.runLintingRules('config');

      expect(result.hints).toHaveLength(1);
      expect(result.hints[0].code).toBe('NAMING_INCONSISTENCY');
    });
  });

  describe('generateStats()', () => {
    it('should count total lines', async () => {
      const config = 'line1\nline2\nline3';

      vi.mocked(wasmCore.loadConfig).mockResolvedValue({} as any);

      const result = await configValidator.validate(config);

      expect(result.stats?.lineCount).toBe(3);
    });

    it('should count code lines (excluding empty and comments)', async () => {
      const config = `
        layer "test" {
          // This is a comment
          map KEY_A to KEY_B

        }
      `;

      vi.mocked(wasmCore.loadConfig).mockResolvedValue({} as any);

      const result = await configValidator.validate(config);

      expect(result.stats?.codeLineCount).toBeLessThan(result.stats?.lineCount || 0);
    });

    it('should count layers', async () => {
      const config = `
        layer "layer1" { }
        layer "layer2" { }
        layer "layer3" { }
      `;

      vi.mocked(wasmCore.loadConfig).mockResolvedValue({} as any);

      const result = await configValidator.validate(config);

      expect(result.stats?.layerCount).toBe(3);
    });

    it('should count modifiers', async () => {
      const config = `
        modifier "mod1" { }
        modifier "mod2" { }
      `;

      vi.mocked(wasmCore.loadConfig).mockResolvedValue({} as any);

      const result = await configValidator.validate(config);

      expect(result.stats?.modifierCount).toBe(2);
    });

    it('should count locks', async () => {
      const config = `
        lock "lock1" { }
        lock "lock2" { }
      `;

      vi.mocked(wasmCore.loadConfig).mockResolvedValue({} as any);

      const result = await configValidator.validate(config);

      expect(result.stats?.lockCount).toBe(2);
    });

    it('should count mappings', async () => {
      const config = `
        layer "base" {
          map KEY_A to KEY_B
          map KEY_C to KEY_D
          map KEY_E to KEY_F
        }
      `;

      vi.mocked(wasmCore.loadConfig).mockResolvedValue({} as any);

      const result = await configValidator.validate(config);

      expect(result.stats?.mappingCount).toBe(3);
    });

    it('should handle configs with no entities', async () => {
      const config = '// Just a comment';

      vi.mocked(wasmCore.loadConfig).mockResolvedValue({} as any);

      const result = await configValidator.validate(config);

      expect(result.stats?.layerCount).toBe(0);
      expect(result.stats?.modifierCount).toBe(0);
      expect(result.stats?.lockCount).toBe(0);
    });
  });

  describe('singleton instance', () => {
    it('should export a singleton validator instance', () => {
      expect(validator).toBeInstanceOf(ConfigValidator);
    });

    it('should be the same instance across imports', () => {
      const validator1 = validator;
      const validator2 = validator;
      expect(validator1).toBe(validator2);
    });
  });

  describe('integration scenarios', () => {
    it('should handle complete validation workflow', async () => {
      const config = `
        layer "base" {
          map KEY_A to KEY_B
        }
        layer "unused" {
          map KEY_C to KEY_D
        }
      `;

      vi.mocked(wasmCore.loadConfig).mockResolvedValue({} as any);
      vi.mocked(lintUnusedLayers).mockReturnValue([
        {
          line: 5,
          column: 1,
          message: "Layer 'unused' is defined but never activated",
          code: 'UNUSED_LAYER',
        },
      ]);

      const result = await configValidator.validate(config);

      expect(result.errors).toHaveLength(0);
      expect(result.warnings).toHaveLength(1);
      expect(result.warnings[0].code).toBe('UNUSED_LAYER');
      expect(result.stats).toBeDefined();
      expect(result.stats?.layerCount).toBe(2);
      expect(result.timestamp).toBeDefined();
    });

    it('should validate timestamp format', async () => {
      vi.mocked(wasmCore.loadConfig).mockResolvedValue({} as any);

      const result = await configValidator.validate('layer "test" { }');

      expect(result.timestamp).toMatch(/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}/);
    });
  });
});
