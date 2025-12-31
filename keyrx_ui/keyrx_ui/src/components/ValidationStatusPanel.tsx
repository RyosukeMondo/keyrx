import React, { useState } from 'react';
import type { ValidationResult, ValidationError, ValidationWarning } from '../types/validation';
import './ValidationStatusPanel.css';

export interface ValidationStatusPanelProps {
  validationResult: ValidationResult | null;
  isValidating: boolean;
  onErrorClick: (error: ValidationError) => void;
  onWarningClick: (warning: ValidationWarning) => void;
}

/**
 * Component that displays validation status summary with error/warning counts
 * and provides jump-to-error functionality.
 */
export const ValidationStatusPanel: React.FC<ValidationStatusPanelProps> = ({
  validationResult,
  isValidating,
  onErrorClick,
  onWarningClick,
}) => {
  const [isExpanded, setIsExpanded] = useState(true);

  const errorCount = validationResult?.errors?.length || 0;
  const warningCount = validationResult?.warnings?.length || 0;
  const hintCount = validationResult?.hints?.length || 0;

  const isValid = validationResult && errorCount === 0 && warningCount === 0;

  const toggleExpanded = () => {
    setIsExpanded((prev) => !prev);
  };

  const handleKeyDown = (event: React.KeyboardEvent) => {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      toggleExpanded();
    }
  };

  // Limit error list to 10 items
  const maxDisplayItems = 10;
  const displayErrors = validationResult?.errors?.slice(0, maxDisplayItems) || [];
  const remainingErrors = Math.max(0, errorCount - maxDisplayItems);

  const displayWarnings = validationResult?.warnings?.slice(0, maxDisplayItems) || [];
  const remainingWarnings = Math.max(0, warningCount - maxDisplayItems);

  return (
    <div className="validation-status-panel" role="region" aria-label="Validation Status">
      {/* Header with badges */}
      <div
        className="validation-status-header"
        onClick={toggleExpanded}
        onKeyDown={handleKeyDown}
        role="button"
        tabIndex={0}
        aria-expanded={isExpanded}
        aria-label={`Validation status: ${errorCount} errors, ${warningCount} warnings. Click to ${isExpanded ? 'collapse' : 'expand'}`}
      >
        <div className="validation-badges">
          {isValidating && (
            <span className="validation-badge validating" aria-live="polite">
              ⏳ Validating...
            </span>
          )}

          {!isValidating && isValid && (
            <span className="validation-badge success" aria-live="polite">
              ✓ Configuration valid
            </span>
          )}

          {!isValidating && errorCount > 0 && (
            <span className="validation-badge error" aria-live="polite">
              ❌ {errorCount} Error{errorCount !== 1 ? 's' : ''}
            </span>
          )}

          {!isValidating && warningCount > 0 && (
            <span className="validation-badge warning" aria-live="polite">
              ⚠️ {warningCount} Warning{warningCount !== 1 ? 's' : ''}
            </span>
          )}

          {!isValidating && hintCount > 0 && (
            <span className="validation-badge hint">
              💡 {hintCount} Hint{hintCount !== 1 ? 's' : ''}
            </span>
          )}
        </div>

        <span className="expand-icon" aria-hidden="true">
          {isExpanded ? '▼' : '▶'}
        </span>
      </div>

      {/* Expandable content */}
      {isExpanded && !isValidating && (
        <div className="validation-status-content">
          {/* Error list */}
          {errorCount > 0 && (
            <div className="validation-section">
              <h4 className="validation-section-title">Errors</h4>
              <ul className="validation-list" aria-label="Validation Errors">
                {displayErrors.map((error, index) => (
                  <li key={index} className="validation-item error-item">
                    <span className="validation-message">
                      Line {error.line}: {error.message}
                    </span>
                    <button
                      className="jump-button"
                      onClick={() => onErrorClick(error)}
                      aria-label={`Jump to error at line ${error.line}`}
                    >
                      Jump
                    </button>
                  </li>
                ))}
              </ul>
              {remainingErrors > 0 && (
                <p className="validation-overflow" aria-live="polite">
                  ...and {remainingErrors} more error{remainingErrors !== 1 ? 's' : ''}
                </p>
              )}
            </div>
          )}

          {/* Warning list */}
          {warningCount > 0 && (
            <div className="validation-section">
              <h4 className="validation-section-title">Warnings</h4>
              <ul className="validation-list" aria-label="Validation Warnings">
                {displayWarnings.map((warning, index) => (
                  <li key={index} className="validation-item warning-item">
                    <span className="validation-message">
                      Line {warning.line}: {warning.message}
                    </span>
                    <button
                      className="jump-button"
                      onClick={() => onWarningClick(warning)}
                      aria-label={`Jump to warning at line ${warning.line}`}
                    >
                      Jump
                    </button>
                  </li>
                ))}
              </ul>
              {remainingWarnings > 0 && (
                <p className="validation-overflow" aria-live="polite">
                  ...and {remainingWarnings} more warning{remainingWarnings !== 1 ? 's' : ''}
                </p>
              )}
            </div>
          )}

          {/* Success message when no errors or warnings */}
          {isValid && (
            <div className="validation-success">
              <p>✓ Your configuration is valid and ready to use.</p>
            </div>
          )}
        </div>
      )}
    </div>
  );
};
