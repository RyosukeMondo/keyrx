/**
 * Component tests for ConfigEditor.
 *
 * Tests cover:
 * - Monaco editor rendering
 * - Validation triggers on typing (debounced)
 * - Save button disabled when errors exist
 * - Keyboard shortcuts (F8 jump to error)
 * - Editor integration with useConfigValidator hook
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, waitFor, fireEvent, act } from '@testing-library/react';
import { ConfigEditor } from './ConfigEditor';
import type { ValidationResult } from '@/types/validation';

// Mock Monaco editor
let mockEditorInstance: any;

const createMockEditorInstance = () => ({
  getValue: vi.fn(),
  setValue: vi.fn(),
  getModel: vi.fn(),
  getPosition: vi.fn(),
  setPosition: vi.fn(),
  revealLineInCenter: vi.fn(),
  focus: vi.fn(),
  addAction: vi.fn(() => ({ dispose: vi.fn() })),
  dispose: vi.fn(),
});

vi.mock('@monaco-editor/react', () => ({
  default: ({ onChange, onMount, value }: any) => {
    // Store onChange callback for testing
    if (onChange) {
      (global as any).__mockMonacoOnChange = onChange;
    }
    // Call onMount when component mounts
    if (onMount) {
      setTimeout(() => onMount(mockEditorInstance), 0);
    }
    return (
      <div className="monaco-editor" data-testid="monaco-editor">
        <textarea
          data-testid="monaco-editor-content"
          value={value}
          onChange={(e) => onChange?.(e.target.value)}
        />
      </div>
    );
  },
  OnMount: {} as any,
}));

// Mock the useConfigValidator hook
vi.mock('@/hooks/useConfigValidator', () => ({
  useConfigValidator: vi.fn(),
}));

// Mock Monaco utilities
vi.mock('@/utils/monacoConfig', () => ({
  registerRhaiLanguage: vi.fn(),
}));

vi.mock('@/utils/monacoMarkers', () => ({
  updateEditorMarkers: vi.fn(),
}));

vi.mock('@/utils/monacoQuickFix', () => ({
  registerQuickFixProvider: vi.fn(() => ({ dispose: vi.fn() })),
  updateQuickFixContext: vi.fn(),
}));

// Mock monaco-editor module
vi.mock('monaco-editor', () => ({
  default: {},
  KeyCode: {
    F8: 66,
  },
  MarkerSeverity: {
    Error: 8,
    Warning: 4,
    Info: 2,
  },
  editor: {
    getModelMarkers: vi.fn(() => []),
  },
}));

import { useConfigValidator } from '@/hooks/useConfigValidator';
import { updateEditorMarkers } from '@/utils/monacoMarkers';
import { updateQuickFixContext } from '@/utils/monacoQuickFix';
import * as monaco from 'monaco-editor';

describe('ConfigEditor', () => {
  const mockOnSave = vi.fn();
  const mockOnValidationChange = vi.fn();
  const mockValidate = vi.fn();
  const mockClearValidation = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();

    // Create fresh mock editor instance for each test
    mockEditorInstance = createMockEditorInstance();

    // Default mock return values for useConfigValidator
    vi.mocked(useConfigValidator).mockReturnValue({
      validationResult: null,
      isValidating: false,
      wasmAvailable: true,
      validate: mockValidate,
      clearValidation: mockClearValidation,
    });

    // Default mock for getModelMarkers
    vi.mocked(monaco.editor.getModelMarkers).mockReturnValue([]);
  });

  afterEach(() => {
    vi.restoreAllMocks();
    delete (global as any).__mockMonacoOnChange;
  });

  describe('rendering', () => {
    it('should render Monaco editor container', () => {
      render(<ConfigEditor onSave={mockOnSave} />);

      expect(screen.getByTestId('monaco-editor')).toBeInTheDocument();
    });

    it('should render editor header with title', () => {
      render(<ConfigEditor onSave={mockOnSave} />);

      expect(screen.getByText('Configuration Editor')).toBeInTheDocument();
    });

    it('should render save button', () => {
      render(<ConfigEditor onSave={mockOnSave} />);

      expect(screen.getByRole('button', { name: /save configuration/i })).toBeInTheDocument();
    });

    it('should render status bar with F8 hint', () => {
      render(<ConfigEditor onSave={mockOnSave} />);

      expect(screen.getByText(/Press F8 to jump to next error/i)).toBeInTheDocument();
    });

    it('should display initial value in editor', () => {
      const initialValue = 'layer "test" {\n  map KEY_A to KEY_B\n}';
      render(<ConfigEditor initialValue={initialValue} onSave={mockOnSave} />);

      const textarea = screen.getByTestId('monaco-editor-content');
      expect(textarea).toHaveValue(initialValue);
    });
  });

  describe('validation integration', () => {
    beforeEach(() => {
      vi.useFakeTimers();
    });

    afterEach(() => {
      vi.useRealTimers();
    });

    it('should trigger validation when content changes', async () => {
      render(<ConfigEditor onSave={mockOnSave} />);

      const textarea = screen.getByTestId('monaco-editor-content');

      await act(async () => {
        fireEvent.change(textarea, { target: { value: 'new config' } });
      });

      expect(mockValidate).toHaveBeenCalledWith('new config');
    });

    it('should display validating status when isValidating is true', () => {
      vi.mocked(useConfigValidator).mockReturnValue({
        validationResult: null,
        isValidating: true,
        wasmAvailable: true,
        validate: mockValidate,
        clearValidation: mockClearValidation,
      });

      render(<ConfigEditor onSave={mockOnSave} />);

      expect(screen.getByText(/Validating.../)).toBeInTheDocument();
    });

    it('should display error count when errors exist', () => {
      const validationResult: ValidationResult = {
        errors: [
          { line: 1, column: 1, message: 'Error 1', code: 'ERR1' },
          { line: 2, column: 1, message: 'Error 2', code: 'ERR2' },
        ],
        warnings: [],
        hints: [],
        timestamp: new Date().toISOString(),
      };

      vi.mocked(useConfigValidator).mockReturnValue({
        validationResult,
        isValidating: false,
        wasmAvailable: true,
        validate: mockValidate,
        clearValidation: mockClearValidation,
      });

      render(<ConfigEditor onSave={mockOnSave} />);

      expect(screen.getByText(/2 errors/)).toBeInTheDocument();
    });

    it('should display warning count when warnings exist', () => {
      const validationResult: ValidationResult = {
        errors: [],
        warnings: [
          { line: 1, column: 1, message: 'Warning 1', code: 'WARN1' },
        ],
        hints: [],
        timestamp: new Date().toISOString(),
      };

      vi.mocked(useConfigValidator).mockReturnValue({
        validationResult,
        isValidating: false,
        wasmAvailable: true,
        validate: mockValidate,
        clearValidation: mockClearValidation,
      });

      render(<ConfigEditor onSave={mockOnSave} />);

      expect(screen.getByText(/1 warning/)).toBeInTheDocument();
    });

    it('should display success message when no errors or warnings', () => {
      const validationResult: ValidationResult = {
        errors: [],
        warnings: [],
        hints: [],
        timestamp: new Date().toISOString(),
      };

      vi.mocked(useConfigValidator).mockReturnValue({
        validationResult,
        isValidating: false,
        wasmAvailable: true,
        validate: mockValidate,
        clearValidation: mockClearValidation,
      });

      render(<ConfigEditor onSave={mockOnSave} />);

      expect(screen.getByText(/No issues found/)).toBeInTheDocument();
    });

    it('should display WASM unavailable message when wasmAvailable is false', () => {
      vi.mocked(useConfigValidator).mockReturnValue({
        validationResult: null,
        isValidating: false,
        wasmAvailable: false,
        validate: mockValidate,
        clearValidation: mockClearValidation,
      });

      render(<ConfigEditor onSave={mockOnSave} />);

      expect(screen.getByText(/Validation unavailable/)).toBeInTheDocument();
    });

    it('should call onValidationChange when validation result updates', () => {
      const validationResult: ValidationResult = {
        errors: [],
        warnings: [],
        hints: [],
        timestamp: new Date().toISOString(),
      };

      vi.mocked(useConfigValidator).mockReturnValue({
        validationResult,
        isValidating: false,
        wasmAvailable: true,
        validate: mockValidate,
        clearValidation: mockClearValidation,
      });

      render(
        <ConfigEditor
          onSave={mockOnSave}
          onValidationChange={mockOnValidationChange}
        />
      );

      expect(mockOnValidationChange).toHaveBeenCalledWith(validationResult);
    });
  });

  describe('save functionality', () => {
    it('should call onSave with current content when save button clicked', async () => {
      const validationResult: ValidationResult = {
        errors: [],
        warnings: [],
        hints: [],
        timestamp: new Date().toISOString(),
      };

      vi.mocked(useConfigValidator).mockReturnValue({
        validationResult,
        isValidating: false,
        wasmAvailable: true,
        validate: mockValidate,
        clearValidation: mockClearValidation,
      });

      mockOnSave.mockResolvedValue(undefined);

      render(<ConfigEditor initialValue="test config" onSave={mockOnSave} />);

      const saveButton = screen.getByRole('button', { name: /save configuration/i });

      await act(async () => {
        fireEvent.click(saveButton);
      });

      expect(mockOnSave).toHaveBeenCalledWith('test config');
    });

    it('should disable save button when errors exist', () => {
      const validationResult: ValidationResult = {
        errors: [{ line: 1, column: 1, message: 'Error', code: 'ERR' }],
        warnings: [],
        hints: [],
        timestamp: new Date().toISOString(),
      };

      vi.mocked(useConfigValidator).mockReturnValue({
        validationResult,
        isValidating: false,
        wasmAvailable: true,
        validate: mockValidate,
        clearValidation: mockClearValidation,
      });

      render(<ConfigEditor onSave={mockOnSave} />);

      const saveButton = screen.getByRole('button', { name: /save configuration/i });
      expect(saveButton).toBeDisabled();
      expect(saveButton).toHaveAttribute('title', 'Fix all errors before saving');
    });

    it('should disable save button when WASM unavailable', () => {
      vi.mocked(useConfigValidator).mockReturnValue({
        validationResult: null,
        isValidating: false,
        wasmAvailable: false,
        validate: mockValidate,
        clearValidation: mockClearValidation,
      });

      render(<ConfigEditor onSave={mockOnSave} />);

      const saveButton = screen.getByRole('button', { name: /save configuration/i });
      expect(saveButton).toBeDisabled();
      expect(saveButton).toHaveAttribute('title', 'WASM module unavailable');
    });

    it('should disable save button when saving', async () => {
      const validationResult: ValidationResult = {
        errors: [],
        warnings: [],
        hints: [],
        timestamp: new Date().toISOString(),
      };

      vi.mocked(useConfigValidator).mockReturnValue({
        validationResult,
        isValidating: false,
        wasmAvailable: true,
        validate: mockValidate,
        clearValidation: mockClearValidation,
      });

      let resolveSave: () => void;
      const savePromise = new Promise<void>((resolve) => {
        resolveSave = resolve;
      });
      mockOnSave.mockReturnValue(savePromise);

      render(<ConfigEditor onSave={mockOnSave} />);

      const saveButton = screen.getByRole('button', { name: /save configuration/i });

      await act(async () => {
        fireEvent.click(saveButton);
      });

      // Button should show "Saving..." and be disabled
      expect(screen.getByRole('button', { name: /saving.../i })).toBeDisabled();

      // Resolve save
      await act(async () => {
        resolveSave!();
      });

      // Button should return to normal
      await waitFor(() => {
        expect(screen.getByRole('button', { name: /save configuration/i })).not.toBeDisabled();
      });
    });

    it('should prevent save when errors exist', () => {
      const validationResult: ValidationResult = {
        errors: [{ line: 1, column: 1, message: 'Error', code: 'ERR' }],
        warnings: [],
        hints: [],
        timestamp: new Date().toISOString(),
      };

      vi.mocked(useConfigValidator).mockReturnValue({
        validationResult,
        isValidating: false,
        wasmAvailable: true,
        validate: mockValidate,
        clearValidation: mockClearValidation,
      });

      render(<ConfigEditor onSave={mockOnSave} />);

      const saveButton = screen.getByRole('button', { name: /save configuration/i });

      // Button should be disabled, preventing save
      expect(saveButton).toBeDisabled();
      expect(saveButton).toHaveAttribute('title', 'Fix all errors before saving');
    });

    it('should handle save errors gracefully', async () => {
      const validationResult: ValidationResult = {
        errors: [],
        warnings: [],
        hints: [],
        timestamp: new Date().toISOString(),
      };

      vi.mocked(useConfigValidator).mockReturnValue({
        validationResult,
        isValidating: false,
        wasmAvailable: true,
        validate: mockValidate,
        clearValidation: mockClearValidation,
      });

      const saveError = new Error('Network error');
      mockOnSave.mockRejectedValue(saveError);

      const alertSpy = vi.spyOn(window, 'alert').mockImplementation(() => {});
      const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

      render(<ConfigEditor onSave={mockOnSave} />);

      const saveButton = screen.getByRole('button', { name: /save configuration/i });

      await act(async () => {
        fireEvent.click(saveButton);
      });

      await waitFor(() => {
        expect(alertSpy).toHaveBeenCalledWith(
          expect.stringContaining('Failed to save configuration: Network error')
        );
        expect(consoleErrorSpy).toHaveBeenCalledWith('Save failed:', saveError);
      });

      alertSpy.mockRestore();
      consoleErrorSpy.mockRestore();
    });
  });

  describe('Monaco editor integration', () => {
    it('should register Rhai language on mount', async () => {
      const { registerRhaiLanguage } = await import('@/utils/monacoConfig');

      render(<ConfigEditor onSave={mockOnSave} />);

      await waitFor(() => {
        expect(registerRhaiLanguage).toHaveBeenCalled();
      });
    });

    it('should register Quick Fix provider on mount', async () => {
      const { registerQuickFixProvider } = await import('@/utils/monacoQuickFix');

      render(<ConfigEditor onSave={mockOnSave} />);

      await waitFor(() => {
        expect(registerQuickFixProvider).toHaveBeenCalled();
      });
    });

    it('should add F8 keyboard shortcut on mount', async () => {
      render(<ConfigEditor onSave={mockOnSave} />);

      await waitFor(() => {
        expect(mockEditorInstance.addAction).toHaveBeenCalledWith(
          expect.objectContaining({
            id: 'jump-to-next-error',
            label: 'Jump to Next Error',
            keybindings: [monaco.KeyCode.F8],
          })
        );
      });
    });

    it('should cleanup disposables on unmount', async () => {
      const mockDispose = vi.fn();
      mockEditorInstance.addAction.mockReturnValue({ dispose: mockDispose });

      const { unmount } = render(<ConfigEditor onSave={mockOnSave} />);

      await waitFor(() => {
        expect(mockEditorInstance.addAction).toHaveBeenCalled();
      });

      unmount();

      expect(mockDispose).toHaveBeenCalled();
    });
  });

  describe('F8 keyboard shortcut', () => {
    it('should jump to next error when F8 is pressed', async () => {
      const mockModel = {
        uri: { toString: () => 'test-uri' },
      };

      mockEditorInstance.getModel.mockReturnValue(mockModel);
      mockEditorInstance.getPosition.mockReturnValue({ lineNumber: 1, column: 1 });

      // Mock markers
      vi.mocked(monaco.editor.getModelMarkers).mockReturnValue([
        {
          startLineNumber: 5,
          startColumn: 10,
          severity: monaco.MarkerSeverity.Error,
        } as any,
        {
          startLineNumber: 10,
          startColumn: 5,
          severity: monaco.MarkerSeverity.Error,
        } as any,
      ]);

      render(<ConfigEditor onSave={mockOnSave} />);

      await waitFor(() => {
        expect(mockEditorInstance.addAction).toHaveBeenCalled();
      });

      // Get the F8 action callback
      const addActionCall = vi.mocked(mockEditorInstance.addAction).mock.calls[0];
      const action = addActionCall[0];

      // Execute F8 action
      action.run(mockEditorInstance);

      expect(mockEditorInstance.setPosition).toHaveBeenCalledWith({
        lineNumber: 5,
        column: 10,
      });
      expect(mockEditorInstance.revealLineInCenter).toHaveBeenCalledWith(5);
      expect(mockEditorInstance.focus).toHaveBeenCalled();
    });

    it('should wrap around to first error when no error after cursor', async () => {
      const mockModel = {
        uri: { toString: () => 'test-uri' },
      };

      mockEditorInstance.getModel.mockReturnValue(mockModel);
      mockEditorInstance.getPosition.mockReturnValue({ lineNumber: 15, column: 1 });

      // Mock markers (all before cursor position)
      vi.mocked(monaco.editor.getModelMarkers).mockReturnValue([
        {
          startLineNumber: 5,
          startColumn: 10,
          severity: monaco.MarkerSeverity.Error,
        } as any,
        {
          startLineNumber: 10,
          startColumn: 5,
          severity: monaco.MarkerSeverity.Error,
        } as any,
      ]);

      render(<ConfigEditor onSave={mockOnSave} />);

      await waitFor(() => {
        expect(mockEditorInstance.addAction).toHaveBeenCalled();
      });

      const addActionCall = vi.mocked(mockEditorInstance.addAction).mock.calls[0];
      const action = addActionCall[0];

      action.run(mockEditorInstance);

      // Should jump to first error (wrap around)
      expect(mockEditorInstance.setPosition).toHaveBeenCalledWith({
        lineNumber: 5,
        column: 10,
      });
    });

    it('should do nothing when no errors exist', async () => {
      const mockModel = {
        uri: { toString: () => 'test-uri' },
      };

      mockEditorInstance.getModel.mockReturnValue(mockModel);
      mockEditorInstance.getPosition.mockReturnValue({ lineNumber: 1, column: 1 });

      // No error markers
      vi.mocked(monaco.editor.getModelMarkers).mockReturnValue([]);

      render(<ConfigEditor onSave={mockOnSave} />);

      await waitFor(() => {
        expect(mockEditorInstance.addAction).toHaveBeenCalled();
      });

      const addActionCall = vi.mocked(mockEditorInstance.addAction).mock.calls[0];
      const action = addActionCall[0];

      action.run(mockEditorInstance);

      expect(mockEditorInstance.setPosition).not.toHaveBeenCalled();
    });
  });
});
