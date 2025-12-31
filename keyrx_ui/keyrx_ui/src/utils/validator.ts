/**
 * Configuration validation service.
 *
 * This module provides the core validation engine that wraps the WASM module,
 * parses errors from WASM, and runs optional linting rules for code quality checks.
 */

import { wasmCore } from '@/wasm/core';
import type {
  ValidationResult,
  ValidationError,
  ValidationWarning,
  ValidationHint,
  ValidationOptions,
  ConfigStats,
} from '@/types/validation';
import { DEFAULT_VALIDATION_OPTIONS } from '@/types/validation';
import { lintUnusedLayers, lintNamingConsistency } from './lintingRules';

/**
 * Regular expression to parse WASM error messages.
 * Matches patterns like: "Parse error at line 4, column 9: Missing semicolon"
 * or "Error at line 10, column 5: Undefined layer 'foo'"
 */
const WASM_ERROR_REGEX = /(?:line|Line)\s+(\d+)(?:,\s*(?:column|Column)\s+(\d+))?\s*:\s*(.+)/;

/**
 * Alternative error regex for different WASM error formats.
 * Matches: "at 4:9 - Missing semicolon"
 */
const ALT_ERROR_REGEX = /at\s+(\d+):(\d+)\s*[-–]\s*(.+)/;

/**
 * ConfigValidator service for validating Rhai configurations.
 *
 * This class wraps the WASM module and provides comprehensive validation
 * including syntax checking and optional linting rules.
 */
export class ConfigValidator {
  /**
   * Validate a Rhai configuration source.
   *
   * This method performs syntax validation via WASM and optionally runs
   * linting rules for code quality checks.
   *
   * @param rhaiSource - The Rhai configuration source code to validate
   * @param options - Validation options (defaults to all checks enabled)
   * @returns ValidationResult with errors, warnings, hints, and statistics
   */
  async validate(
    rhaiSource: string,
    options: ValidationOptions = DEFAULT_VALIDATION_OPTIONS
  ): Promise<ValidationResult> {
    const errors: ValidationError[] = [];
    const warnings: ValidationWarning[] = [];
    const hints: ValidationHint[] = [];
    const timestamp = new Date().toISOString();

    // Check if source is empty
    if (!rhaiSource || rhaiSource.trim().length === 0) {
      errors.push({
        line: 1,
        column: 1,
        message: 'Configuration cannot be empty',
        code: 'EMPTY_CONFIG',
      });

      return {
        errors,
        warnings,
        hints,
        timestamp,
      };
    }

    // Try to load config via WASM for syntax validation
    try {
      await wasmCore.loadConfig(rhaiSource);
      // Config loaded successfully - no syntax errors
    } catch (error) {
      // Parse WASM error and convert to ValidationError
      const parsedErrors = this.parseWasmError(error);
      errors.push(...parsedErrors);

      // Limit errors if maxErrors specified
      if (options.maxErrors && errors.length > options.maxErrors) {
        errors.splice(options.maxErrors);
        errors.push({
          line: 1,
          column: 1,
          message: `Too many errors (showing first ${options.maxErrors})`,
          code: 'TOO_MANY_ERRORS',
        });
      }
    }

    // Run linting rules if enabled and no syntax errors
    if (options.enableLinting && errors.length === 0) {
      const lintResults = this.runLintingRules(rhaiSource);
      warnings.push(...lintResults.warnings);
      if (options.includeHints) {
        hints.push(...lintResults.hints);
      }

      // Limit warnings if maxWarnings specified
      if (options.maxWarnings && warnings.length > options.maxWarnings) {
        warnings.splice(options.maxWarnings);
      }
    }

    // Generate statistics
    const stats = this.generateStats(rhaiSource);

    return {
      errors,
      warnings,
      hints,
      stats,
      timestamp,
    };
  }

