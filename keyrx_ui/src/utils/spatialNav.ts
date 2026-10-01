/**
 * Arrow-key navigation over a 2D key layout (roving tabindex support).
 *
 * Pure geometry so it is testable without a DOM: given the key rectangles in
 * layout units and the index of the focused key, return the index of the key
 * that should receive focus next.
 */

export interface KeyRect {
  x: number;
  y: number;
  w: number;
  h: number;
}

export type NavKey =
  | 'ArrowLeft'
  | 'ArrowRight'
  | 'ArrowUp'
  | 'ArrowDown'
  | 'Home'
  | 'End';

/** Keys closer than this (in layout units) vertically count as the same row. */
const ROW_TOLERANCE = 0.45;

const cx = (k: KeyRect) => k.x + k.w / 2;
const cy = (k: KeyRect) => k.y + k.h / 2;

function sameRow(a: KeyRect, b: KeyRect): boolean {
  return Math.abs(cy(a) - cy(b)) < ROW_TOLERANCE;
}

function horizontal(keys: KeyRect[], from: number, dir: 1 | -1): number {
  const cur = keys[from];
  let best = from;
  let bestDist = Infinity;
  keys.forEach((k, i) => {
    if (i === from || !sameRow(cur, k)) return;
    const dx = (cx(k) - cx(cur)) * dir;
    if (dx > 0 && dx < bestDist) {
      best = i;
      bestDist = dx;
    }
  });
  return best;
}

function vertical(keys: KeyRect[], from: number, dir: 1 | -1): number {
  const cur = keys[from];
  const candidates = keys
    .map((k, i) => ({
      i,
      dy: (cy(k) - cy(cur)) * dir,
      dx: Math.abs(cx(k) - cx(cur)),
    }))
    .filter((c) => c.dy > ROW_TOLERANCE);
  if (candidates.length === 0) return from;
  const nearestRow = Math.min(...candidates.map((c) => c.dy));
  const inRow = candidates.filter((c) => c.dy < nearestRow + ROW_TOLERANCE);
  return inRow.reduce((a, b) => (b.dx < a.dx ? b : a)).i;
}

function rowEdge(keys: KeyRect[], from: number, dir: 1 | -1): number {
  const cur = keys[from];
  let best = from;
  keys.forEach((k, i) => {
    if (sameRow(cur, k) && (cx(k) - cx(keys[best])) * dir > 0) best = i;
  });
  return best;
}

/** Index of the key to focus after `key` is pressed on `keys[from]`. */
export function nextKeyIndex(
  keys: KeyRect[],
  from: number,
  key: NavKey
): number {
  if (keys.length === 0 || from < 0 || from >= keys.length) return from;
  switch (key) {
    case 'ArrowLeft':
      return horizontal(keys, from, -1);
    case 'ArrowRight':
      return horizontal(keys, from, 1);
    case 'ArrowUp':
      return vertical(keys, from, -1);
    case 'ArrowDown':
      return vertical(keys, from, 1);
    case 'Home':
      return rowEdge(keys, from, -1);
    case 'End':
      return rowEdge(keys, from, 1);
  }
}

export const NAV_KEYS: ReadonlySet<string> = new Set([
  'ArrowLeft',
  'ArrowRight',
  'ArrowUp',
  'ArrowDown',
  'Home',
  'End',
]);
