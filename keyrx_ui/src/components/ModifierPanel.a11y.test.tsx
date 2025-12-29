import { describe, it, expect, beforeEach } from 'vitest';
import { render } from '@testing-library/react';
import { axe } from 'vitest-axe';
import { DndContext } from '@dnd-kit/core';
import { ModifierPanel } from './ModifierPanel';
import { useConfigBuilderStore } from '@/store/configBuilderStore';

describe('ModifierPanel - Accessibility', () => {
  beforeEach(() => {
    useConfigBuilderStore.getState().resetConfig();
  });

  it('should have no accessibility violations on initial render', async () => {
    const { container } = render(
      <DndContext>
        <ModifierPanel />
      </DndContext>
    );
    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });

  it('should have no violations with modifiers added', async () => {
    const store = useConfigBuilderStore.getState();

    store.addModifier('Shift', 'VK_LSHIFT');
    store.addModifier('Ctrl', 'VK_LCTRL');
    store.addModifier('Alt', 'VK_LALT');

    const { container } = render(
      <DndContext>
        <ModifierPanel />
      </DndContext>
    );
    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });

  it('should have no violations with locks added', async () => {
    const store = useConfigBuilderStore.getState();

    store.addLock('CapsLock', 'VK_CAPITAL');
    store.addLock('NumLock', 'VK_NUMLOCK');

    const { container } = render(
      <DndContext>
        <ModifierPanel />
      </DndContext>
    );
    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });

  it('should have no violations with mixed modifiers and locks', async () => {
    const store = useConfigBuilderStore.getState();

    store.addModifier('Shift', 'VK_LSHIFT');
    store.addLock('CapsLock', 'VK_CAPITAL');
    store.addModifier('Ctrl', 'VK_LCTRL');
    store.addLock('NumLock', 'VK_NUMLOCK');

    const { container } = render(
      <DndContext>
        <ModifierPanel />
      </DndContext>
    );
    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });

  it('should maintain accessibility when modifiers are in use', async () => {
    const store = useConfigBuilderStore.getState();
    const currentLayerId = store.currentLayerId;

    // Add modifiers
    store.addModifier('Shift', 'VK_LSHIFT');
    store.addModifier('Ctrl', 'VK_LCTRL');

    // Use modifiers in mappings
    store.addMapping(currentLayerId, {
      sourceKey: 'VK_A',
      targetKey: 'VK_B',
      modifiers: ['MOD_1'],
      locks: [],
    });

    store.addMapping(currentLayerId, {
      sourceKey: 'VK_C',
      targetKey: 'VK_D',
      modifiers: ['MOD_1', 'MOD_2'],
      locks: [],
    });

    const { container } = render(
      <DndContext>
        <ModifierPanel />
      </DndContext>
    );
    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });
});
