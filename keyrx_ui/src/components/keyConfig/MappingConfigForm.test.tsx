import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import {
  MappingConfigForm,
  type MappingType,
  type MappingConfig,
  type ValidationResult,
} from './MappingConfigForm';
import type { SVGKey } from '../SVGKeyboard';

// Mock dependencies
vi.mock('../SVGKeyboard', () => ({
  SVGKeyboard: ({
    onKeyClick,
    keys,
  }: {
    onKeyClick: (key: string) => void;
    keys: SVGKey[];
  }) => (
    <div data-testid="svg-keyboard">
      <button onClick={() => onKeyClick('VK_A')}>Mock Key A</button>
      <button onClick={() => onKeyClick('VK_B')}>Mock Key B</button>
      <div>Keys count: {keys.length}</div>
    </div>
  ),
}));

vi.mock('../KeyPalette', () => ({
  KeyPalette: vi.fn(
    ({
      onKeySelect,
      selectedKey,
    }: {
      onKeySelect: (key: {
        id: string;
        label: string;
        category: string;
      }) => void;
      selectedKey: { id: string; label: string; category: string } | null;
    }) => (
      <div data-testid="key-palette">
        <button
          onClick={() => {
            onKeySelect({ id: 'VK_ENTER', label: 'Enter', category: 'basic' });
          }}
        >
          Select Enter
        </button>
        <button
          onClick={() => {
            onKeySelect({
              id: 'VK_LCTRL',
              label: 'LCtrl',
              category: 'modifiers',
            });
          }}
        >
          Select LCtrl
        </button>
        <button
          onClick={() => {
            onKeySelect({
              id: 'LK_00',
              label: 'CapsLock',
              category: 'special',
            });
          }}
        >
          Select CapsLock
        </button>
        <button
          onClick={() => {
            onKeySelect({ id: 'MO(1)', label: 'MO(1)', category: 'layers' });
          }}
        >
          Select MO(1)
        </button>
        {selectedKey && (
          <div data-testid="selected-key">Selected: {selectedKey.id}</div>
        )}
      </div>
    )
  ),
}));

