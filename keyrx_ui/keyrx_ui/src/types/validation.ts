/**
 * Validation type definitions for configuration validation system.
 *
 * This module provides type-safe contracts for validation data structures
 * used throughout the config validation feature, including errors, warnings,
 * hints, quick fixes, and statistics.
 */

/**
 * Represents a single validation error in the configuration.
 * Errors indicate critical issues that must be fixed before the config can be used.
 */
export interface ValidationError {
  /** The line number where the error occurs (1-indexed) */
  readonly line: number;

  /** The column number where the error starts (1-indexed) */
  readonly column: number;

  /** The line number where the error ends (optional, defaults to line) */
  readonly endLine?: number;

  /** The column number where the error ends (optional, for multi-character errors) */
  readonly endColumn?: number;

  /** Human-readable error message describing what went wrong */
  readonly message: string;

  /** Machine-readable error code for categorization and tooling */
  readonly code: string;

  /** Optional quick fix suggestion to automatically resolve the error */
  readonly quickFix?: QuickFix;
}

/**
 * Represents a validation warning about potential issues or code quality concerns.
 * Warnings don't prevent config usage but indicate areas for improvement.
 */
export interface ValidationWarning {
  /** The line number where the warning occurs (1-indexed) */
  readonly line: number;

  /** The column number where the warning starts (1-indexed) */
  readonly column: number;

  /** The line number where the warning ends (optional) */
  readonly endLine?: number;

  /** The column number where the warning ends (optional) */
  readonly endColumn?: number;

  /** Human-readable warning message describing the concern */
  readonly message: string;

  /** Machine-readable warning code for categorization */
  readonly code: string;

  /** Optional quick fix suggestion to address the warning */
  readonly quickFix?: QuickFix;
}

/**
 * Represents a helpful hint about code style or best practices.
 * Hints are purely informational and don't indicate actual problems.
 */
export interface ValidationHint {
  /** The line number where the hint applies (1-indexed) */
  readonly line: number;

  /** The column number where the hint starts (1-indexed) */
  readonly column: number;

  /** The line number where the hint ends (optional) */
  readonly endLine?: number;

  /** The column number where the hint ends (optional) */
  readonly endColumn?: number;

  /** Human-readable hint message with suggestions */
  readonly message: string;

  /** Machine-readable hint code for categorization */
  readonly code: string;
}

/**
 * Represents an automated fix that can be applied to resolve an error or warning.
 * Quick fixes provide "one-click" solutions to common validation issues.
 */
export interface QuickFix {
  /** Short title describing the fix (shown in UI) */
  readonly title: string;

  /** Detailed explanation of what the fix will do */
  readonly description: string;

  /** Array of text edits to apply when the quick fix is executed */
  readonly edits: readonly TextEdit[];
}

/**
 * Represents a single text edit operation for applying a quick fix.
 * Edits can replace, insert, or delete text at specific locations.
 */
export interface TextEdit {
  /** The line number where the edit starts (1-indexed) */
  readonly startLine: number;

  /** The column number where the edit starts (1-indexed) */
  readonly startColumn: number;

  /** The line number where the edit ends (1-indexed) */
  readonly endLine: number;

  /** The column number where the edit ends (1-indexed) */
  readonly endColumn: number;

  /** The new text to insert (empty string for deletion) */
  readonly newText: string;
}

/**
 * Complete validation result containing all errors, warnings, and hints.
 * This is the primary data structure returned by the validator.
 */
export interface ValidationResult {
  /** Array of critical errors that must be fixed */
  readonly errors: readonly ValidationError[];

  /** Array of warnings about potential issues */
  readonly warnings: readonly ValidationWarning[];

  /** Array of helpful hints about code style */
  readonly hints: readonly ValidationHint[];

  /** Optional statistics about the configuration */
  readonly stats?: ConfigStats;

  /** Timestamp when validation was performed (ISO 8601 format) */
  readonly timestamp: string;
}

/**
 * Statistics about the configuration being validated.
 * Provides insights into config size and complexity.
 */
export interface ConfigStats {
  /** Total number of lines in the configuration */
  readonly lineCount: number;

  /** Number of non-empty, non-comment lines */
  readonly codeLineCount: number;

  /** Number of defined layers in the configuration */
  readonly layerCount: number;

  /** Number of defined modifiers in the configuration */
  readonly modifierCount: number;

  /** Number of defined locks in the configuration */
  readonly lockCount: number;

  /** Number of key mappings defined */
  readonly mappingCount: number;
}

/**
 * Options for configuring validation behavior.
 * Allows customization of which checks to run.
 */
export interface ValidationOptions {
  /** Whether to run linting rules (code quality checks) */
  readonly enableLinting: boolean;

  /** Whether to include hints in the results */
  readonly includeHints: boolean;

  /** Maximum number of errors to report (prevents UI overload) */
  readonly maxErrors?: number;

  /** Maximum number of warnings to report */
  readonly maxWarnings?: number;
}

/**
 * Default validation options with all checks enabled.
 */
export const DEFAULT_VALIDATION_OPTIONS: ValidationOptions = {
  enableLinting: true,
  includeHints: true,
  maxErrors: 100,
  maxWarnings: 50,
} as const;

/**
 * Type guard to check if a validation result has any errors.
 *
 * @param result - The validation result to check
 * @returns true if the result contains one or more errors
 */
export function hasErrors(result: ValidationResult | null): boolean {
  return result !== null && result.errors.length > 0;
}

/**
 * Type guard to check if a validation result has any warnings.
 *
 * @param result - The validation result to check
 * @returns true if the result contains one or more warnings
 */
export function hasWarnings(result: ValidationResult | null): boolean {
  return result !== null && result.warnings.length > 0;
}

/**
 * Type guard to check if a validation result is completely clean (no issues).
 *
 * @param result - The validation result to check
 * @returns true if the result has no errors, warnings, or hints
 */
export function isClean(result: ValidationResult | null): boolean {
  return (
    result !== null &&
    result.errors.length === 0 &&
    result.warnings.length === 0 &&
    result.hints.length === 0
  );
}
