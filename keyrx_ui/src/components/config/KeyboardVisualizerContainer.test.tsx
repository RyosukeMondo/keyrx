/**
 * Unit tests for KeyboardVisualizerContainer component
 */

import React from 'react';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { KeyboardVisualizerContainer } from './KeyboardVisualizerContainer';
import type { KeyMapping } from '@/types';
import { useKeyboardLayout } from '@/hooks/useKeyboardLayout';

// Mock the KeyboardVisualizer component
interface MockKeyboardVisualizerProps {
  layout: string;
  keyMappings: Map<string, KeyMapping>;
  onKeyClick: (key: string) => void;
}

vi.mock('@/components/KeyboardVisualizer', () => ({
  KeyboardVisualizer: ({
    layout,
    keyMappings,
    onKeyClick,
  }: MockKeyboardVisualizerProps) => (
    <div data-testid="keyboard-visualizer" data-layout={layout}>
      <button onClick={() => onKeyClick('VK_A')}>Mock Key A</button>
      <div data-testid="key-mappings-count">{keyMappings.size}</div>
    </div>
  ),
}));

// Mock the useKeyboardLayout hook
vi.mock('@/hooks/useKeyboardLayout');
const mockUseKeyboardLayout = vi.mocked(useKeyboardLayout);

describe('KeyboardVisualizerContainer', () => {
  const mockSetLayout = vi.fn();

  const defaultProps = {
    profileName: 'Default',
    activeLayer: 'base',
    mappings: new Map<string, KeyMapping>(),
    onKeyClick: vi.fn(),
  };

  beforeEach(() => {
    vi.clearAllMocks();

    // Default mock implementation
    mockUseKeyboardLayout.mockReturnValue({
      layout: 'ANSI_104',
      setLayout: mockSetLayout,
      layoutKeys: [],
    });
  });

  it('renders layout selector with correct options', () => {
    render(<KeyboardVisualizerContainer {...defaultProps} />);

    const layoutSelector = screen.getByLabelText('Select keyboard layout');
    expect(layoutSelector).toBeInTheDocument();

    // Check that all layout options are present
    const options = screen.getAllByRole('option');
    expect(options).toHaveLength(11);
    expect(
      screen.getByRole('option', { name: 'ANSI Full (104)' })
    ).toBeInTheDocument();
    expect(
      screen.getByRole('option', { name: 'ISO Full (105)' })
    ).toBeInTheDocument();
    expect(screen.getByRole('option', { name: 'HHKB' })).toBeInTheDocument();
  });

  it('renders KeyboardVisualizer component', () => {
    render(<KeyboardVisualizerContainer {...defaultProps} />);

    const visualizer = screen.getByTestId('keyboard-visualizer');
    expect(visualizer).toBeInTheDocument();
  });

  it('passes correct props to KeyboardVisualizer', () => {
    const mappings = new Map<string, KeyMapping>([
      ['VK_A', { type: 'simple', tapAction: 'VK_B' }],
      ['VK_C', { type: 'simple', tapAction: 'VK_D' }],
    ]);

    render(
      <KeyboardVisualizerContainer {...defaultProps} mappings={mappings} />
    );

    // Verify mappings are passed
    expect(screen.getByTestId('key-mappings-count')).toHaveTextContent('2');
  });

  it('calls onKeyClick when a key is clicked', async () => {
    const user = userEvent.setup();
    const onKeyClick = vi.fn();

    render(
      <KeyboardVisualizerContainer {...defaultProps} onKeyClick={onKeyClick} />
    );

    const mockKey = screen.getByText('Mock Key A');
    await user.click(mockKey);

    expect(onKeyClick).toHaveBeenCalledWith('VK_A');
    expect(onKeyClick).toHaveBeenCalledTimes(1);
  });

  it('updates layout when selector changes', async () => {
    const user = userEvent.setup();

    render(<KeyboardVisualizerContainer {...defaultProps} />);

    const layoutSelector = screen.getByLabelText('Select keyboard layout');
    await user.selectOptions(layoutSelector, 'ISO_105');

    expect(mockSetLayout).toHaveBeenCalledWith('ISO_105');
  });

  it('uses initial layout from props', () => {
    render(
      <KeyboardVisualizerContainer
        {...defaultProps}
        initialLayout="COMPACT_60"
      />
    );

    expect(mockUseKeyboardLayout).toHaveBeenCalledWith('COMPACT_60');
  });

  it('defaults to ANSI_104 if no initial layout provided', () => {
    render(<KeyboardVisualizerContainer {...defaultProps} />);

    expect(mockUseKeyboardLayout).toHaveBeenCalledWith('ANSI_104');
  });

  it('applies custom className', () => {
    const { container } = render(
      <KeyboardVisualizerContainer {...defaultProps} className="custom-class" />
    );

    expect(container.firstChild).toHaveClass('custom-class');
  });

  it('displays helper text', () => {
    render(<KeyboardVisualizerContainer {...defaultProps} />);

    expect(
      screen.getByText('Click any key to configure mappings')
    ).toBeInTheDocument();
  });

  it('renders with empty mappings', () => {
    const emptyMappings = new Map<string, KeyMapping>();

    render(
      <KeyboardVisualizerContainer {...defaultProps} mappings={emptyMappings} />
    );

    expect(screen.getByTestId('key-mappings-count')).toHaveTextContent('0');
  });

  it('handles multiple layout changes', async () => {
    const user = userEvent.setup();

    render(<KeyboardVisualizerContainer {...defaultProps} />);

    const layoutSelector = screen.getByLabelText('Select keyboard layout');

    await user.selectOptions(layoutSelector, 'ISO_105');
    expect(mockSetLayout).toHaveBeenCalledWith('ISO_105');

    await user.selectOptions(layoutSelector, 'COMPACT_60');
    expect(mockSetLayout).toHaveBeenCalledWith('COMPACT_60');

    await user.selectOptions(layoutSelector, 'HHKB');
    expect(mockSetLayout).toHaveBeenCalledWith('HHKB');

    expect(mockSetLayout).toHaveBeenCalledTimes(3);
  });

  it('renders layout selector label correctly', () => {
    render(<KeyboardVisualizerContainer {...defaultProps} />);

    const label = screen.getByText('Layout:');
    expect(label).toBeInTheDocument();
    expect(label).toHaveClass('text-sm', 'font-medium', 'text-slate-300');
  });
});