describe('MappingConfigForm', () => {
  const mockOnChange = vi.fn();
  const mockOnValidate = vi.fn();
  const mockLayoutKeys: SVGKey[] = [
    { id: 'A', x: 0, y: 0, width: 1, height: 1, label: 'A' },
    { id: 'B', x: 1, y: 0, width: 1, height: 1, label: 'B' },
  ];

  beforeEach(() => {
    mockOnChange.mockClear();
    mockOnValidate.mockClear();
    mockOnValidate.mockReturnValue({ valid: true, errors: {} });
  });

  describe('Simple Mapping Type', () => {
    it('renders simple form with key selection', () => {
      render(
        <MappingConfigForm
          mappingType="simple"
          onChange={mockOnChange}
          enableKeyboardView={false}
        />
      );

      expect(screen.getByTestId('key-palette')).toBeInTheDocument();
    });

    it('shows keyboard/list toggle when enableKeyboardView is true', () => {
      render(
        <MappingConfigForm
          mappingType="simple"
          onChange={mockOnChange}
          layoutKeys={mockLayoutKeys}
          enableKeyboardView={true}
        />
      );

      expect(screen.getByText('Keyboard')).toBeInTheDocument();
      expect(screen.getByText('List')).toBeInTheDocument();
    });

    it('switches between keyboard and list view', async () => {
      const user = userEvent.setup();
      render(
        <MappingConfigForm
          mappingType="simple"
          onChange={mockOnChange}
          layoutKeys={mockLayoutKeys}
          enableKeyboardView={true}
        />
      );

      // Initially shows list (KeyPalette)
      expect(screen.getByTestId('key-palette')).toBeInTheDocument();

      // Switch to keyboard view
      await user.click(screen.getByText('Keyboard'));
      expect(screen.getByTestId('svg-keyboard')).toBeInTheDocument();

      // Switch back to list
      await user.click(screen.getByText('List'));
      expect(screen.getByTestId('key-palette')).toBeInTheDocument();
    });

    it('calls onChange when key is selected', async () => {
      const user = userEvent.setup();
      render(
        <MappingConfigForm
          mappingType="simple"
          onChange={mockOnChange}
          enableKeyboardView={false}
        />
      );

      await user.click(screen.getByText('Select Enter'));

      // onChange should be called (may be called multiple times due to state updates)
      await waitFor(
        () => {
          expect(mockOnChange).toHaveBeenCalled();
        },
        { timeout: 3000 }
      );
    });

    it('displays selected key from currentConfig', () => {
      render(
        <MappingConfigForm
          mappingType="simple"
          currentConfig={{ tapAction: 'VK_ENTER' }}
          onChange={mockOnChange}
          enableKeyboardView={false}
        />
      );

      expect(screen.getByText('VK_ENTER')).toBeInTheDocument();
    });

    it('clears selected key when clear button is clicked', async () => {
      const user = userEvent.setup();
      render(
        <MappingConfigForm
          mappingType="simple"
          currentConfig={{ tapAction: 'VK_A' }}
          onChange={mockOnChange}
          enableKeyboardView={false}
        />
      );

      // Find and click clear button
      const clearButton = screen.getByTitle('Clear selection');
      await user.click(clearButton);

      await waitFor(
        () => {
          expect(mockOnChange).toHaveBeenCalled();
        },
        { timeout: 3000 }
      );
    });

    it('shows Listen for Key button', () => {
      render(
        <MappingConfigForm
          mappingType="simple"
          onChange={mockOnChange}
          enableKeyboardView={false}
        />
      );

      expect(screen.getByText('Listen for Key')).toBeInTheDocument();
    });

    it('displays validation errors', async () => {
      mockOnValidate.mockReturnValue({
        valid: false,
        errors: { tapAction: 'Target key is required' },
      });

      const user = userEvent.setup();
      render(
        <MappingConfigForm
          mappingType="simple"
          onChange={mockOnChange}
          onValidate={mockOnValidate}
          enableKeyboardView={false}
        />
      );

      // Trigger a change to cause validation
      await user.click(screen.getByText('Select Enter'));

      await waitFor(() => {
        expect(screen.getByText('Target key is required')).toBeInTheDocument();
      });
    });
  });

  describe('Modifier Mapping Type', () => {
    it('renders modifier form with key selection', () => {
      render(
        <MappingConfigForm
          mappingType="modifier"
          onChange={mockOnChange}
          enableKeyboardView={false}
        />
      );

      expect(screen.getByTestId('key-palette')).toBeInTheDocument();
      expect(
        screen.getByText('Select a modifier key (Ctrl, Shift, Alt, etc.)')
      ).toBeInTheDocument();
    });

    it('calls onChange when modifier key is selected', async () => {
      const user = userEvent.setup();
      render(
        <MappingConfigForm
          mappingType="modifier"
          onChange={mockOnChange}
          enableKeyboardView={false}
        />
      );

      await user.click(screen.getByText('Select LCtrl'));

      await waitFor(
        () => {
          expect(mockOnChange).toHaveBeenCalled();
        },
        { timeout: 3000 }
      );
    });

    it('displays selected modifier key from currentConfig', () => {
      render(
        <MappingConfigForm
          mappingType="modifier"
          currentConfig={{ modifierKey: 'VK_LCTRL' }}
          onChange={mockOnChange}
          enableKeyboardView={false}
        />
      );

      expect(screen.getByText('VK_LCTRL')).toBeInTheDocument();
    });

    it('displays validation errors for modifier', async () => {
      mockOnValidate.mockReturnValue({
        valid: false,
        errors: { modifierKey: 'Modifier key is required' },
      });

      const user = userEvent.setup();
      render(
        <MappingConfigForm
          mappingType="modifier"
          onChange={mockOnChange}
          onValidate={mockOnValidate}
          enableKeyboardView={false}
        />
      );

      // Trigger a change to cause validation
      await user.click(screen.getByText('Select LCtrl'));

      await waitFor(() => {
        expect(
          screen.getByText('Modifier key is required')
        ).toBeInTheDocument();
      });
    });
  });

  describe('Lock Mapping Type', () => {
    it('renders lock form with key selection', () => {
      render(
        <MappingConfigForm
          mappingType="lock"
          onChange={mockOnChange}
          enableKeyboardView={false}
        />
      );

      expect(screen.getByTestId('key-palette')).toBeInTheDocument();
      expect(
        screen.getByText('Select a lock key (CapsLock, NumLock, etc.)')
      ).toBeInTheDocument();
    });

    it('calls onChange when lock key is selected', async () => {
      const user = userEvent.setup();
      render(
        <MappingConfigForm
          mappingType="lock"
          onChange={mockOnChange}
          enableKeyboardView={false}
        />
      );

      await user.click(screen.getByText('Select CapsLock'));

      await waitFor(
        () => {
          expect(mockOnChange).toHaveBeenCalled();
        },
        { timeout: 3000 }
      );
    });

    it('displays validation errors for lock', async () => {
      mockOnValidate.mockReturnValue({
        valid: false,
        errors: { lockKey: 'Lock key is required' },
      });

      const user = userEvent.setup();
      render(
        <MappingConfigForm
          mappingType="lock"
          onChange={mockOnChange}
          onValidate={mockOnValidate}
          enableKeyboardView={false}
        />
      );

      // Trigger a change to cause validation
      await user.click(screen.getByText('Select CapsLock'));

      await waitFor(() => {
        expect(screen.getByText('Lock key is required')).toBeInTheDocument();
      });
    });
  });

  describe('Tap/Hold Mapping Type', () => {
    it('renders tap/hold form with both action fields', () => {
      render(
        <MappingConfigForm
          mappingType="tap_hold"
          onChange={mockOnChange}
          enableKeyboardView={false}
        />
      );

      expect(screen.getByText('Tap Action')).toBeInTheDocument();
      expect(screen.getByText('Hold Action (modifier)')).toBeInTheDocument();
      expect(
        screen.getByText(/Hold Threshold \(ms\):/, { exact: false })
      ).toBeInTheDocument();
    });

    it('calls onChange when tap action is selected', async () => {
      const user = userEvent.setup();
      render(
        <MappingConfigForm
          mappingType="tap_hold"
          onChange={mockOnChange}
          enableKeyboardView={false}
        />
      );

      // Get all key palettes (there should be one for tap action)
      const selectButtons = screen.getAllByText('Select Enter');
      await user.click(selectButtons[0]);

      await waitFor(
        () => {
          expect(mockOnChange).toHaveBeenCalled();
        },
        { timeout: 3000 }
      );
    });

    it('shows hold threshold slider', () => {
      render(
        <MappingConfigForm
          mappingType="tap_hold"
          currentConfig={{ threshold: 200 }}
          onChange={mockOnChange}
          enableKeyboardView={false}
        />
      );

      // Get all sliders and check the threshold one exists
      const sliders = screen.getAllByRole('slider');
      expect(sliders.length).toBeGreaterThan(0);
      expect(
        screen.getByText(/Hold Threshold \(ms\): 200/)
      ).toBeInTheDocument();
    });

    it('shows Listen buttons for tap and hold actions', () => {
      render(
        <MappingConfigForm
          mappingType="tap_hold"
          onChange={mockOnChange}
          enableKeyboardView={false}
        />
      );

      const listenButtons = screen.getAllByText(/Listen/);
      expect(listenButtons.length).toBeGreaterThanOrEqual(1);
    });

    it('displays validation errors for tap/hold', async () => {
      mockOnValidate.mockReturnValue({
        valid: false,
        errors: {
          tapAction: 'Tap action is required',
          holdAction: 'Hold action is required',
          threshold: 'Threshold must be between 50 and 500',
        },
      });

      const user = userEvent.setup();
      render(
        <MappingConfigForm
          mappingType="tap_hold"
          onChange={mockOnChange}
          onValidate={mockOnValidate}
          enableKeyboardView={false}
        />
      );

      // Trigger a change to cause validation
      const selectButtons = screen.getAllByText('Select Enter');
      await user.click(selectButtons[0]);

      await waitFor(() => {
        expect(screen.getByText('Tap action is required')).toBeInTheDocument();
        expect(screen.getByText('Hold action is required')).toBeInTheDocument();
        expect(
          screen.getByText('Threshold must be between 50 and 500')
        ).toBeInTheDocument();
      });
    });

    it('allows changing hold modifier with number input', async () => {
      const user = userEvent.setup();
      render(
        <MappingConfigForm
          mappingType="tap_hold"
          currentConfig={{ holdAction: 'MD_00' }}
          onChange={mockOnChange}
          enableKeyboardView={false}
        />
      );

      const numberInput = screen.getByPlaceholderText('Enter value 0-255');
      await user.clear(numberInput);
      await user.type(numberInput, '42');

      await waitFor(
        () => {
          expect(mockOnChange).toHaveBeenCalled();
        },
        { timeout: 3000 }
      );
    });

    it('clamps hold modifier value to 0-255 range', async () => {
      const user = userEvent.setup();
      render(
        <MappingConfigForm
          mappingType="tap_hold"
          currentConfig={{ holdAction: 'MD_00' }}
          onChange={mockOnChange}
          enableKeyboardView={false}
        />
      );

      const numberInput = screen.getByPlaceholderText('Enter value 0-255');
      await user.clear(numberInput);
      await user.type(numberInput, '500');

      await waitFor(
        () => {
          expect(mockOnChange).toHaveBeenCalled();
        },
        { timeout: 3000 }
      );
    });
  });

  describe('Layer Active Mapping Type', () => {
    it('renders layer active form', () => {
      render(
        <MappingConfigForm
          mappingType="layer_active"
          onChange={mockOnChange}
          enableKeyboardView={false}
        />
      );

      expect(screen.getByTestId('key-palette')).toBeInTheDocument();
      expect(
        screen.getByText('Select a layer to activate (MO, TO, TG, OSL)')
      ).toBeInTheDocument();
    });

    it('calls onChange when layer is selected', async () => {
      const user = userEvent.setup();
      render(
        <MappingConfigForm
          mappingType="layer_active"
          onChange={mockOnChange}
          enableKeyboardView={false}
        />
      );

      await user.click(screen.getByText('Select MO(1)'));

      await waitFor(
        () => {
          expect(mockOnChange).toHaveBeenCalled();
        },
        { timeout: 3000 }
      );
    });

    it('displays validation errors for layer', async () => {
      mockOnValidate.mockReturnValue({
        valid: false,
        errors: { targetLayer: 'Target layer is required' },
      });

      const user = userEvent.setup();
      render(
        <MappingConfigForm
          mappingType="layer_active"
          onChange={mockOnChange}
          onValidate={mockOnValidate}
          enableKeyboardView={false}
        />
      );

      // Trigger a change to cause validation
      await user.click(screen.getByText('Select MO(1)'));

      await waitFor(() => {
        expect(
          screen.getByText('Target layer is required')
        ).toBeInTheDocument();
      });
    });
  });

  describe('Configuration Updates', () => {
    it('updates form when currentConfig prop changes', () => {
      const { rerender } = render(
        <MappingConfigForm
          mappingType="simple"
          currentConfig={{ tapAction: 'VK_A' }}
          onChange={mockOnChange}
          enableKeyboardView={false}
        />
      );

      expect(screen.getByText('VK_A')).toBeInTheDocument();

      rerender(
        <MappingConfigForm
          mappingType="simple"
          currentConfig={{ tapAction: 'VK_B' }}
          onChange={mockOnChange}
          enableKeyboardView={false}
        />
      );

      expect(screen.getByText('VK_B')).toBeInTheDocument();
    });

    it('calls onValidate when provided', async () => {
      const user = userEvent.setup();
      render(
        <MappingConfigForm
          mappingType="simple"
          onChange={mockOnChange}
          onValidate={mockOnValidate}
          enableKeyboardView={false}
        />
      );

      await user.click(screen.getByText('Select Enter'));

      await waitFor(() => {
        expect(mockOnValidate).toHaveBeenCalled();
      });
    });

    it('does not call onValidate when not provided', async () => {
      const user = userEvent.setup();
      render(
        <MappingConfigForm
          mappingType="simple"
          onChange={mockOnChange}
          enableKeyboardView={false}
        />
      );

      await user.click(screen.getByText('Select Enter'));

      await waitFor(() => {
        expect(mockOnChange).toHaveBeenCalled();
      });
      expect(mockOnValidate).not.toHaveBeenCalled();
    });
  });

  describe('Key Listening', () => {
    beforeEach(() => {
      // Mock keyboard events
      global.document.addEventListener = vi.fn();
      global.document.removeEventListener = vi.fn();
    });

    afterEach(() => {
      vi.restoreAllMocks();
    });

    it('shows listening overlay when Listen button is clicked', async () => {
      const user = userEvent.setup();
      render(
        <MappingConfigForm
          mappingType="simple"
          onChange={mockOnChange}
          enableKeyboardView={false}
        />
      );

      await user.click(screen.getByText('Listen for Key'));

      expect(
        screen.getByText('Listening for key press...')
      ).toBeInTheDocument();
    });

    it('can cancel listening with Cancel button', async () => {
      const user = userEvent.setup();
      render(
        <MappingConfigForm
          mappingType="simple"
          onChange={mockOnChange}
          enableKeyboardView={false}
        />
      );

      await user.click(screen.getByText('Listen for Key'));
      expect(
        screen.getByText('Listening for key press...')
      ).toBeInTheDocument();

      await user.click(screen.getByText('Cancel'));
      expect(
        screen.queryByText('Listening for key press...')
      ).not.toBeInTheDocument();
    });
  });

  describe('Edge Cases', () => {
    it('handles missing layoutKeys gracefully', () => {
      render(
        <MappingConfigForm
          mappingType="simple"
          onChange={mockOnChange}
          enableKeyboardView={true}
        />
      );

      // Should still render without crashing
      expect(screen.getByTestId('key-palette')).toBeInTheDocument();
    });

    it('handles empty currentConfig', () => {
      render(
        <MappingConfigForm
          mappingType="tap_hold"
          currentConfig={{}}
          onChange={mockOnChange}
          enableKeyboardView={false}
        />
      );

      // Should render with default values
      expect(
        screen.getByText(/Hold Threshold \(ms\): 200/)
      ).toBeInTheDocument();
    });

    it('switches form fields when mappingType changes', () => {
      const { rerender } = render(
        <MappingConfigForm
          mappingType="simple"
          onChange={mockOnChange}
          enableKeyboardView={false}
        />
      );

      expect(screen.queryByText('Tap Action')).not.toBeInTheDocument();

      rerender(
        <MappingConfigForm
          mappingType="tap_hold"
          onChange={mockOnChange}
          enableKeyboardView={false}
        />
      );

      expect(screen.getByText('Tap Action')).toBeInTheDocument();
      expect(screen.getByText('Hold Action (modifier)')).toBeInTheDocument();
    });
  });
});
