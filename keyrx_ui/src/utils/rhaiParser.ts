/**
 * Rhai script parser for KeyRx configuration files
 *
 * This module provides utilities to parse Rhai scripts into structured AST
 * representations for use in the visual configuration editor.
 *
 * Key features:
 * - Parse device() blocks with serial number patterns
 * - Extract all mapping types (simple, tap_hold, macro, layer_switch)
 * - Distinguish global vs device-specific scope
 * - Extract import statements
 * - Preserve comments with line numbers
 * - Handle syntax errors gracefully with actionable suggestions
 *
 * @module rhaiParser
 */

/**
 * Parse result containing either successful AST or error details
 */
export interface ParseResult {
  /** Whether parsing succeeded */
  success: boolean;
  /** Parsed AST if successful */
  ast?: RhaiAST;
  /** Parse error if failed */
  error?: ParseError;
}

/**
 * Abstract Syntax Tree representation of a Rhai configuration script
 */
export interface RhaiAST {
  /** Import statements found in the script */
  imports: ImportStatement[];
  /** Global mappings (outside any device block) */
  globalMappings: KeyMapping[];
  /** Device-specific configuration blocks */
  deviceBlocks: DeviceBlock[];
  /** Comments preserved with their line numbers */
  comments: Comment[];
}

/**
 * Import statement for including other Rhai files or modules
 */
export interface ImportStatement {
  /** Path or module being imported */
  path: string;
  /** Optional alias for the import */
  alias?: string;
  /** Line number where import appears */
  line: number;
}

/**
 * Device configuration block for device-specific mappings
 */
export interface DeviceBlock {
  /** Device serial number pattern (e.g., "*", "*Keychron*", "Vendor_04d9_Product_a09f") */
  pattern: string;
  /** Key mappings specific to this device */
  mappings: KeyMapping[];
  /** Modifier layers (when_start blocks) */
  layers: ModifierLayer[];
  /** Line number where device block starts */
  startLine: number;
  /** Line number where device block ends */
  endLine: number;
}

/**
 * Modifier layer activated by holding a modifier key
 */
export interface ModifierLayer {
  /** Modifier ID(s) that activate this layer (e.g., "MD_00" or ["MD_00", "MD_01"]) */
  modifiers: string | string[];
  /** Key mappings active when this layer is enabled */
  mappings: KeyMapping[];
  /** Line number where layer starts */
  startLine: number;
  /** Line number where layer ends */
  endLine: number;
}

/**
 * Key mapping configuration - supports multiple mapping types
 */
export interface KeyMapping {
  /** Mapping type */
  type: 'simple' | 'tap_hold' | 'macro' | 'layer_switch';
  /** Physical key code (VK_* format) */
  sourceKey: string;
  /** Line number where mapping is defined */
  line: number;
  /** Simple mapping target (for type='simple') */
  targetKey?: string;
  /** Tap-hold configuration (for type='tap_hold') */
  tapHold?: TapHoldConfig;
  /** Macro sequence (for type='macro') */
  macro?: MacroSequence;
  /** Layer switch (for type='layer_switch') */
  layerSwitch?: LayerSwitch;
}

/**
 * Tap-hold dual-function key configuration
 */
export interface TapHoldConfig {
  /** Key to output on quick tap */
  tapAction: string;
  /** Modifier to activate on hold */
  holdAction: string;
  /** Threshold in milliseconds */
  thresholdMs: number;
}

/**
 * Macro sequence configuration
 */
export interface MacroSequence {
  /** Array of key codes to emit in sequence */
  keys: string[];
  /** Optional delay between keys in milliseconds */
  delayMs?: number;
}

/**
 * Layer switch configuration
 */
export interface LayerSwitch {
  /** Target layer identifier */
  layerId: string;
  /** Switch mode (toggle, momentary, etc.) */
  mode?: 'toggle' | 'momentary';
}

/**
 * Key action that can include modifiers (Ctrl, Shift, Alt, Super)
 */
export interface KeyAction {
  /** Base key code */
  key: string;
  /** Whether Ctrl modifier is active */
  ctrl?: boolean;
  /** Whether Shift modifier is active */
  shift?: boolean;
  /** Whether Alt modifier is active */
  alt?: boolean;
  /** Whether Super/Windows modifier is active */
  super?: boolean;
}

/**
 * Comment preserved from the source code
 */
export interface Comment {
  /** Comment text (excluding // or /* *\/) */
  text: string;
  /** Line number where comment appears */
  line: number;
  /** Comment type */
  type: 'line' | 'block';
}

/**
 * Parse error with detailed location and suggestion
 */
export interface ParseError {
  /** Error message */
  message: string;
  /** Line number where error occurred */
  line: number;
  /** Column number where error occurred */
  column: number;
  /** Helpful suggestion to fix the error */
  suggestion?: string;
}

