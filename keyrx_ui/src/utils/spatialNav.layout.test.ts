import { describe, expect, it } from 'vitest';
import { parseKLEToSVG } from './kle-parser';
import { nextKeyIndex } from './spatialNav';
import ANSI_104 from '../data/layouts/ANSI_104.json';
import JIS_109 from '../data/layouts/JIS_109.json';
import ISO_105 from '../data/layouts/ISO_105.json';

const layouts = { ANSI_104, JIS_109, ISO_105 };

/** x where the numpad cluster starts (gap after the arrow cluster). */
function clusterStart(keys: { x: number }[]): number {
  return Math.max(...keys.map((k) => k.x)) - 3.01;
}

describe.each(Object.entries(layouts))('arrow navigation on %s', (_n, data) => {
  const keys = parseKLEToSVG(data as never);
  const numpadX = clusterStart(keys);

  it('ArrowDown/Up never jumps between the main block and the numpad', () => {
    const offenders: string[] = [];
    keys.forEach((k, i) => {
      for (const dir of ['ArrowDown', 'ArrowUp'] as const) {
        const to = nextKeyIndex(keys, i, dir);
        if (to === i) continue;
        const target = keys[to];
        const fromPad = k.x >= numpadX - 0.01;
        const toPad = target.x >= numpadX - 0.01;
        // main block <-> numpad jumps are wrong when the target has no key
        // directly above/below (horizontal overlap)
        const overlap =
          Math.min(k.x + k.w, target.x + target.w) - Math.max(k.x, target.x);
        if (fromPad !== toPad && overlap <= 0) {
          offenders.push(`${k.code} ${dir} -> ${target.code}`);
        }
      }
    });
    expect(offenders).toEqual([]);
  });

  it('ArrowDown from the space bar stays in its own cluster', () => {
    const space = keys.findIndex((k) => /SPC|SPACE/i.test(k.code));
    const up = nextKeyIndex(keys, space, 'ArrowUp');
    expect(keys[up].x).toBeLessThan(numpadX);
  });
});
