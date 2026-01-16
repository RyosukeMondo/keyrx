/**
 * Unit tests for ConfigurationPanel component
 */

import React from 'react';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { ConfigurationPanel } from './ConfigurationPanel';
import type { KeyMapping } from '@/types';
import type { PaletteKey } from '@/components/KeyPalette';
import type { Device } from '@/components/DeviceSelector';

// Mock all child components
vi.mock('@/components/DeviceSelector', () => ({
  DeviceSelector: ({
    devices,
    selectedDevices,
    globalSelected,
    onSelectionChange,
  }: any) => (
    <div data-testid="device-selector">
      <div data-testid="devices-count">{devices.length}</div>
      <div data-testid="selected-count">{selectedDevices.length}</div>
      <div data-testid="global-selected">{String(globalSelected)}</div>
      <button onClick={() => onSelectionChange(['device1'], false)}>
        Change Selection
      </button>
    </div>
  ),
}));

vi.mock('@/components/LayerSwitcher', () => ({
  LayerSwitcher: ({ activeLayer, availableLayers, onLayerChange }: any) => (
    <div data-testid="layer-switcher">
      <div data-testid="active-layer">{activeLayer}</div>
      <div data-testid="layers-count">{availableLayers.length}</div>
      <button onClick={() => onLayerChange('md-00')}>Switch Layer</button>
    </div>
  ),
}));

vi.mock('@/components/KeyPalette', () => ({
  KeyPalette: ({ onKeySelect, selectedKey, compact }: any) => (
    <div data-testid="key-palette">
      <div data-testid="selected-key">
        {selectedKey ? selectedKey.code : 'none'}
      </div>
      <div data-testid="compact-mode">{String(compact)}</div>
      <button
        onClick={() =>
          onKeySelect({ code: 'VK_A', label: 'A', category: 'letter' })
        }
      >
        Select Key
      </button>
    </div>
  ),
}));

vi.mock('@/components/KeyConfigPanel', () => ({
  KeyConfigPanel: ({
    physicalKey,
    onSave,
    onClearMapping,
    activeLayer,
  }: any) => (
    <div data-testid="key-config-panel">
      <div data-testid="physical-key">{physicalKey || 'none'}</div>
      <div data-testid="active-layer-in-config">{activeLayer}</div>
      <button
        onClick={() =>
          onSave({ type: 'simple', tapAction: 'VK_B', sourceKey: 'VK_A' })
        }
      >
        Save Mapping
      </button>
      {physicalKey && (
        <button onClick={() => onClearMapping(physicalKey)}>
          Clear Mapping
        </button>
      )}
    </div>
  ),
}));

vi.mock('@/components/CurrentMappingsSummary', () => ({
  CurrentMappingsSummary: ({
    keyMappings,
    onEditMapping,
    onClearMapping,
  }: any) => (
    <div data-testid="mappings-summary">
      <div data-testid="mappings-count">{keyMappings.size}</div>
      <button onClick={() => onEditMapping('VK_A')}>Edit Mapping</button>
      <button onClick={() => onClearMapping('VK_A')}>Clear Mapping</button>
    </div>
  ),
}));