  /**
   * Parse WASM error messages and convert to ValidationError format.
   *
   * Extracts line and column numbers from error messages using regex patterns.
   * Handles various WASM error formats gracefully.
   *
   * @param error - The error thrown by WASM (Error object or string)
   * @returns Array of ValidationError objects
   */
  parseWasmError(error: unknown): ValidationError[] {
    const errors: ValidationError[] = [];

    // Extract error message
    let errorMessage: string;
    if (error instanceof Error) {
      errorMessage = error.message;
    } else if (typeof error === 'string') {
      errorMessage = error;
    } else {
      // Unknown error type - create generic error
      return [
        {
          line: 1,
          column: 1,
          message: 'Unknown WASM error occurred',
          code: 'WASM_UNKNOWN_ERROR',
        },
      ];
    }

    // Try to extract line/column from error message
    const match = errorMessage.match(WASM_ERROR_REGEX);
    const altMatch = !match ? errorMessage.match(ALT_ERROR_REGEX) : null;

    if (match) {
      const line = parseInt(match[1], 10);
      const column = match[2] ? parseInt(match[2], 10) : 1;
      const message = match[3].trim();

      errors.push({
        line,
        column,
        message,
        code: 'WASM_PARSE_ERROR',
      });
    } else if (altMatch) {
      const line = parseInt(altMatch[1], 10);
      const column = parseInt(altMatch[2], 10);
      const message = altMatch[3].trim();

      errors.push({
        line,
        column,
        message,
        code: 'WASM_PARSE_ERROR',
      });
    } else {
      // Could not parse line/column - create generic error
      errors.push({
        line: 1,
        column: 1,
        message: errorMessage,
        code: 'WASM_ERROR',
      });
    }

    return errors;
  }

  /**
   * Run linting rules for code quality checks.
   *
   * This method performs non-critical checks like unused layers,
   * naming consistency, and config size warnings.
   *
   * @param rhaiSource - The Rhai configuration source code
   * @returns Object containing warnings and hints arrays
   */
  runLintingRules(rhaiSource: string): {
    warnings: ValidationWarning[];
    hints: ValidationHint[];
  } {
    const warnings: ValidationWarning[] = [];
    const hints: ValidationHint[] = [];

    // Check for large configs
    const lines = rhaiSource.split('\n');
    if (lines.length > 500) {
      warnings.push({
        line: 1,
        column: 1,
        message: `Configuration is large (${lines.length} lines). Consider splitting into multiple files.`,
        code: 'LARGE_CONFIG',
      });
    }

    // Check for unused layers
    const unusedLayerWarnings = lintUnusedLayers(rhaiSource);
    warnings.push(...unusedLayerWarnings);

    // Check for naming consistency
    const namingHints = lintNamingConsistency(rhaiSource);
    hints.push(...namingHints);

    return { warnings, hints };
  }

  /**
   * Generate configuration statistics.
   *
   * @param rhaiSource - The Rhai configuration source code
   * @returns ConfigStats object with line counts and entity counts
   */
  private generateStats(rhaiSource: string): ConfigStats {
    const lines = rhaiSource.split('\n');
    const lineCount = lines.length;

    // Count non-empty, non-comment lines
    let codeLineCount = 0;
    for (const line of lines) {
      const trimmed = line.trim();
      if (trimmed.length > 0 && !trimmed.startsWith('//')) {
        codeLineCount++;
      }
    }

    // Count layers
    const layerMatches = rhaiSource.match(/layer\s+"[^"]+"/g);
    const layerCount = layerMatches ? layerMatches.length : 0;

    // Count modifiers
    const modifierMatches = rhaiSource.match(/modifier\s+"[^"]+"/g);
    const modifierCount = modifierMatches ? modifierMatches.length : 0;

    // Count locks
    const lockMatches = rhaiSource.match(/lock\s+"[^"]+"/g);
    const lockCount = lockMatches ? lockMatches.length : 0;

    // Count mappings (simplified - counts "map" keyword)
    const mappingMatches = rhaiSource.match(/\bmap\b/g);
    const mappingCount = mappingMatches ? mappingMatches.length : 0;

    return {
      lineCount,
      codeLineCount,
      layerCount,
      modifierCount,
      lockCount,
      mappingCount,
    };
  }
}

/**
 * Singleton instance of ConfigValidator for application-wide use.
 *
 * Import and use this instance instead of creating new instances:
 * ```typescript
 * import { validator } from '@/utils/validator';
 * const result = await validator.validate(configSource);
 * ```
 */
export const validator = new ConfigValidator();
