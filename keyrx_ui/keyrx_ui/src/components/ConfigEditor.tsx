/**
 * ConfigEditor - Monaco-based configuration editor with real-time validation.
 *
 * This component provides an IDE-like experience for editing Rhai configurations
 * with syntax highlighting, error detection, and quick fix suggestions.
 */

import { useState, useRef, useCallback, useEffect } from 'react';
import type { OnMount } from '@monaco-editor/react';
import Editor from '@monaco-editor/react';
import * as monaco from 'monaco-editor';
import { useConfigValidator } from '@/hooks/useConfigValidator';
import { registerRhaiLanguage } from '@/utils/monacoConfig';
import { updateEditorMarkers } from '@/utils/monacoMarkers';
import {
  registerQuickFixProvider,
  updateQuickFixContext,
} from '@/utils/monacoQuickFix';
import type { ValidationResult } from '@/types/validation';
import { hasErrors } from '@/types/validation';
import './ConfigEditor.css';

/**
 * Props for ConfigEditor component
 */
export interface ConfigEditorProps {
  /** Initial configuration content to display in editor */
  initialValue?: string;

  /** Callback invoked when user saves the configuration */
  onSave: (content: string) => Promise<void>;

  /** Callback invoked when validation result changes */
  onValidationChange?: (result: ValidationResult | null) => void;
}

/**
 * ConfigEditor component with Monaco editor integration and real-time validation.
 *
 * Features:
 * - Syntax highlighting for Rhai language
 * - Real-time validation with 500ms debounce
 * - Error/warning markers with quick fixes
 * - F8 keyboard shortcut to jump to next error
 * - Save blocked when validation errors exist
 */
