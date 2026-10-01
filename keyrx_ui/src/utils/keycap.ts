/**
 * Keycap geometry, styling and text helpers for the SVG keyboard.
 */
import type { KeyMapping } from '@/types';
import { formatKeyLabel, friendlyKeyName } from '@/utils/keyNames';

// Constants for SVG rendering
export const UNIT_SIZE = 54; // pixels per key unit (1u)
export const KEY_GAP = 2; // gap between keys
export const KEY_RADIUS = 6; // border radius
export const KEY_INSET = 3; // 3D effect inset

export interface SVGKey {
  code: string;
  label: string;
  x: number;
  y: number;
  w: number;
  h: number;
  /** Special shape: 'iso-enter' | 'standard' */
  shape?: 'iso-enter' | 'standard';
}

/** Minimum on-screen size (px) the keyboard may be scaled down to. */
export const MIN_SCALE = 0.6;
export const LABEL_MAX = 13;
export const LABEL_MIN = 10;
export const MAPPING_MAX = 14;
export const MAPPING_MIN = 10;
export const MONO_EM = 0.62; // monospace advance width in em

/** Fit `text` into `width` px: shrink to `min`, then truncate with an ellipsis. */
export function fitText(text: string, width: number, max: number, min: number) {
  if (!text) return { text, size: max };
  const size = Math.max(min, Math.min(max, width / (text.length * MONO_EM)));
  const maxChars = Math.max(1, Math.floor(width / (size * MONO_EM)));
  return {
    text: text.length > maxChars ? `${text.slice(0, maxChars - 1)}…` : text,
    size,
  };
}

const spoken = (value: string | undefined) =>
  value ? friendlyKeyName(value) : '';

/**
 * Plain-language description of what a key does, for tooltips and screen
 * readers. Uses the names printed on the keys ("Space", "Numpad 3",
 * "Backtick"), not the DSL spellings ("Num3", "Grave").
 */
export function describeKey(code: string, mapping?: KeyMapping): string {
  const name = friendlyKeyName(code);
  if (!mapping) return `${name}, not remapped`;
  switch (mapping.type) {
    case 'simple':
      return `${name}, acts as ${spoken(mapping.tapAction)}`;
    case 'tap_hold':
      return `${name}, tap for ${spoken(mapping.tapAction)}, hold for ${spoken(mapping.holdAction)} after ${mapping.threshold ?? 200} ms`;
    case 'macro':
      return `${name}, runs a macro of ${mapping.macroSteps?.length ?? 0} steps`;
    case 'layer_switch':
      return `${name}, switches to layer ${mapping.targetLayer ?? ''}`;
    default:
      return `${name}, not remapped`;
  }
}

/**
 * Generate SVG path for ISO Enter key (L-shaped)
 * The ISO Enter spans 2 rows with different widths
 */
export function generateISOEnterPath(
  x: number,
  y: number,
  w: number,
  h: number
): string {
  const px = x * UNIT_SIZE;
  const py = y * UNIT_SIZE;
  const topWidth = 1.5 * UNIT_SIZE - KEY_GAP; // Top part is 1.5u
  const bottomWidth = w * UNIT_SIZE - KEY_GAP; // Bottom uses actual width
  const halfHeight = (h * UNIT_SIZE) / 2 - KEY_GAP / 2;
  const r = KEY_RADIUS;

  // L-shape path (clockwise from top-left)
  // Note: The top part extends further left than the bottom
  const leftOffset = topWidth - bottomWidth;

  return `
    M ${px + leftOffset + r} ${py}
    L ${px + topWidth - r} ${py}
    Q ${px + topWidth} ${py} ${px + topWidth} ${py + r}
    L ${px + topWidth} ${py + h * UNIT_SIZE - KEY_GAP - r}
    Q ${px + topWidth} ${py + h * UNIT_SIZE - KEY_GAP} ${px + topWidth - r} ${
      py + h * UNIT_SIZE - KEY_GAP
    }
    L ${px + leftOffset + r} ${py + h * UNIT_SIZE - KEY_GAP}
    Q ${px + leftOffset} ${py + h * UNIT_SIZE - KEY_GAP} ${px + leftOffset} ${
      py + h * UNIT_SIZE - KEY_GAP - r
    }
    L ${px + leftOffset} ${py + halfHeight + r}
    Q ${px + leftOffset} ${py + halfHeight} ${px + leftOffset - r} ${
      py + halfHeight
    }
    L ${px + r} ${py + halfHeight}
    Q ${px} ${py + halfHeight} ${px} ${py + halfHeight - r}
    L ${px} ${py + r}
    Q ${px} ${py} ${px + r} ${py}
    Z
  `.trim();
}

