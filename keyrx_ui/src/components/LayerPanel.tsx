/**
 * LayerPanel component for managing keyboard configuration layers
 *
 * Features:
 * - List all layers with current layer highlighting
 * - Drag-to-reorder layers
 * - Add/delete/rename layer actions
 * - Base layer protection (cannot be deleted)
 */

import React, { useState } from 'react';
import {
  DndContext,
  closestCenter,
  KeyboardSensor,
  PointerSensor,
  useSensor,
  useSensors,
  DragEndEvent,
} from '@dnd-kit/core';
import {
  arrayMove,
  SortableContext,
  sortableKeyboardCoordinates,
  useSortable,
  verticalListSortingStrategy,
} from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
import { useConfigBuilderStore } from '../store/configBuilderStore';
import { Layer } from '../types/configBuilder';
import './LayerPanel.css';

interface SortableLayerItemProps {
  layer: Layer;
  isActive: boolean;
  onSelect: () => void;
  onRename: () => void;
  onDelete: () => void;
}

/**
 * Individual sortable layer item
 */
function SortableLayerItem({
  layer,
  isActive,
  onSelect,
  onRename,
  onDelete,
}: SortableLayerItemProps) {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id: layer.id });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.5 : 1,
  };

  return (
    <div
      ref={setNodeRef}
      style={style}
      className={`layer-item ${isActive ? 'active' : ''} ${layer.isBase ? 'base' : ''}`}
    >
      <button
        className="layer-drag-handle"
        {...attributes}
        {...listeners}
        aria-label={`Drag to reorder ${layer.name}`}
      >
        <span className="drag-icon" aria-hidden="true">⋮⋮</span>
      </button>
      <div className="layer-info" onClick={onSelect} role="button" tabIndex={0} onKeyDown={(e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault();
          onSelect();
        }
      }}>
        <span className="layer-name">{layer.name}</span>
        <span className="layer-mapping-count">
          {layer.mappings.length} mapping{layer.mappings.length !== 1 ? 's' : ''}
        </span>
      </div>
      <div className="layer-actions">
        <button
          className="layer-action-btn rename-btn"
          onClick={onRename}
          aria-label={`Rename ${layer.name}`}
          title="Rename layer"
        >
          <span aria-hidden="true">✏️</span>
        </button>
        {!layer.isBase && (
          <button
            className="layer-action-btn delete-btn"
            onClick={onDelete}
            aria-label={`Delete ${layer.name}`}
            title="Delete layer"
          >
            <span aria-hidden="true">🗑️</span>
          </button>
        )}
      </div>
    </div>
  );
}

interface RenameDialogProps {
  currentName: string;
  onConfirm: (newName: string) => void;
  onCancel: () => void;
}

/**
 * Simple rename dialog
 */
function RenameDialog({ currentName, onConfirm, onCancel }: RenameDialogProps) {
  const [newName, setNewName] = useState(currentName);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (newName.trim()) {
      onConfirm(newName.trim());
    }
  };

  return (
    <div className="rename-dialog-overlay" onClick={onCancel}>
      <div className="rename-dialog" onClick={(e) => e.stopPropagation()}>
        <h3>Rename Layer</h3>
        <form onSubmit={handleSubmit}>
          <input
            type="text"
            value={newName}
            onChange={(e) => setNewName(e.target.value)}
            autoFocus
            placeholder="Layer name"
          />
          <div className="dialog-buttons">
            <button type="button" onClick={onCancel}>
              Cancel
            </button>
            <button type="submit" disabled={!newName.trim()}>
              Rename
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

/**
 * LayerPanel component
 */
export function LayerPanel() {
  const {
    layers,
    currentLayerId,
    addLayer,
    removeLayer,
    renameLayer,
    setCurrentLayer,
    reorderLayers,
  } = useConfigBuilderStore();

  const [renamingLayerId, setRenamingLayerId] = useState<string | null>(null);

  const sensors = useSensors(
    useSensor(PointerSensor),
    useSensor(KeyboardSensor, {
      coordinateGetter: sortableKeyboardCoordinates,
    })
  );

  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event;

    if (over && active.id !== over.id) {
      const oldIndex = layers.findIndex((l) => l.id === active.id);
      const newIndex = layers.findIndex((l) => l.id === over.id);

      if (oldIndex !== -1 && newIndex !== -1) {
        reorderLayers(oldIndex, newIndex);
      }
    }
  };

  const handleAddLayer = () => {
    addLayer();
  };

  const handleDeleteLayer = (layerId: string) => {
    if (window.confirm('Are you sure you want to delete this layer?')) {
      removeLayer(layerId);
    }
  };

  const handleRenameLayer = (layerId: string, newName: string) => {
    renameLayer(layerId, newName);
    setRenamingLayerId(null);
  };

  const renamingLayer = layers.find((l) => l.id === renamingLayerId);

  return (
    <div className="layer-panel">
      <div className="layer-panel-header">
        <h2>Layers</h2>
        <button className="add-layer-btn" onClick={handleAddLayer} title="Add new layer">
          + Add Layer
        </button>
      </div>

      <DndContext
        sensors={sensors}
        collisionDetection={closestCenter}
        onDragEnd={handleDragEnd}
      >
        <SortableContext items={layers.map((l) => l.id)} strategy={verticalListSortingStrategy}>
          <div className="layer-list">
            {layers.map((layer) => (
              <SortableLayerItem
                key={layer.id}
                layer={layer}
                isActive={layer.id === currentLayerId}
                onSelect={() => setCurrentLayer(layer.id)}
                onRename={() => setRenamingLayerId(layer.id)}
                onDelete={() => handleDeleteLayer(layer.id)}
              />
            ))}
          </div>
        </SortableContext>
      </DndContext>

      {renamingLayer && (
        <RenameDialog
          currentName={renamingLayer.name}
          onConfirm={(newName) => handleRenameLayer(renamingLayer.id, newName)}
          onCancel={() => setRenamingLayerId(null)}
        />
      )}
    </div>
  );
}
