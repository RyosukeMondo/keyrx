/**
 * CodePreview component
 *
 * Displays generated Rhai configuration code in a read-only Monaco editor
 * with syntax highlighting, validation, and a copy-to-clipboard button.
 */

import React, { useState, useEffect } from 'react';
import Editor from '@monaco-editor/react';
import { useConfigBuilderStore } from '../store/configBuilderStore';
import { generateRhaiCode } from '../utils/rhaiGenerator';
import { useConfigValidator } from '../hooks/useConfigValidator';
import { hasErrors, hasWarnings } from '../types/validation';
import './CodePreview.css';

export const CodePreview: React.FC = () => {
  const [copyStatus, setCopyStatus] = useState<'idle' | 'copied' | 'error'>('idle');
  const layers = useConfigBuilderStore((state) => state.layers);
  const modifiers = useConfigBuilderStore((state) => state.modifiers);
  const locks = useConfigBuilderStore((state) => state.locks);

  const config = { layers, modifiers, locks };

  const generatedCode = generateRhaiCode(config);

  const { validationResult, isValidating, wasmAvailable, validate } = useConfigValidator();

  // Trigger validation whenever generated code changes
  useEffect(() => {
    if (wasmAvailable && generatedCode) {
      validate(generatedCode);
    }
  }, [generatedCode, wasmAvailable, validate]);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(generatedCode);
      setCopyStatus('copied');
      setTimeout(() => setCopyStatus('idle'), 2000);
    } catch (error) {
      console.error('Failed to copy code:', error);
      setCopyStatus('error');
      setTimeout(() => setCopyStatus('idle'), 2000);
    }
  };

  // Determine validation status for UI
  const validationStatus = !wasmAvailable
    ? 'unavailable'
    : isValidating
    ? 'validating'
    : validationResult && hasErrors(validationResult)
    ? 'error'
    : validationResult && hasWarnings(validationResult)
    ? 'warning'
    : validationResult
    ? 'valid'
    : 'idle';

  return (
    <div className="code-preview">
      <div className="code-preview-header">
        <div className="header-left">
          <h3>Generated Rhai Configuration</h3>
          {validationStatus !== 'idle' && (
            <span className={`validation-badge ${validationStatus}`}>
              {validationStatus === 'unavailable' && '⚠ Validation Unavailable'}
              {validationStatus === 'validating' && '⟳ Validating...'}
              {validationStatus === 'error' && `✗ ${validationResult?.errors.length} Error${(validationResult?.errors.length ?? 0) > 1 ? 's' : ''}`}
              {validationStatus === 'warning' && `⚠ ${validationResult?.warnings.length} Warning${(validationResult?.warnings.length ?? 0) > 1 ? 's' : ''}`}
              {validationStatus === 'valid' && '✓ Valid'}
            </span>
          )}
        </div>
        <button
          className={`copy-button ${copyStatus}`}
          onClick={handleCopy}
          disabled={copyStatus !== 'idle'}
        >
          {copyStatus === 'idle' && 'Copy Code'}
          {copyStatus === 'copied' && '✓ Copied!'}
          {copyStatus === 'error' && '✗ Failed'}
        </button>
      </div>

      {/* Validation messages panel */}
      {validationResult && (validationResult.errors.length > 0 || validationResult.warnings.length > 0) && (
        <div className="validation-messages">
          {validationResult.errors.map((error, idx) => (
            <div key={`error-${idx}`} className="validation-message error">
              <span className="message-icon">✗</span>
              <span className="message-location">Line {error.line}:{error.column}</span>
              <span className="message-text">{error.message}</span>
              {error.code && <span className="message-code">[{error.code}]</span>}
            </div>
          ))}
          {validationResult.warnings.map((warning, idx) => (
            <div key={`warning-${idx}`} className="validation-message warning">
              <span className="message-icon">⚠</span>
              <span className="message-location">Line {warning.line}:{warning.column}</span>
              <span className="message-text">{warning.message}</span>
              {warning.code && <span className="message-code">[{warning.code}]</span>}
            </div>
          ))}
        </div>
      )}

      <div className="code-preview-editor">
        <Editor
          height="100%"
          defaultLanguage="javascript"
          value={generatedCode}
          theme="vs-dark"
          options={{
            readOnly: true,
            minimap: { enabled: false },
            scrollBeyondLastLine: false,
            fontSize: 14,
            lineNumbers: 'on',
            folding: true,
            automaticLayout: true,
          }}
        />
      </div>
    </div>
  );
};
