import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { describe, expect, it } from 'vitest';
import { DSL_KEY_TABLE } from '@/data/dslKeyTable';
import {
  allDslKeyNames,
  canonicalDslKeyName,
  isDslKeyName,
  toDslKeyRef,
} from './dslKeys';

/** Rows of `parse_key_name` in the Rust parser: the single source of truth. */
function rustKeyRows(): string[][] {
  const path = resolve(process.cwd(), '../keyrx_core/src/parser/validators.rs');
  const src = readFileSync(path, 'utf8');
  const body = src.slice(src.indexOf('pub fn parse_key_name'));
  const rows: string[][] = [];
  for (const line of body.split('\n')) {
    const m = line.match(/^\s*((?:"[^"]+"\s*\|?\s*)+)=>\s*KeyCode::/);
    if (m) rows.push([...m[1].matchAll(/"([^"]+)"/g)].map((x) => x[1]));
  }
  return rows;
}

describe('dslKeyTable drift', () => {
  it('matches parse_key_name in keyrx_core exactly', () => {
    expect(DSL_KEY_TABLE.map((r) => [...r])).toEqual(rustKeyRows());
  });
});

describe('canonicalDslKeyName', () => {
  it.each([
    ['SPACE', 'Space'],
    ['VK_SPACE', 'Space'],
    ['space', 'Space'],
    ['Space', 'Space'],
    ['SPC', 'Space'],
    ['CAPSLOCK', 'CapsLock'],
    ['LEFTSHIFT', 'LShift'],
    ['LEFTCONTROL', 'LCtrl'],
    ['PAGEDOWN', 'PageDown'],
    ['Numpad3', 'Numpad3'],
    ['1', 'Num1'],
    ['a', 'A'],
    ['全角', 'Zenkaku'],
  ])('%s -> %s', (input, expected) => {
    expect(canonicalDslKeyName(input)).toBe(expected);
  });

  it('returns null for things that are not keys', () => {
    expect(canonicalDslKeyName('MD_00')).toBeNull();
    expect(canonicalDslKeyName('NOPE')).toBeNull();
  });

  it('only ever yields names the parser accepts verbatim', () => {
    for (const name of allDslKeyNames()) {
      const canonical = canonicalDslKeyName(name.toUpperCase());
      if (canonical) expect(isDslKeyName(canonical)).toBe(true);
    }
  });

  it('toDslKeyRef keeps unknown names so the compiler can report them', () => {
    expect(toDslKeyRef('SPACE')).toBe('VK_Space');
    expect(toDslKeyRef('VK_Nope')).toBe('VK_Nope');
  });
});
