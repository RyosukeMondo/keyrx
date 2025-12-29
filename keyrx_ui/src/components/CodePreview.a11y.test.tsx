import { describe, it, expect, beforeEach } from 'vitest';
import { render } from '@testing-library/react';
import { axe } from 'vitest-axe';
import { CodePreview } from './CodePreview';
import { useConfigBuilderStore } from '@/store/configBuilderStore';

describe('CodePreview - Accessibility', () => {
  beforeEach(() => {
    useConfigBuilderStore.getState().resetConfig();
  });

  it('should have no accessibility violations on initial render', async () => {
    const { container } = render(<CodePreview />);
    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });

  it('should have no violations with empty configuration', async () => {
    const { container } = render(<CodePreview />);
    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });

  it('should have no violations with generated code', async () => {
    const store = useConfigBuilderStore.getState();
    const currentLayerId = store.currentLayerId;

    store.addMapping(currentLayerId, {
      sourceKey: 'VK_A',
      targetKey: 'VK_B',
      modifiers: [],
      locks: [],
    });

    const { container } = render(<CodePreview />);
    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });

  it('should have no violations with complex configuration', async () => {
    const store = useConfigBuilderStore.getState();

    // Add layers
    store.addLayer();

    // Add modifiers
    store.addModifier({
      id: 'MOD_1',
      name: 'Shift',
      type: 'modifier',
      sourceKey: 'VK_LSHIFT',
    });

    // Add locks
    store.addModifier({
      id: 'LOCK_1',
      name: 'CapsLock',
      type: 'lock',
      sourceKey: 'VK_CAPITAL',
    });

    // Add mappings
    store.layers.forEach(layer => {
      store.addMapping(layer.id, {
        sourceKey: 'VK_A',
        targetKey: 'VK_B',
        modifiers: ['MOD_1'],
        locks: ['LOCK_1'],
      });
    });

    const { container } = render(<CodePreview />);
    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });

  it('should maintain accessibility with validation errors', async () => {
    const store = useConfigBuilderStore.getState();
    const currentLayerId = store.currentLayerId;

    // Add a mapping that might cause validation issues
    store.addMapping(currentLayerId, {
      sourceKey: 'VK_A',
      targetKey: 'VK_B',
      modifiers: [],
      locks: [],
    });

    const { container } = render(<CodePreview />);
    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });

  it('should have no violations with multiple mappings', async () => {
    const store = useConfigBuilderStore.getState();
    const currentLayerId = store.currentLayerId;

    // Add multiple mappings
    for (let i = 0; i < 10; i++) {
      store.addMapping(currentLayerId, {
        sourceKey: `VK_${String.fromCharCode(65 + i)}`,
        targetKey: `VK_${String.fromCharCode(75 + i)}`,
        modifiers: [],
        locks: [],
      });
    }

    const { container } = render(<CodePreview />);
    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });
});
