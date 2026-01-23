/**
 * Rhai code generator for KeyRx configuration files
 *
 * This module provides utilities to generate Rhai scripts from structured AST
 * representations created by the visual configuration editor.
 *
 * Key features:
 * - Generate clean, formatted Rhai code from AST
 * - Support all mapping types (simple, tap_hold, macro, layer_switch)
 * - Generate device() blocks with proper grouping
 * - Preserve comments and formatting
 * - Configurable formatting options
 *
 * @module rhaiCodeGen
 */

import type {
  RhaiAST,
  DeviceBlock,
  KeyMapping,
  ModifierLayer,
  ImportStatement,
  Comment,
} from './rhaiParser';

/**
 * Formatting options for code generation
 */
export interface FormatOptions {
  /** Number of spaces for indentation (default: 4) */
  indentSize?: number;
  /** Maximum line length (default: 100) */
  maxLineLength?: number;
  /** Number of blank lines between device blocks (default: 1) */
  blankLinesBetweenDevices?: number;
  /** Number of blank lines between sections (imports, global, devices) (default: 2) */
  blankLinesBetweenSections?: number;
}

/**
 * Default formatting options
 */
const DEFAULT_FORMAT_OPTIONS: Required<FormatOptions> = {
  indentSize: 4,
  maxLineLength: 100,
  blankLinesBetweenDevices: 1,
  blankLinesBetweenSections: 2,
};

/**
 * Generate a complete Rhai script from AST
 *
 * This function generates a properly formatted Rhai configuration script
 * from a parsed AST structure, preserving comments and applying consistent
 * formatting rules.
 *
 * @param ast - Parsed AST structure
 * @param options - Optional formatting configuration
 * @returns Generated Rhai script as string
 *
 * @example
 * ```typescript
 * const ast: RhaiAST = {
 *   imports: [],
 *   globalMappings: [
 *     { type: 'simple', sourceKey: 'VK_A', targetKey: 'VK_B', line: 1 }
 *   ],
 *   deviceBlocks: [],
 *   comments: []
 * };
 *
 * const script = generateRhaiScript(ast);
 * console.log(script);
 * // Output:
 * // map("VK_A", "VK_B");
 * ```
 */
export function generateRhaiScript(
  ast: RhaiAST,
  options?: FormatOptions
): string {
  const opts = { ...DEFAULT_FORMAT_OPTIONS, ...options };
  const lines: string[] = [];

  // Generate import statements
  if (ast.imports.length > 0) {
    for (const importStmt of ast.imports) {
      lines.push(generateImportStatement(importStmt));
    }
    // Add blank lines after imports
    for (let i = 0; i < opts.blankLinesBetweenSections; i++) {
      lines.push('');
    }
  }

  // Generate global mappings wrapped in device_start("*") block
  // This ensures compatibility with the Rhai validator which requires all mappings inside device blocks
  if (ast.globalMappings.length > 0) {
    // Add comments that appear before global mappings
    const globalComments = getCommentsForSection(
      ast.comments,
      0,
      getFirstDeviceLineOrEnd(ast)
    );
    for (const comment of globalComments) {
      lines.push(generateComment(comment));
    }

    // Wrap global mappings in device_start("*") block
    lines.push('device_start("*");');
    const indent = ' '.repeat(opts.indentSize);
    for (const mapping of ast.globalMappings) {
      lines.push(indent + generateKeyMapping(mapping, 1, opts));
    }
    lines.push('device_end();');

    // Add blank lines after global mappings if there are device blocks
    // Use blankLinesBetweenDevices since global mappings are now a device block
    if (ast.deviceBlocks.length > 0) {
      for (let i = 0; i < opts.blankLinesBetweenDevices; i++) {
        lines.push('');
      }
    }
  }

  // Generate device blocks
  for (let i = 0; i < ast.deviceBlocks.length; i++) {
    const deviceBlock = ast.deviceBlocks[i];
    lines.push(...generateDeviceBlock(deviceBlock, ast.comments, opts));

    // Add blank lines between device blocks (except after last one)
    if (i < ast.deviceBlocks.length - 1) {
      for (let j = 0; j < opts.blankLinesBetweenDevices; j++) {
        lines.push('');
      }
    }
  }

  return lines.join('\n');
}

