/**
 * Linting rules for configuration validation.
 * Provides code quality checks beyond syntax validation.
 */

import type { ValidationWarning, ValidationHint } from '@/types/validation';

/**
 * Detects layers that are defined but never activated in the configuration.
 * @param configSource - The Rhai configuration source code
 * @returns Array of warnings for unused layers
 */
export function lintUnusedLayers(configSource: string): ValidationWarning[] {
  const warnings: ValidationWarning[] = [];
  const definedLayers = new Map<string, number>(); // layer name -> line number
  const activatedLayers = new Set<string>();

  try {
    // Find all layer definitions
    // Matches: layer "layer_name" { ... }
    const layerDefRegex = /layer\s+"([^"]+)"\s*\{/g;
    let match;

    while ((match = layerDefRegex.exec(configSource)) !== null) {
      const layerName = match[1];
      const lineNumber = getLineNumber(configSource, match.index);
      definedLayers.set(layerName, lineNumber);
    }

    // Find all layer activations
    // Matches various activation patterns:
    // - activate_layer "layer_name"
    // - activate("layer_name")
    // - to_layer "layer_name"
    // - switch_layer "layer_name"
    const activationPatterns = [
      /activate_layer\s+"([^"]+)"/g,
      /activate\s*\(\s*"([^"]+)"\s*\)/g,
      /to_layer\s+"([^"]+)"/g,
      /switch_layer\s+"([^"]+)"/g,
    ];

    for (const pattern of activationPatterns) {
      let activationMatch;
      while ((activationMatch = pattern.exec(configSource)) !== null) {
        const layerName = activationMatch[1];
        activatedLayers.add(layerName);
      }
    }

    // Check for unused layers
    for (const [layerName, lineNumber] of definedLayers) {
      if (!activatedLayers.has(layerName)) {
        warnings.push({
          line: lineNumber,
          column: 1,
          endLine: lineNumber,
          endColumn: 100, // Highlight the whole line
          message: `Layer '${layerName}' is defined but never activated`,
          code: 'UNUSED_LAYER',
        });
      }
    }
  } catch (error) {
    // Gracefully handle regex errors
    console.error('Error in lintUnusedLayers:', error);
  }

  return warnings;
}

/**
 * Detects inconsistent naming conventions in layer and modifier names.
 * Suggests using a consistent style (all camelCase or all snake_case).
 * @param configSource - The Rhai configuration source code
 * @returns Array of hints for naming inconsistencies
 */
export function lintNamingConsistency(configSource: string): ValidationHint[] {
  const hints: ValidationHint[] = [];
  const names: string[] = [];

  try {
    // Extract layer names
    const layerRegex = /layer\s+"([^"]+)"/g;
    let match;

    while ((match = layerRegex.exec(configSource)) !== null) {
      names.push(match[1]);
    }

    // Extract modifier names
    // Matches: modifier "modifier_name"
    const modifierRegex = /modifier\s+"([^"]+)"/g;
    while ((match = modifierRegex.exec(configSource)) !== null) {
      names.push(match[1]);
    }

    // Classify naming styles
    let camelCaseCount = 0;
    let snakeCaseCount = 0;

    for (const name of names) {
      if (isCamelCase(name)) {
        camelCaseCount++;
      } else if (isSnakeCase(name)) {
        snakeCaseCount++;
      }
    }

    // Only hint if both styles are present
    if (camelCaseCount > 0 && snakeCaseCount > 0) {
      hints.push({
        line: 1,
        column: 1,
        endLine: 1,
        endColumn: 1,
        message: `Consider using consistent naming (e.g., all snake_case). Found: camelCase (${camelCaseCount}) and snake_case (${snakeCaseCount})`,
        code: 'NAMING_INCONSISTENCY',
      });
    }
  } catch (error) {
    // Gracefully handle regex errors
    console.error('Error in lintNamingConsistency:', error);
  }

  return hints;
}

/**
 * Checks if a name follows camelCase convention.
 * @param name - The name to check
 * @returns True if the name is camelCase
 */
function isCamelCase(name: string): boolean {
  // camelCase: starts with lowercase, has uppercase in middle, no underscores
  return /^[a-z][a-zA-Z0-9]*$/.test(name) && /[A-Z]/.test(name);
}

/**
 * Checks if a name follows snake_case convention.
 * @param name - The name to check
 * @returns True if the name is snake_case
 */
function isSnakeCase(name: string): boolean {
  // snake_case: lowercase with underscores
  return /^[a-z][a-z0-9_]*$/.test(name) && /_/.test(name);
}

/**
 * Calculates the line number for a given character index in the source.
 * @param source - The source code
 * @param index - Character index in the source
 * @returns Line number (1-based)
 */
function getLineNumber(source: string, index: number): number {
  const upToIndex = source.substring(0, index);
  const lines = upToIndex.split('\n');
  return lines.length;
}
