import { describe, it, expect, beforeEach } from 'vitest';
import { render } from '@testing-library/react';
import { axe } from 'vitest-axe';
import { DndContext } from '@dnd-kit/core';
import { LayerPanel } from './LayerPanel';
import { useConfigBuilderStore } from '@/store/configBuilderStore';

describe('LayerPanel - Accessibility', () => {
  beforeEach(() => {
    useConfigBuilderStore.getState().resetConfig();
  });

  it('should have no accessibility violations on initial render', async () => {
    const { container } = render(
      <DndContext>
        <LayerPanel />
      </DndContext>
    );
    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });

  it('should have no violations with multiple layers', async () => {
    const store = useConfigBuilderStore.getState();
    store.addLayer();
    store.addLayer();
    store.addLayer();

    const { container } = render(
      <DndContext>
        <LayerPanel />
      </DndContext>
    );
    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });

  it('should have no violations with renamed layers', async () => {
    const store = useConfigBuilderStore.getState();
    const firstLayerId = store.layers[0].id;

    store.addLayer();
    store.addLayer();

    store.renameLayer(firstLayerId, 'Custom Layer Name');

    const { container } = render(
      <DndContext>
        <LayerPanel />
      </DndContext>
    );
    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });

  it('should have no violations with active layer selection', async () => {
    const store = useConfigBuilderStore.getState();
    store.addLayer();

    // Get fresh state to ensure we have the latest layers
    const freshStore = useConfigBuilderStore.getState();
    const secondLayerId = freshStore.layers[1].id;

    store.setCurrentLayer(secondLayerId);

    const { container } = render(
      <DndContext>
        <LayerPanel />
      </DndContext>
    );
    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });

  it('should maintain accessibility after layer reordering', async () => {
    const store = useConfigBuilderStore.getState();
    store.addLayer();
    store.addLayer();
    store.addLayer();

    const layerIds = store.layers.map(l => l.id);
    const reordered = [layerIds[2], layerIds[0], layerIds[1]];
    store.reorderLayers(reordered);

    const { container } = render(
      <DndContext>
        <LayerPanel />
      </DndContext>
    );
    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });

  it('should have no violations with layers containing mappings', async () => {
    const store = useConfigBuilderStore.getState();
    store.addLayer();

    store.layers.forEach(layer => {
      store.addMapping(layer.id, {
        sourceKey: 'VK_A',
        targetKey: 'VK_B',
        modifiers: [],
        locks: [],
      });
    });

    const { container } = render(
      <DndContext>
        <LayerPanel />
      </DndContext>
    );
    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });
});
