/**
 * Unit tests for LayerPanel component
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { LayerPanel } from './LayerPanel';
import { useConfigBuilderStore } from '../store/configBuilderStore';

// Mock window.confirm
global.confirm = vi.fn(() => true);

describe('LayerPanel', () => {
  beforeEach(() => {
    // Reset store to initial state before each test
    useConfigBuilderStore.getState().resetConfig();
    vi.clearAllMocks();
  });

  describe('rendering', () => {
    it('should render the panel with header', () => {
      render(<LayerPanel />);
      expect(screen.getByText('Layers')).toBeInTheDocument();
      expect(screen.getByText('+ Add Layer')).toBeInTheDocument();
    });

    it('should render the base layer by default', () => {
      render(<LayerPanel />);
      expect(screen.getByText('base')).toBeInTheDocument();
      expect(screen.getByText('0 mappings')).toBeInTheDocument();
    });

    it('should show mapping count correctly', () => {
      const store = useConfigBuilderStore.getState();
      store.addMapping('base', 'a', 'b');
      store.addMapping('base', 'c', 'd');

      render(<LayerPanel />);
      expect(screen.getByText('2 mappings')).toBeInTheDocument();
    });

    it('should render multiple layers', () => {
      const store = useConfigBuilderStore.getState();
      store.addLayer('layer1');
      store.addLayer('layer2');

      render(<LayerPanel />);
      expect(screen.getByText('base')).toBeInTheDocument();
      expect(screen.getByText('layer1')).toBeInTheDocument();
      expect(screen.getByText('layer2')).toBeInTheDocument();
    });

    it('should highlight the active layer', () => {
      const store = useConfigBuilderStore.getState();
      store.addLayer('test-layer');
      store.setCurrentLayer('base');

      render(<LayerPanel />);
      const baseLayer = screen.getByText('base').closest('.layer-item');
      expect(baseLayer).toHaveClass('active');
    });

    it('should mark base layer with special class', () => {
      render(<LayerPanel />);
      const baseLayer = screen.getByText('base').closest('.layer-item');
      expect(baseLayer).toHaveClass('base');
    });
  });

  describe('layer selection', () => {
    it('should switch active layer on click', () => {
      const initialStore = useConfigBuilderStore.getState();
      initialStore.addLayer('test-layer');

      render(<LayerPanel />);

      const testLayer = screen.getByText('test-layer').closest('.layer-info');
      fireEvent.click(testLayer!);

      const updatedStore = useConfigBuilderStore.getState();
      const testLayerId = updatedStore.layers.find((l) => l.name === 'test-layer')?.id;
      expect(updatedStore.currentLayerId).toBe(testLayerId);
    });
  });

  describe('adding layers', () => {
    it('should add a new layer when clicking Add Layer button', () => {
      render(<LayerPanel />);

      const addButton = screen.getByText('+ Add Layer');
      fireEvent.click(addButton);

      const store = useConfigBuilderStore.getState();
      expect(store.layers.length).toBe(2); // base + new layer
    });

    it('should auto-generate layer name', () => {
      render(<LayerPanel />);

      const addButton = screen.getByText('+ Add Layer');
      fireEvent.click(addButton);

      expect(screen.getByText('layer_1')).toBeInTheDocument();
    });

    it('should set new layer as current', () => {
      render(<LayerPanel />);

      const addButton = screen.getByText('+ Add Layer');
      fireEvent.click(addButton);

      const updatedStore = useConfigBuilderStore.getState();
      const newLayer = updatedStore.layers.find((l) => l.name === 'layer_1');
      expect(updatedStore.currentLayerId).toBe(newLayer?.id);
    });
  });

  describe('deleting layers', () => {
    it('should show delete button for non-base layers', () => {
      const store = useConfigBuilderStore.getState();
      store.addLayer('test-layer');

      render(<LayerPanel />);

      const testLayerItem = screen.getByText('test-layer').closest('.layer-item');
      const deleteButton = testLayerItem?.querySelector('.delete-btn');
      expect(deleteButton).toBeInTheDocument();
    });

    it('should not show delete button for base layer', () => {
      render(<LayerPanel />);

      const baseLayerItem = screen.getByText('base').closest('.layer-item');
      const deleteButton = baseLayerItem?.querySelector('.delete-btn');
      expect(deleteButton).not.toBeInTheDocument();
    });

    it('should delete layer after confirmation', async () => {
      const store = useConfigBuilderStore.getState();
      store.addLayer('test-layer');

      render(<LayerPanel />);

      const testLayerItem = screen.getByText('test-layer').closest('.layer-item');
      const deleteButton = testLayerItem?.querySelector('.delete-btn') as HTMLElement;

      fireEvent.click(deleteButton);

      await waitFor(() => {
        expect(screen.queryByText('test-layer')).not.toBeInTheDocument();
      });
    });

    it('should show confirmation dialog before deleting', () => {
      const store = useConfigBuilderStore.getState();
      store.addLayer('test-layer');

      render(<LayerPanel />);

      const testLayerItem = screen.getByText('test-layer').closest('.layer-item');
      const deleteButton = testLayerItem?.querySelector('.delete-btn') as HTMLElement;

      fireEvent.click(deleteButton);

      expect(global.confirm).toHaveBeenCalledWith(
        'Are you sure you want to delete this layer?'
      );
    });
  });

  describe('renaming layers', () => {
    it('should show rename button for all layers', () => {
      const store = useConfigBuilderStore.getState();
      store.addLayer('test-layer');

      render(<LayerPanel />);

      const renameButtons = screen.getAllByTitle('Rename layer');
      expect(renameButtons.length).toBe(2); // base + test-layer
    });

    it('should open rename dialog on rename button click', () => {
      render(<LayerPanel />);

      const renameButton = screen.getByTitle('Rename layer');
      fireEvent.click(renameButton);

      expect(screen.getByText('Rename Layer')).toBeInTheDocument();
      expect(screen.getByPlaceholderText('Layer name')).toBeInTheDocument();
    });

    it('should populate input with current layer name', () => {
      const store = useConfigBuilderStore.getState();
      store.addLayer('test-layer');

      render(<LayerPanel />);

      const testLayerItem = screen.getByText('test-layer').closest('.layer-item');
      const renameButton = testLayerItem?.querySelector('.rename-btn') as HTMLElement;
      fireEvent.click(renameButton);

      const input = screen.getByPlaceholderText('Layer name') as HTMLInputElement;
      expect(input.value).toBe('test-layer');
    });

    it('should rename layer on form submit', async () => {
      const store = useConfigBuilderStore.getState();
      store.addLayer('test-layer');

      render(<LayerPanel />);

      const testLayerItem = screen.getByText('test-layer').closest('.layer-item');
      const renameButton = testLayerItem?.querySelector('.rename-btn') as HTMLElement;
      fireEvent.click(renameButton);

      const input = screen.getByPlaceholderText('Layer name');
      fireEvent.change(input, { target: { value: 'renamed-layer' } });

      const submitButton = screen.getByText('Rename');
      fireEvent.click(submitButton);

      await waitFor(() => {
        expect(screen.getByText('renamed-layer')).toBeInTheDocument();
        expect(screen.queryByText('test-layer')).not.toBeInTheDocument();
      });
    });

    it('should close dialog on cancel', async () => {
      render(<LayerPanel />);

      const renameButton = screen.getByTitle('Rename layer');
      fireEvent.click(renameButton);

      const cancelButton = screen.getByText('Cancel');
      fireEvent.click(cancelButton);

      await waitFor(() => {
        expect(screen.queryByText('Rename Layer')).not.toBeInTheDocument();
      });
    });

    it('should close dialog on overlay click', async () => {
      render(<LayerPanel />);

      const renameButton = screen.getByTitle('Rename layer');
      fireEvent.click(renameButton);

      const overlay = document.querySelector('.rename-dialog-overlay') as HTMLElement;
      fireEvent.click(overlay);

      await waitFor(() => {
        expect(screen.queryByText('Rename Layer')).not.toBeInTheDocument();
      });
    });

    it('should not close dialog on dialog content click', () => {
      render(<LayerPanel />);

      const renameButton = screen.getByTitle('Rename layer');
      fireEvent.click(renameButton);

      const dialog = document.querySelector('.rename-dialog') as HTMLElement;
      fireEvent.click(dialog);

      expect(screen.getByText('Rename Layer')).toBeInTheDocument();
    });

    it('should disable submit button with empty name', () => {
      render(<LayerPanel />);

      const renameButton = screen.getByTitle('Rename layer');
      fireEvent.click(renameButton);

      const input = screen.getByPlaceholderText('Layer name');
      fireEvent.change(input, { target: { value: '' } });

      const submitButton = screen.getByText('Rename');
      expect(submitButton).toBeDisabled();
    });

    it('should trim whitespace from layer name', async () => {
      const store = useConfigBuilderStore.getState();
      store.addLayer('test-layer');

      render(<LayerPanel />);

      const testLayerItem = screen.getByText('test-layer').closest('.layer-item');
      const renameButton = testLayerItem?.querySelector('.rename-btn') as HTMLElement;
      fireEvent.click(renameButton);

      const input = screen.getByPlaceholderText('Layer name');
      fireEvent.change(input, { target: { value: '  trimmed  ' } });

      const submitButton = screen.getByText('Rename');
      fireEvent.click(submitButton);

      await waitFor(() => {
        expect(screen.getByText('trimmed')).toBeInTheDocument();
      });
    });
  });

  describe('drag and drop', () => {
    it('should render drag handles for all layers', () => {
      const store = useConfigBuilderStore.getState();
      store.addLayer('layer1');
      store.addLayer('layer2');

      render(<LayerPanel />);

      const dragHandles = document.querySelectorAll('.layer-drag-handle');
      expect(dragHandles.length).toBe(3); // base + layer1 + layer2
    });

    it('should display drag icon in handles', () => {
      render(<LayerPanel />);

      const dragIcon = document.querySelector('.drag-icon');
      expect(dragIcon).toBeInTheDocument();
      expect(dragIcon?.textContent).toBe('⋮⋮');
    });
  });

  describe('state management integration', () => {
    it('should mark config as dirty when adding layer', () => {
      render(<LayerPanel />);

      const initialStore = useConfigBuilderStore.getState();
      expect(initialStore.isDirty).toBe(false);

      const addButton = screen.getByText('+ Add Layer');
      fireEvent.click(addButton);

      const updatedStore = useConfigBuilderStore.getState();
      expect(updatedStore.isDirty).toBe(true);
    });

    it('should mark config as dirty when deleting layer', () => {
      const initialStore = useConfigBuilderStore.getState();
      initialStore.addLayer('test-layer');
      initialStore.markClean();

      render(<LayerPanel />);

      const testLayerItem = screen.getByText('test-layer').closest('.layer-item');
      const deleteButton = testLayerItem?.querySelector('.delete-btn') as HTMLElement;
      fireEvent.click(deleteButton);

      const updatedStore = useConfigBuilderStore.getState();
      expect(updatedStore.isDirty).toBe(true);
    });

    it('should mark config as dirty when renaming layer', async () => {
      const initialStore = useConfigBuilderStore.getState();
      initialStore.addLayer('test-layer');
      initialStore.markClean();

      render(<LayerPanel />);

      const testLayerItem = screen.getByText('test-layer').closest('.layer-item');
      const renameButton = testLayerItem?.querySelector('.rename-btn') as HTMLElement;
      fireEvent.click(renameButton);

      const input = screen.getByPlaceholderText('Layer name');
      fireEvent.change(input, { target: { value: 'renamed' } });

      const submitButton = screen.getByText('Rename');
      fireEvent.click(submitButton);

      await waitFor(() => {
        const updatedStore = useConfigBuilderStore.getState();
        expect(updatedStore.isDirty).toBe(true);
      });
    });
  });
});
