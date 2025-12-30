/**
 * VisualBuilderPage - Main page for the visual configuration builder.
 *
 * Provides a drag-and-drop interface for building keyboard configurations
 * with import/export functionality for Rhai files.
 */

import { useRef } from 'react';
import { VirtualKeyboard } from './VirtualKeyboard';
import { LayerPanel } from './LayerPanel';
import { ModifierPanel } from './ModifierPanel';
import { CodePreview } from './CodePreview';
import { ErrorToast } from './ErrorToast';
import { useConfigBuilderStore } from '@/store/configBuilderStore';
import { parseRhaiConfig } from '@/utils/rhaiParser';
import { generateRhaiConfig } from '@/utils/rhaiGenerator';
import './VisualBuilderPage.css';

/**
 * VisualBuilderPage component.
 *
 * Features:
 * - Drag-and-drop visual keyboard configuration
 * - Layer management panel
 * - Modifier/lock management
 * - Live code preview
 * - Import existing Rhai configs
 * - Export to Rhai file
 */
export function VisualBuilderPage() {
  const fileInputRef = useRef<HTMLInputElement>(null);
  const { config, setConfig, setError } = useConfigBuilderStore();

  /**
   * Handle import Rhai file
   */
  const handleImport = () => {
    fileInputRef.current?.click();
  };

  /**
   * Handle file selection
   */
  const handleFileChange = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (!file) return;

    try {
      const text = await file.text();
      const parsedConfig = parseRhaiConfig(text);
      setConfig(parsedConfig);

      // Clear file input so the same file can be selected again
      if (fileInputRef.current) {
        fileInputRef.current.value = '';
      }
    } catch (error) {
      console.error('Failed to import configuration:', error);
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      setError(`Failed to import configuration: ${errorMessage}`);
    }
  };

  /**
   * Handle export to Rhai file
   */
  const handleExport = () => {
    try {
      const rhaiCode = generateRhaiConfig(config);
      const blob = new Blob([rhaiCode], { type: 'text/plain' });
      const url = URL.createObjectURL(blob);

      const a = document.createElement('a');
      a.href = url;
      a.download = 'keyboard-config.rhai';
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
    } catch (error) {
      console.error('Failed to export configuration:', error);
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      setError(`Failed to export configuration: ${errorMessage}`);
    }
  };

  /**
   * Handle reset to default config
   */
  const handleReset = () => {
    if (confirm('Are you sure you want to reset to default configuration? This will clear all your current settings.')) {
      setConfig({
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
    }
  };

  return (
    <div className="visual-builder-page">
      {/* Error Toast */}
      <ErrorToast />

      {/* Hidden file input for import */}
      <input
        ref={fileInputRef}
        type="file"
        accept=".rhai,.txt"
        style={{ display: 'none' }}
        onChange={handleFileChange}
      />

      {/* Page Header */}
      <div className="page-header">
        <div className="header-content">
          <h1>Visual Config Builder</h1>
          <p className="header-subtitle">
            Build your keyboard configuration visually with drag-and-drop
          </p>
        </div>
        <div className="header-actions">
          <button className="action-button secondary" onClick={handleImport}>
            Import .rhai
          </button>
          <button className="action-button secondary" onClick={handleExport}>
            Export .rhai
          </button>
          <button className="action-button danger" onClick={handleReset}>
            Reset
          </button>
        </div>
      </div>

      {/* Main Content Area */}
      <div className="page-content">
        {/* Left Panel: Keyboard and Layers */}
        <div className="left-panel">
          <div className="keyboard-section">
            <h2>Virtual Keyboard</h2>
            <VirtualKeyboard />
          </div>
          <div className="layers-section">
            <h2>Layers</h2>
            <LayerPanel />
          </div>
        </div>

        {/* Center Panel: Modifiers */}
        <div className="center-panel">
          <h2>Modifiers & Locks</h2>
          <ModifierPanel />
        </div>

        {/* Right Panel: Code Preview */}
        <div className="right-panel">
          <h2>Generated Code</h2>
          <CodePreview />
        </div>
      </div>

      {/* Help Text */}
      <div className="page-footer">
        <div className="help-text">
          <strong>How to use:</strong> Drag keys from the keyboard to layer mappings.
          Create custom modifiers and locks in the modifiers panel.
          Import existing .rhai files or export your configuration.
        </div>
      </div>
    </div>
  );
}

export default VisualBuilderPage;
