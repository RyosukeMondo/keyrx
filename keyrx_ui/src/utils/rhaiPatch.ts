/**
 * Minimal, source-preserving edits of a Rhai profile.
 *
 * The visual editor only models `map`, `tap_hold`, `macro` and `layer_switch`
 * statements inside device / when blocks. Regenerating the whole file from
 * that model (generateRhaiScript) silently drops everything else: comments,
 * `sequence`, `hold_only`, `home_row`, trailing remarks, blank-line layout and
 * device blocks that are not selected. `patchRhaiScript` instead diffs the
 * new AST against the AST of the current source and edits only the lines of
 * mappings that were added, changed or removed; every other byte is kept.
 *
 * With an unchanged AST the output is byte-identical to the input.
 */
import {
  parseRhaiScript,
  type DeviceBlock,
  type KeyMapping,
  type ModifierLayer,
  type RhaiAST,
} from './rhaiParser';
import {
  generateDeviceBlock,
  generateKeyMapping,
  generateModifierLayer,
  generateRhaiScript,
} from './rhaiCodeGen';
import { canonicalDslKeyName } from './dslKeys';

const STEP = '    ';
const LAYER_OPTS = {
  indentSize: 4,
  maxLineLength: 100,
  blankLinesBetweenDevices: 1,
  blankLinesBetweenSections: 2,
};

/** Line-level edit; `line` is a 0-based index into the source lines. */
interface Edit {
  line: number;
  /** Replace the character span [from, to) of the line. */
  replace?: { from: number; to: number; text: string };
  /** Drop the whole line. */
  remove?: boolean;
  /** Lines to insert before this line (kept in order). */
  insertBefore?: string[];
}

