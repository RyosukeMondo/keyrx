/**
 * CodePreview component tests
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { CodePreview } from './CodePreview';
import { useConfigBuilderStore } from '../store/configBuilderStore';

// Mock Monaco editor
vi.mock('@monaco-editor/react', () => ({
  default: ({ value }: { value: string }) => (
    <div data-testid="monaco-editor">{value}</div>
  ),
}));

// Mock the Rhai generator
vi.mock('../utils/rhaiGenerator', () => ({
  generateRhaiCode: vi.fn((config) => {
    if (config.layers.length === 0) return '// Empty configuration';
    return `// Generated code for ${config.layers.length} layers`;
  }),
}));

describe('CodePreview', () => {
  beforeEach(() => {
    useConfigBuilderStore.setState({
      layers: [
        {
          id: 'base',
          name: 'base',
          mappings: [],
          isBase: true,
        },
      ],
      modifiers: [],
      locks: [],
      currentLayerId: 'base',
      isDirty: false,
    });
  });

  it('renders the component with header', () => {
    render(<CodePreview />);
    expect(screen.getByText('Generated Rhai Configuration')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /copy code/i })).toBeInTheDocument();
  });

  it('displays generated code in Monaco editor', () => {
    render(<CodePreview />);
    const editor = screen.getByTestId('monaco-editor');
    expect(editor).toHaveTextContent(/Generated code for 1 layers/);
  });

  it('copies code to clipboard when copy button is clicked', async () => {
    const user = userEvent.setup();
    const writeTextSpy = vi.spyOn(navigator.clipboard, 'writeText').mockResolvedValue();

    render(<CodePreview />);

    const copyButton = screen.getByRole('button', { name: /copy code/i });
    await user.click(copyButton);

    expect(writeTextSpy).toHaveBeenCalledWith(expect.stringContaining('Generated code'));
    await waitFor(() => {
      expect(screen.getByText('✓ Copied!')).toBeInTheDocument();
    });

    writeTextSpy.mockRestore();
  });

  it('shows error state when clipboard write fails', async () => {
    const user = userEvent.setup();
    const writeTextSpy = vi.spyOn(navigator.clipboard, 'writeText')
      .mockRejectedValue(new Error('Clipboard error'));

    render(<CodePreview />);

    const copyButton = screen.getByRole('button', { name: /copy code/i });
    await user.click(copyButton);

    await waitFor(() => {
      expect(screen.getByText('✗ Failed')).toBeInTheDocument();
    });

    writeTextSpy.mockRestore();
  });

  it('resets copy button status after timeout', async () => {
    const user = userEvent.setup();
    const writeTextSpy = vi.spyOn(navigator.clipboard, 'writeText').mockResolvedValue();

    render(<CodePreview />);

    const copyButton = screen.getByRole('button', { name: /copy code/i });
    await user.click(copyButton);

    await waitFor(() => {
      expect(screen.getByText('✓ Copied!')).toBeInTheDocument();
    });

    // Wait for the 2-second timeout to reset the button
    await waitFor(
      () => {
        expect(screen.getByText('Copy Code')).toBeInTheDocument();
      },
      { timeout: 3000 }
    );

    writeTextSpy.mockRestore();
  });

  it('updates code when config changes', () => {
    const { rerender } = render(<CodePreview />);

    // Add a layer
    useConfigBuilderStore.getState().addLayer('custom');

    rerender(<CodePreview />);

    const editor = screen.getByTestId('monaco-editor');
    expect(editor).toHaveTextContent(/Generated code for 2 layers/);
  });

  it('disables copy button while copying', async () => {
    const user = userEvent.setup();
    const writeTextSpy = vi.spyOn(navigator.clipboard, 'writeText').mockResolvedValue();

    render(<CodePreview />);

    const copyButton = screen.getByRole('button', { name: /copy code/i });

    // Click and immediately check that button becomes disabled with "Copied" state
    await user.click(copyButton);

    // Button should show "✓ Copied!" and be disabled
    await waitFor(() => {
      const button = screen.getByRole('button', { name: /✓ copied!/i });
      expect(button).toBeDisabled();
    });

    writeTextSpy.mockRestore();
  });
});
