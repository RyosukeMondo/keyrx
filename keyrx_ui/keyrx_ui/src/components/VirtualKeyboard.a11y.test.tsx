import { describe, it, expect, beforeEach } from 'vitest';
import { render } from '@testing-library/react';
import { axe } from 'vitest-axe';
import { DndContext } from '@dnd-kit/core';
import { VirtualKeyboard } from './VirtualKeyboard';
import { useConfigBuilderStore } from '@/store/configBuilderStore';

describe('VirtualKeyboard - Accessibility', () => {
  beforeEach(() => {
    useConfigBuilderStore.getState().resetConfig();
  });

  it('should have no accessibility violations on initial render', async () => {
    const { container } = render(
      <DndContext>
        <VirtualKeyboard />
      </DndContext>
    );
    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });

  it('should have no violations with mapped keys', async () => {
    const store = useConfigBuilderStore.getState();
    const currentLayerId = store.currentLayerId;

    store.addMapping(currentLayerId, {
      sourceKey: 'VK_A',
      targetKey: 'VK_B',
      modifiers: [],
      locks: [],
    });

    store.addMapping(currentLayerId, {
      sourceKey: 'VK_S',
      targetKey: 'VK_C',
      modifiers: [],
      locks: [],
    });

    const { container } = render(
      <DndContext>
        <VirtualKeyboard />
      </DndContext>
    );
    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });

  it('should have no violations with modifier keys highlighted', async () => {
    const store = useConfigBuilderStore.getState();

    store.addModifier('Shift', 'VK_LSHIFT');
    store.addModifier('Ctrl', 'VK_LCTRL');

    const { container } = render(
      <DndContext>
        <VirtualKeyboard />
      </DndContext>
    );
    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });

  it('should have no violations with lock keys', async () => {
    const store = useConfigBuilderStore.getState();

    store.addLock('CapsLock', 'VK_CAPITAL');
    store.addLock('NumLock', 'VK_NUMLOCK');

    const { container } = render(
      <DndContext>
        <VirtualKeyboard />
      </DndContext>
    );
    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });

  it('should maintain accessibility with all key types', async () => {
    const store = useConfigBuilderStore.getState();
    const currentLayerId = store.currentLayerId;

    // Add mappings
    store.addMapping(currentLayerId, {
      sourceKey: 'VK_A',
      targetKey: 'VK_B',
      modifiers: [],
      locks: [],
    });

    // Add modifiers
    store.addModifier('Shift', 'VK_LSHIFT');

    // Add locks
    store.addLock('CapsLock', 'VK_CAPITAL');

    const { container } = render(
      <DndContext>
        <VirtualKeyboard />
      </DndContext>
    );
    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });
});
