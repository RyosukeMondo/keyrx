/**
 * Plain-language diff between two versions of a profile's Rhai source.
 *
 * The raw Rhai diff stays available behind a "Details" toggle; this is what
 * the Save review shows first: "Caps Lock → Ctrl", one line per change.
 */
import { parseRhaiScript, type KeyMapping, type RhaiAST } from './rhaiParser';
import { friendlyKeyName } from './keyNames';

export type ChangeKind = 'added' | 'changed' | 'removed';

export interface MappingChange {
  kind: ChangeKind;
  /** Source key as a friendly name ("Caps Lock"). */
  from: string;
  /** What it does now (target / tap+hold), friendly. Empty for removals. */
  to: string;
  tapHold?: { tap: string; hold: string };
  /** Layer the mapping lives on, e.g. "MD_00"; undefined for the base layer. */
  layer?: string;
  /** Device pattern the mapping is scoped to; undefined for global (`*`). */
  device?: string;
}

interface FlatMapping {
  id: string;
  from: string;
  layer?: string;
  device?: string;
  signature: string;
  to: string;
  tapHold?: { tap: string; hold: string };
}

function flatten(ast: RhaiAST): FlatMapping[] {
  const out: FlatMapping[] = [];
  const push = (m: KeyMapping, device?: string, layer?: string) => {
    const from = friendlyKeyName(m.sourceKey);
    let to = '';
    let tapHold: FlatMapping['tapHold'];
    if (m.type === 'tap_hold' && m.tapHold) {
      tapHold = {
        tap: friendlyKeyName(m.tapHold.tapAction),
        hold: friendlyKeyName(m.tapHold.holdAction),
      };
      to = `tap ${tapHold.tap} / hold ${tapHold.hold}`;
    } else if (m.targetKey) {
      to = friendlyKeyName(m.targetKey);
    }
    out.push({
      id: `${device ?? '*'}|${layer ?? 'base'}|${m.sourceKey}`,
      from,
      layer,
      device,
      signature: `${m.type}:${m.targetKey ?? ''}:${JSON.stringify(m.tapHold ?? null)}`,
      to,
      tapHold,
    });
  };
  ast.globalMappings.forEach((m) => push(m));
  for (const block of ast.deviceBlocks) {
    const device = block.pattern === '*' ? undefined : block.pattern;
    block.mappings.forEach((m) => push(m, device));
    for (const layer of block.layers) {
      const layerNames = Array.isArray(layer.modifiers)
        ? layer.modifiers
        : [layer.modifiers];
      layerNames.forEach((name) =>
        layer.mappings.forEach((m) => push(m, device, name))
      );
    }
  }
  return out;
}

const toChange = (m: FlatMapping, kind: ChangeKind): MappingChange => ({
  kind,
  from: m.from,
  to: kind === 'removed' ? '' : m.to,
  tapHold: kind === 'removed' ? undefined : m.tapHold,
  layer: m.layer,
  device: m.device,
});

/** Changes needed to get from `original` to `modified`; null if either is unparsable. */
export function diffMappings(
  original: string,
  modified: string
): MappingChange[] | null {
  const a = parseRhaiScript(original);
  const b = parseRhaiScript(modified);
  if (!a.success || !a.ast || !b.success || !b.ast) return null;

  const before = new Map(flatten(a.ast).map((m) => [m.id, m]));
  const after = new Map(flatten(b.ast).map((m) => [m.id, m]));
  const changes: MappingChange[] = [];

  for (const [id, m] of after) {
    const prev = before.get(id);
    if (!prev) changes.push(toChange(m, 'added'));
    else if (prev.signature !== m.signature)
      changes.push(toChange(m, 'changed'));
  }
  for (const [id, m] of before) {
    if (!after.has(id)) changes.push(toChange(m, 'removed'));
  }
  return changes;
}
