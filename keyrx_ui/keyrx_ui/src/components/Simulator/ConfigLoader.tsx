/**
 * ConfigLoader - Component for loading Rhai configuration files.
 *
 * Supports both textarea input and file upload with validation and
 * error display including line numbers for parse errors.
 */

import { useState, useRef, useCallback } from 'react';
import type { ChangeEvent } from 'react';
import './ConfigLoader.css';

interface ConfigLoaderProps {
  onLoad: (rhaiSource: string) => Promise<void>;
  isLoading?: boolean;
  error?: string | null;
}

const MAX_FILE_SIZE = 1024 * 1024; // 1MB
const ALLOWED_EXTENSIONS = ['.rhai'];

/**
 * Extract line number from error message if present.
 * Common patterns: "line 42", "at line 42", "on line 42"
 */
function extractLineNumber(error: string): number | null {
  const lineMatch = error.match(/(?:line|at line|on line)\s+(\d+)/i);
  return lineMatch ? parseInt(lineMatch[1], 10) : null;
}

export function ConfigLoader({ onLoad, isLoading = false, error = null }: ConfigLoaderProps) {
  const [inputMode, setInputMode] = useState<'textarea' | 'file'>('textarea');
  const [textareaContent, setTextareaContent] = useState('');
  const [fileName, setFileName] = useState<string | null>(null);
  const [fileError, setFileError] = useState<string | null>(null);
  const [highlightLine, setHighlightLine] = useState<number | null>(null);

  const fileInputRef = useRef<HTMLInputElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  // Extract line number from error if present
  const errorLineNumber = error ? extractLineNumber(error) : null;
  const effectiveHighlightLine = errorLineNumber || highlightLine;

  /**
   * Handle textarea input change.
   */
  const handleTextareaChange = useCallback((e: ChangeEvent<HTMLTextAreaElement>) => {
    setTextareaContent(e.target.value);
    setHighlightLine(null);
    setFileError(null);
  }, []);

  /**
   * Handle file selection and validation.
   */
  const handleFileSelect = useCallback((e: ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    setFileError(null);
    setHighlightLine(null);

    if (!file) {
      setFileName(null);
      return;
    }

    // Validate file extension
    const fileExt = file.name.substring(file.name.lastIndexOf('.')).toLowerCase();
    if (!ALLOWED_EXTENSIONS.includes(fileExt)) {
      setFileError(`Invalid file type. Please select a .rhai file (got ${fileExt})`);
      setFileName(null);
      if (fileInputRef.current) {
        fileInputRef.current.value = '';
      }
      return;
    }

    // Validate file size
    if (file.size > MAX_FILE_SIZE) {
      const sizeMB = (file.size / (1024 * 1024)).toFixed(2);
      setFileError(`File too large: ${sizeMB}MB (maximum 1MB)`);
      setFileName(null);
      if (fileInputRef.current) {
        fileInputRef.current.value = '';
      }
      return;
    }

    // Read file content
    const reader = new FileReader();
    reader.onload = (evt) => {
      const content = evt.target?.result as string;
      setTextareaContent(content);
      setFileName(file.name);
    };
    reader.onerror = () => {
      setFileError('Failed to read file');
      setFileName(null);
    };
    reader.readAsText(file);
  }, []);

  /**
   * Handle load button click.
   */
  const handleLoadClick = useCallback(async () => {
    if (!textareaContent.trim()) {
      setFileError('Please enter or upload a configuration');
      return;
    }

    setFileError(null);
    setHighlightLine(null);

    try {
      await onLoad(textareaContent);
    } catch (err) {
      // Error is handled by parent component
      // Extract line number if present for highlighting
      if (err instanceof Error) {
        const lineNum = extractLineNumber(err.message);
        if (lineNum) {
          setHighlightLine(lineNum);
          // Scroll to error line in textarea
          if (textareaRef.current) {
            const lines = textareaContent.split('\n');
            const charPosition = lines.slice(0, lineNum - 1).join('\n').length + lineNum;
            textareaRef.current.focus();
            textareaRef.current.setSelectionRange(charPosition, charPosition);
          }
        }
      }
    }
  }, [textareaContent, onLoad]);

  /**
   * Handle mode switch between textarea and file upload.
   */
  const handleModeSwitch = useCallback((mode: 'textarea' | 'file') => {
    setInputMode(mode);
    setFileError(null);
    setHighlightLine(null);

    if (mode === 'file' && fileInputRef.current) {
      fileInputRef.current.click();
    }
  }, []);

  return (
    <div className="config-loader">
      {/* Input Mode Toggle */}
      <div className="input-mode-toggle">
        <button
          type="button"
          className={`mode-button ${inputMode === 'textarea' ? 'active' : ''}`}
          onClick={() => handleModeSwitch('textarea')}
          disabled={isLoading}
        >
          Paste Configuration
        </button>
        <button
          type="button"
          className={`mode-button ${inputMode === 'file' ? 'active' : ''}`}
          onClick={() => handleModeSwitch('file')}
          disabled={isLoading}
        >
          Upload File
        </button>
      </div>

      {/* File Upload (Hidden Input) */}
      <input
        ref={fileInputRef}
        type="file"
        accept=".rhai"
        onChange={handleFileSelect}
        style={{ display: 'none' }}
      />

      {/* File Name Display */}
      {fileName && (
        <div className="file-name-display">
          <span className="file-icon">📄</span>
          <span className="file-name">{fileName}</span>
        </div>
      )}

      {/* Textarea Input */}
      <div className="textarea-container">
        <textarea
          ref={textareaRef}
          className={`config-textarea ${effectiveHighlightLine ? 'has-error-line' : ''}`}
          value={textareaContent}
          onChange={handleTextareaChange}
          placeholder="Paste your Rhai configuration here or upload a .rhai file..."
          disabled={isLoading}
          rows={15}
          spellCheck={false}
        />

        {/* Line Number Overlay for Errors */}
        {effectiveHighlightLine && (
          <div className="error-line-indicator">
            Error on line {effectiveHighlightLine}
          </div>
        )}
      </div>

      {/* File Error Display */}
      {fileError && (
        <div className="inline-error">
          {fileError}
        </div>
      )}

      {/* Parse Error Display with Line Numbers */}
      {error && errorLineNumber && (
        <div className="parse-error">
          <strong>Parse Error (line {errorLineNumber}):</strong>
          <pre>{error}</pre>
        </div>
      )}
      {error && !errorLineNumber && (
        <div className="parse-error">
          <strong>Error:</strong>
          <pre>{error}</pre>
        </div>
      )}

      {/* Load Button */}
      <button
        type="button"
        className="load-button"
        onClick={handleLoadClick}
        disabled={isLoading || !textareaContent.trim()}
      >
        {isLoading ? 'Loading Configuration...' : 'Load Configuration'}
      </button>

      {/* Help Text */}
      <div className="help-text">
        <p>
          <strong>Tip:</strong> Your Rhai configuration will be compiled in the browser.
          Parse errors will be shown with line numbers to help you debug.
        </p>
      </div>
    </div>
  );
}

export default ConfigLoader;
