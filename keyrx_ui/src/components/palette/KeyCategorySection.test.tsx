import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { KeyCategorySection } from './KeyCategorySection';
import type { PaletteKey } from '../KeyPalette';

// Mock KeyPaletteItem component
vi.mock('../KeyPaletteItem', () => ({
  KeyPaletteItem: ({
    keyItem,
    isSelected,
    isFavorite,
    onClick,
    onToggleFavorite,
    showStar,
  }: {
    keyItem: PaletteKey;
    isSelected: boolean;
    isFavorite: boolean;
    onClick: () => void;
    onToggleFavorite?: () => void;
    showStar: boolean;
  }) => (
    <div
      data-testid={`key-item-${keyItem.id}`}
      data-selected={isSelected}
      data-favorite={isFavorite}
      data-show-star={showStar}
    >
      <button onClick={onClick}>{keyItem.label}</button>
      {showStar && onToggleFavorite && (
        <button onClick={onToggleFavorite} data-testid={`star-${keyItem.id}`}>
          Star
        </button>
      )}
    </div>
  ),
}));

describe('KeyCategorySection', () => {
  const mockKeys: PaletteKey[] = [
    { id: 'A', label: 'A', category: 'basic' },
    { id: 'B', label: 'B', category: 'basic' },
    { id: 'C', label: 'C', category: 'basic' },
  ];

  const mockOnKeySelect = vi.fn();
  const mockOnToggleFavorite = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Rendering', () => {
    it('renders section with title and keys', () => {
      render(
        <KeyCategorySection
          title="Test Section"
          keys={mockKeys}
          onKeySelect={mockOnKeySelect}
        />
      );

      expect(screen.getByText('Test Section')).toBeInTheDocument();
      expect(screen.getByText('(3)')).toBeInTheDocument();
      expect(screen.getByTestId('key-item-A')).toBeInTheDocument();
      expect(screen.getByTestId('key-item-B')).toBeInTheDocument();
      expect(screen.getByTestId('key-item-C')).toBeInTheDocument();
    });

    it('renders icon when provided', () => {
      const icon = <span data-testid="custom-icon">★</span>;

      render(
        <KeyCategorySection
          title="Test Section"
          icon={icon}
          keys={mockKeys}
          onKeySelect={mockOnKeySelect}
        />
      );

      expect(screen.getByTestId('custom-icon')).toBeInTheDocument();
      expect(screen.getByText('★')).toBeInTheDocument();
    });

    it('renders nothing when keys array is empty', () => {
      const { container } = render(
        <KeyCategorySection
          title="Empty Section"
          keys={[]}
          onKeySelect={mockOnKeySelect}
        />
      );

      expect(container.firstChild).toBeNull();
    });

    it('shows key count in header', () => {
      render(
        <KeyCategorySection
          title="Test Section"
          keys={mockKeys}
          onKeySelect={mockOnKeySelect}
        />
      );

      expect(screen.getByText(`(${mockKeys.length})`)).toBeInTheDocument();
    });
  });

  describe('Key Selection', () => {
    it('calls onKeySelect when a key is clicked', async () => {
      const user = userEvent.setup();

      render(
        <KeyCategorySection
          title="Test Section"
          keys={mockKeys}
          onKeySelect={mockOnKeySelect}
        />
      );

      await user.click(screen.getByText('A'));

      expect(mockOnKeySelect).toHaveBeenCalledWith(mockKeys[0]);
      expect(mockOnKeySelect).toHaveBeenCalledTimes(1);
    });

    it('highlights selected key', () => {
      render(
        <KeyCategorySection
          title="Test Section"
          keys={mockKeys}
          onKeySelect={mockOnKeySelect}
          selectedKeyId="B"
        />
      );

      const keyB = screen.getByTestId('key-item-B');
      expect(keyB).toHaveAttribute('data-selected', 'true');

      const keyA = screen.getByTestId('key-item-A');
      expect(keyA).toHaveAttribute('data-selected', 'false');
    });
  });

  describe('Favorites', () => {
    it('shows favorite stars when showFavorites is true', () => {
      render(
        <KeyCategorySection
          title="Test Section"
          keys={mockKeys}
          onKeySelect={mockOnKeySelect}
          showFavorites={true}
          onToggleFavorite={mockOnToggleFavorite}
        />
      );

      expect(screen.getByTestId('star-A')).toBeInTheDocument();
      expect(screen.getByTestId('star-B')).toBeInTheDocument();
      expect(screen.getByTestId('star-C')).toBeInTheDocument();
    });

    it('hides favorite stars when showFavorites is false', () => {
      render(
        <KeyCategorySection
          title="Test Section"
          keys={mockKeys}
          onKeySelect={mockOnKeySelect}
          showFavorites={false}
        />
      );

      expect(screen.queryByTestId('star-A')).not.toBeInTheDocument();
      expect(screen.queryByTestId('star-B')).not.toBeInTheDocument();
    });

    it('marks favorited keys correctly', () => {
      render(
        <KeyCategorySection
          title="Test Section"
          keys={mockKeys}
          onKeySelect={mockOnKeySelect}
          favoriteKeyIds={['A', 'C']}
          showFavorites={true}
          onToggleFavorite={mockOnToggleFavorite}
        />
      );

      expect(screen.getByTestId('key-item-A')).toHaveAttribute(
        'data-favorite',
        'true'
      );
      expect(screen.getByTestId('key-item-B')).toHaveAttribute(
        'data-favorite',
        'false'
      );
      expect(screen.getByTestId('key-item-C')).toHaveAttribute(
        'data-favorite',
        'true'
      );
    });

    it('calls onToggleFavorite when star is clicked', async () => {
      const user = userEvent.setup();

      render(
        <KeyCategorySection
          title="Test Section"
          keys={mockKeys}
          onKeySelect={mockOnKeySelect}
          showFavorites={true}
          onToggleFavorite={mockOnToggleFavorite}
        />
      );

      await user.click(screen.getByTestId('star-B'));

      expect(mockOnToggleFavorite).toHaveBeenCalledWith('B');
      expect(mockOnToggleFavorite).toHaveBeenCalledTimes(1);
    });
  });

  describe('Collapsible Behavior', () => {
    it('renders as non-collapsible by default', () => {
      render(
        <KeyCategorySection
          title="Test Section"
          keys={mockKeys}
          onKeySelect={mockOnKeySelect}
        />
      );

      // Should not have expand/collapse button
      expect(
        screen.queryByLabelText(/expand|collapse/i)
      ).not.toBeInTheDocument();

      // Keys should be visible
      expect(screen.getByTestId('key-item-A')).toBeInTheDocument();
    });

    it('renders collapse/expand button when collapsible', () => {
      render(
        <KeyCategorySection
          title="Test Section"
          keys={mockKeys}
          onKeySelect={mockOnKeySelect}
          collapsible={true}
        />
      );

      expect(
        screen.getByLabelText('Collapse Test Section section')
      ).toBeInTheDocument();
    });

    it('starts expanded by default when collapsible', () => {
      render(
        <KeyCategorySection
          title="Test Section"
          keys={mockKeys}
          onKeySelect={mockOnKeySelect}
          collapsible={true}
        />
      );

      expect(screen.getByTestId('key-item-A')).toBeInTheDocument();
      expect(
        screen.getByLabelText('Collapse Test Section section')
      ).toHaveAttribute('aria-expanded', 'true');
    });

    it('starts collapsed when defaultCollapsed is true', () => {
      render(
        <KeyCategorySection
          title="Test Section"
          keys={mockKeys}
          onKeySelect={mockOnKeySelect}
          collapsible={true}
          defaultCollapsed={true}
        />
      );

      expect(screen.queryByTestId('key-item-A')).not.toBeInTheDocument();
      expect(
        screen.getByLabelText('Expand Test Section section')
      ).toHaveAttribute('aria-expanded', 'false');
    });

    it('toggles collapsed state when button is clicked', async () => {
      const user = userEvent.setup();

      render(
        <KeyCategorySection
          title="Test Section"
          keys={mockKeys}
          onKeySelect={mockOnKeySelect}
          collapsible={true}
        />
      );

      // Initially expanded
      expect(screen.getByTestId('key-item-A')).toBeInTheDocument();

      // Click to collapse
      await user.click(screen.getByLabelText('Collapse Test Section section'));

      // Should be hidden
      expect(screen.queryByTestId('key-item-A')).not.toBeInTheDocument();

      // Click to expand
      await user.click(screen.getByLabelText('Expand Test Section section'));

      // Should be visible again
      expect(screen.getByTestId('key-item-A')).toBeInTheDocument();
    });
  });

  describe('View Modes', () => {
    it('applies grid view mode classes', () => {
      const { container } = render(
        <KeyCategorySection
          title="Test Section"
          keys={mockKeys}
          onKeySelect={mockOnKeySelect}
          viewMode="grid"
        />
      );

      const content = container.querySelector('[role="group"]');
      expect(content).toHaveClass('grid', 'grid-cols-8', 'gap-2');
    });

    it('applies list view mode classes', () => {
      const { container } = render(
        <KeyCategorySection
          title="Test Section"
          keys={mockKeys}
          onKeySelect={mockOnKeySelect}
          viewMode="list"
        />
      );

      const content = container.querySelector('[role="group"]');
      expect(content).toHaveClass('flex', 'flex-col', 'gap-2');
    });
  });

  describe('Accessibility', () => {
    it('has proper ARIA labels for key group', () => {
      render(
        <KeyCategorySection
          title="Test Section"
          keys={mockKeys}
          onKeySelect={mockOnKeySelect}
        />
      );

      expect(screen.getByRole('group')).toHaveAttribute(
        'aria-label',
        'Test Section keys'
      );
    });

    it('has proper ARIA attributes for collapsible button', () => {
      render(
        <KeyCategorySection
          title="Test Section"
          keys={mockKeys}
          onKeySelect={mockOnKeySelect}
          collapsible={true}
        />
      );

      const button = screen.getByLabelText('Collapse Test Section section');
      expect(button).toHaveAttribute('aria-expanded', 'true');
    });

    it('updates aria-expanded when collapsed state changes', async () => {
      const user = userEvent.setup();

      render(
        <KeyCategorySection
          title="Test Section"
          keys={mockKeys}
          onKeySelect={mockOnKeySelect}
          collapsible={true}
        />
      );

      const button = screen.getByLabelText('Collapse Test Section section');
      expect(button).toHaveAttribute('aria-expanded', 'true');

      await user.click(button);

      const expandButton = screen.getByLabelText('Expand Test Section section');
      expect(expandButton).toHaveAttribute('aria-expanded', 'false');
    });
  });

  describe('Edge Cases', () => {
    it('handles single key correctly', () => {
      const singleKey: PaletteKey[] = [
        { id: 'A', label: 'A', category: 'basic' },
      ];

      render(
        <KeyCategorySection
          title="Single Key"
          keys={singleKey}
          onKeySelect={mockOnKeySelect}
        />
      );

      expect(screen.getByText('(1)')).toBeInTheDocument();
      expect(screen.getByTestId('key-item-A')).toBeInTheDocument();
    });

    it('handles large number of keys', () => {
      const manyKeys: PaletteKey[] = Array.from({ length: 100 }, (_, i) => ({
        id: `KEY_${i}`,
        label: `Key ${i}`,
        category: 'basic' as const,
      }));

      render(
        <KeyCategorySection
          title="Many Keys"
          keys={manyKeys}
          onKeySelect={mockOnKeySelect}
        />
      );

      expect(screen.getByText('(100)')).toBeInTheDocument();
      expect(screen.getByTestId('key-item-KEY_0')).toBeInTheDocument();
      expect(screen.getByTestId('key-item-KEY_99')).toBeInTheDocument();
    });

    it('handles missing favoriteKeyIds gracefully', () => {
      render(
        <KeyCategorySection
          title="Test Section"
          keys={mockKeys}
          onKeySelect={mockOnKeySelect}
          showFavorites={true}
          onToggleFavorite={mockOnToggleFavorite}
        />
      );

      // All keys should be non-favorites
      expect(screen.getByTestId('key-item-A')).toHaveAttribute(
        'data-favorite',
        'false'
      );
    });

    it('handles missing onToggleFavorite gracefully', () => {
      render(
        <KeyCategorySection
          title="Test Section"
          keys={mockKeys}
          onKeySelect={mockOnKeySelect}
          showFavorites={true}
        />
      );

      // Should still render but without star buttons functionality
      const keyItems = screen.getAllByTestId(/key-item-/);
      expect(keyItems).toHaveLength(3);
    });
  });
});