export function ConfigEditor({
  initialValue = '',
  onSave,
  onValidationChange,
}: ConfigEditorProps) {
  // State management
  const [content, setContent] = useState(initialValue);
  const [isSaving, setIsSaving] = useState(false);
  const editorRef = useRef<monaco.editor.IStandaloneCodeEditor | null>(null);
  const disposablesRef = useRef<monaco.IDisposable[]>([]);

  // Validation hook
  const { validationResult, isValidating, wasmAvailable, validate } =
    useConfigValidator();

  /**
   * Handle editor mount - set up Rhai language and quick fix provider.
   */
  const handleEditorMount: OnMount = useCallback((editor) => {
    editorRef.current = editor;

    // Register Rhai language (idempotent)
    registerRhaiLanguage();

    // Register Quick Fix provider
    const quickFixDisposable = registerQuickFixProvider();
    disposablesRef.current.push(quickFixDisposable);

    // Add F8 keyboard shortcut for jumping to next error
    const jumpToErrorAction = editor.addAction({
      id: 'jump-to-next-error',
      label: 'Jump to Next Error',
      keybindings: [monaco.KeyCode.F8],
      run: (ed) => {
        const model = ed.getModel();
        if (!model) return;

        const markers = monaco.editor.getModelMarkers({
          resource: model.uri,
        }).filter(m => m.severity === monaco.MarkerSeverity.Error);

        if (markers.length === 0) return;

        const currentPosition = ed.getPosition();
        if (!currentPosition) return;

        // Find next error after current position
        let nextMarker = markers.find(
          (m) =>
            m.startLineNumber > currentPosition.lineNumber ||
            (m.startLineNumber === currentPosition.lineNumber &&
              m.startColumn > currentPosition.column)
        );

        // Wrap around to first error if no error found after cursor
        if (!nextMarker) {
          nextMarker = markers[0];
        }

        // Jump to error position
        ed.setPosition({
          lineNumber: nextMarker.startLineNumber,
          column: nextMarker.startColumn,
        });

        // Reveal line in center of viewport
        ed.revealLineInCenter(nextMarker.startLineNumber);

        // Focus editor
        ed.focus();
      },
    });
    disposablesRef.current.push(jumpToErrorAction);
  }, []);

  /**
   * Handle editor content change - trigger debounced validation.
   */
  const handleEditorChange = useCallback(
    (value: string | undefined) => {
      const newContent = value ?? '';
      setContent(newContent);
      validate(newContent);
    },
    [validate]
  );

  /**
   * Handle save button click.
   */
  const handleSave = useCallback(async () => {
    // Block save if validation errors exist
    if (hasErrors(validationResult)) {
      alert(
        'Cannot save configuration with errors. Please fix all errors before saving.'
      );
      return;
    }

    setIsSaving(true);

    try {
      await onSave(content);
    } catch (error) {
      console.error('Save failed:', error);
      alert(
        `Failed to save configuration: ${
          error instanceof Error ? error.message : 'Unknown error'
        }`
      );
    } finally {
      setIsSaving(false);
    }
  }, [content, validationResult, onSave]);

  /**
   * Update Monaco markers when validation result changes.
   */
  useEffect(() => {
    if (editorRef.current) {
      updateEditorMarkers(editorRef.current, validationResult);
      updateQuickFixContext(validationResult);
    }
  }, [validationResult]);

  /**
   * Notify parent of validation changes.
   */
  useEffect(() => {
    if (onValidationChange) {
      onValidationChange(validationResult);
    }
  }, [validationResult, onValidationChange]);

  /**
   * Cleanup disposables on unmount.
   */
  useEffect(() => {
    return () => {
      disposablesRef.current.forEach((d) => d.dispose());
      disposablesRef.current = [];
    };
  }, []);

  // Calculate error/warning counts for status bar
  const errorCount = validationResult?.errors.length ?? 0;
  const warningCount = validationResult?.warnings.length ?? 0;
  const hasValidationErrors = hasErrors(validationResult);

  return (
    <div className="config-editor">
      {/* Editor Header */}
      <div className="config-editor-header">
        <h3>Configuration Editor</h3>
        <div className="editor-actions">
          <button
            type="button"
            className="save-button"
            onClick={handleSave}
            disabled={isSaving || hasValidationErrors || !wasmAvailable}
            title={
              hasValidationErrors
                ? 'Fix all errors before saving'
                : !wasmAvailable
                ? 'WASM module unavailable'
                : 'Save configuration (Ctrl+S)'
            }
          >
            {isSaving ? 'Saving...' : 'Save Configuration'}
          </button>
        </div>
      </div>

      {/* Monaco Editor */}
      <div className="config-editor-container">
        <Editor
          height="600px"
          language="rhai"
          value={content}
          onChange={handleEditorChange}
          onMount={handleEditorMount}
          options={{
            minimap: { enabled: false },
            fontSize: 14,
            lineNumbers: 'on',
            scrollBeyondLastLine: false,
            automaticLayout: true,
            tabSize: 2,
            insertSpaces: true,
            wordWrap: 'on',
            quickSuggestions: false,
            folding: true,
            renderWhitespace: 'selection',
            glyphMargin: true,
            lightbulb: {
              enabled: true,
            },
          }}
          theme="vs-dark"
        />
      </div>

      {/* Status Bar */}
      <div className="config-editor-status">
        <div className="status-left">
          {isValidating && (
            <span className="status-item validating">
              <span className="spinner" />
              Validating...
            </span>
          )}
          {!isValidating && !wasmAvailable && (
            <span className="status-item error">
              ❌ Validation unavailable
            </span>
          )}
          {!isValidating && wasmAvailable && errorCount === 0 && warningCount === 0 && (
            <span className="status-item success">
              ✓ No issues found
            </span>
          )}
          {!isValidating && errorCount > 0 && (
            <span className="status-item error">
              ❌ {errorCount} {errorCount === 1 ? 'error' : 'errors'}
            </span>
          )}
          {!isValidating && warningCount > 0 && (
            <span className="status-item warning">
              ⚠️ {warningCount} {warningCount === 1 ? 'warning' : 'warnings'}
            </span>
          )}
        </div>
        <div className="status-right">
          <span className="status-item hint">
            💡 Press F8 to jump to next error
          </span>
        </div>
      </div>
    </div>
  );
}

export default ConfigEditor;
