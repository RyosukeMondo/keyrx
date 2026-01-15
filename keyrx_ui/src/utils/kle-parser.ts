/**
 * KLE (Keyboard Layout Editor) JSON Parser
 * Converts layout data into renderable KeyButton objects with grid positioning
 * Also supports SVG rendering with shape detection for ISO Enter, etc.
 */

export interface KLEKey {
  code: string;
  label: string;
  x: number;
  y: number;
  w?: number;
  h?: number;
}

export interface KLEData {
  name: string;
  keys: KLEKey[];
}

export interface KeyButton {
  keyCode: string;
  label: string;
  gridRow: number;
  gridColumn: number;
  gridColumnSpan: number;
  width: number;
}

/** Key data for SVG rendering */
export interface SVGKeyData {
  code: string;
  label: string;
  x: number;
  y: number;
  w: number;
  h: number;
  shape: 'iso-enter' | 'standard';
}

/**
 * Convert QMK keycode (KC_*) to Windows Virtual Key code (VK_*)
 * This fixes the issue where layout files use QMK convention but Windows expects VK_ prefix
 *
 * @param code - Keycode from layout file (e.g., "KC_A", "KC_ENTER")
 * @returns Windows Virtual Key code (e.g., "VK_A", "VK_ENTER")
 *
 * @example
 * convertToVirtualKey("KC_A") // "VK_A"
 * convertToVirtualKey("KC_ENTER") // "VK_ENTER"
 * convertToVirtualKey("VK_A") // "VK_A" (already correct)
 */
function convertToVirtualKey(code: string): string {
  // If already using VK_ prefix, return as-is
  if (code.startsWith('VK_')) {
    return code;
  }

  // Convert KC_ prefix to VK_ prefix
  if (code.startsWith('KC_')) {
    return code.replace(/^KC_/, 'VK_');
  }

  // If no prefix, return as-is (shouldn't happen but handle gracefully)
  return code;
}

/**
 * Parse KLE JSON data into KeyButton objects for grid layout
 * @param kleData - Keyboard layout data from JSON
 * @returns Array of KeyButton objects with grid positioning
 */
export function parseKLEJson(kleData: KLEData): KeyButton[] {
  return kleData.keys.map((key) => ({
    keyCode: convertToVirtualKey(key.code),
    label: key.label,
    gridRow: Math.floor(key.y) + 1, // 1-indexed for CSS Grid
    gridColumn: Math.floor(key.x) + 1,
    gridColumnSpan: Math.ceil(key.w || 1),
    width: key.w || 1,
  }));
}

/**
 * Detect if a key should be rendered as ISO Enter
 * ISO Enter: height=2, specific codes (KC_ENT in ISO layouts)
 */
function detectKeyShape(key: KLEKey, layoutName: string): 'iso-enter' | 'standard' {
  const isISOLayout = layoutName.includes('ISO');
  const isEnterKey = key.code === 'KC_ENT' || key.code === 'KC_ENTER';
  const isTallKey = (key.h || 1) >= 2;

  // ISO Enter is only in ISO layouts, on the main Enter key, with height 2
  if (isISOLayout && isEnterKey && isTallKey) {
    return 'iso-enter';
  }

  return 'standard';
}

/**
 * Parse KLE JSON data into SVG-compatible key objects
 * Handles height, shape detection, and special keys like ISO Enter
 *
 * @param kleData - Keyboard layout data from JSON
 * @returns Array of SVGKeyData objects for SVG rendering
 */
export function parseKLEToSVG(kleData: KLEData): SVGKeyData[] {
  return kleData.keys.map((key) => ({
    code: convertToVirtualKey(key.code),
    label: key.label,
    x: key.x,
    y: key.y,
    w: key.w || 1,
    h: key.h || 1,
    shape: detectKeyShape(key, kleData.name),
  }));
}
