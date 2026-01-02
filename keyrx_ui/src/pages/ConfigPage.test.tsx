import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import { renderWithProviders } from '../../tests/testUtils';
import userEvent from '@testing-library/user-event';
import { ConfigPage } from './ConfigPage';
import * as useUnifiedApiModule from '@/hooks/useUnifiedApi';

// Mock the useUnifiedApi hook
const mockQuery = vi.fn();
const mockCommand = vi.fn();
const mockSubscribe = vi.fn();
const mockUnsubscribe = vi.fn();

vi.mock('@/hooks/useUnifiedApi', () => ({
  useUnifiedApi: vi.fn(),
}));

// Mock MonacoEditor component
vi.mock('@/components/MonacoEditor', () => ({
  MonacoEditor: ({
    value,
    onChange,
    onValidate,
  }: {
    value: string;
    onChange: (value: string) => void;
    onValidate: (errors: any[]) => void;
  }) => (
    <div data-testid="monaco-editor">
      <textarea
        data-testid="monaco-textarea"
        value={value}
        onChange={(e) => onChange(e.target.value)}
      />
      <button
        data-testid="trigger-validation"
        onClick={() => onValidate([])}
      >
        Trigger Validation
      </button>
      <button
        data-testid="trigger-validation-error"
        onClick={() =>
          onValidate([
            {
              line: 5,
              column: 10,
              length: 5,
              message: 'Syntax error: unexpected token',
            },
          ])
        }
      >
        Trigger Validation Error
      </button>
    </div>
  ),
}));

// Mock KeyboardVisualizer component
vi.mock('@/components/KeyboardVisualizer', () => ({
  KeyboardVisualizer: () => <div data-testid="keyboard-visualizer">Keyboard Visualizer</div>,
}));

