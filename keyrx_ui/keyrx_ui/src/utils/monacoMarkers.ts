/**
 * Monaco Editor Markers Integration
 *
 * Converts validation results to Monaco editor markers for displaying
 * errors, warnings, and hints with appropriate visual indicators.
 */

import * as monaco from 'monaco-editor';
import type {
  ValidationResult,
  ValidationError,
  ValidationWarning,
  ValidationHint,
} from '../types/validation';

/**
 * Updates Monaco editor markers based on validation results.
 * Clears existing markers and displays new errors, warnings, and hints.
 *
 * @param editor - Monaco editor instance
 * @param validationResult - Validation result containing errors, warnings, and hints
 */
export function updateEditorMarkers(
  editor: monaco.editor.IStandaloneCodeEditor,
  validationResult: ValidationResult | null
): void {
  const model = editor.getModel();

  if (!model) {
    console.warn('updateEditorMarkers: No model available');
    return;
  }

  // Clear markers if validation result is null or no issues
  if (!validationResult) {
    monaco.editor.setModelMarkers(model, 'config-validator', []);
    return;
  }

  const markers: monaco.editor.IMarkerData[] = [];

  // Convert errors to markers
  if (validationResult.errors) {
    validationResult.errors.forEach((error) => {
      markers.push(createErrorMarker(error));
    });
  }

  // Convert warnings to markers
  if (validationResult.warnings) {
    validationResult.warnings.forEach((warning) => {
      markers.push(createWarningMarker(warning));
    });
  }

  // Convert hints to markers
  if (validationResult.hints) {
    validationResult.hints.forEach((hint) => {
      markers.push(createHintMarker(hint));
    });
  }

  // Update markers in editor
  monaco.editor.setModelMarkers(model, 'config-validator', markers);
}

/**
 * Creates a Monaco marker for a validation error.
 * Displays red squiggly underline.
 *
 * @param error - Validation error
 * @returns Monaco marker data
 */
function createErrorMarker(error: ValidationError): monaco.editor.IMarkerData {
  return {
    severity: monaco.MarkerSeverity.Error,
    startLineNumber: error.line,
    startColumn: error.column,
    endLineNumber: error.endLine ?? error.line,
    endColumn: error.endColumn ?? error.column + 10,
    message: error.message,
    code: error.code ? String(error.code) : undefined,
    tags: error.tags,
  };
}

/**
 * Creates a Monaco marker for a validation warning.
 * Displays orange squiggly underline.
 *
 * @param warning - Validation warning
 * @returns Monaco marker data
 */
function createWarningMarker(warning: ValidationWarning): monaco.editor.IMarkerData {
  return {
    severity: monaco.MarkerSeverity.Warning,
    startLineNumber: warning.line,
    startColumn: warning.column,
    endLineNumber: warning.endLine ?? warning.line,
    endColumn: warning.endColumn ?? warning.column + 10,
    message: warning.message,
    code: warning.code ? String(warning.code) : undefined,
    tags: warning.tags,
  };
}

/**
 * Creates a Monaco marker for a validation hint.
 * Displays blue information indicator.
 *
 * @param hint - Validation hint
 * @returns Monaco marker data
 */
function createHintMarker(hint: ValidationHint): monaco.editor.IMarkerData {
  return {
    severity: monaco.MarkerSeverity.Info,
    startLineNumber: hint.line,
    startColumn: hint.column,
    endLineNumber: hint.endLine ?? hint.line,
    endColumn: hint.endColumn ?? hint.column + 10,
    message: hint.message,
    code: hint.code ? String(hint.code) : undefined,
    tags: hint.tags,
  };
}

/**
 * Clears all validation markers from the editor.
 *
 * @param editor - Monaco editor instance
 */
export function clearEditorMarkers(
  editor: monaco.editor.IStandaloneCodeEditor
): void {
  const model = editor.getModel();

  if (!model) {
    return;
  }

  monaco.editor.setModelMarkers(model, 'config-validator', []);
}