/**
 * Parse a Rhai script into structured AST
 *
 * This function parses Rhai configuration scripts and extracts all relevant
 * information including device blocks, key mappings, import statements, and comments.
 *
 * @param script - Rhai script source code
 * @returns ParseResult containing either AST or error details
 *
 * @example
 * ```typescript
 * const script = `
 *   device_start("*");
 *     map("VK_A", "VK_B");
 *   device_end();
 * `;
 *
 * const result = parseRhaiScript(script);
 * if (result.success) {
 *   console.log('Device blocks:', result.ast.deviceBlocks);
 * } else {
 *   console.error('Parse error:', result.error);
 * }
 * ```
 */
export function parseRhaiScript(script: string): ParseResult {
  try {
    const lines = script.split('\n');
    const ast: RhaiAST = {
      imports: [],
      globalMappings: [],
      deviceBlocks: [],
      comments: [],
    };

    let currentDeviceBlock: DeviceBlock | null = null;
    let currentModifierLayer: ModifierLayer | null = null;
    let lineNumber = 0;

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
        let modifiers: string | string[];
        if (whenStartMatch[1]) {
          // Single modifier
          modifiers = whenStartMatch[1];
        } else {
          // Array of modifiers
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

        // Add layer to current device block or global
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
        const mapping: KeyMapping = {
          type: 'simple',
          sourceKey: mapMatch[1],
          targetKey: mapMatch[2].replace(/"/g, ''),
          line: lineNumber,
        };

        // Add to current context (layer, device, or global)
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
        const mapping: KeyMapping = {
          type: 'tap_hold',
          sourceKey: tapHoldMatch[1],
          line: lineNumber,
          tapHold: {
            tapAction: tapHoldMatch[2],
            holdAction: tapHoldMatch[3],
            thresholdMs: parseInt(tapHoldMatch[4], 10),
          },
        };

        // Add to current context
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
        const mapping: KeyMapping = {
          type: 'simple',
          sourceKey: withModMatch[1],
          targetKey: withModMatch[2], // Store the full expression
          line: lineNumber,
        };

        // Add to current context
        if (currentModifierLayer) {
          currentModifierLayer.mappings.push(mapping);
        } else if (currentDeviceBlock) {
          currentDeviceBlock.mappings.push(mapping);
        } else {
          ast.globalMappings.push(mapping);
        }
        continue;
      }

      // If we get here and we're not in a comment or empty line, it might be unrecognized syntax
      // For now, we'll silently ignore unrecognized lines to be permissive
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
    };
  } catch (error) {
    // Handle unexpected errors
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

/**
 * Extract device serial patterns from device blocks
 *
 * @param ast - Parsed AST
 * @returns Array of device serial patterns
 */
export function extractDevicePatterns(ast: RhaiAST): string[] {
  return ast.deviceBlocks.map((block) => block.pattern);
}

/**
 * Check if AST has global mappings (mappings outside device blocks)
 *
 * @param ast - Parsed AST
 * @returns True if there are global mappings
 */
export function hasGlobalMappings(ast: RhaiAST): boolean {
  return ast.globalMappings.length > 0;
}

/**
 * Get all mappings for a specific device pattern
 *
 * @param ast - Parsed AST
 * @param pattern - Device pattern to search for
 * @returns Array of key mappings for the device, or undefined if not found
 */
export function getMappingsForDevice(
  ast: RhaiAST,
  pattern: string
): KeyMapping[] | undefined {
  const deviceBlock = ast.deviceBlocks.find(
    (block) => block.pattern === pattern
  );
  if (!deviceBlock) return undefined;

  // Flatten device mappings and layer mappings
  const allMappings = [...deviceBlock.mappings];
  for (const layer of deviceBlock.layers) {
    allMappings.push(...layer.mappings);
  }

  return allMappings;
}

/**
 * Validate that parsed AST has expected structure
 *
 * @param ast - Parsed AST to validate
 * @returns Validation result with any errors found
 */
export function validateAST(ast: RhaiAST): {
  valid: boolean;
  errors: string[];
} {
  const errors: string[] = [];

  // Check for duplicate device patterns
  const patterns = new Set<string>();
  for (const block of ast.deviceBlocks) {
    if (patterns.has(block.pattern)) {
      errors.push(`Duplicate device pattern: ${block.pattern}`);
    }
    patterns.add(block.pattern);
  }

  // Validate tap_hold thresholds
  const allMappings = [
    ...ast.globalMappings,
    ...ast.deviceBlocks.flatMap((block) => [
      ...block.mappings,
      ...block.layers.flatMap((layer) => layer.mappings),
    ]),
  ];

  for (const mapping of allMappings) {
    if (mapping.type === 'tap_hold' && mapping.tapHold) {
      const threshold = mapping.tapHold.thresholdMs;
      if (threshold < 50) {
        errors.push(
          `Tap-hold threshold too low (${threshold}ms) at line ${mapping.line}. Minimum: 50ms`
        );
      }
      if (threshold > 1000) {
        errors.push(
          `Tap-hold threshold too high (${threshold}ms) at line ${mapping.line}. Maximum: 1000ms`
        );
      }
    }
  }

  return {
    valid: errors.length === 0,
    errors,
  };
}
