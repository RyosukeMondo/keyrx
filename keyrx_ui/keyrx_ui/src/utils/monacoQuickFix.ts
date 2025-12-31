/**
 * Monaco Editor Quick Fix Integration
 *
 * Provides code action provider for "Quick Fix" suggestions that can
 * automatically resolve validation errors and warnings.
 */

import * as monaco from 'monaco-editor';
import type { QuickFix, TextEdit, ValidationResult } from '../types/validation';

/**
 * Stores the current validation result for the code action provider.
 * Updated by registerQuickFixProvider when validation results change.
 */
let currentValidationResult: ValidationResult | null = null;

/**
 * Registers a Monaco code action provider for Quick Fix suggestions.
 * This provider reads validation results and offers quick fixes for
 * errors and warnings at the cursor position.
 *
 * @returns Disposable to unregister the provider when no longer needed
 */
export function registerQuickFixProvider(): monaco.IDisposable {
  return monaco.languages.registerCodeActionProvider('rhai', {
    provideCodeActions(
      model: monaco.editor.ITextModel,
      range: monaco.Range,
      context: monaco.languages.CodeActionContext
    ): monaco.languages.CodeActionList | undefined {
      if (!currentValidationResult) {
        return undefined;
      }

      const actions: monaco.languages.CodeAction[] = [];

      // Check markers at current position for quick fixes
      for (const marker of context.markers) {
        // Find corresponding error or warning in validation result
        const quickFix = findQuickFixForMarker(marker, currentValidationResult);

        if (quickFix) {
          actions.push(createCodeAction(model, marker, quickFix));
        }
      }

      return {
        actions,
        dispose: () => {}, // No cleanup needed
      };
    },
  });
}

/**
 * Updates the validation result used by the Quick Fix provider.
 * Call this whenever validation results change.
 *
 * @param validationResult - Latest validation result
 */
export function updateQuickFixContext(
  validationResult: ValidationResult | null
): void {
  currentValidationResult = validationResult;
}

/**
 * Finds a quick fix suggestion for a given marker.
 *
 * @param marker - Monaco marker data
 * @param validationResult - Current validation result
 * @returns QuickFix if available, undefined otherwise
 */
function findQuickFixForMarker(
  marker: monaco.editor.IMarker,
  validationResult: ValidationResult
): QuickFix | undefined {
  // Search errors
  for (const error of validationResult.errors) {
    if (
      error.line === marker.startLineNumber &&
      error.column === marker.startColumn &&
      error.quickFix
    ) {
      return error.quickFix;
    }
  }

  // Search warnings
  for (const warning of validationResult.warnings) {
    if (
      warning.line === marker.startLineNumber &&
      warning.column === marker.startColumn &&
      warning.quickFix
    ) {
      return warning.quickFix;
    }
  }

  return undefined;
}

/**
 * Creates a Monaco code action from a quick fix suggestion.
 *
 * @param model - Editor text model
 * @param marker - Marker associated with the issue
 * @param quickFix - Quick fix to apply
 * @returns Monaco code action
 */
function createCodeAction(
  model: monaco.editor.ITextModel,
  marker: monaco.editor.IMarker,
  quickFix: QuickFix
): monaco.languages.CodeAction {
  return {
    title: quickFix.title,
    diagnostics: [marker],
    kind: 'quickfix',
    edit: {
      edits: quickFix.edits.map((edit) => ({
        resource: model.uri,
        versionId: model.getVersionId(),
        textEdit: convertTextEdit(edit),
      })),
    },
    isPreferred: true, // Show as primary action
  };
}

/**
 * Converts a TextEdit to Monaco IIdentifiedSingleEditOperation format.
 *
 * @param edit - TextEdit to convert
 * @returns Monaco edit operation
 */
function convertTextEdit(
  edit: TextEdit
): { range: monaco.IRange; text: string } {
  return {
    range: {
      startLineNumber: edit.startLine,
      startColumn: edit.startColumn,
      endLineNumber: edit.endLine,
      endColumn: edit.endColumn,
    },
    text: edit.newText,
  };
}

/**
 * Applies quick fix edits directly to an editor.
 * Useful for programmatic application without user interaction.
 *
 * @param editor - Monaco editor instance
 * @param quickFix - Quick fix to apply
 */
export function applyQuickFix(
  editor: monaco.editor.IStandaloneCodeEditor,
  quickFix: QuickFix
): void {
  const model = editor.getModel();

  if (!model) {
    console.warn('applyQuickFix: No model available');
    return;
  }

  // Convert edits to Monaco edit operations
  const edits: monaco.editor.IIdentifiedSingleEditOperation[] =
    quickFix.edits.map((edit) => ({
      range: new monaco.Range(
        edit.startLine,
        edit.startColumn,
        edit.endLine,
        edit.endColumn
      ),
      text: edit.newText,
      forceMoveMarkers: true,
    }));

  // Apply all edits atomically
  editor.executeEdits('quick-fix', edits);
}
