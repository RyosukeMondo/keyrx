/**
 * Unit tests for linting rules.
 */

import { describe, it, expect } from 'vitest';
import { lintUnusedLayers, lintNamingConsistency } from './lintingRules';

describe('lintUnusedLayers', () => {
  it('should detect unused layers', () => {
    const config = `
      layer "base" {
        map KEY_A to KEY_B
      }

      layer "unused_layer" {
        map KEY_C to KEY_D
      }

      layer "active_layer" {
        map KEY_E to KEY_F
      }

      // Activate only some layers
      on KEY_X {
        activate_layer "base"
        activate_layer "active_layer"
      }
    `;

    const warnings = lintUnusedLayers(config);

    expect(warnings).toHaveLength(1);
    expect(warnings[0]).toMatchObject({
      message: "Layer 'unused_layer' is defined but never activated",
      code: 'UNUSED_LAYER',
    });
    expect(warnings[0].line).toBeGreaterThan(0);
  });

  it('should not warn when all layers are used', () => {
    const config = `
      layer "layer1" {
        map KEY_A to KEY_B
      }

      layer "layer2" {
        map KEY_C to KEY_D
      }

      on KEY_X {
        activate_layer "layer1"
      }

      on KEY_Y {
        activate_layer "layer2"
      }
    `;

    const warnings = lintUnusedLayers(config);

    expect(warnings).toHaveLength(0);
  });

  it('should handle activate() function syntax', () => {
    const config = `
      layer "test_layer" {
        map KEY_A to KEY_B
      }

      on KEY_X {
        activate("test_layer")
      }
    `;

    const warnings = lintUnusedLayers(config);

    expect(warnings).toHaveLength(0);
  });

  it('should handle to_layer syntax', () => {
    const config = `
      layer "navigation" {
        map KEY_A to KEY_B
      }

      on KEY_X {
        to_layer "navigation"
      }
    `;

    const warnings = lintUnusedLayers(config);

    expect(warnings).toHaveLength(0);
  });

  it('should handle switch_layer syntax', () => {
    const config = `
      layer "symbols" {
        map KEY_A to KEY_B
      }

      on KEY_X {
        switch_layer "symbols"
      }
    `;

    const warnings = lintUnusedLayers(config);

    expect(warnings).toHaveLength(0);
  });

  it('should handle multiple unused layers', () => {
    const config = `
      layer "used" {
        map KEY_A to KEY_B
      }

      layer "unused1" {
        map KEY_C to KEY_D
      }

      layer "unused2" {
        map KEY_E to KEY_F
      }

      on KEY_X {
        activate_layer "used"
      }
    `;

    const warnings = lintUnusedLayers(config);

    expect(warnings).toHaveLength(2);
    expect(warnings[0].message).toContain('unused1');
    expect(warnings[1].message).toContain('unused2');
  });

  it('should handle empty config gracefully', () => {
    const config = '';

    const warnings = lintUnusedLayers(config);

    expect(warnings).toHaveLength(0);
  });

  it('should handle config with no layers', () => {
    const config = `
      modifier "ctrl" {
        map KEY_A to KEY_B
      }
    `;

    const warnings = lintUnusedLayers(config);

    expect(warnings).toHaveLength(0);
  });

  it('should correctly identify line numbers', () => {
    const config = `layer "base" { }

layer "unused" { }

layer "active" { }

on KEY_X { activate_layer "base" }
on KEY_Y { activate_layer "active" }`;

    const warnings = lintUnusedLayers(config);

    expect(warnings).toHaveLength(1);
    expect(warnings[0].line).toBe(3); // "unused" is on line 3
  });
});

describe('lintNamingConsistency', () => {
  it('should detect mixed camelCase and snake_case', () => {
    const config = `
      layer "base_layer" {
        map KEY_A to KEY_B
      }

      layer "navigationLayer" {
        map KEY_C to KEY_D
      }

      modifier "ctrl_modifier" {
        map KEY_E to KEY_F
      }

      modifier "shiftKey" {
        map KEY_G to KEY_H
      }
    `;

    const hints = lintNamingConsistency(config);

    expect(hints).toHaveLength(1);
    expect(hints[0]).toMatchObject({
      message: expect.stringContaining('Consider using consistent naming'),
      code: 'NAMING_INCONSISTENCY',
    });
    expect(hints[0].message).toContain('camelCase (2)');
    expect(hints[0].message).toContain('snake_case (2)');
  });

  it('should not hint when all names are snake_case', () => {
    const config = `
      layer "base_layer" {
        map KEY_A to KEY_B
      }

      layer "navigation_layer" {
        map KEY_C to KEY_D
      }

      modifier "ctrl_modifier" {
        map KEY_E to KEY_F
      }
    `;

    const hints = lintNamingConsistency(config);

    expect(hints).toHaveLength(0);
  });

  it('should not hint when all names are camelCase', () => {
    const config = `
      layer "baseLayer" {
        map KEY_A to KEY_B
      }

      layer "navigationLayer" {
        map KEY_C to KEY_D
      }

      modifier "ctrlModifier" {
        map KEY_E to KEY_F
      }
    `;

    const hints = lintNamingConsistency(config);

    expect(hints).toHaveLength(0);
  });

  it('should handle empty config gracefully', () => {
    const config = '';

    const hints = lintNamingConsistency(config);

    expect(hints).toHaveLength(0);
  });

  it('should handle config with no layers or modifiers', () => {
    const config = `
      on KEY_X {
        map KEY_A to KEY_B
      }
    `;

    const hints = lintNamingConsistency(config);

    expect(hints).toHaveLength(0);
  });

  it('should handle single-word names (neither camelCase nor snake_case)', () => {
    const config = `
      layer "base" {
        map KEY_A to KEY_B
      }

      layer "navigation" {
        map KEY_C to KEY_D
      }
    `;

    const hints = lintNamingConsistency(config);

    expect(hints).toHaveLength(0);
  });

  it('should count both layers and modifiers', () => {
    const config = `
      layer "snake_layer1" {
        map KEY_A to KEY_B
      }

      layer "snake_layer2" {
        map KEY_C to KEY_D
      }

      modifier "camelModifier1" {
        map KEY_E to KEY_F
      }

      modifier "camelModifier2" {
        map KEY_G to KEY_H
      }

      modifier "camelModifier3" {
        map KEY_I to KEY_J
      }
    `;

    const hints = lintNamingConsistency(config);

    expect(hints).toHaveLength(1);
    expect(hints[0].message).toContain('camelCase (3)');
    expect(hints[0].message).toContain('snake_case (2)');
  });

  it('should correctly identify camelCase names', () => {
    const config = `
      layer "myLayer" {
        map KEY_A to KEY_B
      }

      layer "anotherCoolLayer" {
        map KEY_C to KEY_D
      }
    `;

    const hints = lintNamingConsistency(config);

    // Both are camelCase, so no hint
    expect(hints).toHaveLength(0);
  });

  it('should correctly identify snake_case names', () => {
    const config = `
      layer "my_layer" {
        map KEY_A to KEY_B
      }

      layer "another_cool_layer" {
        map KEY_C to KEY_D
      }
    `;

    const hints = lintNamingConsistency(config);

    // Both are snake_case, so no hint
    expect(hints).toHaveLength(0);
  });
});
