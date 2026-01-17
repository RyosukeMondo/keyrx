import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { KeyConfigModal } from './KeyConfigModal';
import type { KeyMapping } from '@/types';
import type { SVGKey } from './SVGKeyboard';

// Mock dependencies
vi.mock('./Modal', () => ({
  Modal: ({
    open,
    onClose,
    title,
    children,
  }: {
    open: boolean;
    onClose: () => void;
    title: string;
    children: React.ReactNode;
  }) =>
    open ? (
      <div role="dialog" aria-label={title} data-testid="modal">
        <h2>{title}</h2>
        <button onClick={onClose} aria-label="Close modal">
          Close
        </button>
        {children}
      </div>
    ) : null,
}));

vi.mock('./CurrentMappingsSummary', () => ({
  CurrentMappingsSummary: ({
    keyMappings,
    onEditMapping,
    onClearMapping,
  }: {
    keyMappings: Map<string, KeyMapping>;
    onEditMapping: (key: string) => void;
    onClearMapping: (key: string) => void;
  }) => (
    <div data-testid="current-mappings-summary">
      <div>Mappings: {keyMappings.size}</div>
      <button onClick={() => onEditMapping('test-key')}>Edit</button>
      <button onClick={() => onClearMapping('test-key')}>Clear</button>
    </div>
  ),
}));

vi.mock('./keyConfig/MappingTypeSelector', () => ({
  MappingTypeSelector: ({
    selectedType,
    onChange,
    supportedTypes,
  }: {
    selectedType: string;
    onChange: (type: string) => void;
    supportedTypes: string[];
  }) => (
    <div data-testid="mapping-type-selector">
      <div>Selected: {selectedType}</div>
      {supportedTypes.map((type) => (
        <button key={type} onClick={() => onChange(type)}>
          {type}
        </button>
      ))}
    </div>
  ),
}));

vi.mock('./keyConfig/MappingConfigForm', () => ({
  MappingConfigForm: ({
    mappingType,
    currentConfig,
    onChange,
    layoutKeys,
    enableKeyboardView,
  }: {
    mappingType: string;
    currentConfig: Record<string, unknown>;
    onChange: (config: Record<string, unknown>) => void;
    layoutKeys: SVGKey[];
    enableKeyboardView: boolean;
  }) => (
    <div data-testid="mapping-config-form">
      <div>Type: {mappingType}</div>
      <div>Keyboard View: {enableKeyboardView ? 'enabled' : 'disabled'}</div>
      <div>Layout Keys: {layoutKeys.length}</div>
      <button
        onClick={() => onChange({ ...currentConfig, tapAction: 'VK_ENTER' })}
      >
        Set Target Key
      </button>
      <button
        onClick={() => onChange({ ...currentConfig, modifierKey: 'VK_LCTRL' })}
      >
        Set Modifier
      </button>
      <button
        onClick={() =>
          onChange({ ...currentConfig, tapAction: 'VK_A', holdAction: 'VK_B' })
        }
      >
        Set Tap/Hold
      </button>
    </div>
  ),
}));

