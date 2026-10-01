import { describe, expect, it } from 'vitest';
import { patchRhaiScript, statementSpan } from './rhaiPatch';
import { generateRhaiScript } from './rhaiCodeGen';
import { parseRhaiScript, type RhaiAST } from './rhaiParser';

const SOURCE = `// my profile
device_start("*");  // everything
    tap_hold("Space", "VK_Space", "MD_00", 200); // space / nav layer
    map("CapsLock", "VK_LCtrl");
    sequence("F9", ["VK_A", "VK_B"]);  // unmodelled construct
    hold_only("LMeta", "MD_07");

    when_start("MD_00");
        map("H", "VK_Left");
        // vim arrows
        map("J", "VK_Down");
    when_end();
device_end();

device_start("*Other*");
    map("A", "VK_B");
device_end();
`;

const ast = (): RhaiAST => {
  const r = parseRhaiScript(SOURCE);
  if (!r.ast) throw new Error('parse');
  return r.ast;
};

describe('patchRhaiScript', () => {
  it('is the identity for an unchanged AST', () => {
    expect(patchRhaiScript(SOURCE, ast())).toBe(SOURCE);
  });

  it('regression: UI-style upper-cased key names do not reach the file', () => {
    const next = ast();
    // what useASTSync used to hand the store for the Space key
    next.deviceBlocks[0].mappings[0].sourceKey = 'VK_SPACE';
    next.deviceBlocks[0].mappings[0].tapHold!.tapAction = 'VK_SPACE';
    next.deviceBlocks[0].mappings[0].tapHold!.thresholdMs = 180;
    const out = patchRhaiScript(SOURCE, next);
    expect(out).toContain('tap_hold("VK_Space", "VK_Space", "MD_00", 180); // space / nav layer');
    expect(out).not.toMatch(/SPACE/);
    expect(generateRhaiScript(next)).not.toMatch(/SPACE/);
  });

  it('edits only the changed mapping and keeps everything else', () => {
    const next = ast();
    next.deviceBlocks[0].mappings[1].targetKey = 'VK_Escape';
    const out = patchRhaiScript(SOURCE, next);
    expect(out).toBe(
      SOURCE.replace('map("CapsLock", "VK_LCtrl")', 'map("VK_CapsLock", "VK_Escape")')
    );
    expect(out).toContain('sequence("F9"');
    expect(out).toContain('// vim arrows');
  });

  it('adds a mapping before the first when block / removes one cleanly', () => {
    const next = ast();
    next.deviceBlocks[0].mappings.push({
      type: 'simple',
      sourceKey: 'VK_Tab',
      targetKey: 'VK_Enter',
      line: 0,
    });
    next.deviceBlocks[0].layers[0].mappings.splice(1, 1); // drop J
    const out = patchRhaiScript(SOURCE, next);
    expect(out).toContain('"VK_LCtrl");\n    map("VK_Tab", "VK_Enter");\n    sequence');
    expect(out).not.toContain('"J"');
    expect(out).toContain('// vim arrows');
    expect(parseRhaiScript(out).success).toBe(true);
  });

  it('leaves blocks that are absent from the new AST untouched', () => {
    const next = ast();
    next.deviceBlocks = [next.deviceBlocks[0]];
    expect(patchRhaiScript(SOURCE, next)).toBe(SOURCE);
  });

  it('removes an emptied layer only when nothing else lives in it', () => {
    const next = ast();
    next.deviceBlocks[0].layers = [];
    const out = patchRhaiScript(SOURCE, next);
    expect(out).not.toContain('when_start');
    expect(out).not.toContain('when_end');
  });

  it('appends a new device block and falls back to generation for empty input', () => {
    const next = ast();
    next.deviceBlocks.push({
      pattern: '*New*',
      mappings: [{ type: 'simple', sourceKey: 'VK_A', targetKey: 'VK_B', line: 0 }],
      layers: [],
      startLine: 0,
      endLine: 0,
    });
    expect(patchRhaiScript(SOURCE, next)).toContain('device_start("*New*");');
    expect(patchRhaiScript('', next)).toBe(generateRhaiScript(next));
  });
});

describe('statementSpan', () => {
  it('handles nested parens, strings and trailing comments', () => {
    const line = '  map("A", with_shift("VK_B")); // note (x)';
    const span = statementSpan(line)!;
    expect(line.slice(span.from, span.to)).toBe('map("A", with_shift("VK_B"));');
  });
});