/** Span of the leading `name( ... );` statement on a line (string/paren aware). */
export function statementSpan(
  line: string
): { from: number; to: number } | null {
  const m = /^(\s*)[A-Za-z_]\w*\s*\(/.exec(line);
  if (!m) return null;
  let depth = 0;
  let quote = false;
  for (let i = m[0].length - 1; i < line.length; i++) {
    const c = line[i];
    if (quote) {
      if (c === '\\') i++;
      else if (c === '"') quote = false;
    } else if (c === '"') quote = true;
    else if (c === '(') depth++;
    else if (c === ')' && --depth === 0) {
      let to = i + 1;
      if (line[to] === ';') to++;
      return { from: m[1].length, to };
    }
  }
  return null;
}

const sourceId = (m: KeyMapping): string =>
  canonicalDslKeyName(m.sourceKey) ?? m.sourceKey.replace(/^VK_/, '');

const layerId = (l: ModifierLayer): string =>
  ([] as string[]).concat(l.modifiers).join(',');

/** Queue of mappings per source key (a key may legitimately repeat). */
function byKey(mappings: KeyMapping[]): Map<string, KeyMapping[]> {
  const map = new Map<string, KeyMapping[]>();
  for (const m of mappings) {
    const id = sourceId(m);
    map.set(id, [...(map.get(id) ?? []), m]);
  }
  return map;
}

function removalEdit(line: string, index: number): Edit {
  const span = statementSpan(line);
  if (!span) return { line: index, remove: true };
  const rest = line.slice(span.to).trim();
  if (rest === '' || rest.startsWith('//'))
    return { line: index, remove: true };
  return { line: index, replace: { from: span.from, to: span.to, text: '' } };
}

interface Scope {
  oldMaps: KeyMapping[];
  newMaps: KeyMapping[];
  /** 0-based line before which additions are inserted. */
  anchor: number;
  indent: string;
}

function diffScope(lines: string[], scope: Scope, edits: Edit[]): void {
  const oldByKey = byKey(scope.oldMaps);
  const added: string[] = [];
  for (const nm of scope.newMaps) {
    const queue = oldByKey.get(sourceId(nm));
    const old = queue?.shift();
    const text = generateKeyMapping(nm);
    if (!old) {
      added.push(scope.indent + text);
      continue;
    }
    if (generateKeyMapping(old) === text) continue;
    const idx = old.line - 1;
    const span = statementSpan(lines[idx]);
    if (span) {
      edits.push({ line: idx, replace: { ...span, text } });
    }
  }
  for (const rest of oldByKey.values()) {
    for (const old of rest)
      edits.push(removalEdit(lines[old.line - 1], old.line - 1));
  }
  if (added.length) edits.push({ line: scope.anchor, insertBefore: added });
}

function indentOf(line: string): string {
  return /^\s*/.exec(line)![0];
}

function firstIndent(lines: string[], maps: KeyMapping[], fallback: string) {
  return maps.length ? indentOf(lines[maps[0].line - 1]) : fallback;
}

function removeLayer(lines: string[], layer: ModifierLayer, edits: Edit[]) {
  const mapLines = new Set(layer.mappings.map((m) => m.line - 1));
  let otherContent = false;
  for (let i = layer.startLine; i < layer.endLine - 1; i++) {
    const t = lines[i].trim();
    if (t && !t.startsWith('//') && !mapLines.has(i)) otherContent = true;
  }
  for (const m of layer.mappings) {
    edits.push(removalEdit(lines[m.line - 1], m.line - 1));
  }
  if (!otherContent) {
    edits.push({ line: layer.startLine - 1, remove: true });
    edits.push({ line: layer.endLine - 1, remove: true });
  }
}

/** Fold layers sharing a modifier set into one (first occurrence's identity). */
function mergeLayers(layers: ModifierLayer[]): ModifierLayer[] {
  const merged = new Map<string, ModifierLayer>();
  for (const l of layers) {
    const have = merged.get(layerId(l));
    if (have) have.mappings = [...have.mappings, ...l.mappings];
    else merged.set(layerId(l), { ...l, mappings: [...l.mappings] });
  }
  return [...merged.values()];
}

/** Fold blocks sharing a pattern into one. */
function mergeBlocks(blocks: DeviceBlock[]): DeviceBlock[] {
  const merged = new Map<string, DeviceBlock>();
  for (const b of blocks) {
    const have = merged.get(b.pattern);
    if (have) {
      have.mappings = [...have.mappings, ...b.mappings];
      have.layers = [...have.layers, ...b.layers];
    } else {
      merged.set(b.pattern, {
        ...b,
        mappings: [...b.mappings],
        layers: [...b.layers],
      });
    }
  }
  return [...merged.values()];
}

/** Insert point inside a scope: after its last mapping, else before its close. */
function anchorAfter(maps: KeyMapping[], closeIdx: number): number {
  return maps.length ? maps[maps.length - 1].line : closeIdx;
}

/**
 * Diff one new block against every old block sharing its pattern (hand-written
 * profiles repeat `device_start("*")` and `when_start("MD_00")`; the visual
 * model has one of each). Additions go into the first occurrence.
 */
function diffBlock(
  lines: string[],
  oldBlocks: DeviceBlock[],
  newB: DeviceBlock,
  edits: Edit[]
): void {
  const first = oldBlocks[0];
  const oldBase = oldBlocks.flatMap((b) => b.mappings);
  const baseIndent = firstIndent(
    lines,
    oldBase,
    indentOf(lines[first.startLine - 1]) + STEP
  );
  // Base additions stay above the first `when` block, as the generator writes.
  const baseClose = (first.layers[0]?.startLine ?? first.endLine) - 1;
  diffScope(
    lines,
    {
      oldMaps: oldBase,
      newMaps: newB.mappings,
      anchor: anchorAfter(first.mappings, baseClose),
      indent: baseIndent,
    },
    edits
  );

  const groups = new Map<string, ModifierLayer[]>();
  for (const l of oldBlocks.flatMap((b) => b.layers)) {
    groups.set(layerId(l), [...(groups.get(layerId(l)) ?? []), l]);
  }
  const extra: string[] = [];
  for (const nl of mergeLayers(newB.layers)) {
    const group = groups.get(layerId(nl));
    groups.delete(layerId(nl));
    if (!group) {
      const level = Math.max(1, Math.round(baseIndent.length / STEP.length));
      extra.push(...generateModifierLayer(nl, level, LAYER_OPTS));
      continue;
    }
    const ol = group[0];
    const oldMaps = group.flatMap((l) => l.mappings);
    diffScope(
      lines,
      {
        oldMaps,
        newMaps: nl.mappings,
        anchor: anchorAfter(ol.mappings, ol.endLine - 1),
        indent: firstIndent(
          lines,
          oldMaps,
          indentOf(lines[ol.startLine - 1]) + STEP
        ),
      },
      edits
    );
  }
  for (const gone of groups.values()) {
    for (const layer of gone) removeLayer(lines, layer, edits);
  }
  if (extra.length)
    edits.push({ line: first.endLine - 1, insertBefore: extra });
}

function applyEdits(lines: string[], edits: Edit[]): string[] {
  const byLine = new Map<number, Edit[]>();
  for (const e of edits) byLine.set(e.line, [...(byLine.get(e.line) ?? []), e]);
  const out: string[] = [];
  lines.forEach((text, i) => {
    const here = byLine.get(i) ?? [];
    for (const e of here) if (e.insertBefore) out.push(...e.insertBefore);
    if (here.some((e) => e.remove)) return;
    const rep = here.find((e) => e.replace)?.replace;
    out.push(
      rep ? text.slice(0, rep.from) + rep.text + text.slice(rep.to) : text
    );
  });
  return out;
}

/**
 * Apply `newAst` to `source` with minimal line edits. Falls back to full
 * generation only when there is nothing to preserve (empty or unparsable
 * source) or when the AST uses top-level mappings the patcher does not place.
 */
export function patchRhaiScript(source: string, newAst: RhaiAST): string {
  const parsed = source.trim() ? parseRhaiScript(source) : null;
  if (!parsed?.success || !parsed.ast || newAst.globalMappings.length > 0) {
    return generateRhaiScript(newAst);
  }
  const lines = source.split('\n');
  const edits: Edit[] = [];
  const oldByPattern = new Map<string, DeviceBlock[]>();
  for (const b of parsed.ast.deviceBlocks) {
    oldByPattern.set(b.pattern, [...(oldByPattern.get(b.pattern) ?? []), b]);
  }
  const appended: string[] = [];

  for (const nb of mergeBlocks(newAst.deviceBlocks)) {
    const group = oldByPattern.get(nb.pattern);
    if (!group) {
      appended.push('', ...generateDeviceBlock(nb, [], LAYER_OPTS));
      continue;
    }
    oldByPattern.delete(nb.pattern);
    diffBlock(lines, group, nb, edits);
  }
  // Old blocks absent from the new AST (devices not selected in the editor)
  // are deliberately left untouched.
  const patched = applyEdits(lines, edits);
  if (appended.length) {
    while (patched.length && patched[patched.length - 1].trim() === '')
      patched.pop();
    patched.push(...appended, '');
  }
  return patched.join('\n');
}
