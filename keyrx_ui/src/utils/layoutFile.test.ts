import { describe, expect, it } from 'vitest';
import {
  MAX_LAYOUT_BYTES,
  bytesToBase64,
  detectLayoutFormat,
  isValidProfileName,
  layoutFileProblem,
  profileNameFromFile,
  uniqueProfileName,
} from './layoutFile';

describe('layoutFile', () => {
  it('detects .krx and .rhai case-insensitively and rejects the rest', () => {
    expect(detectLayoutFormat('a.krx')).toBe('krx');
    expect(detectLayoutFormat('A.RHAI')).toBe('rhai');
    expect(detectLayoutFormat('a.txt')).toBeNull();
    expect(detectLayoutFormat('krx')).toBeNull();
  });

  it('reports why a file cannot be loaded', () => {
    expect(layoutFileProblem({ name: 'a.krx', size: 10 })).toBeNull();
    expect(layoutFileProblem({ name: 'a.exe', size: 10 })).toBe('type');
    expect(layoutFileProblem({ name: 'a.krx', size: 0 })).toBe('empty');
    expect(layoutFileProblem({ name: 'a.rhai', size: MAX_LAYOUT_BYTES + 1 })).toBe(
      'tooLarge'
    );
    expect(layoutFileProblem({ name: 'a.rhai', size: MAX_LAYOUT_BYTES })).toBeNull();
  });

  it('derives a daemon-valid profile name from any file name', () => {
    for (const file of [
      'my layout.v2.krx',
      '日本語.krx',
      '../../etc/passwd.rhai',
      '.krx',
      `${'x'.repeat(200)}.rhai`,
    ]) {
      expect(isValidProfileName(profileNameFromFile(file)), file).toBe(true);
    }
    expect(profileNameFromFile('my layout.v2.krx')).toBe('my-layout-v2');
  });

  it('suggests the first free name, keeping it within the length limit', () => {
    expect(uniqueProfileName('a', ['b'])).toBe('a');
    expect(uniqueProfileName('a', ['a', 'a-2'])).toBe('a-3');
    const long = 'x'.repeat(64);
    const next = uniqueProfileName(long, [long]);
    expect(next).not.toBe(long);
    expect(isValidProfileName(next)).toBe(true);
  });

  it('base64-encodes large byte arrays without overflowing the stack', () => {
    expect(bytesToBase64(new Uint8Array([104, 105]))).toBe('aGk=');
    const big = new Uint8Array(MAX_LAYOUT_BYTES).fill(65);
    expect(atob(bytesToBase64(big))).toHaveLength(MAX_LAYOUT_BYTES);
  });
});