describe('KeyConfigModal', () => {
  const mockOnClose = vi.fn();
  const mockOnSave = vi.fn();
  const mockLayoutKeys: SVGKey[] = [
    { id: 'A', x: 0, y: 0, width: 1, height: 1, label: 'A' },
    { id: 'B', x: 1, y: 0, width: 1, height: 1, label: 'B' },
  ];

  const defaultProps = {
    isOpen: true,
    onClose: mockOnClose,
    physicalKey: 'VK_CAPSLOCK',
    onSave: mockOnSave,
    activeLayer: 'base',
    keyMappings: new Map(),
    layoutKeys: mockLayoutKeys,
  };

  beforeEach(() => {
    mockOnClose.mockClear();
    mockOnSave.mockClear();
  });

  describe('Rendering', () => {
    it('renders modal when isOpen is true', () => {
      render(<KeyConfigModal {...defaultProps} />);

      expect(screen.getByRole('dialog')).toBeInTheDocument();
      expect(screen.getByText('Configure Key Mapping')).toBeInTheDocument();
    });

    it('does not render when isOpen is false', () => {
      render(<KeyConfigModal {...defaultProps} isOpen={false} />);

      expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    });

    it('renders all child components', () => {
      render(<KeyConfigModal {...defaultProps} />);

      expect(screen.getByTestId('mapping-type-selector')).toBeInTheDocument();
      expect(screen.getByTestId('mapping-config-form')).toBeInTheDocument();
      expect(screen.getByText('Preview')).toBeInTheDocument();
    });

    it('displays physical key name', () => {
      render(<KeyConfigModal {...defaultProps} physicalKey="VK_A" />);

      expect(screen.getByText('VK_A')).toBeInTheDocument();
    });

    it('displays active layer', () => {
      render(<KeyConfigModal {...defaultProps} activeLayer="layer-1" />);

      expect(screen.getByText('LAYER_1')).toBeInTheDocument();
    });

    it('displays base layer correctly', () => {
      render(<KeyConfigModal {...defaultProps} activeLayer="base" />);

      expect(screen.getByText('Base')).toBeInTheDocument();
    });

    it('renders CurrentMappingsSummary when keyMappings exist', () => {
      const keyMappings = new Map([
        ['VK_A', { type: 'simple', tapAction: 'VK_B' } as KeyMapping],
      ]);
      render(<KeyConfigModal {...defaultProps} keyMappings={keyMappings} />);

      expect(
        screen.getByTestId('current-mappings-summary')
      ).toBeInTheDocument();
      expect(screen.getByText('Mappings: 1')).toBeInTheDocument();
    });

    it('does not render CurrentMappingsSummary when no mappings', () => {
      render(<KeyConfigModal {...defaultProps} keyMappings={new Map()} />);

      expect(
        screen.queryByTestId('current-mappings-summary')
      ).not.toBeInTheDocument();
    });

    it('passes layoutKeys to MappingConfigForm', () => {
      render(<KeyConfigModal {...defaultProps} />);

      expect(screen.getByText('Layout Keys: 2')).toBeInTheDocument();
    });

    it('enables keyboard view in MappingConfigForm', () => {
      render(<KeyConfigModal {...defaultProps} />);

      expect(screen.getByText('Keyboard View: enabled')).toBeInTheDocument();
    });
  });

  describe('Mapping Type Selection', () => {
    it('initializes with simple type when no currentMapping', () => {
      render(<KeyConfigModal {...defaultProps} />);

      expect(screen.getByText('Selected: simple')).toBeInTheDocument();
    });

    it('initializes with currentMapping type', () => {
      const currentMapping: KeyMapping = {
        type: 'tap_hold',
        tapAction: 'VK_A',
        holdAction: 'VK_B',
        threshold: 200,
      };
      render(
        <KeyConfigModal {...defaultProps} currentMapping={currentMapping} />
      );

      expect(screen.getByText('Selected: tap_hold')).toBeInTheDocument();
    });

    it('changes mapping type when type selector is used', async () => {
      const user = userEvent.setup();
      render(<KeyConfigModal {...defaultProps} />);

      await user.click(screen.getByText('modifier'));

      expect(screen.getByText('Selected: modifier')).toBeInTheDocument();
    });

    it('supports all 5 mapping types', () => {
      render(<KeyConfigModal {...defaultProps} />);

      expect(screen.getByText('simple')).toBeInTheDocument();
      expect(screen.getByText('modifier')).toBeInTheDocument();
      expect(screen.getByText('lock')).toBeInTheDocument();
      expect(screen.getByText('tap_hold')).toBeInTheDocument();
      expect(screen.getByText('layer_active')).toBeInTheDocument();
    });
  });

  describe('Configuration Changes', () => {
    it('updates config when form changes', async () => {
      const user = userEvent.setup();
      render(<KeyConfigModal {...defaultProps} />);

      await user.click(screen.getByText('Set Target Key'));

      // Preview should update
      await waitFor(() => {
        expect(
          screen.getByText(/Press VK_CAPSLOCK → Output VK_ENTER/)
        ).toBeInTheDocument();
      });
    });

    it('preserves currentMapping in config', () => {
      const currentMapping: KeyMapping = {
        type: 'simple',
        tapAction: 'VK_A',
      };
      render(
        <KeyConfigModal {...defaultProps} currentMapping={currentMapping} />
      );

      expect(
        screen.getByText(/Press VK_CAPSLOCK → Output VK_A/)
      ).toBeInTheDocument();
    });
  });

  describe('Preview Text', () => {
    it('shows default preview for simple with no config', () => {
      render(<KeyConfigModal {...defaultProps} />);

      expect(
        screen.getByText('Select a target key to map to')
      ).toBeInTheDocument();
    });

    it('shows preview for simple mapping', async () => {
      const user = userEvent.setup();
      render(<KeyConfigModal {...defaultProps} />);

      await user.click(screen.getByText('Set Target Key'));

      await waitFor(() => {
        expect(
          screen.getByText(/Press VK_CAPSLOCK → Output VK_ENTER/)
        ).toBeInTheDocument();
      });
    });

    it('shows preview for modifier mapping', async () => {
      const user = userEvent.setup();
      render(<KeyConfigModal {...defaultProps} />);

      await user.click(screen.getByText('modifier'));
      await user.click(screen.getByText('Set Modifier'));

      await waitFor(() => {
        expect(
          screen.getByText('VK_CAPSLOCK acts as VK_LCTRL modifier')
        ).toBeInTheDocument();
      });
    });

    it('shows preview for tap_hold mapping', async () => {
      const user = userEvent.setup();
      render(<KeyConfigModal {...defaultProps} />);

      await user.click(screen.getByText('tap_hold'));
      await user.click(screen.getByText('Set Tap/Hold'));

      await waitFor(() => {
        expect(
          screen.getByText(/Quick tap: VK_CAPSLOCK → VK_A/)
        ).toBeInTheDocument();
      });
    });

    it('shows default tap_hold preview when not configured', async () => {
      const user = userEvent.setup();
      render(<KeyConfigModal {...defaultProps} />);

      await user.click(screen.getByText('tap_hold'));

      expect(
        screen.getByText('Configure tap and hold actions')
      ).toBeInTheDocument();
    });
  });

  describe('Save and Cancel Actions', () => {
    it('calls onClose when cancel button is clicked', async () => {
      const user = userEvent.setup();
      render(<KeyConfigModal {...defaultProps} />);

      await user.click(screen.getByText('Cancel'));

      expect(mockOnClose).toHaveBeenCalledTimes(1);
    });

    it('calls onClose from Modal close button', async () => {
      const user = userEvent.setup();
      render(<KeyConfigModal {...defaultProps} />);

      await user.click(screen.getByLabelText('Close modal'));

      expect(mockOnClose).toHaveBeenCalledTimes(1);
    });

    it('calls onSave with config and closes modal', async () => {
      const user = userEvent.setup();
      render(<KeyConfigModal {...defaultProps} />);

      await user.click(screen.getByText('Set Target Key'));
      await user.click(screen.getByText('Save Mapping'));

      await waitFor(() => {
        expect(mockOnSave).toHaveBeenCalledTimes(1);
        expect(mockOnSave).toHaveBeenCalledWith(
          expect.objectContaining({
            tapAction: 'VK_ENTER',
          })
        );
        expect(mockOnClose).toHaveBeenCalledTimes(1);
      });
    });

    it('disables save button when config is invalid for simple', () => {
      render(<KeyConfigModal {...defaultProps} />);

      const saveButton = screen.getByText('Save Mapping');
      expect(saveButton).toBeDisabled();
    });

    it('enables save button when config is valid for simple', async () => {
      const user = userEvent.setup();
      render(<KeyConfigModal {...defaultProps} />);

      await user.click(screen.getByText('Set Target Key'));

      await waitFor(() => {
        const saveButton = screen.getByText('Save Mapping');
        expect(saveButton).not.toBeDisabled();
      });
    });

    it('disables save button when modifier not configured', async () => {
      const user = userEvent.setup();
      render(<KeyConfigModal {...defaultProps} />);

      await user.click(screen.getByText('modifier'));

      const saveButton = screen.getByText('Save Mapping');
      expect(saveButton).toBeDisabled();
    });

    it('enables save button when modifier configured', async () => {
      const user = userEvent.setup();
      render(<KeyConfigModal {...defaultProps} />);

      await user.click(screen.getByText('modifier'));
      await user.click(screen.getByText('Set Modifier'));

      await waitFor(() => {
        const saveButton = screen.getByText('Save Mapping');
        expect(saveButton).not.toBeDisabled();
      });
    });

    it('disables save button when tap_hold partially configured', async () => {
      const user = userEvent.setup();
      render(<KeyConfigModal {...defaultProps} />);

      await user.click(screen.getByText('tap_hold'));
      await user.click(screen.getByText('Set Target Key'));

      await waitFor(() => {
        const saveButton = screen.getByText('Save Mapping');
        expect(saveButton).toBeDisabled();
      });
    });

    it('enables save button when tap_hold fully configured', async () => {
      const user = userEvent.setup();
      render(<KeyConfigModal {...defaultProps} />);

      await user.click(screen.getByText('tap_hold'));
      await user.click(screen.getByText('Set Tap/Hold'));

      await waitFor(() => {
        const saveButton = screen.getByText('Save Mapping');
        expect(saveButton).not.toBeDisabled();
      });
    });
  });

  describe('Integration with Child Components', () => {
    it('passes correct props to MappingTypeSelector', () => {
      render(<KeyConfigModal {...defaultProps} />);

      const selector = screen.getByTestId('mapping-type-selector');
      expect(selector).toBeInTheDocument();
      expect(screen.getByText('simple')).toBeInTheDocument();
      expect(screen.getByText('modifier')).toBeInTheDocument();
      expect(screen.getByText('lock')).toBeInTheDocument();
      expect(screen.getByText('tap_hold')).toBeInTheDocument();
      expect(screen.getByText('layer_active')).toBeInTheDocument();
    });

    it('passes correct props to MappingConfigForm', async () => {
      const user = userEvent.setup();
      render(<KeyConfigModal {...defaultProps} />);

      await user.click(screen.getByText('modifier'));

      await waitFor(() => {
        expect(screen.getByText('Type: modifier')).toBeInTheDocument();
      });
    });

    it('updates form when type changes', async () => {
      const user = userEvent.setup();
      render(<KeyConfigModal {...defaultProps} />);

      expect(screen.getByText('Type: simple')).toBeInTheDocument();

      await user.click(screen.getByText('tap_hold'));

      await waitFor(() => {
        expect(screen.getByText('Type: tap_hold')).toBeInTheDocument();
      });
    });
  });

  describe('Edge Cases', () => {
    it('handles invalid mapping type in currentMapping', () => {
      const currentMapping = {
        type: 'invalid_type',
        tapAction: 'VK_A',
      } as unknown as KeyMapping;
      render(
        <KeyConfigModal {...defaultProps} currentMapping={currentMapping} />
      );

      // Should default to simple
      expect(screen.getByText('Selected: simple')).toBeInTheDocument();
    });

    it('handles undefined currentMapping', () => {
      render(<KeyConfigModal {...defaultProps} currentMapping={undefined} />);

      expect(screen.getByText('Selected: simple')).toBeInTheDocument();
    });

    it('handles empty keyMappings', () => {
      render(<KeyConfigModal {...defaultProps} keyMappings={new Map()} />);

      expect(
        screen.queryByTestId('current-mappings-summary')
      ).not.toBeInTheDocument();
    });

    it('handles empty layoutKeys', () => {
      render(<KeyConfigModal {...defaultProps} layoutKeys={[]} />);

      expect(screen.getByText('Layout Keys: 0')).toBeInTheDocument();
    });

    it('handles changing config after type change', async () => {
      const user = userEvent.setup();
      render(<KeyConfigModal {...defaultProps} />);

      await user.click(screen.getByText('Set Target Key'));
      await user.click(screen.getByText('modifier'));
      await user.click(screen.getByText('Set Modifier'));

      await waitFor(() => {
        expect(
          screen.getByText('VK_CAPSLOCK acts as VK_LCTRL modifier')
        ).toBeInTheDocument();
      });
    });
  });

  describe('Accessibility', () => {
    it('has dialog role', () => {
      render(<KeyConfigModal {...defaultProps} />);

      expect(screen.getByRole('dialog')).toBeInTheDocument();
    });

    it('has proper title', () => {
      render(<KeyConfigModal {...defaultProps} />);

      expect(screen.getByText('Configure Key Mapping')).toBeInTheDocument();
    });

    it('has cancel and save buttons', () => {
      render(<KeyConfigModal {...defaultProps} />);

      expect(screen.getByText('Cancel')).toBeInTheDocument();
      expect(screen.getByText('Save Mapping')).toBeInTheDocument();
    });

    it('disables save button when invalid', () => {
      render(<KeyConfigModal {...defaultProps} />);

      const saveButton = screen.getByText('Save Mapping');
      expect(saveButton).toBeDisabled();
      expect(saveButton).toHaveClass('disabled:opacity-50');
      expect(saveButton).toHaveClass('disabled:cursor-not-allowed');
    });
  });
});