/**
 * Generate import statement
 *
 * @param importStmt - Import statement to generate
 * @returns Generated import statement as string
 */
function generateImportStatement(importStmt: ImportStatement): string {
  if (importStmt.alias) {
    return `import "${importStmt.path}" as ${importStmt.alias};`;
  }
  return `import "${importStmt.path}";`;
}

/**
 * Generate comment
 *
 * @param comment - Comment to generate
 * @returns Generated comment as string
 */
function generateComment(comment: Comment): string {
  if (comment.type === 'block') {
    return `/* ${comment.text} */`;
  }
  return `// ${comment.text}`;
}

/**
 * Generate device block with all its mappings and layers
 *
 * @param device - Device block to generate
 * @param allComments - All comments from the AST (for context preservation)
 * @param options - Formatting options
 * @returns Array of lines for the device block
 */
export function generateDeviceBlock(
  device: DeviceBlock,
  allComments: Comment[] = [],
  options?: FormatOptions
): string[] {
  const opts = { ...DEFAULT_FORMAT_OPTIONS, ...options };
  const lines: string[] = [];
  const indent = ' '.repeat(opts.indentSize);

  // Add comments that appear before this device block
  const blockComments = getCommentsForSection(
    allComments,
    device.startLine - 5, // Look for comments within 5 lines before
    device.startLine
  );
  for (const comment of blockComments) {
    lines.push(generateComment(comment));
  }

  // Device start
  lines.push(`device_start("${device.pattern}");`);

  // Device mappings
  for (const mapping of device.mappings) {
    lines.push(indent + generateKeyMapping(mapping, 1, opts));
  }

  // Modifier layers
  for (const layer of device.layers) {
    lines.push(...generateModifierLayer(layer, 1, opts));
  }

  // Device end
  lines.push('device_end();');

  return lines;
}

/**
 * Generate modifier layer (when block)
 *
 * @param layer - Modifier layer to generate
 * @param baseIndentLevel - Base indentation level (0 = no indent, 1 = one level, etc.)
 * @param options - Formatting options
 * @returns Array of lines for the modifier layer
 */
function generateModifierLayer(
  layer: ModifierLayer,
  baseIndentLevel: number,
  options: Required<FormatOptions>
): string[] {
  const lines: string[] = [];
  const baseIndent = ' '.repeat(baseIndentLevel * options.indentSize);
  const innerIndent = ' '.repeat((baseIndentLevel + 1) * options.indentSize);

  // Format modifiers - single string or array
  let modifiersArg: string;
  if (Array.isArray(layer.modifiers)) {
    const quotedMods = layer.modifiers.map((m) => `"${m}"`).join(', ');
    modifiersArg = `[${quotedMods}]`;
  } else {
    modifiersArg = `"${layer.modifiers}"`;
  }

  // When start
  lines.push(`${baseIndent}when_start(${modifiersArg});`);

  // Layer mappings
  for (const mapping of layer.mappings) {
    lines.push(
      innerIndent + generateKeyMapping(mapping, baseIndentLevel + 1, options)
    );
  }

  // When end
  lines.push(`${baseIndent}when_end();`);

  return lines;
}

/**
 * Generate a single key mapping
 *
 * @param mapping - Key mapping to generate
 * @param indentLevel - Current indentation level
 * @param options - Formatting options
 * @returns Generated mapping statement as string
 */
export function generateKeyMapping(
  mapping: KeyMapping,
  _indentLevel: number = 0,
  _options?: FormatOptions
): string {
  switch (mapping.type) {
    case 'simple':
      return generateSimpleMapping(mapping);

    case 'tap_hold':
      return generateTapHoldMapping(mapping);

    case 'macro':
      return generateMacroMapping(mapping);

    case 'layer_switch':
      return generateLayerSwitchMapping(mapping);

    default:
      // This should never happen as all cases are handled
      return `// Unknown mapping type`;
  }
}

/**
 * Ensure key has VK_ prefix
 */
