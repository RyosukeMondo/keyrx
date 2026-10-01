/**
 * parse -> generate -> parse round trip over every shipped profile (examples,
 * daemon templates, compiler examples). Guards the Save path that rewrites a
 * hand-written profile: it must emit only key names the parser accepts and
 * must not corrupt constructs the visual editor does not model.
 */
import { readdirSync, readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { describe, expect, it } from 'vitest';
import { generateRhaiScript } from './rhaiCodeGen';
import { patchRhaiScript } from './rhaiPatch';
import { parseRhaiScript, type KeyMapping, type RhaiAST } from './rhaiParser';
import { canonicalDslKeyName, isDslKeyName } from './dslKeys';

const ROOT = resolve(process.cwd(), '..');
const DIRS = ['examples', 'keyrx_daemon/templates', 'keyrx_compiler/examples'];

const files = DIRS.flatMap((dir) =>
  readdirSync(resolve(ROOT, dir))
    .filter((f) => f.endsWith('.rhai'))
    .map((f) => ({ name: `${dir}/${f}`, path: resolve(ROOT, dir, f) }))
);

const parse = (src: string): RhaiAST => {
  const r = parseRhaiScript(src);
  if (!r.success || !r.ast) throw new Error(r.error?.message);
  return r.ast;
};

/** Mappings with line numbers removed and key names canonicalised. */
function shape(ast: RhaiAST): unknown {
  const key = (k: string) => canonicalDslKeyName(k) ?? k.replace(/^VK_/, '');
  const norm = (m: KeyMapping) => ({
    type: m.type,
    src: key(m.sourceKey),
    target: m.targetKey && key(m.targetKey),
    tapHold: m.tapHold && {
      tap: key(m.tapHold.tapAction),
      hold: key(m.tapHold.holdAction),
      ms: m.tapHold.thresholdMs,
    },
    macro: m.macro && { ...m.macro, keys: m.macro.keys.map(key) },
    layerSwitch: m.layerSwitch,
  });
  return ast.deviceBlocks.map((b) => ({
    pattern: b.pattern,
    maps: b.mappings.map(norm),
    layers: b.layers.map((l) => ({
      m: l.modifiers,
      maps: l.mappings.map(norm),
    })),
  }));
}

/** Every key-like operand written into generated code. */
function emittedKeys(code: string): string[] {
  const out: string[] = [];
  for (const m of code.matchAll(/"VK_([^"]+)"/g)) out.push(m[1]);
  return out.filter((k) => !/^(MD|LK)_/.test(k));
}

describe('profile round trip', () => {
  it('finds profiles to test', () => {
    expect(files.length).toBeGreaterThanOrEqual(10);
  });

  describe.each(files)('$name', ({ path }) => {
    const source = readFileSync(path, 'utf8');
    const ast = parse(source);

    it('patching with an unchanged AST is byte-identical', () => {
      expect(patchRhaiScript(source, ast)).toBe(source);
    });

    it('parse -> generate -> parse preserves every modelled mapping', () => {
      const generated = generateRhaiScript(ast);
      expect(shape(parse(generated))).toEqual(shape(ast));
    });

    it('generated code only names keys the parser accepts', () => {
      for (const key of emittedKeys(generateRhaiScript(ast))) {
        expect(isDslKeyName(key), `unknown key name ${key}`).toBe(true);
      }
    });
  });
});
