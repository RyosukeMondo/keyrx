import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { KeyMappingDialog } from './KeyMappingDialog';
import type { EditorKeyMapping } from '@/types/config';

describe('KeyMappingDialog', () => {
  const mockOnClose = vi.fn();
  const mockOnSave = vi.fn();

  const defaultProps = {
    open: true,
    onClose: mockOnClose,
    keyCode: 'CapsLock',
    onSave: mockOnSave,
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Rendering', () => {
    it('should render dialog when open', () => {
      render(<KeyMappingDialog {...defaultProps} />);
      expect(screen.getByRole('dialog')).toBeInTheDocument();
      expect(screen.getByText('Configure CapsLock')).toBeInTheDocument();
    });

    it('should not render dialog when closed', () => {
      render(<KeyMappingDialog {...defaultProps} open={false} />);
      expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    });

    it('should render all mapping type buttons', () => {
      render(<KeyMappingDialog {...defaultProps} />);
      expect(screen.getByText('Simple Mapping')).toBeInTheDocument();
      expect(screen.getByText('Tap-Hold (Dual Function)')).toBeInTheDocument();
      expect(screen.getByText('Macro Sequence')).toBeInTheDocument();
      expect(screen.getByText('Layer Switch')).toBeInTheDocument();
    });

    it('should default to simple mapping type', () => {
      render(<KeyMappingDialog {...defaultProps} />);
      const simpleButton = screen.getByRole('button', {
        name: /Select Simple Mapping/i,
      });
      expect(simpleButton).toHaveAttribute('aria-pressed', 'true');
    });
  });

  describe('Simple Mapping', () => {
    it('should display simple action input', () => {
      render(<KeyMappingDialog {...defaultProps} />);
      expect(screen.getByLabelText('Simple action')).toBeInTheDocument();
      expect(
        screen.getByPlaceholderText(/VK_A, VK_ENTER/i)
      ).toBeInTheDocument();
    });

    it('should update simple action value', async () => {
      const user = userEvent.setup();
      render(<KeyMappingDialog {...defaultProps} />);

      const input = screen.getByLabelText('Simple action');
      await user.type(input, 'VK_A');

      expect(input).toHaveValue('VK_A');
    });

    it('should validate simple action is required', async () => {
      const user = userEvent.setup();
      render(<KeyMappingDialog {...defaultProps} />);

      const saveButton = screen.getByRole('button', { name: /Save mapping/i });
      await user.click(saveButton);

      await waitFor(() => {
        expect(
          screen.getByText('Simple action is required')
        ).toBeInTheDocument();
      });
      expect(mockOnSave).not.toHaveBeenCalled();
    });

    it('should call onSave with correct simple mapping', async () => {
      const user = userEvent.setup();
      mockOnSave.mockResolvedValueOnce(undefined);

      render(<KeyMappingDialog {...defaultProps} />);

      const input = screen.getByLabelText('Simple action');
      await user.type(input, 'VK_A');

      const saveButton = screen.getByRole('button', { name: /Save mapping/i });
      await user.click(saveButton);

      await waitFor(() => {
        expect(mockOnSave).toHaveBeenCalledWith({
          keyCode: 'CapsLock',
          type: 'simple',
          simple: 'VK_A',
        });
      });
    });
  });

  describe('Tap-Hold Mapping', () => {
    beforeEach(async () => {
      const user = userEvent.setup();
      render(<KeyMappingDialog {...defaultProps} />);

      const tapHoldButton = screen.getByRole('button', {
        name: /Select Tap-Hold/i,
      });
      await user.click(tapHoldButton);
    });

    it('should display tap-hold form fields', () => {
      expect(screen.getByLabelText('Tap action')).toBeInTheDocument();
      expect(screen.getByLabelText('Hold action')).toBeInTheDocument();
      expect(
        screen.getByLabelText('Timeout in milliseconds')
      ).toBeInTheDocument();
    });

    it('should validate tap action is required', async () => {
      const user = userEvent.setup();

      const saveButton = screen.getByRole('button', { name: /Save mapping/i });
      await user.click(saveButton);

      await waitFor(() => {
        expect(screen.getByText('Tap action is required')).toBeInTheDocument();
      });
    });

    it('should validate hold action is required', async () => {
      const user = userEvent.setup();

      const tapInput = screen.getByLabelText('Tap action');
      await user.type(tapInput, 'VK_ESCAPE');

      const saveButton = screen.getByRole('button', { name: /Save mapping/i });
      await user.click(saveButton);

      await waitFor(() => {
        expect(screen.getByText('Hold action is required')).toBeInTheDocument();
      });
    });

    it('should validate timeout range (100-500ms)', async () => {
      const user = userEvent.setup();

      const tapInput = screen.getByLabelText('Tap action');
      await user.type(tapInput, 'VK_ESCAPE');

      const holdInput = screen.getByLabelText('Hold action');
      await user.type(holdInput, 'MD_CTRL');

      const timeoutInput = screen.getByLabelText('Timeout in milliseconds');
      await user.clear(timeoutInput);
      await user.type(timeoutInput, '50');

      const saveButton = screen.getByRole('button', { name: /Save mapping/i });
      await user.click(saveButton);

      await waitFor(() => {
        expect(
          screen.getByText('Timeout must be between 100-500ms')
        ).toBeInTheDocument();
      });
    });

    it('should update timeout with slider', async () => {
      const { fireEvent } = await import('@testing-library/react');

      const slider = screen.getByLabelText(
        'Timeout slider'
      ) as HTMLInputElement;

      // Use fireEvent for range input as userEvent doesn't support it well
      fireEvent.change(slider, { target: { value: '350' } });

      const timeoutInput = screen.getByLabelText(
        'Timeout in milliseconds'
      ) as HTMLInputElement;
      expect(timeoutInput.value).toBe('350');
    });

    it('should call onSave with correct tap-hold mapping', async () => {
      const user = userEvent.setup();
      mockOnSave.mockResolvedValueOnce(undefined);

      const tapInput = screen.getByLabelText('Tap action');
      await user.type(tapInput, 'VK_ESCAPE');

      const holdInput = screen.getByLabelText('Hold action');
      await user.type(holdInput, 'MD_CTRL');

      const timeoutInput = screen.getByLabelText('Timeout in milliseconds');
      await user.clear(timeoutInput);
      await user.type(timeoutInput, '250');

      const saveButton = screen.getByRole('button', { name: /Save mapping/i });
      await user.click(saveButton);

      await waitFor(() => {
        expect(mockOnSave).toHaveBeenCalledWith({
          keyCode: 'CapsLock',
          type: 'tap_hold',
          tapHold: {
            tap: 'VK_ESCAPE',
            hold: 'MD_CTRL',
            timeoutMs: 250,
          },
        });
      });
    });
  });

  describe('Macro Mapping', () => {
    beforeEach(async () => {
      const user = userEvent.setup();
      render(<KeyMappingDialog {...defaultProps} />);

      const macroButton = screen.getByRole('button', {
        name: /Select Macro Sequence/i,
      });
      await user.click(macroButton);
    });

    it('should display macro sequence input', () => {
      expect(screen.getByLabelText('Macro sequence')).toBeInTheDocument();
      expect(
        screen.getByPlaceholderText(/VK_H, VK_E, VK_L, VK_L, VK_O/i)
      ).toBeInTheDocument();
    });

    it('should validate macro sequence is required', async () => {
      const user = userEvent.setup();

      const saveButton = screen.getByRole('button', { name: /Save mapping/i });
      await user.click(saveButton);

      await waitFor(() => {
        expect(
          screen.getByText('Macro sequence is required')
        ).toBeInTheDocument();
      });
    });

    it('should parse comma-separated macro sequence', async () => {
      const user = userEvent.setup();
      mockOnSave.mockResolvedValueOnce(undefined);

      const input = screen.getByLabelText('Macro sequence');
      await user.type(input, 'VK_H, VK_E, VK_L, VK_L, VK_O');

      const saveButton = screen.getByRole('button', { name: /Save mapping/i });
      await user.click(saveButton);

      await waitFor(() => {
        expect(mockOnSave).toHaveBeenCalledWith({
          keyCode: 'CapsLock',
          type: 'macro',
          macro: ['VK_H', 'VK_E', 'VK_L', 'VK_L', 'VK_O'],
        });
      });
    });

    it('should trim whitespace from macro keys', async () => {
      const user = userEvent.setup();
      mockOnSave.mockResolvedValueOnce(undefined);

      const input = screen.getByLabelText('Macro sequence');
      await user.type(input, '  VK_A  ,  VK_B  ,  VK_C  ');

      const saveButton = screen.getByRole('button', { name: /Save mapping/i });
      await user.click(saveButton);

      await waitFor(() => {
        expect(mockOnSave).toHaveBeenCalledWith({
          keyCode: 'CapsLock',
          type: 'macro',
          macro: ['VK_A', 'VK_B', 'VK_C'],
        });
      });
    });
  });

  describe('Layer Switch Mapping', () => {
    beforeEach(async () => {
      const user = userEvent.setup();
      render(<KeyMappingDialog {...defaultProps} />);

      const layerButton = screen.getByRole('button', {
        name: /Select Layer Switch/i,
      });
      await user.click(layerButton);
    });

    it('should display layer name input', () => {
      expect(screen.getByLabelText('Layer name')).toBeInTheDocument();
      expect(screen.getByPlaceholderText(/nav, num, fn/i)).toBeInTheDocument();
    });

    it('should validate layer name is required', async () => {
      const user = userEvent.setup();

      const saveButton = screen.getByRole('button', { name: /Save mapping/i });
      await user.click(saveButton);

      await waitFor(() => {
        expect(screen.getByText('Layer name is required')).toBeInTheDocument();
      });
    });

    it('should call onSave with correct layer-switch mapping', async () => {
      const user = userEvent.setup();
      mockOnSave.mockResolvedValueOnce(undefined);

      const input = screen.getByLabelText('Layer name');
      await user.type(input, 'nav');

      const saveButton = screen.getByRole('button', { name: /Save mapping/i });
      await user.click(saveButton);

      await waitFor(() => {
        expect(mockOnSave).toHaveBeenCalledWith({
          keyCode: 'CapsLock',
          type: 'layer_switch',
          layer: 'nav',
        });
      });
    });
  });

  describe('Editing Existing Mapping', () => {
    it('should populate form from currentMapping for simple type', () => {
      const currentMapping: EditorKeyMapping = {
        keyCode: 'CapsLock',
        type: 'simple',
        simple: 'VK_A',
      };

      render(
        <KeyMappingDialog {...defaultProps} currentMapping={currentMapping} />
      );

      const input = screen.getByLabelText('Simple action');
      expect(input).toHaveValue('VK_A');
    });

    it('should populate form from currentMapping for tap-hold type', () => {
      const currentMapping: EditorKeyMapping = {
        keyCode: 'CapsLock',
        type: 'tap_hold',
        tapHold: {
          tap: 'VK_ESCAPE',
          hold: 'MD_CTRL',
          timeoutMs: 300,
        },
      };

      render(
        <KeyMappingDialog {...defaultProps} currentMapping={currentMapping} />
      );

      expect(screen.getByLabelText('Tap action')).toHaveValue('VK_ESCAPE');
      expect(screen.getByLabelText('Hold action')).toHaveValue('MD_CTRL');
      // Number input returns string value
      const timeoutInput = screen.getByLabelText(
        'Timeout in milliseconds'
      ) as HTMLInputElement;
      expect(timeoutInput.value).toBe('300');
    });

    it('should populate form from currentMapping for macro type', () => {
      const currentMapping: EditorKeyMapping = {
        keyCode: 'CapsLock',
        type: 'macro',
        macro: ['VK_H', 'VK_I'],
      };

      render(
        <KeyMappingDialog {...defaultProps} currentMapping={currentMapping} />
      );

      const input = screen.getByLabelText('Macro sequence');
      expect(input).toHaveValue('VK_H, VK_I');
    });

    it('should populate form from currentMapping for layer-switch type', () => {
      const currentMapping: EditorKeyMapping = {
        keyCode: 'CapsLock',
        type: 'layer_switch',
        layer: 'nav',
      };

      render(
        <KeyMappingDialog {...defaultProps} currentMapping={currentMapping} />
      );

      const input = screen.getByLabelText('Layer name');
      expect(input).toHaveValue('nav');
    });
  });

  describe('Dialog Actions', () => {
    it('should call onClose when Cancel is clicked', async () => {
      const user = userEvent.setup();
      render(<KeyMappingDialog {...defaultProps} />);

      const cancelButton = screen.getByRole('button', { name: /Cancel/i });
      await user.click(cancelButton);

      expect(mockOnClose).toHaveBeenCalled();
    });

    it('should call onClose when Escape is pressed', async () => {
      const user = userEvent.setup();
      render(<KeyMappingDialog {...defaultProps} />);

      await user.keyboard('{Escape}');

      expect(mockOnClose).toHaveBeenCalled();
    });

    it('should close dialog after successful save', async () => {
      const user = userEvent.setup();
      mockOnSave.mockResolvedValueOnce(undefined);

      render(<KeyMappingDialog {...defaultProps} />);

      const input = screen.getByLabelText('Simple action');
      await user.type(input, 'VK_A');

      const saveButton = screen.getByRole('button', { name: /Save mapping/i });
      await user.click(saveButton);

      await waitFor(() => {
        expect(mockOnClose).toHaveBeenCalled();
      });
    });

    it('should show error message when save fails', async () => {
      const user = userEvent.setup();
      mockOnSave.mockRejectedValueOnce(new Error('Network error'));

      render(<KeyMappingDialog {...defaultProps} />);

      const input = screen.getByLabelText('Simple action');
      await user.type(input, 'VK_A');

      const saveButton = screen.getByRole('button', { name: /Save mapping/i });
      await user.click(saveButton);

      await waitFor(() => {
        expect(screen.getByText('Network error')).toBeInTheDocument();
      });
      expect(mockOnClose).not.toHaveBeenCalled();
    });

    it('should disable buttons during save', async () => {
      const user = userEvent.setup();
      mockOnSave.mockImplementation(
        () => new Promise((resolve) => setTimeout(resolve, 100))
      );

      render(<KeyMappingDialog {...defaultProps} />);

      const input = screen.getByLabelText('Simple action');
      await user.type(input, 'VK_A');

      const saveButton = screen.getByRole('button', { name: /Save mapping/i });
      await user.click(saveButton);

      // Check buttons are disabled during save
      expect(saveButton).toBeDisabled();
      expect(screen.getByRole('button', { name: /Cancel/i })).toBeDisabled();

      await waitFor(
        () => {
          expect(mockOnSave).toHaveBeenCalled();
        },
        { timeout: 200 }
      );
    });
  });

  describe('Accessibility', () => {
    it('should have accessible button labels', () => {
      render(<KeyMappingDialog {...defaultProps} />);

      expect(
        screen.getByRole('button', { name: /Select Simple Mapping/i })
      ).toBeInTheDocument();
      expect(
        screen.getByRole('button', {
          name: /Select Tap-Hold \(Dual Function\)/i,
        })
      ).toBeInTheDocument();
      expect(
        screen.getByRole('button', { name: /Select Macro Sequence/i })
      ).toBeInTheDocument();
      expect(
        screen.getByRole('button', { name: /Select Layer Switch/i })
      ).toBeInTheDocument();
    });

    it('should have aria-pressed on mapping type buttons', () => {
      render(<KeyMappingDialog {...defaultProps} />);

      const simpleButton = screen.getByRole('button', {
        name: /Select Simple Mapping/i,
      });
      expect(simpleButton).toHaveAttribute('aria-pressed', 'true');

      const tapHoldButton = screen.getByRole('button', {
        name: /Select Tap-Hold/i,
      });
      expect(tapHoldButton).toHaveAttribute('aria-pressed', 'false');
    });

    it('should trap focus within dialog', async () => {
      const user = userEvent.setup();
      render(<KeyMappingDialog {...defaultProps} />);

      // Tab should move focus within dialog
      await user.tab();
      expect(document.activeElement?.tagName).toBe('BUTTON');

      // Should not tab outside dialog
      await user.tab();
      await user.tab();
      await user.tab();
      expect(screen.getByRole('dialog')).toContainElement(
        document.activeElement
      );
    });
  });
});
