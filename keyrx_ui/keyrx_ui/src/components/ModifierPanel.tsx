/**
 * ModifierPanel Component
 *
 * Displays and manages modifiers and locks in the visual config builder.
 * Users can add, remove, and assign trigger keys via drag-and-drop.
 */

import React, { useState } from 'react';
import { useDroppable } from '@dnd-kit/core';
import { useConfigBuilderStore } from '../store/configBuilderStore';
import type { DragData } from '../types/configBuilder';
import './ModifierPanel.css';

interface ModifierItemProps {
  id: string;
  name: string;
  triggerKey: string;
  type: 'modifier' | 'lock';
  onRemove: () => void;
}

/**
 * Individual modifier or lock item with droppable area for trigger key assignment
 */
const ModifierItem: React.FC<ModifierItemProps> = ({
  id,
  name,
  triggerKey,
  type,
  onRemove,
}) => {
  const { setNodeRef, isOver } = useDroppable({
    id: `${type}-${id}`,
    data: { type, id } as DragData,
  });

  return (
    <div className="modifier-item">
      <div className="modifier-info">
        <span className="modifier-name">{name}</span>
        <div
          ref={setNodeRef}
          className={`modifier-trigger ${isOver ? 'drag-over' : ''}`}
        >
          {triggerKey || 'Drop key here'}
        </div>
      </div>
      <button
        className="modifier-remove"
        onClick={onRemove}
        aria-label={`Remove ${name}`}
      >
        ×
      </button>
    </div>
  );
};

interface AddDialogProps {
  type: 'modifier' | 'lock';
  onAdd: (name: string) => void;
  onCancel: () => void;
}

/**
 * Dialog for adding a new modifier or lock
 */
const AddDialog: React.FC<AddDialogProps> = ({ type, onAdd, onCancel }) => {
  const [name, setName] = useState('');

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    const trimmedName = name.trim();
    if (trimmedName) {
      onAdd(trimmedName);
      setName('');
    }
  };

  const handleOverlayClick = (e: React.MouseEvent) => {
    if (e.target === e.currentTarget) {
      onCancel();
    }
  };

  return (
    <div className="dialog-overlay" onClick={handleOverlayClick}>
      <div className="dialog">
        <h3>Add {type === 'modifier' ? 'Modifier' : 'Lock'}</h3>
        <form onSubmit={handleSubmit}>
          <input
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder={`Enter ${type} name`}
            autoFocus
          />
          <div className="dialog-actions">
            <button type="button" onClick={onCancel}>
              Cancel
            </button>
            <button type="submit" disabled={!name.trim()}>
              Add
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};

/**
 * ModifierPanel Component
 *
 * Displays all modifiers and locks with their trigger keys.
 * Allows adding/removing and assigning trigger keys via drag-and-drop.
 */
export const ModifierPanel: React.FC = () => {
  const { modifiers, locks, addModifier, removeModifier, addLock, removeLock } =
    useConfigBuilderStore();
  const [showAddModifier, setShowAddModifier] = useState(false);
  const [showAddLock, setShowAddLock] = useState(false);

  const handleAddModifier = (name: string) => {
    addModifier(name, '');
    setShowAddModifier(false);
  };

  const handleAddLock = (name: string) => {
    addLock(name, '');
    setShowAddLock(false);
  };

  return (
    <div className="modifier-panel">
      <div className="modifier-section">
        <div className="section-header">
          <h3>Modifiers</h3>
          <button
            className="add-button"
            onClick={() => setShowAddModifier(true)}
            aria-label="Add modifier"
          >
            + Add
          </button>
        </div>
        <div className="modifier-list">
          {modifiers.length === 0 ? (
            <p className="empty-message">
              No modifiers. Click &quot;+ Add&quot; to create one.
            </p>
          ) : (
            modifiers.map((modifier) => (
              <ModifierItem
                key={modifier.id}
                id={modifier.id}
                name={modifier.name}
                triggerKey={modifier.triggerKey}
                type="modifier"
                onRemove={() => removeModifier(modifier.id)}
              />
            ))
          )}
        </div>
      </div>

      <div className="modifier-section">
        <div className="section-header">
          <h3>Locks</h3>
          <button
            className="add-button"
            onClick={() => setShowAddLock(true)}
            aria-label="Add lock"
          >
            + Add
          </button>
        </div>
        <div className="modifier-list">
          {locks.length === 0 ? (
            <p className="empty-message">
              No locks. Click &quot;+ Add&quot; to create one.
            </p>
          ) : (
            locks.map((lock) => (
              <ModifierItem
                key={lock.id}
                id={lock.id}
                name={lock.name}
                triggerKey={lock.triggerKey}
                type="lock"
                onRemove={() => removeLock(lock.id)}
              />
            ))
          )}
        </div>
      </div>

      {showAddModifier && (
        <AddDialog
          type="modifier"
          onAdd={handleAddModifier}
          onCancel={() => setShowAddModifier(false)}
        />
      )}

      {showAddLock && (
        <AddDialog
          type="lock"
          onAdd={handleAddLock}
          onCancel={() => setShowAddLock(false)}
        />
      )}
    </div>
  );
};
