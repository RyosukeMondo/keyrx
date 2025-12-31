/**
 * ConfigurationPage - Main page for editing Rhai configuration files.
 *
 * This page integrates the ConfigEditor and ValidationStatusPanel components
 * to provide a comprehensive configuration editing experience.
 */

import { useState, useCallback } from 'react';
import { ConfigEditor } from './ConfigEditor';
import { ValidationStatusPanel } from './ValidationStatusPanel';
import type { ValidationResult, ValidationError, ValidationWarning } from '@/types/validation';
import './ConfigurationPage.css';

/**
 * Props for ConfigurationPage component
 */
export interface ConfigurationPageProps {
  /** Initial configuration content to load */
  initialConfig?: string;
}

/**
 * ConfigurationPage component.
 *
 * Features:
 * - Monaco-based code editor with syntax highlighting
 * - Real-time validation with error/warning display
 * - Validation status panel with jump-to-error functionality
 * - Save configuration with validation checks
 */
export function ConfigurationPage({
  initialConfig = '',
}: ConfigurationPageProps) {
  const [validationResult, setValidationResult] = useState<ValidationResult | null>(null);
  const [isValidating] = useState(false);
  const [saveStatus, setSaveStatus] = useState<'idle' | 'success' | 'error'>('idle');

  /**
   * Handle save configuration.
   * In a real app, this would POST to an API endpoint.
   */
  const handleSaveConfig = useCallback(async (content: string) => {
    try {
      // TODO: Replace with actual API call
      console.log('Saving configuration:', content);

      // Simulate API delay
      await new Promise((resolve) => setTimeout(resolve, 500));

      // For now, just save to localStorage as a demo
      localStorage.setItem('keyrx_config', content);

      setSaveStatus('success');
      setTimeout(() => setSaveStatus('idle'), 3000);
    } catch (error) {
      console.error('Failed to save configuration:', error);
      setSaveStatus('error');
      setTimeout(() => setSaveStatus('idle'), 3000);
      throw error;
    }
  }, []);

  /**
   * Handle validation result changes from the editor.
   */
  const handleValidationChange = useCallback((result: ValidationResult | null) => {
    setValidationResult(result);
  }, []);

  /**
   * Handle click on error in validation panel - jump to error in editor.
   */
  const handleErrorClick = useCallback((error: ValidationError) => {
    // The editor's F8 functionality handles jumping to errors
    // This could be enhanced to directly control the editor position
    console.log('Jump to error:', error);
  }, []);

  /**
   * Handle click on warning in validation panel.
   */
  const handleWarningClick = useCallback((warning: ValidationWarning) => {
    console.log('Jump to warning:', warning);
  }, []);

  return (
    <div className="configuration-page">
      {/* Page Header */}
      <div className="page-header">
        <div className="header-content">
          <h1>Configuration Editor</h1>
          <p className="header-subtitle">
            Edit your Rhai keyboard configuration with real-time validation
          </p>
        </div>
        {saveStatus === 'success' && (
          <div className="save-notification success">
            ✓ Configuration saved successfully
          </div>
        )}
        {saveStatus === 'error' && (
          <div className="save-notification error">
            ✗ Failed to save configuration
          </div>
        )}
      </div>

      {/* Main Content Area */}
      <div className="page-content">
        {/* Left Panel: Editor */}
        <div className="editor-panel">
          <ConfigEditor
            initialValue={initialConfig}
            onSave={handleSaveConfig}
            onValidationChange={handleValidationChange}
          />
        </div>

        {/* Right Panel: Validation Status */}
        <div className="validation-panel">
          <ValidationStatusPanel
            validationResult={validationResult}
            isValidating={isValidating}
            onErrorClick={handleErrorClick}
            onWarningClick={handleWarningClick}
          />
        </div>
      </div>

      {/* Help Text */}
      <div className="page-footer">
        <div className="help-text">
          <strong>Keyboard Shortcuts:</strong> F8 - Jump to next error | Ctrl+S - Save (if valid)
        </div>
      </div>
    </div>
  );
}

export default ConfigurationPage;
