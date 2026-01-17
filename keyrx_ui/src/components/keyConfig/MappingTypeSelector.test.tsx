import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MappingTypeSelector, type MappingType } from './MappingTypeSelector';

describe('MappingTypeSelector', () => {
  const mockOnChange = vi.fn();

  afterEach(() => {
    mockOnChange.mockClear();
  });

  describe('Rendering', () => {
    it('renders all supported types', () => {
      const supportedTypes: MappingType[] = ['simple', 'tap_hold'];
      render(
        <MappingTypeSelector
          selectedType="simple"
          onChange={mockOnChange}
          supportedTypes={supportedTypes}
        />
      );

      expect(screen.getByText('Simple')).toBeInTheDocument();
      expect(screen.getByText('Tap/Hold')).toBeInTheDocument();
    });

    it('renders all 5 types when all are supported', () => {
      const supportedTypes: MappingType[] = [
        'simple',
        'modifier',
        'lock',
        'tap_hold',
        'layer_active',
      ];
      render(
        <MappingTypeSelector
          selectedType="simple"
          onChange={mockOnChange}
          supportedTypes={supportedTypes}
        />
      );

      expect(screen.getByText('Simple')).toBeInTheDocument();
      expect(screen.getByText('Modifier')).toBeInTheDocument();
      expect(screen.getByText('Lock')).toBeInTheDocument();
      expect(screen.getByText('Tap/Hold')).toBeInTheDocument();
      expect(screen.getByText('Layer Active')).toBeInTheDocument();
    });

    it('highlights the selected type', () => {
      render(
        <MappingTypeSelector
          selectedType="tap_hold"
          onChange={mockOnChange}
          supportedTypes={['simple', 'tap_hold']}
        />
      );

      const tapHoldButton = screen.getByRole('radio', { checked: true });
      expect(tapHoldButton).toHaveTextContent('Tap/Hold');
      expect(tapHoldButton).toHaveClass('bg-primary-500');
    });

    it('applies horizontal layout by default', () => {
      const { container } = render(
        <MappingTypeSelector
          selectedType="simple"
          onChange={mockOnChange}
          supportedTypes={['simple', 'tap_hold']}
        />
      );

      const buttonsContainer = container.querySelector('[role="radiogroup"]');
      expect(buttonsContainer).toHaveClass('flex-wrap');
      expect(buttonsContainer).not.toHaveClass('flex-col');
    });

    it('applies vertical layout when specified', () => {
      const { container } = render(
        <MappingTypeSelector
          selectedType="simple"
          onChange={mockOnChange}
          supportedTypes={['simple', 'tap_hold']}
          layout="vertical"
        />
      );

      const buttonsContainer = container.querySelector('[role="radiogroup"]');
      expect(buttonsContainer).toHaveClass('flex-col');
    });
  });

  describe('Interaction', () => {
    it('calls onChange with correct type when clicked', async () => {
      const user = userEvent.setup();
      render(
        <MappingTypeSelector
          selectedType="simple"
          onChange={mockOnChange}
          supportedTypes={['simple', 'tap_hold']}
        />
      );

      const tapHoldButton = screen.getByText('Tap/Hold');
      await user.click(tapHoldButton);

      expect(mockOnChange).toHaveBeenCalledTimes(1);
      expect(mockOnChange).toHaveBeenCalledWith('tap_hold');
    });

    it('calls onChange for each type when clicked in sequence', async () => {
      const user = userEvent.setup();
      const supportedTypes: MappingType[] = [
        'simple',
        'modifier',
        'tap_hold',
      ];
      render(
        <MappingTypeSelector
          selectedType="simple"
          onChange={mockOnChange}
          supportedTypes={supportedTypes}
        />
      );

      await user.click(screen.getByText('Modifier'));
      expect(mockOnChange).toHaveBeenCalledWith('modifier');

      await user.click(screen.getByText('Tap/Hold'));
      expect(mockOnChange).toHaveBeenCalledWith('tap_hold');

      await user.click(screen.getByText('Simple'));
      expect(mockOnChange).toHaveBeenCalledWith('simple');

      expect(mockOnChange).toHaveBeenCalledTimes(3);
    });

    it('does not call onChange when clicking the already selected type', async () => {
      const user = userEvent.setup();
      render(
        <MappingTypeSelector
          selectedType="simple"
          onChange={mockOnChange}
          supportedTypes={['simple', 'tap_hold']}
        />
      );

      const simpleButton = screen.getByText('Simple');
      await user.click(simpleButton);

      // onChange is still called, but with the same value
      expect(mockOnChange).toHaveBeenCalledTimes(1);
      expect(mockOnChange).toHaveBeenCalledWith('simple');
    });
  });

  describe('Accessibility', () => {
    it('has radiogroup role', () => {
      render(
        <MappingTypeSelector
          selectedType="simple"
          onChange={mockOnChange}
          supportedTypes={['simple', 'tap_hold']}
        />
      );

      expect(screen.getByRole('radiogroup')).toBeInTheDocument();
    });

    it('marks selected button as checked', () => {
      render(
        <MappingTypeSelector
          selectedType="simple"
          onChange={mockOnChange}
          supportedTypes={['simple', 'tap_hold']}
        />
      );

      const checkedRadio = screen.getByRole('radio', { checked: true });
      expect(checkedRadio).toHaveTextContent('Simple');
    });

    it('marks non-selected buttons as not checked', () => {
      render(
        <MappingTypeSelector
          selectedType="simple"
          onChange={mockOnChange}
          supportedTypes={['simple', 'tap_hold']}
        />
      );

      const uncheckedRadios = screen.getAllByRole('radio', { checked: false });
      expect(uncheckedRadios).toHaveLength(1);
      expect(uncheckedRadios[0]).toHaveTextContent('Tap/Hold');
    });

    it('has descriptive aria-labels', () => {
      render(
        <MappingTypeSelector
          selectedType="simple"
          onChange={mockOnChange}
          supportedTypes={['simple', 'modifier']}
        />
      );

      expect(
        screen.getByLabelText('Simple: Map to a single key')
      ).toBeInTheDocument();
      expect(
        screen.getByLabelText('Modifier: Act as a modifier key')
      ).toBeInTheDocument();
    });

    it('has title attributes for tooltips', () => {
      render(
        <MappingTypeSelector
          selectedType="simple"
          onChange={mockOnChange}
          supportedTypes={['simple', 'tap_hold']}
        />
      );

      const simpleButton = screen.getByText('Simple').closest('button');
      const tapHoldButton = screen.getByText('Tap/Hold').closest('button');

      expect(simpleButton).toHaveAttribute('title', 'Map to a single key');
      expect(tapHoldButton).toHaveAttribute(
        'title',
        'Different actions for tap vs hold'
      );
    });
  });

  describe('Edge Cases', () => {
    it('handles single supported type', () => {
      render(
        <MappingTypeSelector
          selectedType="simple"
          onChange={mockOnChange}
          supportedTypes={['simple']}
        />
      );

      const buttons = screen.getAllByRole('radio');
      expect(buttons).toHaveLength(1);
      expect(buttons[0]).toHaveTextContent('Simple');
    });

    it('renders correctly with empty supportedTypes array', () => {
      render(
        <MappingTypeSelector
          selectedType="simple"
          onChange={mockOnChange}
          supportedTypes={[]}
        />
      );

      const buttons = screen.queryAllByRole('radio');
      expect(buttons).toHaveLength(0);
    });

    it('handles selectedType not in supportedTypes', () => {
      render(
        <MappingTypeSelector
          selectedType="modifier"
          onChange={mockOnChange}
          supportedTypes={['simple', 'tap_hold']}
        />
      );

      // No button should be marked as checked
      const checkedButtons = screen.queryAllByRole('radio', { checked: true });
      expect(checkedButtons).toHaveLength(0);
    });
  });
});
