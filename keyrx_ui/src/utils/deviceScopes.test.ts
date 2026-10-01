import { describe, it, expect } from 'vitest';
import { buildScopes, scopeMatchesBlock } from './deviceScopes';

const kbd = {
  id: 'path-/dev/input/event3',
  name: 'USB Keyboard',
  serial: null,
  path: '/dev/input/event3',
};
const pad = {
  id: 'path-/dev/input/event7',
  name: 'A11y Numpad',
  serial: 'SN7',
  path: '/dev/input/event7',
};

describe('buildScopes', () => {
  it('shows a glob pattern as one pattern scope listing the devices it matches', () => {
    const scopes = buildScopes(['a11y*'], [kbd, pad]);
    expect(scopes.map((s) => s.id)).toEqual(['pattern-a11y*', kbd.id]);
    expect(scopes[0]).toMatchObject({
      name: 'a11y*',
      pattern: 'a11y*',
      connected: true,
      matchedNames: ['A11y Numpad'],
    });
  });

  it('does not list a device twice when a pattern already covers it', () => {
    const scopes = buildScopes(['usb*'], [kbd, pad]);
    expect(scopes.filter((s) => s.id === kbd.id)).toHaveLength(0);
    expect(scopes.map((s) => s.pattern)).toEqual(['usb*', undefined]);
  });

  it('keeps an unmatched pattern as a disconnected scope', () => {
    const [scope] = buildScopes(['nothing*'], [kbd]);
    expect(scope).toMatchObject({
      id: 'disconnected-nothing*',
      connected: false,
      matchedNames: [],
    });
  });

  it('resolves an exact (case-insensitive) name to the device itself', () => {
    const [scope] = buildScopes(['usb keyboard'], [kbd]);
    expect(scope).toMatchObject({ id: kbd.id, pattern: 'usb keyboard' });
  });

  it('never lists the wildcard-only pattern', () => {
    expect(buildScopes(['*'], []).length).toBe(0);
  });
});

describe('scopeMatchesBlock', () => {
  it('matches pattern scopes by pattern and plain devices by glob rule', () => {
    const [patternScope] = buildScopes(['a11y*'], [pad]);
    expect(scopeMatchesBlock(patternScope, [pad], 'a11y*')).toBe(true);
    expect(scopeMatchesBlock(patternScope, [pad], 'other')).toBe(false);

    const [plain] = buildScopes([], [kbd]);
    expect(scopeMatchesBlock(plain, [kbd], 'usb*')).toBe(true);
    expect(scopeMatchesBlock(plain, [kbd], '*mouse*')).toBe(false);
  });
});
