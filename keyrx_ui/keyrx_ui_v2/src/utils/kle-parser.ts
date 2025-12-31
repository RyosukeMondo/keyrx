/**
 * KLE (Keyboard Layout Editor) JSON Parser
 * Converts layout data into renderable KeyButton objects with grid positioning
 */

export interface KLEKey {
  code: string;
  label: string;
  x: number;
  y: number;
  w?: number;
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

/**
 * Parse KLE JSON data into KeyButton objects for grid layout
 * @param kleData - Keyboard layout data from JSON
 * @returns Array of KeyButton objects with grid positioning
 */
export function parseKLEJson(kleData: KLEData): KeyButton[] {
  return kleData.keys.map((key) => ({
    keyCode: key.code,
    label: key.label,
    gridRow: Math.floor(key.y) + 1, // 1-indexed for CSS Grid
    gridColumn: Math.floor(key.x) + 1,
    gridColumnSpan: Math.ceil(key.w || 1),
    width: key.w || 1,
  }));
}
