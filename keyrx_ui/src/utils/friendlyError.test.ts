import { describe, it, expect } from 'vitest';
import {
  editDistance,
  friendlyErrorMessage,
  userFacingError,
} from './errorUtils';

// Real daemon text captured from set_profile_config on a scratch daemon.
const UNKNOWN_KEY =
  "Failed to set profile config: Compilation error: Compilation failed: /tmp/kxc/cfg/profiles/demo.rhai.tmp:2:2: Syntax error: Runtime error: Invalid 'from' key: Unknown key name: 'Foo'\n\nDid you mean one of these?\n  - F\n  - O\n  - F1\n (line 2, position 2)\n\nHelp: Check your Rhai script syntax at the indicated location.";
const SYNTAX =
  "Failed to set profile config: Compilation error: Compilation failed: /tmp/kxc/cfg/profiles/demo.rhai.tmp:2:13: Syntax error: Syntax error: Expecting ',' to separate the arguments to function call 'map' (line 2, position 13)\n\nHelp: Check your Rhai script syntax at the indicated location.";

describe('editDistance', () => {
  it('computes Levenshtein distance case-insensitively', () => {
    expect(editDistance('Foo', 'foo')).toBe(0);
    expect(editDistance('Foo', 'F')).toBe(2);
    expect(editDistance('CapsLok', 'CapsLock')).toBe(1);
    expect(editDistance('', 'abc')).toBe(3);
  });
});

describe('friendlyErrorMessage', () => {
  it('never leaks temp or absolute paths', () => {
    for (const raw of [UNKNOWN_KEY, SYNTAX]) {
      const out = friendlyErrorMessage(raw);
      expect(out).not.toMatch(/\/tmp|\.tmp|\.rhai|\/home|[A-Za-z]:\\/);
      expect(out).not.toMatch(/Failed to set profile config|Compilation/);
    }
  });

  it('reports the line and drops implausible "Did you mean" suggestions', () => {
    expect(friendlyErrorMessage(UNKNOWN_KEY)).toBe(
      "Line 2: Syntax error: Invalid 'from' key: Unknown key name: 'Foo'"
    );
  });

  it('keeps suggestions that are a small edit away', () => {
    const raw =
      "Unknown key name: 'CapsLok'\n\nDid you mean one of these?\n  - CapsLock\n  - Caps\n  - Left";
    expect(friendlyErrorMessage(raw)).toBe(
      "Unknown key name: 'CapsLok' Did you mean CapsLock?"
    );
  });

  it('collapses duplicated syntax prefixes and removes position noise', () => {
    expect(friendlyErrorMessage(SYNTAX)).toBe(
      "Line 2: Syntax error: Expecting ',' to separate the arguments to function call 'map'"
    );
  });

  it('normalizes the WASM validator wording too', () => {
    expect(
      friendlyErrorMessage(
        "Parse error: Runtime error: Invalid 'from' key: Unknown key name: 'Foo'"
      )
    ).toBe("Syntax error: Invalid 'from' key: Unknown key name: 'Foo'");
  });

  it('removes Windows paths too', () => {
    expect(
      friendlyErrorMessage(
        'Compilation failed: C:\\Users\\me\\AppData\\x\\p.rhai.tmp:7:1: boom'
      )
    ).toBe('Line 7: boom');
  });

  it('userFacingError extracts then sanitizes', () => {
    expect(userFacingError(new Error(SYNTAX), 'x')).toMatch(/^Line 2:/);
    expect(userFacingError(null, 'Operation failed')).toBe('Operation failed');
  });
});
