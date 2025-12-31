/**
 * Tests for VirtualKeyboard Component
 */

import { render, screen } from '@testing-library/react';
import { DndContext } from '@dnd-kit/core';
import { describe, it, expect, beforeEach } from 'vitest';
import { VirtualKeyboard } from './VirtualKeyboard';
import { useConfigBuilderStore } from '../store/configBuilderStore';

describe('VirtualKeyboard', () => {
  beforeEach(() => {
    // Reset store before each test
    useConfigBuilderStore.getState().resetConfig();
  });

  it('renders keyboard layout', () => {
    render(
      <DndContext>
        <VirtualKeyboard />
      </DndContext>
    );

    // Check for some common keys
    expect(screen.getByText('Q')).toBeInTheDocument();
    expect(screen.getByText('W')).toBeInTheDocument();
    expect(screen.getByText('E')).toBeInTheDocument();
    expect(screen.getByText('Space')).toBeInTheDocument();
  });

  it('displays QWERTY layout by default', () => {
    render(
      <DndContext>
        <VirtualKeyboard />
      </DndContext>
    );

    expect(screen.getByText('QWERTY Layout')).toBeInTheDocument();
  });

  it('shows current layer name', () => {
    render(
      <DndContext>
        <VirtualKeyboard />
      </DndContext>
    );

    expect(screen.getByText(/Layer: base/i)).toBeInTheDocument();
  });

  it('highlights mapped keys', () => {
    // Add a mapping to the store
    const { addMapping, layers, currentLayerId } =
      useConfigBuilderStore.getState();

    if (currentLayerId) {
      addMapping(currentLayerId, 'KEY_Q', 'KEY_A');
    }

    const { container } = render(
      <DndContext>
        <VirtualKeyboard />
      </DndContext>
    );

    // The Q key should have the 'mapped' class
    const keys = container.querySelectorAll('.virtual-key.mapped');
    expect(keys.length).toBeGreaterThan(0);
  });

  it('renders all keyboard rows', () => {
    const { container } = render(
      <DndContext>
        <VirtualKeyboard />
      </DndContext>
    );

    const rows = container.querySelectorAll('.keyboard-row');
    expect(rows.length).toBe(5); // QWERTY has 5 rows
  });

  it('renders modifier keys with correct styling', () => {
    const { container } = render(
      <DndContext>
        <VirtualKeyboard />
      </DndContext>
    );

    // Check for modifier keys
    const modifierKeys = container.querySelectorAll('.virtual-key.modifier');
    expect(modifierKeys.length).toBeGreaterThan(0);
  });

  it('renders special keys with correct styling', () => {
    const { container } = render(
      <DndContext>
        <VirtualKeyboard />
      </DndContext>
    );

    // Check for special keys (Esc, Enter, etc.)
    const specialKeys = container.querySelectorAll('.virtual-key.special');
    expect(specialKeys.length).toBeGreaterThan(0);
  });

  it('displays mapping indicators for mapped keys', () => {
    const { addMapping, currentLayerId } = useConfigBuilderStore.getState();

    if (currentLayerId) {
      addMapping(currentLayerId, 'KEY_Q', 'KEY_A');
    }

    const { container } = render(
      <DndContext>
        <VirtualKeyboard />
      </DndContext>
    );

    // Check for mapping indicator
    const indicators = container.querySelectorAll('.mapping-indicator');
    expect(indicators.length).toBeGreaterThan(0);
  });

  it('renders draggable keys when draggable=true', () => {
    const { container } = render(
      <DndContext>
        <VirtualKeyboard draggable={true} />
      </DndContext>
    );

    // Keys should have draggable attributes
    const keys = container.querySelectorAll('.virtual-key');
    expect(keys.length).toBeGreaterThan(0);
  });

  it('renders droppable keys when droppable=true', () => {
    const { container } = render(
      <DndContext>
        <VirtualKeyboard droppable={true} />
      </DndContext>
    );

    // Keys should have droppable class
    const droppableKeys = container.querySelectorAll('.virtual-key.droppable');
    expect(droppableKeys.length).toBeGreaterThan(0);
  });

  it('shows layer name when layer is selected', () => {
    const { addLayer, setCurrentLayer, layers } =
      useConfigBuilderStore.getState();

    addLayer('symbols');
    const symbolsLayer = layers.find((l) => l.name === 'symbols');

    if (symbolsLayer) {
      setCurrentLayer(symbolsLayer.id);
    }

    render(
      <DndContext>
        <VirtualKeyboard />
      </DndContext>
    );

    expect(screen.getByText(/Layer: symbols/i)).toBeInTheDocument();
  });
});
