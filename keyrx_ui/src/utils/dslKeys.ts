/**
 * Canonical key names for the Rhai DSL.
 *
 * The parser (keyrx_core `parse_key_name`) is case-sensitive: "Space" and
 * "SPC" are keys, "SPACE" is an error. Everything the UI writes into a profile
 * therefore goes through `canonicalDslKeyName()`, which maps any spelling the
 * UI or older code may produce (upper-cased, QMK, legacy long names) onto the
 * one spelling the parser accepts. The table itself lives in
 * `data/dslKeyTable.ts` and is checked against the Rust source by a test.
 */
import { DSL_KEY_TABLE } from '@/data/dslKeyTable';

/** Spellings older UI code produced that are not in the parser table. */
const LEGACY_ALIASES: Record<string, string> = {
  leftshift: 'LShift',
  rightshift: 'RShift',
  leftcontrol: 'LCtrl',
  rightcontrol: 'RCtrl',
  leftctrl: 'LCtrl',
  rightctrl: 'RCtrl',
  leftalt: 'LAlt',
  rightalt: 'RAlt',
  leftmeta: 'LMeta',
  rightmeta: 'RMeta',
  leftwin: 'LMeta',
  rightwin: 'RMeta',
};

const EXACT = new Map<string, string>();
const FOLDED = new Map<string, string>();

for (const row of DSL_KEY_TABLE) {
  const [canonical] = row;
  for (const name of row) {
    EXACT.set(name, canonical);
    const folded = name.toLowerCase();
    if (!FOLDED.has(folded)) FOLDED.set(folded, canonical);
  }
}

/** True when the keyrx parser accepts `name` verbatim (no `VK_` prefix). */
export function isDslKeyName(name: string): boolean {
  return EXACT.has(name);
}

/** Every spelling the parser accepts. */
export function allDslKeyNames(): string[] {
  return [...EXACT.keys()];
}

/**
 * Canonical parser spelling of `raw` (with or without `VK_`), or `null` when
 * the name is not a key at all (layer ids like `MD_00`, helpers, typos).
 * `"SPACE"`/`"space"`/`"VK_SPACE"` -> `"Space"`, `"LEFTSHIFT"` -> `"LShift"`.
 */
export function canonicalDslKeyName(raw: string): string | null {
  const name = raw.replace(/^VK_/, '');
  const exact = EXACT.get(name);
  if (exact) return exact;
  const folded = name.toLowerCase();
  return FOLDED.get(folded) ?? LEGACY_ALIASES[folded] ?? null;
}

/**
 * `VK_`-prefixed canonical form for writing into a profile. Unknown names are
 * returned unchanged (still `VK_`-prefixed) so the daemon's own error message,
 * not silent rewriting, tells the user about a typo.
 */
export function toDslKeyRef(raw: string): string {
  const bare = raw.replace(/^VK_/, '');
  return `VK_${canonicalDslKeyName(bare) ?? bare}`;
}
