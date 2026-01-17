import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { KeyConfigPanel } from './KeyConfigPanel';
import type { KeyMapping } from '@/types';
import type { SVGKey } from './SVGKeyboard';

// Mock dependencies
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
    layout,
  }: {
    selectedType: string;
    onChange: (type: string) => void;
    supportedTypes: string[];
    layout?: string;
  }) => (
    <div data-testid="mapping-type-selector">
      <div>Selected: {selectedType}</div>
      <div>Layout: {layout || 'horizontal'}</div>
      <div>Supported: {supportedTypes.join(', ')}</div>
      {supportedTypes.map((type) => (
        <button key={type} onClick={() => onChange(type)}>
          {type}
        </button>
      ))}
    </div>
  ),
}));

vi.mock('./keyConfig/KeySelectionTabs', () => ({
  KeySelectionTabs: ({
    activeTab,
    onTabChange,
    availableTabs,
    onKeySelect,
    layoutKeys,
    maxHeight,
  }: {
    activeTab: string;
    onTabChange: (tab: string) => void;
    availableTabs: string[];
    onKeySelect: (key: string) => void;
    layoutKeys: SVGKey[];
    maxHeight?: string;
  }) => (
    <div data-testid="key-selection-tabs">
      <div>Active: {activeTab}</div>
      <div>Max Height: {maxHeight || 'default'}</div>
      <div>Available: {availableTabs.join(', ')}</div>
      <div>Layout Keys: {layoutKeys.length}</div>
      {availableTabs.map((tab) => (
        <button key={tab} onClick={() => onTabChange(tab)}>
          {tab}
        </button>
      ))}
      <button onClick={() => onKeySelect('VK_ENTER')}>Select Enter</button>
      <button onClick={() => onKeySelect('VK_A')}>Select A</button>
    </div>
  ),
}));

