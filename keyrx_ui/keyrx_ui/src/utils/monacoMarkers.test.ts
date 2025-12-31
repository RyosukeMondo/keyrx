/**
 * Tests for Monaco Editor Markers Integration
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import * as monaco from 'monaco-editor';
import { updateEditorMarkers, clearEditorMarkers } from './monacoMarkers';
import type {
  ValidationResult,
  ValidationError,
  ValidationWarning,
  ValidationHint,
} from '../types/validation';

// Mock Monaco editor types
interface ITextModel {
  // Minimal mock interface
}

interface IStandaloneCodeEditor {
  getModel(): ITextModel | null;
}

// Spy on Monaco editor methods
const mockSetModelMarkers = vi.fn();

describe('monacoMarkers', () => {
  let mockEditor: IStandaloneCodeEditor;
  let mockModel: ITextModel;

  beforeEach(() => {
    // Replace Monaco's setModelMarkers with our spy
    monaco.editor.setModelMarkers = mockSetModelMarkers;
    mockSetModelMarkers.mockClear();

    // Create mock model
    mockModel = {} as ITextModel;

    // Create mock editor
    mockEditor = {
      getModel: vi.fn(() => mockModel),
    } as unknown as IStandaloneCodeEditor;
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe('updateEditorMarkers', () => {
    it('should clear markers when validation result is null', () => {
      updateEditorMarkers(mockEditor, null);

      expect(mockSetModelMarkers).toHaveBeenCalledWith(
        mockModel,
        'config-validator',
        []
      );
    });

    it('should handle editor with no model gracefully', () => {
      const editorNoModel = {
        getModel: vi.fn(() => null),
      } as unknown as IStandaloneCodeEditor;

      // Should not throw
      expect(() => updateEditorMarkers(editorNoModel, null)).not.toThrow();

      // Should not call setModelMarkers
      expect(mockSetModelMarkers).not.toHaveBeenCalled();
    });

    it('should create error markers with correct severity', () => {
      const error: ValidationError = {
        line: 5,
        column: 10,
        message: 'Syntax error',
        code: 'SYNTAX_ERROR',
      };

      const validationResult: ValidationResult = {
        errors: [error],
        warnings: [],
        hints: [],
        stats: {
          totalLines: 10,
          totalLayers: 1,
          totalMappings: 5,
        },
      };

      updateEditorMarkers(mockEditor, validationResult);

      expect(mockSetModelMarkers).toHaveBeenCalledWith(
        mockModel,
        'config-validator',
        [
          {
            severity: 8, // MarkerSeverity.Error
            startLineNumber: 5,
            startColumn: 10,
            endLineNumber: 5,
            endColumn: 20, // column + 10
            message: 'Syntax error',
            code: 'SYNTAX_ERROR',
            tags: undefined,
          },
        ]
      );
    });

    it('should create warning markers with correct severity', () => {
      const warning: ValidationWarning = {
        line: 3,
        column: 1,
        message: 'Unused layer',
        code: 'UNUSED_LAYER',
      };

      const validationResult: ValidationResult = {
        errors: [],
        warnings: [warning],
        hints: [],
        stats: {
          totalLines: 10,
          totalLayers: 2,
          totalMappings: 5,
        },
      };

      updateEditorMarkers(mockEditor, validationResult);

      expect(mockSetModelMarkers).toHaveBeenCalledWith(
        mockModel,
        'config-validator',
        [
          {
            severity: 4, // MarkerSeverity.Warning
            startLineNumber: 3,
            startColumn: 1,
            endLineNumber: 3,
            endColumn: 11,
            message: 'Unused layer',
            code: 'UNUSED_LAYER',
            tags: undefined,
          },
        ]
      );
    });

    it('should create hint markers with correct severity', () => {
      const hint: ValidationHint = {
        line: 1,
        column: 1,
        message: 'Consider consistent naming',
        code: 'NAMING_INCONSISTENCY',
      };

      const validationResult: ValidationResult = {
        errors: [],
        warnings: [],
        hints: [hint],
        stats: {
          totalLines: 10,
          totalLayers: 1,
          totalMappings: 5,
        },
      };

      updateEditorMarkers(mockEditor, validationResult);

      expect(mockSetModelMarkers).toHaveBeenCalledWith(
        mockModel,
        'config-validator',
        [
          {
            severity: 2, // MarkerSeverity.Info
            startLineNumber: 1,
            startColumn: 1,
            endLineNumber: 1,
            endColumn: 11,
            message: 'Consider consistent naming',
            code: 'NAMING_INCONSISTENCY',
            tags: undefined,
          },
        ]
      );
    });

    it('should handle errors with explicit endLine and endColumn', () => {
      const error: ValidationError = {
        line: 5,
        column: 10,
        endLine: 5,
        endColumn: 25,
        message: 'Invalid token range',
        code: 'INVALID_TOKEN',
      };

      const validationResult: ValidationResult = {
        errors: [error],
        warnings: [],
        hints: [],
        stats: {
          totalLines: 10,
          totalLayers: 1,
          totalMappings: 5,
        },
      };

      updateEditorMarkers(mockEditor, validationResult);

      expect(mockSetModelMarkers).toHaveBeenCalledWith(
        mockModel,
        'config-validator',
        [
          {
            severity: 8, // MarkerSeverity.Error
            startLineNumber: 5,
            startColumn: 10,
            endLineNumber: 5,
            endColumn: 25,
            message: 'Invalid token range',
            code: 'INVALID_TOKEN',
            tags: undefined,
          },
        ]
      );
    });

    it('should create multiple markers for multiple issues', () => {
      const validationResult: ValidationResult = {
        errors: [
          {
            line: 1,
            column: 1,
            message: 'Error 1',
            code: 'ERR1',
          },
          {
            line: 2,
            column: 5,
            message: 'Error 2',
            code: 'ERR2',
          },
        ],
        warnings: [
          {
            line: 3,
            column: 10,
            message: 'Warning 1',
            code: 'WARN1',
          },
        ],
        hints: [
          {
            line: 4,
            column: 1,
            message: 'Hint 1',
            code: 'HINT1',
          },
        ],
        stats: {
          totalLines: 10,
          totalLayers: 1,
          totalMappings: 5,
        },
      };

      updateEditorMarkers(mockEditor, validationResult);

      const markers = mockSetModelMarkers.mock.calls[0][2];
      expect(markers).toHaveLength(4);
      expect(markers[0].severity).toBe(8); // MarkerSeverity.Error
      expect(markers[1].severity).toBe(8); // MarkerSeverity.Error
      expect(markers[2].severity).toBe(4); // MarkerSeverity.Warning
      expect(markers[3].severity).toBe(2); // MarkerSeverity.Info
    });

    it('should clear markers when validation has no issues', () => {
      const validationResult: ValidationResult = {
        errors: [],
        warnings: [],
        hints: [],
        stats: {
          totalLines: 10,
          totalLayers: 1,
          totalMappings: 5,
        },
      };

      updateEditorMarkers(mockEditor, validationResult);

      expect(mockSetModelMarkers).toHaveBeenCalledWith(
        mockModel,
        'config-validator',
        []
      );
    });
  });

  describe('clearEditorMarkers', () => {
    it('should clear all markers', () => {
      clearEditorMarkers(mockEditor);

      expect(mockSetModelMarkers).toHaveBeenCalledWith(
        mockModel,
        'config-validator',
        []
      );
    });

    it('should handle editor with no model gracefully', () => {
      const editorNoModel = {
        getModel: vi.fn(() => null),
      } as unknown as IStandaloneCodeEditor;

      // Should not throw
      expect(() => clearEditorMarkers(editorNoModel)).not.toThrow();

      // Should not call setModelMarkers
      expect(mockSetModelMarkers).not.toHaveBeenCalled();
    });
  });
});
