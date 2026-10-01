/**
 * Device pattern matching for `device_start(pattern)` / `when_device(pattern)`.
 *
 * This is a line-for-line port of `keyrx_core::runtime::device_pattern`, the
 * single rule the daemon uses. It exists in TypeScript only because the WASM
 * module does not export it yet (follow-up: export `device_pattern_matches`
 * from keyrx_core::wasm and delete this file). The shared test vectors in
 * `devicePattern.test.ts` are copied from the Rust unit tests so drift is
 * caught on both sides.
 *
 * `*` is a wildcard; matching is ASCII case-insensitive; a device matches if
 * any of its identities (id, name, path, serial) matches.
 */

const asciiLower = (s: string): string =>
  s.replace(/[A-Z]/g, (c) => c.toLowerCase());

/** Does `pattern` match the single identity `id`? */
export function matchesPattern(id: string, pattern: string): boolean {
  const target = asciiLower(id);
  const pat = asciiLower(pattern);
  if (!pat.includes('*')) return target === pat;

  const parts = pat.split('*');
  if (parts.length === 2) {
    const [prefix, suffix] = parts;
    if (prefix === '' && suffix === '') return true;
    if (prefix === '') return target.endsWith(suffix);
    if (suffix === '') return target.startsWith(prefix);
    return target.startsWith(prefix) && target.endsWith(suffix);
  }
  if (parts.length === 3) {
    const [prefix, middle, suffix] = parts;
    if (prefix === '' && suffix === '') return target.includes(middle);
    return (
      target.startsWith(prefix) &&
      target.endsWith(suffix) &&
      target.includes(middle)
    );
  }
  let remaining = target;
  for (let i = 0; i < parts.length; i++) {
    const part = parts[i];
    if (part === '') continue;
    if (i === 0) {
      if (!remaining.startsWith(part)) return false;
      remaining = remaining.slice(part.length);
    } else if (i === parts.length - 1) {
      if (!remaining.endsWith(part)) return false;
    } else {
      const pos = remaining.indexOf(part);
      if (pos < 0) return false;
      remaining = remaining.slice(pos + part.length);
    }
  }
  return true;
}

/** Does `pattern` match any of the device's identities? */
export function matchesAnyIdentity(
  identities: ReadonlyArray<string | null | undefined>,
  pattern: string
): boolean {
  return identities.some(
    (id) => typeof id === 'string' && id !== '' && matchesPattern(id, pattern)
  );
}

/** True when the pattern contains a wildcard. */
export const isGlobPattern = (pattern: string): boolean =>
  pattern.includes('*');
