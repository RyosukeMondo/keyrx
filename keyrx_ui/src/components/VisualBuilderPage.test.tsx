/**
 * Tests for VisualBuilderPage component
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { VisualBuilderPage } from './VisualBuilderPage';
import { useConfigBuilderStore } from '@/store/configBuilderStore';

// Mock child components
vi.mock('./VirtualKeyboard', () => ({
  VirtualKeyboard: () => <div data-testid="virtual-keyboard">Virtual Keyboard</div>,
}));

vi.mock('./LayerPanel', () => ({
  LayerPanel: () => <div data-testid="layer-panel">Layer Panel</div>,
}));

vi.mock('./ModifierPanel', () => ({
  ModifierPanel: () => <div data-testid="modifier-panel">Modifier Panel</div>,
}));

vi.mock('./CodePreview', () => ({
  CodePreview: () => <div data-testid="code-preview">Code Preview</div>,
}));

// Mock Rhai utilities
vi.mock('@/utils/rhaiParser', () => ({
  parseRhaiConfig: vi.fn((text: string) => ({
    layers: [{ id: 'base', name: 'base', mappings: [], isBase: true }],
    modifiers: [],
    locks: [],
    currentLayerId: 'base',
    isDirty: false,
  })),
}));

vi.mock('@/utils/rhaiGenerator', () => ({
  generateRhaiConfig: vi.fn(() => '// Generated Rhai code'),
}));

describe('VisualBuilderPage', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useConfigBuilderStore.setState({
      layers: [{ id: 'base', name: 'base', mappings: [], isBase: true }],
      modifiers: [],
      locks: [],
      currentLayerId: 'base',
      isDirty: false,
    });
  });

  it('renders all main sections', () => {
    render(<VisualBuilderPage />);

    expect(screen.getByText('Visual Config Builder')).toBeInTheDocument();
    expect(screen.getByTestId('virtual-keyboard')).toBeInTheDocument();
    expect(screen.getByTestId('layer-panel')).toBeInTheDocument();
    expect(screen.getByTestId('modifier-panel')).toBeInTheDocument();
    expect(screen.getByTestId('code-preview')).toBeInTheDocument();
  });

  it('renders action buttons', () => {
    render(<VisualBuilderPage />);

    expect(screen.getByText('Import .rhai')).toBeInTheDocument();
    expect(screen.getByText('Export .rhai')).toBeInTheDocument();
    expect(screen.getByText('Reset')).toBeInTheDocument();
  });

  it('triggers file input when Import button is clicked', () => {
    render(<VisualBuilderPage />);

    const fileInput = document.querySelector('input[type="file"]') as HTMLInputElement;
    const clickSpy = vi.spyOn(fileInput, 'click');

    const importButton = screen.getByText('Import .rhai');
    fireEvent.click(importButton);

    expect(clickSpy).toHaveBeenCalled();
  });

  it('handles file import', async () => {
    const { parseRhaiConfig } = await import('@/utils/rhaiParser');
    render(<VisualBuilderPage />);

    const fileInput = document.querySelector('input[type="file"]') as HTMLInputElement;
    const fileContent = 'device("test") { }';
    const file = new File([fileContent], 'test.rhai', { type: 'text/plain' });

    // Mock the File.text() method for JSDOM
    Object.defineProperty(file, 'text', {
      value: vi.fn().mockResolvedValue(fileContent),
    });

    fireEvent.change(fileInput, { target: { files: [file] } });

    await waitFor(() => {
      expect(parseRhaiConfig).toHaveBeenCalledWith(fileContent);
    });

    const state = useConfigBuilderStore.getState();
    expect(state.layers).toHaveLength(1);
  });

  it('handles export button click', async () => {
    const { generateRhaiConfig } = await import('@/utils/rhaiGenerator');
    render(<VisualBuilderPage />);

    // Mock URL.createObjectURL and URL.revokeObjectURL
    const mockCreateObjectURL = vi.fn(() => 'blob:mock-url');
    const mockRevokeObjectURL = vi.fn();
    global.URL.createObjectURL = mockCreateObjectURL;
    global.URL.revokeObjectURL = mockRevokeObjectURL;

    // Mock document.createElement and click
    const mockLink = document.createElement('a');
    const clickSpy = vi.spyOn(mockLink, 'click');
    const createElementSpy = vi.spyOn(document, 'createElement').mockReturnValue(mockLink);

    const exportButton = screen.getByText('Export .rhai');
    fireEvent.click(exportButton);

    await waitFor(() => {
      expect(generateRhaiConfig).toHaveBeenCalled();
      expect(mockCreateObjectURL).toHaveBeenCalled();
      expect(clickSpy).toHaveBeenCalled();
      expect(mockRevokeObjectURL).toHaveBeenCalledWith('blob:mock-url');
    });

    createElementSpy.mockRestore();
  });

  it('handles reset with confirmation', () => {
    const confirmSpy = vi.spyOn(window, 'confirm').mockReturnValue(true);
    render(<VisualBuilderPage />);

    // Set some state first
    useConfigBuilderStore.setState({
      layers: [
        { id: 'base', name: 'base', mappings: [], isBase: true },
        { id: 'layer1', name: 'layer1', mappings: [], isBase: false },
      ],
      modifiers: [{ id: 'mod1', name: 'MyMod', triggerKey: 'KEY_A', active: false }],
      locks: [],
      currentLayerId: 'layer1',
      isDirty: true,
    });

    const resetButton = screen.getByText('Reset');
    fireEvent.click(resetButton);

    expect(confirmSpy).toHaveBeenCalled();

    const state = useConfigBuilderStore.getState();
    expect(state.layers).toHaveLength(1);
    expect(state.layers[0].name).toBe('base');
    expect(state.modifiers).toHaveLength(0);
    expect(state.isDirty).toBe(false);

    confirmSpy.mockRestore();
  });

  it('does not reset when confirmation is cancelled', () => {
    const confirmSpy = vi.spyOn(window, 'confirm').mockReturnValue(false);
    render(<VisualBuilderPage />);

    // Set some state
    useConfigBuilderStore.setState({
      layers: [
        { id: 'base', name: 'base', mappings: [], isBase: true },
        { id: 'layer1', name: 'layer1', mappings: [], isBase: false },
      ],
      modifiers: [{ id: 'mod1', name: 'MyMod', triggerKey: 'KEY_A', active: false }],
      locks: [],
      currentLayerId: 'layer1',
      isDirty: true,
    });

    const resetButton = screen.getByText('Reset');
    fireEvent.click(resetButton);

    expect(confirmSpy).toHaveBeenCalled();

    const state = useConfigBuilderStore.getState();
    expect(state.layers).toHaveLength(2); // Not reset
    expect(state.modifiers).toHaveLength(1); // Not reset

    confirmSpy.mockRestore();
  });

  it('clears file input after import', async () => {
    render(<VisualBuilderPage />);

    const fileInput = document.querySelector('input[type="file"]') as HTMLInputElement;
    const fileContent = 'device("test") { }';
    const file = new File([fileContent], 'test.rhai', { type: 'text/plain' });

    // Mock the File.text() method for JSDOM
    Object.defineProperty(file, 'text', {
      value: vi.fn().mockResolvedValue(fileContent),
    });

    fireEvent.change(fileInput, { target: { files: [file] } });

    await waitFor(() => {
      expect(fileInput.value).toBe('');
    });
  });

  it('shows help text', () => {
    render(<VisualBuilderPage />);

    expect(
      screen.getByText(/Drag keys from the keyboard to layer mappings/i)
    ).toBeInTheDocument();
  });
});
