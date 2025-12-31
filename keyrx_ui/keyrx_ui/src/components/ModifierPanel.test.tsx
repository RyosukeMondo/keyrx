/**
 * ModifierPanel Component Unit Tests
 *
 * Tests modifier and lock management functionality including
 * adding, removing, and drag-and-drop trigger key assignment.
 */

import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { DndContext } from '@dnd-kit/core';
import { ModifierPanel } from './ModifierPanel';
import { useConfigBuilderStore } from '../store/configBuilderStore';

// Mock the store
vi.mock('../store/configBuilderStore');

const mockUseConfigBuilderStore = useConfigBuilderStore as ReturnType<typeof vi.fn>;

describe('ModifierPanel', () => {
  const mockAddModifier = vi.fn();
  const mockRemoveModifier = vi.fn();
  const mockAddLock = vi.fn();
  const mockRemoveLock = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
    mockUseConfigBuilderStore.mockReturnValue({
      modifiers: [],
      locks: [],
      addModifier: mockAddModifier,
      removeModifier: mockRemoveModifier,
      addLock: mockAddLock,
      removeLock: mockRemoveLock,
      // Other store methods (unused in this component)
      layers: [],
      currentLayerId: 'base',
      isDirty: false,
      addLayer: vi.fn(),
      removeLayer: vi.fn(),
      renameLayer: vi.fn(),
      setCurrentLayer: vi.fn(),
      reorderLayers: vi.fn(),
      addMapping: vi.fn(),
      removeMapping: vi.fn(),
      updateMapping: vi.fn(),
      clearMappings: vi.fn(),
      updateModifier: vi.fn(),
      updateLock: vi.fn(),
      setConfig: vi.fn(),
      resetConfig: vi.fn(),
      markDirty: vi.fn(),
      markClean: vi.fn(),
    });
  });

  describe('Rendering', () => {
    it('renders empty state for modifiers and locks', () => {
      render(
        <DndContext>
          <ModifierPanel />
        </DndContext>
      );

      expect(screen.getByText('Modifiers')).toBeInTheDocument();
      expect(screen.getByText('Locks')).toBeInTheDocument();
      expect(
        screen.getByText(/No modifiers. Click/)
      ).toBeInTheDocument();
      expect(screen.getByText(/No locks. Click/)).toBeInTheDocument();
    });

    it('renders modifiers when present', () => {
      mockUseConfigBuilderStore.mockReturnValue({
        ...mockUseConfigBuilderStore(),
        modifiers: [
          { id: 'm1', name: 'my_shift', triggerKey: 'KEY_CAPSLOCK', active: false },
          { id: 'm2', name: 'my_ctrl', triggerKey: '', active: false },
        ],
      });

      render(
        <DndContext>
          <ModifierPanel />
        </DndContext>
      );

      expect(screen.getByText('my_shift')).toBeInTheDocument();
      expect(screen.getByText('my_ctrl')).toBeInTheDocument();
      expect(screen.getByText('KEY_CAPSLOCK')).toBeInTheDocument();
      expect(screen.getByText('Drop key here')).toBeInTheDocument();
    });

    it('renders locks when present', () => {
      mockUseConfigBuilderStore.mockReturnValue({
        ...mockUseConfigBuilderStore(),
        locks: [
          { id: 'l1', name: 'caps_lock', triggerKey: 'KEY_F1', active: false },
          { id: 'l2', name: 'num_lock', triggerKey: 'KEY_F2', active: false },
        ],
      });

      render(
        <DndContext>
          <ModifierPanel />
        </DndContext>
      );

      expect(screen.getByText('caps_lock')).toBeInTheDocument();
      expect(screen.getByText('num_lock')).toBeInTheDocument();
      expect(screen.getByText('KEY_F1')).toBeInTheDocument();
      expect(screen.getByText('KEY_F2')).toBeInTheDocument();
    });
  });

  describe('Adding Modifiers', () => {
    it('opens add modifier dialog when button clicked', () => {
      render(
        <DndContext>
          <ModifierPanel />
        </DndContext>
      );

      const addButton = screen.getAllByLabelText('Add modifier')[0];
      fireEvent.click(addButton);

      expect(screen.getByText('Add Modifier')).toBeInTheDocument();
      expect(screen.getByPlaceholderText('Enter modifier name')).toBeInTheDocument();
    });

    it('adds modifier when dialog form submitted', () => {
      render(
        <DndContext>
          <ModifierPanel />
        </DndContext>
      );

      const addButton = screen.getAllByLabelText('Add modifier')[0];
      fireEvent.click(addButton);

      const input = screen.getByPlaceholderText('Enter modifier name');
      fireEvent.change(input, { target: { value: 'my_modifier' } });

      const submitButton = screen.getByText('Add');
      fireEvent.click(submitButton);

      expect(mockAddModifier).toHaveBeenCalledWith('my_modifier', '');
    });

    it('trims whitespace from modifier name', () => {
      render(
        <DndContext>
          <ModifierPanel />
        </DndContext>
      );

      const addButton = screen.getAllByLabelText('Add modifier')[0];
      fireEvent.click(addButton);

      const input = screen.getByPlaceholderText('Enter modifier name');
      fireEvent.change(input, { target: { value: '  spaced_name  ' } });

      const submitButton = screen.getByText('Add');
      fireEvent.click(submitButton);

      expect(mockAddModifier).toHaveBeenCalledWith('spaced_name', '');
    });

    it('closes dialog on cancel', () => {
      render(
        <DndContext>
          <ModifierPanel />
        </DndContext>
      );

      const addButton = screen.getAllByLabelText('Add modifier')[0];
      fireEvent.click(addButton);

      expect(screen.getByText('Add Modifier')).toBeInTheDocument();

      const cancelButton = screen.getByText('Cancel');
      fireEvent.click(cancelButton);

      expect(screen.queryByText('Add Modifier')).not.toBeInTheDocument();
    });

    it('closes dialog on overlay click', () => {
      render(
        <DndContext>
          <ModifierPanel />
        </DndContext>
      );

      const addButton = screen.getAllByLabelText('Add modifier')[0];
      fireEvent.click(addButton);

      const overlay = screen.getByText('Add Modifier').closest('.dialog-overlay');
      fireEvent.click(overlay!);

      expect(screen.queryByText('Add Modifier')).not.toBeInTheDocument();
    });
  });

  describe('Adding Locks', () => {
    it('opens add lock dialog when button clicked', () => {
      render(
        <DndContext>
          <ModifierPanel />
        </DndContext>
      );

      const addButton = screen.getAllByLabelText('Add lock')[0];
      fireEvent.click(addButton);

      expect(screen.getByText('Add Lock')).toBeInTheDocument();
      expect(screen.getByPlaceholderText('Enter lock name')).toBeInTheDocument();
    });

    it('adds lock when dialog form submitted', () => {
      render(
        <DndContext>
          <ModifierPanel />
        </DndContext>
      );

      const addButton = screen.getAllByLabelText('Add lock')[0];
      fireEvent.click(addButton);

      const input = screen.getByPlaceholderText('Enter lock name');
      fireEvent.change(input, { target: { value: 'my_lock' } });

      const submitButton = screen.getByText('Add');
      fireEvent.click(submitButton);

      expect(mockAddLock).toHaveBeenCalledWith('my_lock', '');
    });
  });

  describe('Removing Modifiers and Locks', () => {
    it('removes modifier when remove button clicked', () => {
      mockUseConfigBuilderStore.mockReturnValue({
        ...mockUseConfigBuilderStore(),
        modifiers: [
          { id: 'm1', name: 'my_shift', triggerKey: 'KEY_CAPSLOCK', active: false },
        ],
      });

      render(
        <DndContext>
          <ModifierPanel />
        </DndContext>
      );

      const removeButton = screen.getByLabelText('Remove my_shift');
      fireEvent.click(removeButton);

      expect(mockRemoveModifier).toHaveBeenCalledWith('m1');
    });

    it('removes lock when remove button clicked', () => {
      mockUseConfigBuilderStore.mockReturnValue({
        ...mockUseConfigBuilderStore(),
        locks: [
          { id: 'l1', name: 'caps_lock', triggerKey: 'KEY_F1', active: false },
        ],
      });

      render(
        <DndContext>
          <ModifierPanel />
        </DndContext>
      );

      const removeButton = screen.getByLabelText('Remove caps_lock');
      fireEvent.click(removeButton);

      expect(mockRemoveLock).toHaveBeenCalledWith('l1');
    });
  });

  describe('Drag and Drop', () => {
    it('renders droppable area for modifier trigger key', () => {
      mockUseConfigBuilderStore.mockReturnValue({
        ...mockUseConfigBuilderStore(),
        modifiers: [
          { id: 'm1', name: 'my_shift', triggerKey: '', active: false },
        ],
      });

      render(
        <DndContext>
          <ModifierPanel />
        </DndContext>
      );

      expect(screen.getByText('Drop key here')).toBeInTheDocument();
    });

    it('renders droppable area for lock trigger key', () => {
      mockUseConfigBuilderStore.mockReturnValue({
        ...mockUseConfigBuilderStore(),
        locks: [
          { id: 'l1', name: 'caps_lock', triggerKey: '', active: false },
        ],
      });

      render(
        <DndContext>
          <ModifierPanel />
        </DndContext>
      );

      expect(screen.getByText('Drop key here')).toBeInTheDocument();
    });
  });
});