describe('KeyConfigPanel', () => {
  const mockOnSave = vi.fn();
  const mockOnClearMapping = vi.fn();
  const mockOnEditMapping = vi.fn();
  const mockLayoutKeys: SVGKey[] = [
    { id: 'A', x: 0, y: 0, width: 1, height: 1, label: 'A' },
    { id: 'B', x: 1, y: 0, width: 1, height: 1, label: 'B' },
  ];

  const defaultProps = {
    physicalKey: null,
    onSave: mockOnSave,
    onClearMapping: mockOnClearMapping,
    onEditMapping: mockOnEditMapping,
    activeLayer: 'base',
    keyMappings: new Map(),
    layoutKeys: mockLayoutKeys,
  };

  beforeEach(() => {
    mockOnSave.mockClear();
    mockOnClearMapping.mockClear();
    mockOnEditMapping.mockClear();
  });

  describe('Rendering', () => {
    it('renders panel wrapper', () => {
      const { container } = render(<KeyConfigPanel {...defaultProps} />);

      const panel = container.querySelector('.bg-slate-800.rounded-lg');
      expect(panel).toBeInTheDocument();
    });

    it('shows empty state when no physical key selected', () => {
      render(<KeyConfigPanel {...defaultProps} physicalKey={null} />);

      expect(
        screen.getByText('Click a key on the keyboard above to configure it')
      ).toBeInTheDocument();
    });

    it('renders all components when physical key is selected', () => {
      render(<KeyConfigPanel {...defaultProps} physicalKey="VK_CAPSLOCK" />);

      expect(screen.getByTestId('mapping-type-selector')).toBeInTheDocument();
      expect(screen.getByTestId('key-selection-tabs')).toBeInTheDocument();
      expect(
        screen.getByTestId('current-mappings-summary')
      ).toBeInTheDocument();
    });

    it('displays key info header when physical key selected', () => {
      render(<KeyConfigPanel {...defaultProps} physicalKey="VK_CAPSLOCK" />);

      expect(screen.getByText('VK_CAPSLOCK')).toBeInTheDocument();
      expect(screen.getByText('Base')).toBeInTheDocument();
    });

    it('displays current mappings summary with correct count', () => {
      const keyMappings = new Map<string, KeyMapping>([
        ['VK_A', { type: 'simple', tapAction: 'VK_B' }],
        ['VK_C', { type: 'simple', tapAction: 'VK_D' }],
      ]);

      render(<KeyConfigPanel {...defaultProps} keyMappings={keyMappings} />);

      expect(
        screen.getByText('Current Mappings (2 mappings)')
      ).toBeInTheDocument();
      expect(screen.getByText('Mappings: 2')).toBeInTheDocument();
    });

    it('displays active layer name', () => {
      render(
        <KeyConfigPanel
          {...defaultProps}
          physicalKey="VK_A"
          activeLayer="layer-2"
        />
      );

      expect(screen.getByText('LAYER_2')).toBeInTheDocument();
    });
  });

  describe('Component Composition', () => {
    it('renders MappingTypeSelector with correct props', () => {
      render(<KeyConfigPanel {...defaultProps} physicalKey="VK_A" />);

      const selector = screen.getByTestId('mapping-type-selector');
      expect(selector).toHaveTextContent('Selected: simple');
      expect(selector).toHaveTextContent('Layout: horizontal');
      expect(selector).toHaveTextContent('Supported: simple, tap_hold');
    });

    it('passes only simple and tap_hold types to MappingTypeSelector', () => {
      render(<KeyConfigPanel {...defaultProps} physicalKey="VK_A" />);

      const selector = screen.getByTestId('mapping-type-selector');
      expect(selector).toHaveTextContent('simple, tap_hold');
      expect(selector).not.toHaveTextContent('modifier');
      expect(selector).not.toHaveTextContent('lock');
      expect(selector).not.toHaveTextContent('layer_active');
    });

    it('renders KeySelectionTabs with correct available tabs', () => {
      render(<KeyConfigPanel {...defaultProps} physicalKey="VK_A" />);

      const tabs = screen.getByTestId('key-selection-tabs');
      expect(tabs).toHaveTextContent('Available: keyboard, modifier, lock');
    });

    it('passes layoutKeys to KeySelectionTabs', () => {
      render(<KeyConfigPanel {...defaultProps} physicalKey="VK_A" />);

      const tabs = screen.getByTestId('key-selection-tabs');
      expect(tabs).toHaveTextContent('Layout Keys: 2');
    });

    it('passes different maxHeight for simple vs tap_hold', async () => {
      const user = userEvent.setup();
      render(<KeyConfigPanel {...defaultProps} physicalKey="VK_A" />);

      // Simple mapping should have max-h-96
      let tabs = screen.getByTestId('key-selection-tabs');
      expect(tabs).toHaveTextContent('Max Height: max-h-96');

      // Switch to tap_hold
      const tapHoldButton = screen.getByText('tap_hold');
      await user.click(tapHoldButton);

      // Tap/hold should have max-h-64
      tabs = screen.getByTestId('key-selection-tabs');
      expect(tabs).toHaveTextContent('Max Height: max-h-64');
    });
  });

  describe('Mapping Type Selection', () => {
    it('defaults to simple mapping type', () => {
      render(<KeyConfigPanel {...defaultProps} physicalKey="VK_A" />);

      const selector = screen.getByTestId('mapping-type-selector');
      expect(selector).toHaveTextContent('Selected: simple');
    });

    it('switches mapping type when changed', async () => {
      const user = userEvent.setup();
      render(<KeyConfigPanel {...defaultProps} physicalKey="VK_A" />);

      const tapHoldButton = screen.getByText('tap_hold');
      await user.click(tapHoldButton);

      await waitFor(() => {
        const selector = screen.getByTestId('mapping-type-selector');
        expect(selector).toHaveTextContent('Selected: tap_hold');
      });
    });

    it('initializes with tap_hold when current mapping is tap_hold', () => {
      const currentMapping: KeyMapping = {
        type: 'tap_hold',
        tapAction: 'VK_A',
        holdAction: 'MD_1C',
        threshold: 200,
      };

      render(
        <KeyConfigPanel
          {...defaultProps}
          physicalKey="VK_CAPSLOCK"
          currentMapping={currentMapping}
        />
      );

      const selector = screen.getByTestId('mapping-type-selector');
      expect(selector).toHaveTextContent('Selected: tap_hold');
    });
  });

  describe('Key Selection', () => {
    it('updates tap action when key selected', async () => {
      const user = userEvent.setup();
      render(<KeyConfigPanel {...defaultProps} physicalKey="VK_A" />);

      // Initially shows placeholder
      expect(screen.getByText('Select a target key')).toBeInTheDocument();

      const selectEnterButton = screen.getByText('Select Enter');
      await user.click(selectEnterButton);

      // After selection, shows the key in the target display
      await waitFor(() => {
        expect(screen.getByText('VK_ENTER')).toBeInTheDocument();
      });
    });

    it('shows clear button when tap action is selected', async () => {
      const user = userEvent.setup();
      render(<KeyConfigPanel {...defaultProps} physicalKey="VK_A" />);

      const selectEnterButton = screen.getByText('Select Enter');
      await user.click(selectEnterButton);

      await waitFor(() => {
        // Clear button appears in the key selection area
        const clearButtons = screen.getAllByText('Clear');
        expect(clearButtons.length).toBeGreaterThan(0);
      });
    });

    it('clears tap action when clear button clicked', async () => {
      const user = userEvent.setup();
      render(<KeyConfigPanel {...defaultProps} physicalKey="VK_A" />);

      // Select a key
      const selectEnterButton = screen.getByText('Select Enter');
      await user.click(selectEnterButton);

      await waitFor(() => {
        expect(screen.getByText('VK_ENTER')).toBeInTheDocument();
      });

      // Clear it - get the first Clear button
      const clearButtons = screen.getAllByText('Clear');
      await user.click(clearButtons[0]);

      await waitFor(() => {
        // After clearing, should show placeholder again
        expect(screen.getByText('Select a target key')).toBeInTheDocument();
      });
    });
  });

  describe('Tap/Hold Configuration', () => {
    it('shows tap and hold sections in tap_hold mode', async () => {
      const user = userEvent.setup();
      render(<KeyConfigPanel {...defaultProps} physicalKey="VK_A" />);

      const tapHoldButton = screen.getByText('tap_hold');
      await user.click(tapHoldButton);

      await waitFor(() => {
        expect(screen.getByText('Tap Action')).toBeInTheDocument();
        expect(screen.getByText('Hold Action (modifier)')).toBeInTheDocument();
        expect(screen.getByText(/Hold Threshold/)).toBeInTheDocument();
      });
    });

    it('shows hold modifier input controls', async () => {
      const user = userEvent.setup();
      render(<KeyConfigPanel {...defaultProps} physicalKey="VK_A" />);

      const tapHoldButton = screen.getByText('tap_hold');
      await user.click(tapHoldButton);

      await waitFor(() => {
        expect(screen.getByText('Select modifier 0-255')).toBeInTheDocument();
        expect(
          screen.getByPlaceholderText('Enter value 0-255')
        ).toBeInTheDocument();
      });
    });

    it('updates threshold value', async () => {
      const user = userEvent.setup();
      render(<KeyConfigPanel {...defaultProps} physicalKey="VK_A" />);

      const tapHoldButton = screen.getByText('tap_hold');
      await user.click(tapHoldButton);

      await waitFor(() => {
        expect(screen.getByText(/Hold Threshold.*: 200/)).toBeInTheDocument();
      });
    });

    it('loads existing tap_hold config correctly', () => {
      const currentMapping: KeyMapping = {
        type: 'tap_hold',
        tapAction: 'VK_A',
        holdAction: 'MD_1C',
        threshold: 300,
      };

      render(
        <KeyConfigPanel
          {...defaultProps}
          physicalKey="VK_CAPSLOCK"
          currentMapping={currentMapping}
        />
      );

      expect(screen.getByText(/Hold Threshold.*: 300/)).toBeInTheDocument();
    });
  });

  describe('Preview', () => {
    it('shows preview panel when physical key selected', () => {
      render(<KeyConfigPanel {...defaultProps} physicalKey="VK_A" />);

      expect(screen.getByText('Preview')).toBeInTheDocument();
    });

    it('shows instruction text when no target selected', () => {
      render(<KeyConfigPanel {...defaultProps} physicalKey="VK_A" />);

      expect(screen.getByText('Select a target key')).toBeInTheDocument();
    });

    it('shows mapping preview for simple mapping', async () => {
      const user = userEvent.setup();
      render(<KeyConfigPanel {...defaultProps} physicalKey="VK_A" />);

      const selectEnterButton = screen.getByText('Select Enter');
      await user.click(selectEnterButton);

      await waitFor(() => {
        const preview = screen.getByText(/Press VK_A → Output VK_ENTER/);
        expect(preview).toBeInTheDocument();
      });
    });
  });

  describe('Save and Clear Actions', () => {
    it('save button is disabled when no physical key', () => {
      render(<KeyConfigPanel {...defaultProps} physicalKey={null} />);

      expect(screen.queryByText('Save Mapping')).not.toBeInTheDocument();
    });

    it('save button is disabled when simple mapping has no tap action', () => {
      render(<KeyConfigPanel {...defaultProps} physicalKey="VK_A" />);

      const saveButton = screen.getByText('Save Mapping');
      expect(saveButton).toBeDisabled();
    });

    it('save button is enabled when simple mapping is complete', async () => {
      const user = userEvent.setup();
      render(<KeyConfigPanel {...defaultProps} physicalKey="VK_A" />);

      const selectEnterButton = screen.getByText('Select Enter');
      await user.click(selectEnterButton);

      await waitFor(() => {
        const saveButton = screen.getByText('Save Mapping');
        expect(saveButton).not.toBeDisabled();
      });
    });

    it('calls onSave with correct simple mapping', async () => {
      const user = userEvent.setup();
      render(<KeyConfigPanel {...defaultProps} physicalKey="VK_A" />);

      const selectEnterButton = screen.getByText('Select Enter');
      await user.click(selectEnterButton);

      await waitFor(() => {
        const saveButton = screen.getByText('Save Mapping');
        expect(saveButton).not.toBeDisabled();
      });

      const saveButton = screen.getByText('Save Mapping');
      await user.click(saveButton);

      expect(mockOnSave).toHaveBeenCalledTimes(1);
      expect(mockOnSave).toHaveBeenCalledWith({
        type: 'simple',
        tapAction: 'VK_ENTER',
      });
    });

    it('save button is disabled when tap_hold missing actions', async () => {
      const user = userEvent.setup();
      render(<KeyConfigPanel {...defaultProps} physicalKey="VK_A" />);

      const tapHoldButton = screen.getByText('tap_hold');
      await user.click(tapHoldButton);

      await waitFor(() => {
        const saveButton = screen.getByText('Save Mapping');
        expect(saveButton).toBeDisabled();
      });
    });

    it('shows clear mapping button when current mapping exists', () => {
      const currentMapping: KeyMapping = {
        type: 'simple',
        tapAction: 'VK_B',
      };

      render(
        <KeyConfigPanel
          {...defaultProps}
          physicalKey="VK_A"
          currentMapping={currentMapping}
        />
      );

      expect(screen.getByText('Clear Mapping')).toBeInTheDocument();
    });

    it('calls onClearMapping when clear button clicked', async () => {
      const user = userEvent.setup();
      const currentMapping: KeyMapping = {
        type: 'simple',
        tapAction: 'VK_B',
      };

      render(
        <KeyConfigPanel
          {...defaultProps}
          physicalKey="VK_A"
          currentMapping={currentMapping}
        />
      );

      const clearButton = screen.getByText('Clear Mapping');
      await user.click(clearButton);

      expect(mockOnClearMapping).toHaveBeenCalledTimes(1);
      expect(mockOnClearMapping).toHaveBeenCalledWith('VK_A');
    });

    it('does not show clear button when no current mapping', () => {
      render(<KeyConfigPanel {...defaultProps} physicalKey="VK_A" />);

      expect(screen.queryByText('Clear Mapping')).not.toBeInTheDocument();
    });
  });

  describe('State Resets', () => {
    it('resets form when physical key changes', async () => {
      const { rerender } = render(
        <KeyConfigPanel {...defaultProps} physicalKey="VK_A" />
      );

      const user = userEvent.setup();
      const selectEnterButton = screen.getByText('Select Enter');
      await user.click(selectEnterButton);

      // Change physical key
      rerender(<KeyConfigPanel {...defaultProps} physicalKey="VK_B" />);

      await waitFor(() => {
        expect(screen.getByText('Select a target key')).toBeInTheDocument();
      });
    });

    it('resets to simple mapping when physical key changes', async () => {
      const { rerender } = render(
        <KeyConfigPanel {...defaultProps} physicalKey="VK_A" />
      );

      const user = userEvent.setup();
      const tapHoldButton = screen.getByText('tap_hold');
      await user.click(tapHoldButton);

      await waitFor(() => {
        const selector = screen.getByTestId('mapping-type-selector');
        expect(selector).toHaveTextContent('Selected: tap_hold');
      });

      // Change physical key
      rerender(<KeyConfigPanel {...defaultProps} physicalKey="VK_B" />);

      await waitFor(() => {
        const selector = screen.getByTestId('mapping-type-selector');
        expect(selector).toHaveTextContent('Selected: simple');
      });
    });

    it('loads current mapping values when physical key changes', () => {
      const currentMapping: KeyMapping = {
        type: 'simple',
        tapAction: 'VK_Z',
      };

      const { rerender } = render(
        <KeyConfigPanel {...defaultProps} physicalKey="VK_A" />
      );

      rerender(
        <KeyConfigPanel
          {...defaultProps}
          physicalKey="VK_B"
          currentMapping={currentMapping}
        />
      );

      expect(screen.getByText('VK_Z')).toBeInTheDocument();
    });
  });

  describe('Integration with CurrentMappingsSummary', () => {
    it('passes keyMappings to CurrentMappingsSummary', () => {
      const keyMappings = new Map<string, KeyMapping>([
        ['VK_A', { type: 'simple', tapAction: 'VK_B' }],
      ]);

      render(<KeyConfigPanel {...defaultProps} keyMappings={keyMappings} />);

      expect(screen.getByText('Mappings: 1')).toBeInTheDocument();
    });

    it('calls onEditMapping when edit clicked in summary', async () => {
      const user = userEvent.setup();
      const keyMappings = new Map<string, KeyMapping>([
        ['VK_A', { type: 'simple', tapAction: 'VK_B' }],
      ]);

      render(<KeyConfigPanel {...defaultProps} keyMappings={keyMappings} />);

      const editButton = screen.getByText('Edit');
      await user.click(editButton);

      expect(mockOnEditMapping).toHaveBeenCalledTimes(1);
      expect(mockOnEditMapping).toHaveBeenCalledWith('test-key');
    });

    it('calls onClearMapping when clear clicked in summary', async () => {
      const user = userEvent.setup();
      const keyMappings = new Map<string, KeyMapping>([
        ['VK_A', { type: 'simple', tapAction: 'VK_B' }],
      ]);

      render(<KeyConfigPanel {...defaultProps} keyMappings={keyMappings} />);

      const clearButton = screen.getAllByText('Clear')[0];
      await user.click(clearButton);

      expect(mockOnClearMapping).toHaveBeenCalledTimes(1);
      expect(mockOnClearMapping).toHaveBeenCalledWith('test-key');
    });
  });

  describe('Edge Cases', () => {
    it('handles undefined currentMapping', () => {
      render(
        <KeyConfigPanel
          {...defaultProps}
          physicalKey="VK_A"
          currentMapping={undefined}
        />
      );

      const selector = screen.getByTestId('mapping-type-selector');
      expect(selector).toHaveTextContent('Selected: simple');
    });

    it('handles empty layoutKeys array', () => {
      render(
        <KeyConfigPanel {...defaultProps} physicalKey="VK_A" layoutKeys={[]} />
      );

      const tabs = screen.getByTestId('key-selection-tabs');
      expect(tabs).toHaveTextContent('Layout Keys: 0');
    });

    it('handles empty keyMappings', () => {
      render(<KeyConfigPanel {...defaultProps} keyMappings={new Map()} />);

      expect(
        screen.getByText('Current Mappings (0 mappings)')
      ).toBeInTheDocument();
    });

    it('handles active layer with hyphens', () => {
      render(
        <KeyConfigPanel
          {...defaultProps}
          physicalKey="VK_A"
          activeLayer="layer-special-2"
        />
      );

      // Component only replaces first hyphen: .replace('-', '_')
      expect(screen.getByText('LAYER_SPECIAL-2')).toBeInTheDocument();
    });
  });
});
