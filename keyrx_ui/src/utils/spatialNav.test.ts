import { describe, it, expect } from 'vitest';
import { nextKeyIndex, type KeyRect } from './spatialNav';

// Row 0: A B C      Row 1: D(wide, 1.5u)  E
const keys: KeyRect[] = [
  { x: 0, y: 0, w: 1, h: 1 }, // 0 A
  { x: 1, y: 0, w: 1, h: 1 }, // 1 B
  { x: 2, y: 0, w: 1, h: 1 }, // 2 C
  { x: 0, y: 1, w: 1.5, h: 1 }, // 3 D
  { x: 1.5, y: 1, w: 1, h: 1 }, // 4 E
];

describe('nextKeyIndex', () => {
  it('moves left/right within a row and stops at the edges', () => {
    expect(nextKeyIndex(keys, 0, 'ArrowRight')).toBe(1);
    expect(nextKeyIndex(keys, 1, 'ArrowRight')).toBe(2);
    expect(nextKeyIndex(keys, 2, 'ArrowRight')).toBe(2);
    expect(nextKeyIndex(keys, 1, 'ArrowLeft')).toBe(0);
    expect(nextKeyIndex(keys, 0, 'ArrowLeft')).toBe(0);
  });

  it('moves up/down to the nearest key in the adjacent row', () => {
    expect(nextKeyIndex(keys, 0, 'ArrowDown')).toBe(3);
    expect(nextKeyIndex(keys, 2, 'ArrowDown')).toBe(4);
    // E sits exactly between B and C: ties resolve to the left key
    expect(nextKeyIndex(keys, 4, 'ArrowUp')).toBe(1);
    expect(nextKeyIndex(keys, 0, 'ArrowUp')).toBe(0);
  });

  it('Home/End jump to the row edges', () => {
    expect(nextKeyIndex(keys, 1, 'Home')).toBe(0);
    expect(nextKeyIndex(keys, 1, 'End')).toBe(2);
    expect(nextKeyIndex(keys, 4, 'Home')).toBe(3);
  });

  it('is a no-op for empty or out-of-range input', () => {
    expect(nextKeyIndex([], 0, 'ArrowRight')).toBe(0);
    expect(nextKeyIndex(keys, 99, 'ArrowRight')).toBe(99);
  });
});