function ensureVKPrefix(key: string): string {
  // Skip if already has VK_ prefix
  if (key.startsWith('VK_')) {
    return key;
  }
  // Skip if it's a helper function (with_ctrl, etc.)
  if (key.startsWith('with_')) {
    return key;
  }
  // Skip if it's a layer modifier (MD_00, etc.)
  if (key.startsWith('MD_') || key.startsWith('MMD_')) {
    return key;
  }
  // Add VK_ prefix
  return `VK_${key}`;
}

/**
 * Generate simple key mapping
 */
function generateSimpleMapping(mapping: KeyMapping): string {
  if (!mapping.targetKey) {
    throw new Error(`Simple mapping missing targetKey at line ${mapping.line}`);
  }

  const sourceKey = ensureVKPrefix(mapping.sourceKey);
  const targetKey = mapping.targetKey;

  // Check if target is a helper function (with_ctrl, with_shift, etc.)
  if (targetKey.startsWith('with_')) {
    return `map("${sourceKey}", ${targetKey});`;
  }

  return `map("${sourceKey}", "${ensureVKPrefix(targetKey)}");`;
}

/**
 * Generate tap-hold mapping
 */
function generateTapHoldMapping(mapping: KeyMapping): string {
  if (!mapping.tapHold) {
    throw new Error(
      `Tap-hold mapping missing tapHold config at line ${mapping.line}`
    );
  }

  const sourceKey = ensureVKPrefix(mapping.sourceKey);
  const { tapAction, holdAction, thresholdMs } = mapping.tapHold;
  return `tap_hold("${sourceKey}", "${ensureVKPrefix(
    tapAction
  )}", "${ensureVKPrefix(holdAction)}", ${thresholdMs});`;
}

/**
 * Generate macro mapping
 */
function generateMacroMapping(mapping: KeyMapping): string {
  if (!mapping.macro) {
    throw new Error(
      `Macro mapping missing macro config at line ${mapping.line}`
    );
  }

  const sourceKey = ensureVKPrefix(mapping.sourceKey);
  const { keys, delayMs } = mapping.macro;
  const keysStr = keys.map((k) => `"${ensureVKPrefix(k)}"`).join(', ');

  if (delayMs !== undefined) {
    return `macro("${sourceKey}", [${keysStr}], ${delayMs});`;
  }

  return `macro("${sourceKey}", [${keysStr}]);`;
}

/**
 * Generate layer switch mapping
 */
function generateLayerSwitchMapping(mapping: KeyMapping): string {
  if (!mapping.layerSwitch) {
    throw new Error(
      `Layer switch mapping missing layerSwitch config at line ${mapping.line}`
    );
  }

  const sourceKey = ensureVKPrefix(mapping.sourceKey);
  const { layerId, mode } = mapping.layerSwitch;

  if (mode) {
    return `layer_switch("${sourceKey}", "${layerId}", "${mode}");`;
  }

  return `layer_switch("${sourceKey}", "${layerId}");`;
}

/**
 * Get comments that appear within a specific line range
 */
function getCommentsForSection(
  comments: Comment[],
  startLine: number,
  endLine: number
): Comment[] {
  return comments.filter((c) => c.line >= startLine && c.line < endLine);
}

/**
 * Get the line number of the first device block, or a large number if no device blocks
 */
function getFirstDeviceLineOrEnd(ast: RhaiAST): number {
  if (ast.deviceBlocks.length === 0) {
    return Number.MAX_SAFE_INTEGER;
  }
  return ast.deviceBlocks[0].startLine;
}

/**
 * Format a Rhai script with consistent indentation and spacing
 *
 * This is a utility function that can re-format existing Rhai code
 * to match project standards. It works by parsing and regenerating.
 *
 * Note: This function is now implemented in rhaiFormatter.ts to avoid
 * circular dependencies. Use that module for formatting functionality.
 *
 * @deprecated Use formatRhaiScript from rhaiFormatter.ts instead
 * @param script - Rhai script to format
 * @param options - Formatting options
 * @returns Formatted script
 */
export function formatRhaiScript(
  _script: string,
  _options?: FormatOptions
): string {
  // This function is deprecated - use rhaiFormatter.formatRhaiScript instead
  throw new Error(
    'formatRhaiScript has been moved to rhaiFormatter.ts. Please import from there instead.'
  );
}
