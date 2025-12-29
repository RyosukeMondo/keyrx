import { describe, it, expect, beforeEach } from 'vitest';
import { render } from '@testing-library/react';
import { axe } from 'vitest-axe';
import { VisualBuilderPage } from './VisualBuilderPage';
import { useConfigBuilderStore } from '@/store/configBuilderStore';

describe('VisualBuilderPage - Accessibility', () => {
  beforeEach(() => {
    useConfigBuilderStore.getState().resetConfig();
  });

  it('should have no accessibility violations on initial render', async () => {
    const { container } = render(<VisualBuilderPage />);
    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });

  it('should have no violations with multiple layers', async () => {
    const store = useConfigBuilderStore.getState();
    store.addLayer();
    store.addLayer();

    const { container } = render(<VisualBuilderPage />);
    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });

  it('should have no violations with modifiers added', async () => {
    const store = useConfigBuilderStore.getState();
    store.addModifier('Test Modifier', 'VK_LSHIFT');

    const { container } = render(<VisualBuilderPage />);
    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });

  it('should have no violations with key mappings', async () => {
    const store = useConfigBuilderStore.getState();
    const currentLayerId = store.currentLayerId;
    store.addMapping(currentLayerId, {
      sourceKey: 'VK_A',
      targetKey: 'VK_B',
      modifiers: [],
      locks: [],
    });

    const { container } = render(<VisualBuilderPage />);
    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });

  it('should maintain accessibility with complex configuration', async () => {
    const store = useConfigBuilderStore.getState();

    // Add multiple layers
    store.addLayer();
    store.addLayer();

    // Add modifiers
    store.addModifier('Shift', 'VK_LSHIFT');
    store.addModifier('Ctrl', 'VK_LCTRL');

    // Add locks
    store.addLock('CapsLock', 'VK_CAPITAL');

    // Add mappings to each layer
    store.layers.forEach(layer => {
      store.addMapping(layer.id, {
        sourceKey: 'VK_A',
        targetKey: 'VK_B',
        modifiers: ['MOD_1'],
        locks: [],
      });
    });

    const { container } = render(<VisualBuilderPage />);
    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });
});
