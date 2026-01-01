import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { KeyConfigDialog } from './KeyConfigDialog';

const mockOnClose = vi.fn();
const mockOnSave = vi.fn();

const defaultProps = {
  isOpen: true,
  onClose: mockOnClose,
  keyCode: 'CapsLock',
  onSave: mockOnSave,
  availableKeys: ['Escape', 'Ctrl', 'Shift', 'Alt', 'A', 'B', 'C'],
  availableLayers: ['Base', 'Layer1', 'Layer2'],
};

describe('KeyConfigDialog', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Rendering', () => {
    it('should render with correct title', () => {
      render(<KeyConfigDialog {...defaultProps} />);
      expect(
        screen.getByText('Configure Key: CapsLock')
      ).toBeInTheDocument();
    });

    it('should render all action type buttons', () => {
      render(<KeyConfigDialog {...defaultProps} />);
      expect(
        screen.getByRole('button', { name: /Select Tap-Hold action type/i })
      ).toBeInTheDocument();
      expect(
        screen.getByRole('button', { name: /Select Simple Remap action type/i })
      ).toBeInTheDocument();
      expect(
        screen.getByRole('button', { name: /Select Macro action type/i })
      ).toBeInTheDocument();
      expect(
        screen.getByRole('button', {
          name: /Select Layer Switch action type/i,
        })
      ).toBeInTheDocument();
    });

    it('should default to tap-hold action type', () => {
      render(<KeyConfigDialog {...defaultProps} />);
      const tapHoldButton = screen.getByRole('button', {
        name: /Select Tap-Hold action type/i,
      });
      expect(tapHoldButton).toHaveClass('bg-primary-500');
    });

    it('should render preview panel', () => {
      render(<KeyConfigDialog {...defaultProps} />);
      expect(screen.getByText('Preview')).toBeInTheDocument();
    });

    it('should render save and cancel buttons', () => {
      render(<KeyConfigDialog {...defaultProps} />);
      expect(
        screen.getByRole('button', { name: /Cancel key configuration/i })
      ).toBeInTheDocument();
      expect(
        screen.getByRole('button', { name: /Save key configuration/i })
      ).toBeInTheDocument();
    });
  });

  describe('Simple Remap', () => {
    it('should show simple remap form when selected', async () => {
      const user = userEvent.setup();
      render(<KeyConfigDialog {...defaultProps} />);

      const simpleButton = screen.getByRole('button', {
        name: /Select Simple Remap action type/i,
      });
      await user.click(simpleButton);

      expect(
        screen.getByLabelText(/Select output key for simple remap/i)
      ).toBeInTheDocument();
    });

    it('should update preview for simple remap', async () => {
      const user = userEvent.setup();
      render(<KeyConfigDialog {...defaultProps} />);

      const simpleButton = screen.getByRole('button', {
        name: /Select Simple Remap action type/i,
      });
      await user.click(simpleButton);

      expect(screen.getByText(/Select an output key/i)).toBeInTheDocument();
    });
  });

  describe('Tap-Hold', () => {
    it('should show tap-hold form by default', () => {
      render(<KeyConfigDialog {...defaultProps} />);

      expect(
        screen.getByLabelText(/Select tap action key/i)
      ).toBeInTheDocument();
      expect(
        screen.getByLabelText(/Select hold action key/i)
      ).toBeInTheDocument();
      expect(
        screen.getByLabelText(/Set hold threshold in milliseconds/i)
      ).toBeInTheDocument();
    });

    it('should display threshold slider with correct range', () => {
      render(<KeyConfigDialog {...defaultProps} />);
      const slider = screen.getByLabelText(
        /Set hold threshold in milliseconds/i
      ) as HTMLInputElement;

      expect(slider.min).toBe('10');
      expect(slider.max).toBe('2000');
      expect(slider.value).toBe('200');
    });

    it('should update threshold value when slider changes', async () => {
      render(<KeyConfigDialog {...defaultProps} />);

      const slider = screen.getByLabelText(
        /Set hold threshold in milliseconds/i
      ) as HTMLInputElement;

      // Simulate slider change using fireEvent (userEvent doesn't support range inputs well)
      const { fireEvent } = await import('@testing-library/react');
      fireEvent.change(slider, { target: { value: '500' } });

      expect(screen.getByText(/Threshold: 500 ms/i)).toBeInTheDocument();
    });

    it('should show correct preview for tap-hold', async () => {
      render(<KeyConfigDialog {...defaultProps} />);
      expect(
        screen.getByText(/Configure tap and hold actions/i)
      ).toBeInTheDocument();
    });
  });

  describe('Macro', () => {
    it('should show macro form when selected', async () => {
      const user = userEvent.setup();
      render(<KeyConfigDialog {...defaultProps} />);

      const macroButton = screen.getByRole('button', {
        name: /Select Macro action type/i,
      });
      await user.click(macroButton);

      expect(screen.getByText('Macro Steps')).toBeInTheDocument();
      expect(
        screen.getByRole('button', { name: /Add macro step/i })
      ).toBeInTheDocument();
    });

    it('should show empty state when no macro steps', async () => {
      const user = userEvent.setup();
      render(<KeyConfigDialog {...defaultProps} />);

      const macroButton = screen.getByRole('button', {
        name: /Select Macro action type/i,
      });
      await user.click(macroButton);

      expect(
        screen.getByText(/No macro steps defined/i)
      ).toBeInTheDocument();
    });

    it('should add macro step when clicking add button', async () => {
      const user = userEvent.setup();
      render(<KeyConfigDialog {...defaultProps} />);

      const macroButton = screen.getByRole('button', {
        name: /Select Macro action type/i,
      });
      await user.click(macroButton);

      const addButton = screen.getByRole('button', {
        name: /Add macro step/i,
      });
      await user.click(addButton);

      expect(
        screen.getByLabelText(/Macro step 1 type/i)
      ).toBeInTheDocument();
    });

    it('should remove macro step when clicking remove button', async () => {
      const user = userEvent.setup();
      render(<KeyConfigDialog {...defaultProps} />);

      const macroButton = screen.getByRole('button', {
        name: /Select Macro action type/i,
      });
      await user.click(macroButton);

      const addButton = screen.getByRole('button', {
        name: /Add macro step/i,
      });
      await user.click(addButton);

      const removeButton = screen.getByRole('button', {
        name: /Remove macro step 1/i,
      });
      await user.click(removeButton);

      expect(
        screen.queryByLabelText(/Macro step 1 type/i)
      ).not.toBeInTheDocument();
    });

    it('should show delay input for delay type macro step', async () => {
      const user = userEvent.setup();
      render(<KeyConfigDialog {...defaultProps} />);

      const macroButton = screen.getByRole('button', {
        name: /Select Macro action type/i,
      });
      await user.click(macroButton);

      const addButton = screen.getByRole('button', {
        name: /Add macro step/i,
      });
      await user.click(addButton);

      const typeSelect = screen.getByLabelText(
        /Macro step 1 type/i
      ) as HTMLSelectElement;
      await user.selectOptions(typeSelect, 'delay');

      expect(
        screen.getByLabelText(/Macro step 1 delay duration/i)
      ).toBeInTheDocument();
    });

    it('should update preview with macro step count', async () => {
      const user = userEvent.setup();
      render(<KeyConfigDialog {...defaultProps} />);

      const macroButton = screen.getByRole('button', {
        name: /Select Macro action type/i,
      });
      await user.click(macroButton);

      const addButton = screen.getByRole('button', {
        name: /Add macro step/i,
      });
      await user.click(addButton);

      expect(screen.getByText(/Macro: 1 step\(s\)/i)).toBeInTheDocument();
    });
  });

  describe('Layer Switch', () => {
    it('should show layer switch form when selected', async () => {
      const user = userEvent.setup();
      render(<KeyConfigDialog {...defaultProps} />);

      const layerButton = screen.getByRole('button', {
        name: /Select Layer Switch action type/i,
      });
      await user.click(layerButton);

      expect(screen.getByText('Target Layer')).toBeInTheDocument();
      expect(
        screen.getByLabelText(/Select target layer/i)
      ).toBeInTheDocument();
    });

    it('should show correct preview for layer switch', async () => {
      const user = userEvent.setup();
      render(<KeyConfigDialog {...defaultProps} />);

      const layerButton = screen.getByRole('button', {
        name: /Select Layer Switch action type/i,
      });
      await user.click(layerButton);

      expect(screen.getByText(/Select target layer/i)).toBeInTheDocument();
    });
  });

  describe('Save and Cancel', () => {
    it('should call onClose when cancel button clicked', async () => {
      const user = userEvent.setup();
      render(<KeyConfigDialog {...defaultProps} />);

      const cancelButton = screen.getByRole('button', {
        name: /Cancel key configuration/i,
      });
      await user.click(cancelButton);

      expect(mockOnClose).toHaveBeenCalledTimes(1);
    });

    it('should call onSave with correct data for simple remap', async () => {
      const user = userEvent.setup();
      mockOnSave.mockResolvedValue(undefined);
      render(<KeyConfigDialog {...defaultProps} />);

      const simpleButton = screen.getByRole('button', {
        name: /Select Simple Remap action type/i,
      });
      await user.click(simpleButton);

      const saveButton = screen.getByRole('button', {
        name: /Save key configuration/i,
      });
      await user.click(saveButton);

      await waitFor(() => {
        expect(mockOnSave).toHaveBeenCalledWith('CapsLock', {
          type: 'simple',
          tapAction: '',
        });
      });
    });

    it('should call onSave with correct data for tap-hold', async () => {
      const user = userEvent.setup();
      mockOnSave.mockResolvedValue(undefined);
      render(<KeyConfigDialog {...defaultProps} />);

      const saveButton = screen.getByRole('button', {
        name: /Save key configuration/i,
      });
      await user.click(saveButton);

      await waitFor(() => {
        expect(mockOnSave).toHaveBeenCalledWith('CapsLock', {
          type: 'tap_hold',
          tapAction: '',
          holdAction: '',
          threshold: 200,
        });
      });
    });

    it('should show loading state when saving', async () => {
      const user = userEvent.setup();
      mockOnSave.mockImplementation(
        () => new Promise((resolve) => setTimeout(resolve, 100))
      );
      render(<KeyConfigDialog {...defaultProps} />);

      const saveButton = screen.getByRole('button', {
        name: /Save key configuration/i,
      });
      await user.click(saveButton);

      expect(saveButton).toHaveAttribute('aria-busy', 'true');
    });

    it('should disable buttons when saving', async () => {
      const user = userEvent.setup();
      mockOnSave.mockImplementation(
        () => new Promise((resolve) => setTimeout(resolve, 100))
      );
      render(<KeyConfigDialog {...defaultProps} />);

      const saveButton = screen.getByRole('button', {
        name: /Save key configuration/i,
      });
      await user.click(saveButton);

      const cancelButton = screen.getByRole('button', {
        name: /Cancel key configuration/i,
      });
      expect(saveButton).toBeDisabled();
      expect(cancelButton).toBeDisabled();
    });

    it('should close dialog after successful save', async () => {
      const user = userEvent.setup();
      mockOnSave.mockResolvedValue(undefined);
      render(<KeyConfigDialog {...defaultProps} />);

      const saveButton = screen.getByRole('button', {
        name: /Save key configuration/i,
      });
      await user.click(saveButton);

      await waitFor(() => {
        // onClose gets called after save completes
        expect(mockOnClose).toHaveBeenCalled();
      });
    });
  });

  describe('Current Mapping', () => {
    it('should populate form with current mapping data', () => {
      const currentMapping = {
        type: 'tap_hold' as const,
        tapAction: 'Escape',
        holdAction: 'Ctrl',
        threshold: 300,
      };

      render(
        <KeyConfigDialog {...defaultProps} currentMapping={currentMapping} />
      );

      expect(
        screen.getByText(/Threshold: 300 ms/i)
      ).toBeInTheDocument();
    });

    it('should populate simple remap form', async () => {
      const user = userEvent.setup();
      const currentMapping = {
        type: 'simple' as const,
        tapAction: 'Escape',
      };

      render(
        <KeyConfigDialog {...defaultProps} currentMapping={currentMapping} />
      );

      const simpleButton = screen.getByRole('button', {
        name: /Select Simple Remap action type/i,
      });
      expect(simpleButton).toHaveClass('bg-primary-500');
    });

    it('should populate layer switch form', async () => {
      const currentMapping = {
        type: 'layer_switch' as const,
        targetLayer: 'Layer1',
      };

      render(
        <KeyConfigDialog {...defaultProps} currentMapping={currentMapping} />
      );

      const layerButton = screen.getByRole('button', {
        name: /Select Layer Switch action type/i,
      });
      expect(layerButton).toHaveClass('bg-primary-500');
    });
  });

  describe('Accessibility', () => {
    it('should have proper aria-labels on all interactive elements', () => {
      render(<KeyConfigDialog {...defaultProps} />);

      expect(
        screen.getByLabelText(/Select Tap-Hold action type/i)
      ).toBeInTheDocument();
      expect(
        screen.getByLabelText(/Select tap action key/i)
      ).toBeInTheDocument();
      expect(
        screen.getByLabelText(/Select hold action key/i)
      ).toBeInTheDocument();
      expect(
        screen.getByLabelText(/Set hold threshold in milliseconds/i)
      ).toBeInTheDocument();
      expect(
        screen.getByLabelText(/Cancel key configuration/i)
      ).toBeInTheDocument();
      expect(
        screen.getByLabelText(/Save key configuration/i)
      ).toBeInTheDocument();
    });

    it('should be keyboard navigable', async () => {
      const user = userEvent.setup();
      render(<KeyConfigDialog {...defaultProps} />);

      const firstButton = screen.getByRole('button', {
        name: /Select Tap-Hold action type/i,
      });
      firstButton.focus();

      await user.keyboard('{Tab}');
      expect(
        screen.getByRole('button', { name: /Select Simple Remap action type/i })
      ).toHaveFocus();
    });
  });
});
