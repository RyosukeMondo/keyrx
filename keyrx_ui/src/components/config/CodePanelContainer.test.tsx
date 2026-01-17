/**
 * Unit tests for CodePanelContainer component
 */

import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { CodePanelContainer } from './CodePanelContainer';
import { useCodePanel } from '@/hooks/useCodePanel';
import type { RhaiSyncEngineResult } from '../RhaiSyncEngine';

// Mock the MonacoEditor component
interface MockMonacoEditorProps {
  value: string;
  onChange: (value: string) => void;
  height: string;
}

vi.mock('../MonacoEditor', () => ({
  MonacoEditor: ({ value, onChange, height }: MockMonacoEditorProps) => (
    <div data-testid="monaco-editor" data-height={height}>
      <textarea
        data-testid="editor-textarea"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        aria-label="Code editor"
      />
    </div>
  ),
}));

// Mock the useCodePanel hook
vi.mock('@/hooks/useCodePanel');
const mockUseCodePanel = vi.mocked(useCodePanel);

describe('CodePanelContainer', () => {
  const mockToggleOpen = vi.fn();
  const mockSetHeight = vi.fn();
  const mockOnChange = vi.fn();
  const mockClearError = vi.fn();

  const createMockSyncEngine = (
    overrides?: Partial<RhaiSyncEngineResult>
  ): RhaiSyncEngineResult =>
    ({
      state: 'idle',
      error: null,
      clearError: mockClearError,
      getCode: vi.fn(() => ''),
      onCodeChange: vi.fn(),
      ...overrides,
    }) as unknown as RhaiSyncEngineResult;

  const defaultProps = {
    profileName: 'Default',
    rhaiCode: 'fn main() { }',
    onChange: mockOnChange,
    syncEngine: createMockSyncEngine(),
  };

  beforeEach(() => {
    vi.clearAllMocks();

    // Default mock implementation - panel open
    mockUseCodePanel.mockReturnValue({
      isOpen: true,
      height: 300,
      toggleOpen: mockToggleOpen,
      setHeight: mockSetHeight,
    });
  });

  describe('Rendering', () => {
    it('renders when isOpen is true', () => {
      render(<CodePanelContainer {...defaultProps} />);

      expect(screen.getByTestId('code-panel-container')).toBeInTheDocument();
    });

    it('does not render when isOpen is false', () => {
      mockUseCodePanel.mockReturnValue({
        isOpen: false,
        height: 300,
        toggleOpen: mockToggleOpen,
        setHeight: mockSetHeight,
      });

      render(<CodePanelContainer {...defaultProps} />);

      expect(
        screen.queryByTestId('code-panel-container')
      ).not.toBeInTheDocument();
    });

    it('displays profile name in header', () => {
      render(<CodePanelContainer {...defaultProps} profileName="Gaming" />);

      expect(screen.getByText(/Code - Gaming/i)).toBeInTheDocument();
    });

    it('renders Monaco editor with correct props', () => {
      render(<CodePanelContainer {...defaultProps} rhaiCode="test code" />);

      const editor = screen.getByTestId('monaco-editor');
      expect(editor).toBeInTheDocument();

      const textarea = screen.getByTestId('editor-textarea');
      expect(textarea).toHaveValue('test code');
    });

    it('renders hide button', () => {
      render(<CodePanelContainer {...defaultProps} />);

      const hideButton = screen.getByRole('button', {
        name: /hide code editor/i,
      });
      expect(hideButton).toBeInTheDocument();
      expect(hideButton).toHaveTextContent('▼ Hide');
    });

    it('renders resize handle', () => {
      render(<CodePanelContainer {...defaultProps} />);

      const resizeHandle = screen.getByLabelText('Resize handle');
      expect(resizeHandle).toBeInTheDocument();
    });

    it('applies correct height style', () => {
      mockUseCodePanel.mockReturnValue({
        isOpen: true,
        height: 450,
        toggleOpen: mockToggleOpen,
        setHeight: mockSetHeight,
      });

      render(<CodePanelContainer {...defaultProps} />);

      const container = screen.getByTestId('code-panel-container');
      expect(container).toHaveStyle({ height: '450px' });
    });
  });

  describe('Toggle Functionality', () => {
    it('calls toggleOpen when hide button is clicked', async () => {
      const user = userEvent.setup();

      render(<CodePanelContainer {...defaultProps} />);

      const hideButton = screen.getByRole('button', {
        name: /hide code editor/i,
      });
      await user.click(hideButton);

      expect(mockToggleOpen).toHaveBeenCalledTimes(1);
    });
  });

  describe('Resize Functionality', () => {
    it('initiates resize on mouse down', () => {
      render(<CodePanelContainer {...defaultProps} />);

      const resizeHandle = screen.getByLabelText('Resize handle');
      fireEvent.mouseDown(resizeHandle, { clientY: 500 });

      // Verify that mouse move handlers are set up by checking if setHeight is not called yet
      expect(mockSetHeight).not.toHaveBeenCalled();
    });

    it('updates height during drag', () => {
      render(<CodePanelContainer {...defaultProps} />);

      const resizeHandle = screen.getByLabelText('Resize handle');

      // Start drag at Y=500
      fireEvent.mouseDown(resizeHandle, { clientY: 500 });

      // Drag upward to Y=400 (delta = -100, should increase height)
      fireEvent.mouseMove(document, { clientY: 400 });

      expect(mockSetHeight).toHaveBeenCalled();
      // Height should increase: 300 + (500 - 400) = 400
      expect(mockSetHeight).toHaveBeenCalledWith(400);
    });

    it('enforces minimum height of 200px', () => {
      render(<CodePanelContainer {...defaultProps} />);

      const resizeHandle = screen.getByLabelText('Resize handle');

      // Start drag at Y=100
      fireEvent.mouseDown(resizeHandle, { clientY: 100 });

      // Drag downward to Y=500 (delta = 400, should try to decrease height below minimum)
      fireEvent.mouseMove(document, { clientY: 500 });

      expect(mockSetHeight).toHaveBeenCalled();
      // Height should be clamped to minimum: Math.max(200, 300 + (100 - 500)) = 200
      const lastCall =
        mockSetHeight.mock.calls[mockSetHeight.mock.calls.length - 1];
      expect(lastCall[0]).toBeGreaterThanOrEqual(200);
    });

    it('enforces maximum height of 600px', () => {
      render(<CodePanelContainer {...defaultProps} />);

      const resizeHandle = screen.getByLabelText('Resize handle');

      // Start drag at Y=500
      fireEvent.mouseDown(resizeHandle, { clientY: 500 });

      // Drag upward to Y=100 (delta = -400, should try to increase height above maximum)
      fireEvent.mouseMove(document, { clientY: 100 });

      expect(mockSetHeight).toHaveBeenCalled();
      // Height should be clamped to maximum: Math.min(600, 300 + (500 - 100)) = 600
      const lastCall =
        mockSetHeight.mock.calls[mockSetHeight.mock.calls.length - 1];
      expect(lastCall[0]).toBeLessThanOrEqual(600);
    });

    it('removes event listeners on mouse up', () => {
      render(<CodePanelContainer {...defaultProps} />);

      const resizeHandle = screen.getByLabelText('Resize handle');

      // Start drag
      fireEvent.mouseDown(resizeHandle, { clientY: 500 });
      fireEvent.mouseMove(document, { clientY: 400 });

      const firstCallCount = mockSetHeight.mock.calls.length;

      // End drag
      fireEvent.mouseUp(document);

      // Move again after mouse up
      fireEvent.mouseMove(document, { clientY: 300 });

      // setHeight should not be called again after mouse up
      expect(mockSetHeight).toHaveBeenCalledTimes(firstCallCount);
    });
  });

  describe('Code Editor Integration', () => {
    it('passes rhaiCode to editor', () => {
      const code = 'fn test() { print("hello"); }';
      render(<CodePanelContainer {...defaultProps} rhaiCode={code} />);

      const textarea = screen.getByTestId('editor-textarea');
      expect(textarea).toHaveValue(code);
    });

    it('calls onChange when editor content changes', async () => {
      const user = userEvent.setup();

      render(<CodePanelContainer {...defaultProps} />);

      const textarea = screen.getByTestId('editor-textarea');
      await user.clear(textarea);
      await user.type(textarea, 'new code');

      expect(mockOnChange).toHaveBeenCalled();
    });
  });

  describe('Sync Status Display', () => {
    it('shows parsing status', () => {
      const syncEngine = createMockSyncEngine({ state: 'parsing' });
      render(<CodePanelContainer {...defaultProps} syncEngine={syncEngine} />);

      expect(screen.getByText('Parsing Rhai script...')).toBeInTheDocument();
    });

    it('shows generating status', () => {
      const syncEngine = createMockSyncEngine({ state: 'generating' });
      render(<CodePanelContainer {...defaultProps} syncEngine={syncEngine} />);

      expect(screen.getByText('Generating code...')).toBeInTheDocument();
    });

    it('shows syncing status', () => {
      const syncEngine = createMockSyncEngine({ state: 'syncing' });
      render(<CodePanelContainer {...defaultProps} syncEngine={syncEngine} />);

      expect(screen.getByText('Syncing...')).toBeInTheDocument();
    });

    it('does not show status indicator when idle', () => {
      const syncEngine = createMockSyncEngine({ state: 'idle' });
      render(<CodePanelContainer {...defaultProps} syncEngine={syncEngine} />);

      expect(
        screen.queryByText('Parsing Rhai script...')
      ).not.toBeInTheDocument();
      expect(screen.queryByText('Generating code...')).not.toBeInTheDocument();
      expect(screen.queryByText('Syncing...')).not.toBeInTheDocument();
    });

    it('shows spinner during non-idle states', () => {
      const syncEngine = createMockSyncEngine({ state: 'parsing' });
      const { container } = render(
        <CodePanelContainer {...defaultProps} syncEngine={syncEngine} />
      );

      const spinner = container.querySelector('.animate-spin');
      expect(spinner).toBeInTheDocument();
    });
  });

  describe('Error Display', () => {
    it('displays parse error message', () => {
      const syncEngine = createMockSyncEngine({
        error: {
          message: 'Unexpected token',
          line: 10,
          column: 5,
        },
      });

      render(<CodePanelContainer {...defaultProps} syncEngine={syncEngine} />);

      expect(screen.getByText(/Parse Error/i)).toBeInTheDocument();
      expect(
        screen.getByText(/Line 10, Column 5: Unexpected token/i)
      ).toBeInTheDocument();
    });

    it('displays error suggestion if provided', () => {
      const syncEngine = createMockSyncEngine({
        error: {
          message: 'Missing semicolon',
          line: 5,
          column: 20,
          suggestion: 'Add a semicolon at the end of the line',
        },
      });

      render(<CodePanelContainer {...defaultProps} syncEngine={syncEngine} />);

      expect(
        screen.getByText(/💡 Add a semicolon at the end of the line/i)
      ).toBeInTheDocument();
    });

    it('does not display error suggestion if not provided', () => {
      const syncEngine = createMockSyncEngine({
        error: {
          message: 'Syntax error',
          line: 1,
          column: 1,
        },
      });

      render(<CodePanelContainer {...defaultProps} syncEngine={syncEngine} />);

      expect(screen.queryByText(/💡/)).not.toBeInTheDocument();
    });

    it('calls clearError when clear button is clicked', async () => {
      const user = userEvent.setup();

      const syncEngine = createMockSyncEngine({
        error: {
          message: 'Test error',
          line: 1,
          column: 1,
        },
      });

      render(<CodePanelContainer {...defaultProps} syncEngine={syncEngine} />);

      const clearButton = screen.getByRole('button', { name: /clear error/i });
      await user.click(clearButton);

      expect(mockClearError).toHaveBeenCalledTimes(1);
    });

    it('does not display error panel when error is null', () => {
      const syncEngine = createMockSyncEngine({ error: null });
      render(<CodePanelContainer {...defaultProps} syncEngine={syncEngine} />);

      expect(screen.queryByText(/Parse Error/i)).not.toBeInTheDocument();
    });
  });

  describe('Height Calculation', () => {
    it('adjusts editor height based on header elements', () => {
      render(<CodePanelContainer {...defaultProps} />);

      const editor = screen.getByTestId('monaco-editor');
      const heightAttr = editor.getAttribute('data-height');

      // Base height 300 - base header 60 = 240px
      expect(heightAttr).toBe('240px');
    });

    it('reduces editor height when sync status is shown', () => {
      const syncEngine = createMockSyncEngine({ state: 'parsing' });
      render(<CodePanelContainer {...defaultProps} syncEngine={syncEngine} />);

      const editor = screen.getByTestId('monaco-editor');
      const heightAttr = editor.getAttribute('data-height');

      // Base height 300 - base header 60 - sync status 60 = 180px
      expect(heightAttr).toBe('180px');
    });

    it('reduces editor height when error is shown', () => {
      const syncEngine = createMockSyncEngine({
        error: { message: 'Error', line: 1, column: 1 },
      });
      render(<CodePanelContainer {...defaultProps} syncEngine={syncEngine} />);

      const editor = screen.getByTestId('monaco-editor');
      const heightAttr = editor.getAttribute('data-height');

      // Base height 300 - base header 60 - error 80 = 160px
      expect(heightAttr).toBe('160px');
    });

    it('reduces editor height when both sync status and error are shown', () => {
      const syncEngine = createMockSyncEngine({
        state: 'parsing',
        error: { message: 'Error', line: 1, column: 1 },
      });
      render(<CodePanelContainer {...defaultProps} syncEngine={syncEngine} />);

      const editor = screen.getByTestId('monaco-editor');
      const heightAttr = editor.getAttribute('data-height');

      // Base height 300 - base header 60 - sync status 60 - error 80 = 100px
      expect(heightAttr).toBe('100px');
    });
  });

  describe('Accessibility', () => {
    it('has accessible label for hide button', () => {
      render(<CodePanelContainer {...defaultProps} />);

      const hideButton = screen.getByRole('button', {
        name: /hide code editor/i,
      });
      expect(hideButton).toHaveAttribute('aria-label', 'Hide code editor');
    });

    it('has accessible label for resize handle', () => {
      render(<CodePanelContainer {...defaultProps} />);

      const resizeHandle = screen.getByLabelText('Resize handle');
      expect(resizeHandle).toBeInTheDocument();
    });

    it('has accessible label for clear error button', () => {
      const syncEngine = createMockSyncEngine({
        error: { message: 'Error', line: 1, column: 1 },
      });

      render(<CodePanelContainer {...defaultProps} syncEngine={syncEngine} />);

      const clearButton = screen.getByRole('button', { name: /clear error/i });
      expect(clearButton).toHaveAttribute('aria-label', 'Clear error');
    });
  });

  describe('Edge Cases', () => {
    it('handles empty rhaiCode', () => {
      render(<CodePanelContainer {...defaultProps} rhaiCode="" />);

      const textarea = screen.getByTestId('editor-textarea');
      expect(textarea).toHaveValue('');
    });

    it('handles very long profile names', () => {
      const longName = 'A'.repeat(100);
      render(<CodePanelContainer {...defaultProps} profileName={longName} />);

      expect(
        screen.getByText(new RegExp(`Code - ${longName}`))
      ).toBeInTheDocument();
    });

    it('handles rapid state changes', () => {
      const { rerender } = render(<CodePanelContainer {...defaultProps} />);

      const states: Array<RhaiSyncEngineResult['state']> = [
        'parsing',
        'generating',
        'syncing',
        'idle',
      ];

      states.forEach((state) => {
        const syncEngine = createMockSyncEngine({ state });
        rerender(
          <CodePanelContainer {...defaultProps} syncEngine={syncEngine} />
        );
        expect(screen.getByTestId('code-panel-container')).toBeInTheDocument();
      });
    });
  });
});