describe('ConfigPage - Tab Switching and Save Functionality', () => {
  beforeEach(() => {
    // Reset all mocks before each test
    vi.clearAllMocks();

    // Setup default mock implementation
    vi.mocked(useUnifiedApiModule.useUnifiedApi).mockReturnValue({
      query: mockQuery,
      command: mockCommand,
      subscribe: mockSubscribe,
      unsubscribe: mockUnsubscribe,
      isConnected: true,
      readyState: 1,
    });

    // Default successful config fetch
    mockQuery.mockResolvedValue({
      code: '// Default configuration\nlet base = Layer::new("base");',
      hash: 'abc123',
    });

    // Default successful config update
    mockCommand.mockResolvedValue({ success: true });
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe('Tab Rendering and Switching', () => {
    it('renders both tab buttons', async () => {
      renderWithProviders(<ConfigPage />, { wrapWithRouter: true });

      // Wait for initial load
      await waitFor(() => {
        expect(screen.queryByText(/Loading configuration/)).not.toBeInTheDocument();
      });

      const visualTab = screen.getByRole('tab', { name: /visual editor/i });
      const codeTab = screen.getByRole('tab', { name: /code editor/i });

      expect(visualTab).toBeInTheDocument();
      expect(codeTab).toBeInTheDocument();
    });

    it('visual tab is active by default (AC1)', async () => {
      renderWithProviders(<ConfigPage />, { wrapWithRouter: true });

      await waitFor(() => {
        expect(screen.queryByText(/Loading configuration/)).not.toBeInTheDocument();
      });

      const visualTab = screen.getByRole('tab', { name: /visual editor/i });

      expect(visualTab).toHaveAttribute('aria-selected', 'true');
      expect(visualTab).toHaveClass('bg-primary-500');
    });

    it('clicking Code tab renders Monaco editor (AC2)', async () => {
      const user = userEvent.setup();
      renderWithProviders(<ConfigPage />, { wrapWithRouter: true });

      await waitFor(() => {
        expect(screen.queryByText(/Loading configuration/)).not.toBeInTheDocument();
      });

      const codeTab = screen.getByRole('tab', { name: /code editor/i });
      await user.click(codeTab);

      expect(screen.getByTestId('monaco-editor')).toBeInTheDocument();
      expect(screen.queryByTestId('keyboard-visualizer')).not.toBeInTheDocument();
    });

    it('clicking Visual tab renders KeyboardVisualizer (AC3)', async () => {
      const user = userEvent.setup();
      renderWithProviders(<ConfigPage />, { wrapWithRouter: true });

      await waitFor(() => {
        expect(screen.queryByText(/Loading configuration/)).not.toBeInTheDocument();
      });

      // First switch to code tab
      const codeTab = screen.getByRole('tab', { name: /code editor/i });
      await user.click(codeTab);

      // Then switch back to visual tab
      const visualTab = screen.getByRole('tab', { name: /visual editor/i });
      await user.click(visualTab);

      expect(screen.getByTestId('keyboard-visualizer')).toBeInTheDocument();
      expect(screen.queryByTestId('monaco-editor')).not.toBeInTheDocument();
    });

    it('active tab has bg-primary-500 class (AC4)', async () => {
      const user = userEvent.setup();
      renderWithProviders(<ConfigPage />, { wrapWithRouter: true });

      await waitFor(() => {
        expect(screen.queryByText(/Loading configuration/)).not.toBeInTheDocument();
      });

      const visualTab = screen.getByRole('tab', { name: /visual editor/i });
      const codeTab = screen.getByRole('tab', { name: /code editor/i });

      // Visual tab is active initially
      expect(visualTab).toHaveClass('bg-primary-500');
      expect(codeTab).not.toHaveClass('bg-primary-500');
      expect(codeTab).toHaveClass('bg-slate-700');

      // Switch to code tab
      await user.click(codeTab);

      expect(codeTab).toHaveClass('bg-primary-500');
      expect(visualTab).not.toHaveClass('bg-primary-500');
      expect(visualTab).toHaveClass('bg-slate-700');
    });
  });

  describe('State Persistence Across Tabs', () => {
    it('typing in Code editor then switching to Visual preserves changes (AC5)', async () => {
      const user = userEvent.setup();
      renderWithProviders(<ConfigPage />, { wrapWithRouter: true });

      await waitFor(() => {
        expect(screen.queryByText(/Loading configuration/)).not.toBeInTheDocument();
      });

      // Switch to code tab
      const codeTab = screen.getByRole('tab', { name: /code editor/i });
      await user.click(codeTab);

      // Type in the editor
      const textarea = screen.getByTestId('monaco-textarea');
      await user.clear(textarea);
      await user.type(textarea, 'let modified = true;');

      // Switch back to visual tab
      const visualTab = screen.getByRole('tab', { name: /visual editor/i });
      await user.click(visualTab);

      // Switch to code tab again
      await user.click(codeTab);

      // Verify the text is still there
      expect(textarea).toHaveValue('let modified = true;');
    });

    it('switching tabs does not lose unsaved changes (AC10)', async () => {
      const user = userEvent.setup();
      renderWithProviders(<ConfigPage />, { wrapWithRouter: true });

      await waitFor(() => {
        expect(screen.queryByText(/Loading configuration/)).not.toBeInTheDocument();
      });

      // Switch to code tab
      const codeTab = screen.getByRole('tab', { name: /code editor/i });
      await user.click(codeTab);

      // Modify the code
      const textarea = screen.getByTestId('monaco-textarea');
      const newCode = '// Modified code\nlet new_layer = Layer::new("test");';
      await user.clear(textarea);
      await user.type(textarea, newCode);

      // Switch tabs multiple times
      const visualTab = screen.getByRole('tab', { name: /visual editor/i });
      await user.click(visualTab);
      await user.click(codeTab);
      await user.click(visualTab);
      await user.click(codeTab);

      // Verify code is still there
      expect(textarea).toHaveValue(newCode);
    });
  });

  describe('Validation Integration', () => {
    it('validation status panel appears in both tabs (AC6)', async () => {
      const user = userEvent.setup();
      renderWithProviders(<ConfigPage />, { wrapWithRouter: true });

      await waitFor(() => {
        expect(screen.queryByText(/Loading configuration/)).not.toBeInTheDocument();
      });

      // Switch to code tab
      const codeTab = screen.getByRole('tab', { name: /code editor/i });
      await user.click(codeTab);

      // Trigger validation error
      const errorButton = screen.getByTestId('trigger-validation-error');
      await user.click(errorButton);

      // Check error appears in code tab
      await waitFor(() => {
        expect(screen.getByText(/1 validation error/)).toBeInTheDocument();
      });

      // Switch to visual tab
      const visualTab = screen.getByRole('tab', { name: /visual editor/i });
      await user.click(visualTab);

      // Error should still be visible
      expect(screen.getByText(/1 validation error/)).toBeInTheDocument();
    });

    it('validation errors disable save button (AC9)', async () => {
      const user = userEvent.setup();
      renderWithProviders(<ConfigPage />, { wrapWithRouter: true });

      await waitFor(() => {
        expect(screen.queryByText(/Loading configuration/)).not.toBeInTheDocument();
      });

      const saveButton = screen.getByRole('button', { name: 'Save configuration' });
      expect(saveButton).not.toBeDisabled();

      // Switch to code tab
      const codeTab = screen.getByRole('tab', { name: /code editor/i });
      await user.click(codeTab);

      // Trigger validation error
      const errorButton = screen.getByTestId('trigger-validation-error');
      await user.click(errorButton);

      await waitFor(() => {
        expect(saveButton).toBeDisabled();
        expect(saveButton).toHaveClass('cursor-not-allowed');
      });
    });

    it('clearing validation errors enables save button', async () => {
      const user = userEvent.setup();
      renderWithProviders(<ConfigPage />, { wrapWithRouter: true });

      await waitFor(() => {
        expect(screen.queryByText(/Loading configuration/)).not.toBeInTheDocument();
      });

      const codeTab = screen.getByRole('tab', { name: /code editor/i });
      await user.click(codeTab);

      // First trigger an error
      const errorButton = screen.getByTestId('trigger-validation-error');
      await user.click(errorButton);

      const saveButton = screen.getByRole('button', { name: 'Save configuration' });
      await waitFor(() => {
        expect(saveButton).toBeDisabled();
      });

      // Then clear it
      const validButton = screen.getByTestId('trigger-validation');
      await user.click(validButton);

      await waitFor(() => {
        expect(saveButton).not.toBeDisabled();
      });
    });
  });

  describe('Save Functionality', () => {
    it('save button calls updateConfig RPC method (AC7)', async () => {
      const user = userEvent.setup();
      renderWithProviders(<ConfigPage />, { wrapWithRouter: true });

      await waitFor(() => {
        expect(screen.queryByText(/Loading configuration/)).not.toBeInTheDocument();
      });

      // Switch to code tab and modify
      const codeTab = screen.getByRole('tab', { name: /code editor/i });
      await user.click(codeTab);

      const textarea = screen.getByTestId('monaco-textarea');
      await user.clear(textarea);
      await user.type(textarea, 'let save_test = true;');

      // Click save button
      const saveButton = screen.getByRole('button', { name: 'Save configuration' });
      await user.click(saveButton);

      await waitFor(() => {
        expect(mockCommand).toHaveBeenCalledWith('update_config', {
          code: 'let save_test = true;',
        });
      });
    });

    it('Ctrl+S keyboard event triggers save (AC8)', async () => {
      const user = userEvent.setup();
      renderWithProviders(<ConfigPage />, { wrapWithRouter: true });

      await waitFor(() => {
        expect(screen.queryByText(/Loading configuration/)).not.toBeInTheDocument();
      });

      // Switch to code tab
      const codeTab = screen.getByRole('tab', { name: /code editor/i });
      await user.click(codeTab);

      // Press Ctrl+S
      await user.keyboard('{Control>}s{/Control}');

      await waitFor(() => {
        expect(mockCommand).toHaveBeenCalled();
      });
    });

    it('save works from Visual tab', async () => {
      const user = userEvent.setup();
      renderWithProviders(<ConfigPage />, { wrapWithRouter: true });

      await waitFor(() => {
        expect(screen.queryByText(/Loading configuration/)).not.toBeInTheDocument();
      });

      // Stay on visual tab
      const saveButton = screen.getByRole('button', { name: 'Save configuration' });
      await user.click(saveButton);

      await waitFor(() => {
        expect(mockCommand).toHaveBeenCalled();
      });
    });

    it('save works from Code tab', async () => {
      const user = userEvent.setup();
      renderWithProviders(<ConfigPage />, { wrapWithRouter: true });

      await waitFor(() => {
        expect(screen.queryByText(/Loading configuration/)).not.toBeInTheDocument();
      });

      // Switch to code tab
      const codeTab = screen.getByRole('tab', { name: /code editor/i });
      await user.click(codeTab);

      const saveButton = screen.getByRole('button', { name: 'Save configuration' });
      await user.click(saveButton);

      await waitFor(() => {
        expect(mockCommand).toHaveBeenCalled();
      });
    });

    it('shows success message after successful save', async () => {
      const user = userEvent.setup();
      renderWithProviders(<ConfigPage />, { wrapWithRouter: true });

      await waitFor(() => {
        expect(screen.queryByText(/Loading configuration/)).not.toBeInTheDocument();
      });

      const saveButton = screen.getByRole('button', { name: 'Save configuration' });
      await user.click(saveButton);

      await waitFor(() => {
        expect(screen.getByText(/saved successfully/i)).toBeInTheDocument();
      });
    });

    it('shows error message when save fails', async () => {
      mockCommand.mockRejectedValueOnce(new Error('Network error'));

      const user = userEvent.setup();
      renderWithProviders(<ConfigPage />, { wrapWithRouter: true });

      await waitFor(() => {
        expect(screen.queryByText(/Loading configuration/)).not.toBeInTheDocument();
      });

      const saveButton = screen.getByRole('button', { name: 'Save configuration' });
      await user.click(saveButton);

      await waitFor(() => {
        expect(screen.getByText(/network error/i)).toBeInTheDocument();
      });
    });

    it('shows error message when trying to save with validation errors', async () => {
      const user = userEvent.setup();
      renderWithProviders(<ConfigPage />, { wrapWithRouter: true });

      await waitFor(() => {
        expect(screen.queryByText(/Loading configuration/)).not.toBeInTheDocument();
      });

      // Switch to code tab
      const codeTab = screen.getByRole('tab', { name: /code editor/i });
      await user.click(codeTab);

      // Trigger validation error
      const errorButton = screen.getByTestId('trigger-validation-error');
      await user.click(errorButton);

      // Try to save via Ctrl+S (button is disabled but keyboard shortcut still works)
      await user.keyboard('{Control>}s{/Control}');

      await waitFor(() => {
        expect(screen.getByText(/cannot save.*validation errors/i)).toBeInTheDocument();
      });

      // Verify RPC was not called
      expect(mockCommand).not.toHaveBeenCalled();
    });

    it('save button shows correct states (idle, saving, success, error)', async () => {
      const user = userEvent.setup();
      renderWithProviders(<ConfigPage />, { wrapWithRouter: true });

      await waitFor(() => {
        expect(screen.queryByText(/Loading configuration/)).not.toBeInTheDocument();
      });

      const saveButton = screen.getByRole('button', { name: 'Save configuration' });

      // Initial state
      expect(saveButton).toHaveTextContent(/save.*ctrl\+s/i);

      // Click save
      mockCommand.mockImplementation(
        () => new Promise((resolve) => setTimeout(() => resolve({ success: true }), 100))
      );
      await user.click(saveButton);

      // Saving state
      await waitFor(() => {
        expect(saveButton).toHaveTextContent(/saving/i);
      });

      // Success state
      await waitFor(() => {
        expect(saveButton).toHaveTextContent(/saved/i);
      });

      // Back to idle after timeout
      await waitFor(
        () => {
          expect(saveButton).toHaveTextContent(/save.*ctrl\+s/i);
        },
        { timeout: 3000 }
      );
    });
  });

  describe('Loading State', () => {
    it('shows loading skeleton when not connected', () => {
      vi.mocked(useUnifiedApiModule.useUnifiedApi).mockReturnValue({
        query: mockQuery,
        command: mockCommand,
        subscribe: mockSubscribe,
        unsubscribe: mockUnsubscribe,
        isConnected: false,
        readyState: 0,
      });

      renderWithProviders(<ConfigPage />, { wrapWithRouter: true });

      // Should show loading state
      expect(screen.queryByText('Configuration Editor')).not.toBeInTheDocument();
    });

    it('loads configuration on mount when connected', async () => {
      renderWithProviders(<ConfigPage />, { wrapWithRouter: true });

      await waitFor(() => {
        expect(mockQuery).toHaveBeenCalledWith('get_config');
      });
    });

    it('displays loaded configuration in code editor', async () => {
      const user = userEvent.setup();
      mockQuery.mockResolvedValue({
        code: '// Loaded from server\nlet layer = Layer::new("base");',
        hash: 'xyz789',
      });

      renderWithProviders(<ConfigPage />, { wrapWithRouter: true });

      await waitFor(() => {
        expect(screen.queryByText(/Loading configuration/)).not.toBeInTheDocument();
      });

      // Switch to code tab
      const codeTab = screen.getByRole('tab', { name: /code editor/i });
      await user.click(codeTab);

      const textarea = screen.getByTestId('monaco-textarea');
      expect(textarea).toHaveValue('// Loaded from server\nlet layer = Layer::new("base");');
    });
  });

  describe('Accessibility', () => {
    it('tab buttons have proper ARIA attributes', async () => {
      renderWithProviders(<ConfigPage />, { wrapWithRouter: true });

      await waitFor(() => {
        expect(screen.queryByText(/Loading configuration/)).not.toBeInTheDocument();
      });

      const visualTab = screen.getByRole('tab', { name: /visual editor/i });
      const codeTab = screen.getByRole('tab', { name: /code editor/i });

      expect(visualTab).toHaveAttribute('aria-selected', 'true');
      expect(visualTab).toHaveAttribute('aria-controls', 'visual-panel');

      expect(codeTab).toHaveAttribute('aria-selected', 'false');
      expect(codeTab).toHaveAttribute('aria-controls', 'code-panel');
    });

    it('tabpanels have proper ARIA attributes', async () => {
      const user = userEvent.setup();
      renderWithProviders(<ConfigPage />, { wrapWithRouter: true });

      await waitFor(() => {
        expect(screen.queryByText(/Loading configuration/)).not.toBeInTheDocument();
      });

      // Visual panel should be visible
      const visualPanel = screen.getByRole('tabpanel', { name: /visual/i });
      expect(visualPanel).toHaveAttribute('id', 'visual-panel');

      // Switch to code tab
      const codeTab = screen.getByRole('tab', { name: /code editor/i });
      await user.click(codeTab);

      // Code panel should be visible
      const codePanel = screen.getByRole('tabpanel', { name: /code/i });
      expect(codePanel).toHaveAttribute('id', 'code-panel');
    });

    it('save button has accessible label', async () => {
      renderWithProviders(<ConfigPage />, { wrapWithRouter: true });

      await waitFor(() => {
        expect(screen.queryByText(/Loading configuration/)).not.toBeInTheDocument();
      });

      const saveButton = screen.getByRole('button', { name: 'Save configuration' });
      expect(saveButton).toHaveAttribute('aria-label', 'Save configuration');
    });
  });
});