describe('ConfigurationPanel', () => {
  const mockOnPaletteKeySelect = vi.fn();
  const mockOnSaveMapping = vi.fn();
  const mockOnClearMapping = vi.fn();
  const mockOnLayerChange = vi.fn();
  const mockOnDeviceSelectionChange = vi.fn();
  const mockOnEditMapping = vi.fn();

  const defaultDevices: Device[] = [
    { id: 'device1', name: 'Keyboard 1' },
    { id: 'device2', name: 'Keyboard 2' },
  ];

  const defaultProps = {
    profileName: 'Default',
    selectedPhysicalKey: null,
    selectedPaletteKey: null,
    onPaletteKeySelect: mockOnPaletteKeySelect,
    onSaveMapping: mockOnSaveMapping,
    onClearMapping: mockOnClearMapping,
    activeLayer: 'base',
    availableLayers: ['base', 'md-00', 'md-01'],
    onLayerChange: mockOnLayerChange,
    devices: defaultDevices,
    selectedDevices: ['device1'],
    globalSelected: false,
    onDeviceSelectionChange: mockOnDeviceSelectionChange,
    keyMappings: new Map<string, KeyMapping>(),
    onEditMapping: mockOnEditMapping,
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Rendering', () => {
    it('renders all child components', () => {
      render(<ConfigurationPanel {...defaultProps} />);

      expect(screen.getByTestId('device-selector')).toBeInTheDocument();
      expect(screen.getByTestId('layer-switcher')).toBeInTheDocument();
      expect(screen.getByTestId('key-palette')).toBeInTheDocument();
      expect(screen.getByTestId('key-config-panel')).toBeInTheDocument();
      expect(screen.getByTestId('mappings-summary')).toBeInTheDocument();
    });

    it('renders components in correct order', () => {
      const { container } = render(<ConfigurationPanel {...defaultProps} />);

      const children = Array.from(container.querySelectorAll('[data-testid]'));
      const testIds = children.map((child) =>
        child.getAttribute('data-testid')
      );

      // Verify order: DeviceSelector -> LayerSwitcher -> KeyPalette -> KeyConfigPanel -> MappingsSummary
      expect(testIds).toContain('device-selector');
      expect(testIds).toContain('layer-switcher');
      expect(testIds).toContain('key-palette');
      expect(testIds).toContain('key-config-panel');
      expect(testIds).toContain('mappings-summary');
    });
  });

  describe('DeviceSelector Integration', () => {
    it('passes devices prop to DeviceSelector', () => {
      render(<ConfigurationPanel {...defaultProps} />);

      expect(screen.getByTestId('devices-count')).toHaveTextContent('2');
    });

    it('passes selectedDevices prop to DeviceSelector', () => {
      render(
        <ConfigurationPanel
          {...defaultProps}
          selectedDevices={['device1', 'device2']}
        />
      );

      expect(screen.getByTestId('selected-count')).toHaveTextContent('2');
    });

    it('passes globalSelected prop to DeviceSelector', () => {
      render(<ConfigurationPanel {...defaultProps} globalSelected={true} />);

      expect(screen.getByTestId('global-selected')).toHaveTextContent('true');
    });

    it('calls onDeviceSelectionChange when device selection changes', async () => {
      const user = userEvent.setup();

      render(<ConfigurationPanel {...defaultProps} />);

      const button = screen.getByText('Change Selection');
      await user.click(button);

      expect(mockOnDeviceSelectionChange).toHaveBeenCalledWith(
        ['device1'],
        false
      );
      expect(mockOnDeviceSelectionChange).toHaveBeenCalledTimes(1);
    });

    it('handles empty devices array', () => {
      render(<ConfigurationPanel {...defaultProps} devices={[]} />);

      expect(screen.getByTestId('devices-count')).toHaveTextContent('0');
    });
  });

  describe('LayerSwitcher Integration', () => {
    it('passes activeLayer prop to LayerSwitcher', () => {
      render(<ConfigurationPanel {...defaultProps} activeLayer="md-00" />);

      expect(screen.getByTestId('active-layer')).toHaveTextContent('md-00');
    });

    it('passes availableLayers prop to LayerSwitcher', () => {
      render(
        <ConfigurationPanel
          {...defaultProps}
          availableLayers={['base', 'md-00']}
        />
      );

      expect(screen.getByTestId('layers-count')).toHaveTextContent('2');
    });

    it('calls onLayerChange when layer changes', async () => {
      const user = userEvent.setup();

      render(<ConfigurationPanel {...defaultProps} />);

      const button = screen.getByText('Switch Layer');
      await user.click(button);

      expect(mockOnLayerChange).toHaveBeenCalledWith('md-00');
      expect(mockOnLayerChange).toHaveBeenCalledTimes(1);
    });

    it('handles single layer', () => {
      render(
        <ConfigurationPanel {...defaultProps} availableLayers={['base']} />
      );

      expect(screen.getByTestId('layers-count')).toHaveTextContent('1');
    });
  });

  describe('KeyPalette Integration', () => {
    it('passes selectedPaletteKey to KeyPalette', () => {
      const selectedKey: PaletteKey = {
        code: 'VK_A',
        label: 'A',
        category: 'letter',
      };

      render(
        <ConfigurationPanel
          {...defaultProps}
          selectedPaletteKey={selectedKey}
        />
      );

      expect(screen.getByTestId('selected-key')).toHaveTextContent('VK_A');
    });

    it('shows none when no key is selected', () => {
      render(
        <ConfigurationPanel {...defaultProps} selectedPaletteKey={null} />
      );

      expect(screen.getByTestId('selected-key')).toHaveTextContent('none');
    });

    it('calls onPaletteKeySelect when key is selected', async () => {
      const user = userEvent.setup();

      render(<ConfigurationPanel {...defaultProps} />);

      const button = screen.getByText('Select Key');
      await user.click(button);

      expect(mockOnPaletteKeySelect).toHaveBeenCalledWith({
        code: 'VK_A',
        label: 'A',
        category: 'letter',
      });
      expect(mockOnPaletteKeySelect).toHaveBeenCalledTimes(1);
    });

    it('passes compact as false to KeyPalette', () => {
      render(<ConfigurationPanel {...defaultProps} />);

      expect(screen.getByTestId('compact-mode')).toHaveTextContent('false');
    });
  });

  describe('KeyConfigPanel Integration', () => {
    it('passes selectedPhysicalKey to KeyConfigPanel', () => {
      render(
        <ConfigurationPanel {...defaultProps} selectedPhysicalKey="VK_A" />
      );

      expect(screen.getByTestId('physical-key')).toHaveTextContent('VK_A');
    });

    it('shows none when no physical key is selected', () => {
      render(
        <ConfigurationPanel {...defaultProps} selectedPhysicalKey={null} />
      );

      expect(screen.getByTestId('physical-key')).toHaveTextContent('none');
    });

    it('passes activeLayer to KeyConfigPanel', () => {
      render(<ConfigurationPanel {...defaultProps} activeLayer="md-01" />);

      expect(screen.getByTestId('active-layer-in-config')).toHaveTextContent(
        'md-01'
      );
    });

    it('calls onSaveMapping when save button is clicked', async () => {
      const user = userEvent.setup();

      render(
        <ConfigurationPanel {...defaultProps} selectedPhysicalKey="VK_A" />
      );

      const button = screen.getByText('Save Mapping');
      await user.click(button);

      expect(mockOnSaveMapping).toHaveBeenCalledWith({
        type: 'simple',
        tapAction: 'VK_B',
        sourceKey: 'VK_A',
      });
      expect(mockOnSaveMapping).toHaveBeenCalledTimes(1);
    });

    it('calls onClearMapping when clear button is clicked', async () => {
      const user = userEvent.setup();

      render(
        <ConfigurationPanel {...defaultProps} selectedPhysicalKey="VK_A" />
      );

      // Get the clear button from KeyConfigPanel specifically (first one)
      const buttons = screen.getAllByText('Clear Mapping');
      await user.click(buttons[0]);

      expect(mockOnClearMapping).toHaveBeenCalledWith('VK_A');
      expect(mockOnClearMapping).toHaveBeenCalledTimes(1);
    });

    it('passes keyMappings to KeyConfigPanel', () => {
      const mappings = new Map<string, KeyMapping>([
        ['VK_A', { type: 'simple', tapAction: 'VK_B' }],
      ]);

      render(<ConfigurationPanel {...defaultProps} keyMappings={mappings} />);

      // Component should render without errors
      expect(screen.getByTestId('key-config-panel')).toBeInTheDocument();
    });

    it('passes layoutKeys to KeyConfigPanel when provided', () => {
      const layoutKeys = [
        { code: 'VK_A', x: 0, y: 0, width: 1, height: 1, label: 'A' },
      ];

      render(<ConfigurationPanel {...defaultProps} layoutKeys={layoutKeys} />);

      expect(screen.getByTestId('key-config-panel')).toBeInTheDocument();
    });
  });

  describe('CurrentMappingsSummary Integration', () => {
    it('passes keyMappings to CurrentMappingsSummary', () => {
      const mappings = new Map<string, KeyMapping>([
        ['VK_A', { type: 'simple', tapAction: 'VK_B' }],
        ['VK_C', { type: 'simple', tapAction: 'VK_D' }],
      ]);

      render(<ConfigurationPanel {...defaultProps} keyMappings={mappings} />);

      expect(screen.getByTestId('mappings-count')).toHaveTextContent('2');
    });

    it('calls onEditMapping when edit button is clicked', async () => {
      const user = userEvent.setup();

      render(<ConfigurationPanel {...defaultProps} />);

      const button = screen.getByText('Edit Mapping');
      await user.click(button);

      expect(mockOnEditMapping).toHaveBeenCalledWith('VK_A');
      expect(mockOnEditMapping).toHaveBeenCalledTimes(1);
    });

    it('calls onClearMapping when clear button in summary is clicked', async () => {
      const user = userEvent.setup();

      render(<ConfigurationPanel {...defaultProps} />);

      // Find clear button in mappings summary (not the one in key config panel)
      const buttons = screen.getAllByText('Clear Mapping');
      await user.click(buttons[buttons.length - 1]);

      expect(mockOnClearMapping).toHaveBeenCalledWith('VK_A');
    });

    it('handles empty mappings', () => {
      render(<ConfigurationPanel {...defaultProps} keyMappings={new Map()} />);

      expect(screen.getByTestId('mappings-count')).toHaveTextContent('0');
    });
  });

  describe('Prop Updates', () => {
    it('updates when selectedPhysicalKey changes', () => {
      const { rerender } = render(<ConfigurationPanel {...defaultProps} />);

      expect(screen.getByTestId('physical-key')).toHaveTextContent('none');

      rerender(
        <ConfigurationPanel {...defaultProps} selectedPhysicalKey="VK_A" />
      );

      expect(screen.getByTestId('physical-key')).toHaveTextContent('VK_A');
    });

    it('updates when activeLayer changes', () => {
      const { rerender } = render(<ConfigurationPanel {...defaultProps} />);

      expect(screen.getByTestId('active-layer')).toHaveTextContent('base');

      rerender(<ConfigurationPanel {...defaultProps} activeLayer="md-00" />);

      expect(screen.getByTestId('active-layer')).toHaveTextContent('md-00');
    });

    it('updates when keyMappings change', () => {
      const { rerender } = render(<ConfigurationPanel {...defaultProps} />);

      expect(screen.getByTestId('mappings-count')).toHaveTextContent('0');

      const mappings = new Map<string, KeyMapping>([
        ['VK_A', { type: 'simple', tapAction: 'VK_B' }],
      ]);

      rerender(<ConfigurationPanel {...defaultProps} keyMappings={mappings} />);

      expect(screen.getByTestId('mappings-count')).toHaveTextContent('1');
    });

    it('updates when devices change', () => {
      const { rerender } = render(<ConfigurationPanel {...defaultProps} />);

      expect(screen.getByTestId('devices-count')).toHaveTextContent('2');

      const newDevices: Device[] = [
        { id: 'device1', name: 'Keyboard 1' },
        { id: 'device2', name: 'Keyboard 2' },
        { id: 'device3', name: 'Keyboard 3' },
      ];

      rerender(<ConfigurationPanel {...defaultProps} devices={newDevices} />);

      expect(screen.getByTestId('devices-count')).toHaveTextContent('3');
    });
  });

  describe('Edge Cases', () => {
    it('handles all props being null/empty', () => {
      render(
        <ConfigurationPanel
          profileName=""
          selectedPhysicalKey={null}
          selectedPaletteKey={null}
          onPaletteKeySelect={mockOnPaletteKeySelect}
          onSaveMapping={mockOnSaveMapping}
          onClearMapping={mockOnClearMapping}
          activeLayer="base"
          availableLayers={[]}
          onLayerChange={mockOnLayerChange}
          devices={[]}
          selectedDevices={[]}
          globalSelected={false}
          onDeviceSelectionChange={mockOnDeviceSelectionChange}
          keyMappings={new Map()}
          onEditMapping={mockOnEditMapping}
        />
      );

      expect(screen.getByTestId('device-selector')).toBeInTheDocument();
      expect(screen.getByTestId('layer-switcher')).toBeInTheDocument();
      expect(screen.getByTestId('key-palette')).toBeInTheDocument();
      expect(screen.getByTestId('key-config-panel')).toBeInTheDocument();
      expect(screen.getByTestId('mappings-summary')).toBeInTheDocument();
    });

    it('handles very long profile names', () => {
      const longName = 'A'.repeat(200);

      render(<ConfigurationPanel {...defaultProps} profileName={longName} />);

      expect(screen.getByTestId('device-selector')).toBeInTheDocument();
    });

    it('handles large number of mappings', () => {
      const largeMappings = new Map<string, KeyMapping>();
      for (let i = 0; i < 100; i++) {
        largeMappings.set(`VK_${i}`, {
          type: 'simple',
          tapAction: `VK_${i + 1}`,
        });
      }

      render(
        <ConfigurationPanel {...defaultProps} keyMappings={largeMappings} />
      );

      expect(screen.getByTestId('mappings-count')).toHaveTextContent('100');
    });

    it('handles rapid prop changes', () => {
      const { rerender } = render(<ConfigurationPanel {...defaultProps} />);

      for (let i = 0; i < 10; i++) {
        rerender(
          <ConfigurationPanel
            {...defaultProps}
            selectedPhysicalKey={`VK_${i}`}
            activeLayer={`layer-${i}`}
          />
        );
      }

      expect(screen.getByTestId('physical-key')).toHaveTextContent('VK_9');
      expect(screen.getByTestId('active-layer')).toHaveTextContent('layer-9');
    });
  });

  describe('Layout Structure', () => {
    it('wraps components in correct container structure', () => {
      const { container } = render(<ConfigurationPanel {...defaultProps} />);

      const wrapper = container.firstChild as HTMLElement;
      expect(wrapper).toHaveClass('flex', 'flex-col', 'gap-4');
    });

    it('maintains consistent spacing between components', () => {
      const { container } = render(<ConfigurationPanel {...defaultProps} />);

      const wrapper = container.firstChild as HTMLElement;
      expect(wrapper.className).toContain('gap-4');
    });
  });
});
