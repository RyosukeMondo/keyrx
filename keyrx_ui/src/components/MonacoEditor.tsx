import { Editor, BeforeMount, OnMount } from '@monaco-editor/react';
import { useCallback, useEffect, useRef, useState } from 'react';
import type { editor } from 'monaco-editor';
import type { ValidationError } from '../hooks/useWasm';
import { useWasmContext } from '../contexts/WasmContext';

/**
 * Monaco Editor component for Rhai configuration editing
 *
 * Features:
 * - Rhai syntax highlighting with custom language definition
 * - F8 keybinding for next error navigation
 * - 500ms debounced validation
 * - Error markers with tooltips
 * - Dark theme optimized for Rhai
 */

interface MonacoEditorProps {
  value: string;
  onChange?: (value: string) => void;
  onValidate?: (errors: ValidationError[]) => void;
  readOnly?: boolean;
  height?: string;
}

export function MonacoEditor({
  value,
  onChange,
  onValidate,
  readOnly = false,
  height = '600px',
}: MonacoEditorProps) {
  const editorRef = useRef<editor.IStandaloneCodeEditor | null>(null);
  const monacoRef = useRef<typeof import('monaco-editor') | null>(null);
  const validationTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const [currentErrorIndex, setCurrentErrorIndex] = useState(0);
  const [errors, setErrors] = useState<ValidationError[]>([]);
  const [validationStatus, setValidationStatus] =
    useState<string>('Initializing...');

  const { isWasmReady, isLoading, validateConfig } = useWasmContext();

  // Update status when WASM loading state changes
  useEffect(() => {
    /* eslint-disable react-hooks/set-state-in-effect */
    if (isLoading) {
      setValidationStatus('⏳ Loading WASM validator...');
    } else if (isWasmReady) {
      setValidationStatus('✓ Ready');
    } else {
      setValidationStatus('⚠ WASM unavailable');
    }
    /* eslint-enable react-hooks/set-state-in-effect */
  }, [isLoading, isWasmReady]);

  /**
   * Register Rhai language with Monaco before editor mounts
   */
  const handleBeforeMount: BeforeMount = (monaco) => {
    monacoRef.current = monaco;

    // Register Rhai language
    monaco.languages.register({ id: 'rhai' });

    // Define Rhai syntax highlighting rules using Monarch tokenizer
    monaco.languages.setMonarchTokensProvider('rhai', {
      keywords: [
        'let',
        'const',
        'if',
        'else',
        'while',
        'for',
        'loop',
        'break',
        'continue',
        'return',
        'fn',
        'true',
        'false',
        'import',
        'export',
        'as',
        'private',
        'this',
        'in',
      ],
      operators: [
        '=',
        '>',
        '<',
        '!',
        '~',
        '?',
        ':',
        '==',
        '<=',
        '>=',
        '!=',
        '&&',
        '||',
        '++',
        '--',
        '+',
        '-',
        '*',
        '/',
        '&',
        '|',
        '^',
        '%',
        '<<',
        '>>',
        '>>>',
        '+=',
        '-=',
        '*=',
        '/=',
        '&=',
        '|=',
        '^=',
        '%=',
        '<<=',
        '>>=',
        '>>>=',
        '=>',
      ],
      symbols: /[=><!~?:&|+\-*/^%]+/,
      escapes:
        /\\(?:[abfnrtv\\"']|x[0-9A-Fa-f]{1,4}|u[0-9A-Fa-f]{4}|U[0-9A-Fa-f]{8})/,

      tokenizer: {
        root: [
          // Identifiers and keywords
          [
            /[a-zA-Z_]\w*/,
            {
              cases: {
                '@keywords': 'keyword',
                '@default': 'identifier',
              },
            },
          ],

          // Whitespace
          { include: '@whitespace' },

          // Delimiters and operators
          [/[{}()[\]]/, '@brackets'],
          [/[<>](?!@symbols)/, '@brackets'],
          [
            /@symbols/,
            {
              cases: {
                '@operators': 'operator',
                '@default': '',
              },
            },
          ],

          // Numbers
          [/\d*\.\d+([eE][-+]?\d+)?/, 'number.float'],
          [/0[xX][0-9a-fA-F]+/, 'number.hex'],
          [/\d+/, 'number'],

          // Strings
          [/"([^"\\]|\\.)*$/, 'string.invalid'],
          [/"/, 'string', '@string_double'],
          [/'([^'\\]|\\.)*$/, 'string.invalid'],
          [/'/, 'string', '@string_single'],
        ],

        whitespace: [
          [/[ \t\r\n]+/, ''],
          [/\/\*/, 'comment', '@comment'],
          [/\/\/.*$/, 'comment'],
        ],

        comment: [
          [/[^/*]+/, 'comment'],
          [/\*\//, 'comment', '@pop'],
          [/[/*]/, 'comment'],
        ],

        string_double: [
          [/[^\\"]+/, 'string'],
          [/@escapes/, 'string.escape'],
          [/\\./, 'string.escape.invalid'],
          [/"/, 'string', '@pop'],
        ],

        string_single: [
          [/[^\\']+/, 'string'],
          [/@escapes/, 'string.escape'],
          [/\\./, 'string.escape.invalid'],
          [/'/, 'string', '@pop'],
        ],
      },
    });

    // Define rhai-dark theme
    monaco.editor.defineTheme('rhai-dark', {
      base: 'vs-dark',
      inherit: true,
      rules: [
        { token: 'keyword', foreground: 'C586C0', fontStyle: 'bold' },
        { token: 'identifier', foreground: '9CDCFE' },
        { token: 'operator', foreground: 'D4D4D4' },
        { token: 'number', foreground: 'B5CEA8' },
        { token: 'number.float', foreground: 'B5CEA8' },
        { token: 'number.hex', foreground: 'B5CEA8' },
        { token: 'string', foreground: 'CE9178' },
        { token: 'comment', foreground: '6A9955', fontStyle: 'italic' },
      ],
      colors: {
        'editor.background': '#1e1e1e',
        'editor.foreground': '#d4d4d4',
        'editorLineNumber.foreground': '#858585',
        'editor.selectionBackground': '#264f78',
        'editor.inactiveSelectionBackground': '#3a3d41',
      },
    });
  };

  /**
   * Configure editor after mount
   */
  const handleMount: OnMount = (editor, monaco) => {
    editorRef.current = editor;
    monacoRef.current = monaco;

    // Configure editor options
    editor.updateOptions({
      minimap: { enabled: false },
      fontSize: 14,
      tabSize: 2,
      rulers: [80, 120],
      automaticLayout: true,
      scrollBeyondLastLine: false,
      wordWrap: 'on',
    });

    // Add F8 keybinding for next error navigation
    editor.addCommand(monaco.KeyCode.F8, () => {
      jumpToNextError();
    });
  };

  /**
   * Jump to next error and center the line
   */
  // eslint-disable-next-line react-hooks/preserve-manual-memoization
  const jumpToNextError = useCallback(() => {
    if (!editorRef.current || !monacoRef.current || errors.length === 0) {
      return;
    }

    const nextIndex = (currentErrorIndex + 1) % errors.length;
    setCurrentErrorIndex(nextIndex);

    const error = errors[nextIndex];
    const position = {
      lineNumber: error.line,
      column: error.column,
    };

    // Jump to error position and center the line
    editorRef.current.setPosition(position);
    editorRef.current.revealPositionInCenter(position);

    // Focus on the editor
    editorRef.current.focus();
  }, [errors, currentErrorIndex]);

  /**
   * Run validation with debouncing
   */
  const runValidation = useCallback(
    async (code: string) => {
      if (!isWasmReady) {
        setValidationStatus('WASM validation unavailable');
        return;
      }

      setValidationStatus('Validating...');

      try {
        const validationErrors = await validateConfig(code);
        setErrors(validationErrors);
        setCurrentErrorIndex(0);

        if (validationErrors.length === 0) {
          setValidationStatus('✓ No errors');
        } else {
          setValidationStatus(
            `✗ ${validationErrors.length} error${
              validationErrors.length > 1 ? 's' : ''
            }`
          );
        }

        // Call onValidate callback
        if (onValidate) {
          onValidate(validationErrors);
        }

        // Update error markers in editor
        if (editorRef.current && monacoRef.current) {
          const model = editorRef.current.getModel();
          if (model) {
            const markers = validationErrors.map((err) => ({
              severity: monacoRef.current!.MarkerSeverity.Error,
              startLineNumber: err.line,
              startColumn: err.column,
              endLineNumber: err.line,
              endColumn: err.column + err.length,
              message: err.message,
            }));

            monacoRef.current.editor.setModelMarkers(model, 'rhai', markers);
          }
        }
      } catch (err) {
        console.error('Validation failed:', err);
        setValidationStatus('Validation failed');
      }
    },
    [isWasmReady, validateConfig, onValidate]
  );

  /**
   * Handle editor value changes with debounced validation
   */
  const handleEditorChange = useCallback(
    (newValue: string | undefined) => {
      if (newValue === undefined) return;

      // Call onChange callback immediately
      if (onChange) {
        onChange(newValue);
      }

      // Clear previous timeout
      if (validationTimeoutRef.current) {
        clearTimeout(validationTimeoutRef.current);
      }

      // Set new timeout for validation (500ms debounce)
      validationTimeoutRef.current = setTimeout(() => {
        runValidation(newValue);
      }, 500);
    },
    [onChange, runValidation]
  );

  /**
   * Run validation when value prop changes (initial load)
   */
  useEffect(() => {
    if (value && isWasmReady) {
      // eslint-disable-next-line react-hooks/set-state-in-effect
      runValidation(value);
    }
  }, [value, isWasmReady, runValidation]);

  /**
   * Cleanup timeout on unmount
   */
  useEffect(() => {
    return () => {
      if (validationTimeoutRef.current) {
        clearTimeout(validationTimeoutRef.current);
      }
    };
  }, []);

  return (
    <div className="flex flex-col gap-2">
      <div className="flex items-center justify-between px-2 py-1 bg-slate-800 rounded text-sm">
        <span className="text-slate-400">Rhai Configuration Editor</span>
        <span
          className={`font-mono ${
            validationStatus.startsWith('✓')
              ? 'text-green-400'
              : validationStatus.startsWith('✗')
                ? 'text-red-400'
                : 'text-slate-400'
          }`}
        >
          {validationStatus}
        </span>
      </div>

      <Editor
        height={height}
        defaultLanguage="rhai"
        theme="rhai-dark"
        value={value}
        onChange={handleEditorChange}
        beforeMount={handleBeforeMount}
        onMount={handleMount}
        options={{
          readOnly,
        }}
      />

      {errors.length > 0 && (
        <div className="px-2 py-1 bg-slate-800 rounded text-xs text-slate-400">
          Press F8 to navigate to next error
        </div>
      )}
    </div>
  );
}
