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

/** Neighbouring keys this close (layout units) side by side still count as aligned. */
const SIDE_TOLERANCE = 0.1;

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

/** Horizontal overlap (>0) or gap (<0) between two keys, in layout units. */
function overlap(a: KeyRect, b: KeyRect): number {
  return Math.min(a.x + a.w, b.x + b.w) - Math.max(a.x, b.x);
}

/**
 * Rows are told apart by key edges, not centres: a 2u-tall numpad "+" has its
 * centre between two rows and used to win "nearest row" for every key on the
 * main block, so ArrowDown jumped from Tab/Q/W... straight into the numpad.
 * Among the keys of the next row, the one whose centre is closest wins; keys that
 * are not (nearly) directly below do not qualify at all.
 */
function vertical(keys: KeyRect[], from: number, dir: 1 | -1): number {
  const cur = keys[from];
  // Edge of a key that faces the key we leave: its top going down, bottom going up.
  const facing = (k: KeyRect) => (dir === 1 ? k.y : k.y + k.h);
  const limit = dir === 1 ? cur.y + cur.h : cur.y;
  // Only keys that are (nearly) directly above/below count; with none, stay
  // put rather than jump to another cluster (main block <-> numpad).
  const candidates = keys
    .map((k, i) => ({ i, k, edge: facing(k) }))
    .filter(
      (c) =>
        c.i !== from &&
        overlap(cur, c.k) > -SIDE_TOLERANCE &&
        (dir === 1
          ? c.edge >= limit - ROW_TOLERANCE
          : c.edge <= limit + ROW_TOLERANCE)
    );
  if (candidates.length === 0) return from;
  const nearest =
    dir === 1
      ? Math.min(...candidates.map((c) => c.edge))
      : Math.max(...candidates.map((c) => c.edge));
  const inRow = candidates.filter(
    (c) => Math.abs(c.edge - nearest) < ROW_TOLERANCE
  );
  const dx = (c: (typeof inRow)[number]) => Math.abs(cx(c.k) - cx(cur));
  return inRow.reduce((a, b) => (dx(b) < dx(a) ? b : a)).i;
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