/**
 * Generate SVG path for standard rectangular key
 */
export function generateRectPath(
  x: number,
  y: number,
  w: number,
  h: number
): string {
  const px = x * UNIT_SIZE;
  const py = y * UNIT_SIZE;
  const width = w * UNIT_SIZE - KEY_GAP;
  const height = h * UNIT_SIZE - KEY_GAP;
  const r = KEY_RADIUS;

  return `
    M ${px + r} ${py}
    L ${px + width - r} ${py}
    Q ${px + width} ${py} ${px + width} ${py + r}
    L ${px + width} ${py + height - r}
    Q ${px + width} ${py + height} ${px + width - r} ${py + height}
    L ${px + r} ${py + height}
    Q ${px} ${py + height} ${px} ${py + height - r}
    L ${px} ${py + r}
    Q ${px} ${py} ${px + r} ${py}
    Z
  `.trim();
}

/**
 * Get styling based on mapping type
 */
export function getMappingStyle(mapping?: KeyMapping) {
  if (!mapping) {
    return {
      fill: '#334155', // slate-700
      stroke: '#475569', // slate-600
      strokeDasharray: '4 2',
    };
  }

  switch (mapping.type) {
    case 'simple':
      return { fill: '#334155', stroke: '#22c55e', strokeDasharray: 'none' }; // green-500
    case 'tap_hold':
      return {
        fill: 'rgba(127, 29, 29, 0.15)',
        stroke: '#ef4444',
        strokeDasharray: 'none',
      }; // red-500
    case 'macro':
      return {
        fill: 'rgba(88, 28, 135, 0.15)',
        stroke: '#a855f7',
        strokeDasharray: 'none',
      }; // purple-500
    case 'layer_switch':
      return {
        fill: 'rgba(113, 63, 18, 0.15)',
        stroke: '#eab308',
        strokeDasharray: 'none',
      }; // yellow-500
    default:
      return { fill: '#334155', stroke: '#475569', strokeDasharray: 'none' };
  }
}

/**
 * Get mapping display text
 */
export function getRemapText(mapping?: KeyMapping): string {
  if (!mapping) return '';

  switch (mapping.type) {
    case 'simple':
      return formatKeyLabel(mapping.tapAction || '');
    case 'tap_hold': {
      const tap = formatKeyLabel(mapping.tapAction || '');
      const hold = formatKeyLabel(mapping.holdAction || '');
      return `${tap}/${hold}`;
    }
    case 'macro':
      return '⚡';
    case 'layer_switch':
      return mapping.targetLayer?.replace(/^MD_/, 'L') || '';
    default:
      return '';
  }
}

/**
 * Mapping type indicator icon
 */
export function getMappingIcon(type?: string): string {
  switch (type) {
    case 'simple':
      return '→';
    case 'tap_hold':
      return '↕';
    case 'macro':
      return '⚡';
    case 'layer_switch':
      return '⇄';
    default:
      return '';
  }
}

export function getMappingIconColor(type?: string): string {
  switch (type) {
    case 'simple':
      return '#4ade80'; // green-400
    case 'tap_hold':
      return '#f87171'; // red-400
    case 'macro':
      return '#c084fc'; // purple-400
    case 'layer_switch':
      return '#facc15'; // yellow-400
    default:
      return '#94a3b8'; // slate-400
  }
}
