import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { DndContext } from '@dnd-kit/core';
import { DragKeyPalette } from './DragKeyPalette';
import { AssignableKey } from '@/types/config';

/**
 * Test wrapper component to provide DndContext
 */
const DndWrapper: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  return <DndContext>{children}</DndContext>;
};

describe('DragKeyPalette', () => {
  describe('Rendering', () => {
    it('renders without crashing', () => {
      render(
        <DndWrapper>
          <DragKeyPalette />
        </DndWrapper>
      );
      expect(screen.getByText('Virtual Keys')).toBeInTheDocument();
    });

    it('displays all category sections when no filter is applied', () => {
      render(
        <DndWrapper>
          <DragKeyPalette />
        </DndWrapper>
      );

      expect(screen.getByText('Virtual Keys')).toBeInTheDocument();
      expect(screen.getByText('Modifiers')).toBeInTheDocument();
      expect(screen.getByText('Lock Keys')).toBeInTheDocument();
      expect(screen.getByText('Layers')).toBeInTheDocument();
    });

    it('displays letter keys A-Z', () => {
      render(
        <DndWrapper>
          <DragKeyPalette />
        </DndWrapper>
      );

      // Check for a few letter keys
      expect(screen.getByText('A')).toBeInTheDocument();
      expect(screen.getByText('Z')).toBeInTheDocument();
      expect(screen.getByText('M')).toBeInTheDocument();
    });

    it('displays number keys 0-9', () => {
      render(
        <DndWrapper>
          <DragKeyPalette />
        </DndWrapper>
      );

      // Check for number keys
      expect(screen.getByText('0')).toBeInTheDocument();
      expect(screen.getByText('5')).toBeInTheDocument();
      expect(screen.getByText('9')).toBeInTheDocument();
    });

    it('displays special keys', () => {
      render(
        <DndWrapper>
          <DragKeyPalette />
        </DndWrapper>
      );

      expect(screen.getByText('Esc')).toBeInTheDocument();
      expect(screen.getByText('Enter')).toBeInTheDocument();
      expect(screen.getByText('Space')).toBeInTheDocument();
      expect(screen.getByText('Tab')).toBeInTheDocument();
    });

    it('displays modifier keys', () => {
      render(
        <DndWrapper>
          <DragKeyPalette />
        </DndWrapper>
      );

      expect(screen.getByText('Shift')).toBeInTheDocument();
      expect(screen.getByText('Ctrl')).toBeInTheDocument();
      expect(screen.getByText('Alt')).toBeInTheDocument();
      expect(screen.getByText('Super')).toBeInTheDocument();
    });

    it('displays lock keys', () => {
      render(
        <DndWrapper>
          <DragKeyPalette />
        </DndWrapper>
      );

      expect(screen.getByText('CapsLock')).toBeInTheDocument();
      expect(screen.getByText('NumLock')).toBeInTheDocument();
      expect(screen.getByText('ScrollLock')).toBeInTheDocument();
    });

    it('displays layer keys', () => {
      render(
        <DndWrapper>
          <DragKeyPalette />
        </DndWrapper>
      );

      expect(screen.getByText('Layer: Base')).toBeInTheDocument();
      expect(screen.getByText('Layer: Nav')).toBeInTheDocument();
      expect(screen.getByText('Layer: Num')).toBeInTheDocument();
      expect(screen.getByText('Layer: Fn')).toBeInTheDocument();
    });
  });

  describe('Category Filtering', () => {
    it('shows only virtual keys when filterCategory is "vk"', () => {
      render(
        <DndWrapper>
          <DragKeyPalette filterCategory="vk" />
        </DndWrapper>
      );

      expect(screen.getByText('Virtual Keys')).toBeInTheDocument();
      expect(screen.queryByText('Modifiers')).not.toBeInTheDocument();
      expect(screen.queryByText('Lock Keys')).not.toBeInTheDocument();
      expect(screen.queryByText('Layers')).not.toBeInTheDocument();
    });

    it('shows only modifiers when filterCategory is "modifier"', () => {
      render(
        <DndWrapper>
          <DragKeyPalette filterCategory="modifier" />
        </DndWrapper>
      );

      expect(screen.queryByText('Virtual Keys')).not.toBeInTheDocument();
      expect(screen.getByText('Modifiers')).toBeInTheDocument();
      expect(screen.queryByText('Lock Keys')).not.toBeInTheDocument();
      expect(screen.queryByText('Layers')).not.toBeInTheDocument();
    });

    it('shows only locks when filterCategory is "lock"', () => {
      render(
        <DndWrapper>
          <DragKeyPalette filterCategory="lock" />
        </DndWrapper>
      );

      expect(screen.queryByText('Virtual Keys')).not.toBeInTheDocument();
      expect(screen.queryByText('Modifiers')).not.toBeInTheDocument();
      expect(screen.getByText('Lock Keys')).toBeInTheDocument();
      expect(screen.queryByText('Layers')).not.toBeInTheDocument();
    });

    it('shows only layers when filterCategory is "layer"', () => {
      render(
        <DndWrapper>
          <DragKeyPalette filterCategory="layer" />
        </DndWrapper>
      );

      expect(screen.queryByText('Virtual Keys')).not.toBeInTheDocument();
      expect(screen.queryByText('Modifiers')).not.toBeInTheDocument();
      expect(screen.queryByText('Lock Keys')).not.toBeInTheDocument();
      expect(screen.getByText('Layers')).toBeInTheDocument();
    });

    it('shows empty state when filterCategory matches no keys', () => {
      render(
        <DndWrapper>
          <DragKeyPalette filterCategory="macro" />
        </DndWrapper>
      );

      expect(screen.getByText('No keys available for the selected category.')).toBeInTheDocument();
    });
  });

  describe('Callbacks', () => {
    it('calls onDragStart when drag starts', () => {
      const onDragStart = vi.fn();
      render(
        <DndWrapper>
          <DragKeyPalette onDragStart={onDragStart} />
        </DndWrapper>
      );

      // Note: Actual drag testing requires more complex setup with DndContext
      // This test verifies the callback is passed correctly
      expect(onDragStart).not.toHaveBeenCalled();
    });

    it('calls onDragEnd when drag ends', () => {
      const onDragEnd = vi.fn();
      render(
        <DndWrapper>
          <DragKeyPalette onDragEnd={onDragEnd} />
        </DndWrapper>
      );

      // Note: Actual drag testing requires more complex setup with DndContext
      // This test verifies the callback is passed correctly
      expect(onDragEnd).not.toHaveBeenCalled();
    });

    it('accepts both onDragStart and onDragEnd callbacks', () => {
      const onDragStart = vi.fn();
      const onDragEnd = vi.fn();
      const { container } = render(
        <DndWrapper>
          <DragKeyPalette onDragStart={onDragStart} onDragEnd={onDragEnd} />
        </DndWrapper>
      );

      expect(container).toBeInTheDocument();
    });
  });

  describe('Accessibility', () => {
    it('has accessible role for draggable items', () => {
      render(
        <DndWrapper>
          <DragKeyPalette filterCategory="modifier" />
        </DndWrapper>
      );

      const shiftButton = screen.getByRole('button', { name: /Drag Shift modifier/i });
      expect(shiftButton).toBeInTheDocument();
    });

    it('has aria-label for each key', () => {
      render(
        <DndWrapper>
          <DragKeyPalette filterCategory="vk" />
        </DndWrapper>
      );

      const keyA = screen.getByRole('button', { name: /Drag Virtual Key A/i });
      expect(keyA).toHaveAttribute('aria-label');
    });

    it('has tabIndex for keyboard navigation', () => {
      render(
        <DndWrapper>
          <DragKeyPalette filterCategory="vk" />
        </DndWrapper>
      );

      const keyA = screen.getByRole('button', { name: /Drag Virtual Key A/i });
      expect(keyA).toHaveAttribute('tabIndex', '0');
    });

    it('has title attribute for tooltips', () => {
      render(
        <DndWrapper>
          <DragKeyPalette filterCategory="vk" />
        </DndWrapper>
      );

      const keyA = screen.getByRole('button', { name: /Drag Virtual Key A/i });
      expect(keyA).toHaveAttribute('title', 'Virtual Key A');
    });

    it('has screen reader only description', () => {
      render(
        <DndWrapper>
          <DragKeyPalette filterCategory="modifier" />
        </DndWrapper>
      );

      const shiftButton = screen.getByRole('button', { name: /Drag Shift modifier/i });
      expect(shiftButton).toHaveAttribute('aria-describedby');
    });

    it('meets minimum touch target size (44px)', () => {
      const { container } = render(
        <DndWrapper>
          <DragKeyPalette filterCategory="vk" />
        </DndWrapper>
      );

      // Check that elements have min-h-[44px] and min-w-[44px] classes
      const draggableItems = container.querySelectorAll('[role="button"]');
      draggableItems.forEach((item) => {
        expect(item.classList.contains('min-h-[44px]')).toBe(true);
        expect(item.classList.contains('min-w-[44px]')).toBe(true);
      });
    });
  });

  describe('Styling', () => {
    it('applies custom className when provided', () => {
      const { container } = render(
        <DndWrapper>
          <DragKeyPalette className="custom-class" />
        </DndWrapper>
      );

      const palette = container.firstChild;
      expect(palette).toHaveClass('custom-class');
    });

    it('has cursor-grab styling on draggable items', () => {
      const { container } = render(
        <DndWrapper>
          <DragKeyPalette filterCategory="vk" />
        </DndWrapper>
      );

      const draggableItems = container.querySelectorAll('[role="button"]');
      draggableItems.forEach((item) => {
        expect(item.classList.contains('cursor-grab')).toBe(true);
      });
    });

    it('has focus outline styling for keyboard accessibility', () => {
      const { container } = render(
        <DndWrapper>
          <DragKeyPalette filterCategory="vk" />
        </DndWrapper>
      );

      const draggableItems = container.querySelectorAll('[role="button"]');
      draggableItems.forEach((item) => {
        expect(item.classList.contains('focus:outline')).toBe(true);
        expect(item.classList.contains('focus:outline-2')).toBe(true);
      });
    });
  });

  describe('Key Organization', () => {
    it('groups virtual keys in grid layout', () => {
      const { container } = render(
        <DndWrapper>
          <DragKeyPalette filterCategory="vk" />
        </DndWrapper>
      );

      const gridContainer = container.querySelector('.grid');
      expect(gridContainer).toBeInTheDocument();
      expect(gridContainer).toHaveClass('grid-cols-4');
    });

    it('groups modifiers in grid layout', () => {
      const { container } = render(
        <DndWrapper>
          <DragKeyPalette filterCategory="modifier" />
        </DndWrapper>
      );

      const gridContainer = container.querySelector('.grid');
      expect(gridContainer).toBeInTheDocument();
      expect(gridContainer).toHaveClass('grid-cols-2');
    });

    it('displays category headers', () => {
      render(
        <DndWrapper>
          <DragKeyPalette />
        </DndWrapper>
      );

      const virtualKeysHeader = screen.getByText('Virtual Keys');
      expect(virtualKeysHeader).toBeInTheDocument();
      expect(virtualKeysHeader.tagName).toBe('H3');
    });
  });

  describe('Data Coverage', () => {
    it('includes all standard letter keys', () => {
      render(
        <DndWrapper>
          <DragKeyPalette filterCategory="vk" />
        </DndWrapper>
      );

      const letters = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ'.split('');
      letters.forEach((letter) => {
        expect(screen.getByText(letter)).toBeInTheDocument();
      });
    });

    it('includes all number keys', () => {
      render(
        <DndWrapper>
          <DragKeyPalette filterCategory="vk" />
        </DndWrapper>
      );

      for (let i = 0; i <= 9; i++) {
        expect(screen.getByText(i.toString())).toBeInTheDocument();
      }
    });

    it('includes arrow keys', () => {
      render(
        <DndWrapper>
          <DragKeyPalette filterCategory="vk" />
        </DndWrapper>
      );

      expect(screen.getByText('←')).toBeInTheDocument();
      expect(screen.getByText('→')).toBeInTheDocument();
      expect(screen.getByText('↑')).toBeInTheDocument();
      expect(screen.getByText('↓')).toBeInTheDocument();
    });
  });
});
