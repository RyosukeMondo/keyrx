/**
 * CodePreview component
 *
 * Displays generated Rhai configuration code in a read-only Monaco editor
 * with syntax highlighting and a copy-to-clipboard button.
 */

import React, { useState } from 'react';
import Editor from '@monaco-editor/react';
import { useConfigBuilderStore } from '../store/configBuilderStore';
import { generateRhaiCode } from '../utils/rhaiGenerator';
import './CodePreview.css';

export const CodePreview: React.FC = () => {
  const [copyStatus, setCopyStatus] = useState<'idle' | 'copied' | 'error'>('idle');
  const layers = useConfigBuilderStore((state) => state.layers);
  const modifiers = useConfigBuilderStore((state) => state.modifiers);
  const locks = useConfigBuilderStore((state) => state.locks);

  const config = { layers, modifiers, locks };

  const generatedCode = generateRhaiCode(config);

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

  return (
    <div className="code-preview">
      <div className="code-preview-header">
        <h3>Generated Rhai Configuration</h3>
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
