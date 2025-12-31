/**
 * Unit tests for Monaco Quick Fix Integration
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import * as monaco from 'monaco-editor';
import {
  registerQuickFixProvider,
  updateQuickFixContext,
  applyQuickFix,
} from './monacoQuickFix';
import type {
  ValidationResult,
  ValidationError,
  ValidationWarning,
  QuickFix,
  TextEdit,
} from '../types/validation';

// Mock Monaco editor
vi.mock('monaco-editor', () => ({
  languages: {
    registerCodeActionProvider: vi.fn((languageId, provider) => ({
      dispose: vi.fn(),
    })),
  },
  Range: class {
    constructor(
      public startLineNumber: number,
      public startColumn: number,
      public endLineNumber: number,
      public endColumn: number
    ) {}
  },
  MarkerSeverity: {
    Error: 8,
    Warning: 4,
    Info: 2,
  },
}));

describe('monacoQuickFix', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('registerQuickFixProvider', () => {
    it('should register code action provider for rhai language', () => {
      const disposable = registerQuickFixProvider();

      expect(monaco.languages.registerCodeActionProvider).toHaveBeenCalledWith(
        'rhai',
        expect.objectContaining({
          provideCodeActions: expect.any(Function),
        })
      );
      expect(disposable).toBeDefined();
      expect(disposable.dispose).toBeDefined();
    });

    it('should return undefined when no validation result is set', () => {
      // Reset validation context
      updateQuickFixContext(null);

      const disposable = registerQuickFixProvider();
      const provider = vi.mocked(monaco.languages.registerCodeActionProvider)
        .mock.calls[0][1];

      const mockModel = {
        uri: { path: '/test.rhai' },
        getVersionId: () => 1,
      } as any;

      const mockRange = new monaco.Range(1, 1, 1, 10);
      const mockContext = {
        markers: [],
      } as any;

      const result = provider.provideCodeActions(
        mockModel,
        mockRange,
        mockContext
      );

      expect(result).toBeUndefined();
    });

    it('should provide code actions for errors with quick fixes', () => {
      const quickFix: QuickFix = {
        title: 'Replace with VK_A',
        description: 'Replace invalid key code with VK_A',
        edits: [
          {
            startLine: 5,
            startColumn: 10,
            endLine: 5,
            endColumn: 20,
            newText: 'VK_A',
          },
        ],
      };

      const validationResult: ValidationResult = {
        errors: [
          {
            line: 5,
            column: 10,
            message: 'Invalid key code',
            code: 'INVALID_KEY',
            quickFix,
          },
        ],
        warnings: [],
        hints: [],
        timestamp: new Date().toISOString(),
      };

      updateQuickFixContext(validationResult);

      const disposable = registerQuickFixProvider();
      const provider = vi.mocked(monaco.languages.registerCodeActionProvider)
        .mock.calls[0][1];

      const mockModel = {
        uri: { path: '/test.rhai' },
        getVersionId: () => 1,
      } as any;

      const mockRange = new monaco.Range(5, 10, 5, 20);
      const mockMarker = {
        startLineNumber: 5,
        startColumn: 10,
        endLineNumber: 5,
        endColumn: 20,
        message: 'Invalid key code',
        severity: monaco.MarkerSeverity.Error,
      };

      const mockContext = {
        markers: [mockMarker],
      } as any;

      const result = provider.provideCodeActions(
        mockModel,
        mockRange,
        mockContext
      );

      expect(result).toBeDefined();
      expect(result?.actions).toHaveLength(1);
      expect(result?.actions[0].title).toBe('Replace with VK_A');
      expect(result?.actions[0].kind).toBe('quickfix');
      expect(result?.actions[0].isPreferred).toBe(true);
    });

    it('should provide code actions for warnings with quick fixes', () => {
      const quickFix: QuickFix = {
        title: 'Rename to snake_case',
        description: 'Rename layer to use snake_case',
        edits: [
          {
            startLine: 3,
            startColumn: 8,
            endLine: 3,
            endColumn: 16,
            newText: 'my_layer',
          },
        ],
      };

      const validationResult: ValidationResult = {
        errors: [],
        warnings: [
          {
            line: 3,
            column: 8,
            message: 'Inconsistent naming style',
            code: 'NAMING_STYLE',
            quickFix,
          },
        ],
        hints: [],
        timestamp: new Date().toISOString(),
      };

      updateQuickFixContext(validationResult);

      const disposable = registerQuickFixProvider();
      const provider = vi.mocked(monaco.languages.registerCodeActionProvider)
        .mock.calls[0][1];

      const mockModel = {
        uri: { path: '/test.rhai' },
        getVersionId: () => 1,
      } as any;

      const mockRange = new monaco.Range(3, 8, 3, 16);
      const mockMarker = {
        startLineNumber: 3,
        startColumn: 8,
        endLineNumber: 3,
        endColumn: 16,
        message: 'Inconsistent naming style',
        severity: monaco.MarkerSeverity.Warning,
      };

      const mockContext = {
        markers: [mockMarker],
      } as any;

      const result = provider.provideCodeActions(
        mockModel,
        mockRange,
        mockContext
      );

      expect(result).toBeDefined();
      expect(result?.actions).toHaveLength(1);
      expect(result?.actions[0].title).toBe('Rename to snake_case');
    });

    it('should not provide actions for markers without quick fixes', () => {
      const validationResult: ValidationResult = {
        errors: [
          {
            line: 5,
            column: 10,
            message: 'Invalid key code',
            code: 'INVALID_KEY',
            // No quickFix
          },
        ],
        warnings: [],
        hints: [],
        timestamp: new Date().toISOString(),
      };

      updateQuickFixContext(validationResult);

      const disposable = registerQuickFixProvider();
      const provider = vi.mocked(monaco.languages.registerCodeActionProvider)
        .mock.calls[0][1];

      const mockModel = {
        uri: { path: '/test.rhai' },
        getVersionId: () => 1,
      } as any;

      const mockRange = new monaco.Range(5, 10, 5, 20);
      const mockMarker = {
        startLineNumber: 5,
        startColumn: 10,
        endLineNumber: 5,
        endColumn: 20,
        message: 'Invalid key code',
        severity: monaco.MarkerSeverity.Error,
      };

      const mockContext = {
        markers: [mockMarker],
      } as any;

      const result = provider.provideCodeActions(
        mockModel,
        mockRange,
        mockContext
      );

      expect(result).toBeDefined();
      expect(result?.actions).toHaveLength(0);
    });

    it('should handle multiple markers with different quick fixes', () => {
      const quickFix1: QuickFix = {
        title: 'Fix error 1',
        description: 'Fix the first error',
        edits: [
          {
            startLine: 5,
            startColumn: 10,
            endLine: 5,
            endColumn: 20,
            newText: 'FIX1',
          },
        ],
      };

      const quickFix2: QuickFix = {
        title: 'Fix error 2',
        description: 'Fix the second error',
        edits: [
          {
            startLine: 7,
            startColumn: 5,
            endLine: 7,
            endColumn: 15,
            newText: 'FIX2',
          },
        ],
      };

      const validationResult: ValidationResult = {
        errors: [
          {
            line: 5,
            column: 10,
            message: 'Error 1',
            code: 'ERROR1',
            quickFix: quickFix1,
          },
          {
            line: 7,
            column: 5,
            message: 'Error 2',
            code: 'ERROR2',
            quickFix: quickFix2,
          },
        ],
        warnings: [],
        hints: [],
        timestamp: new Date().toISOString(),
      };

      updateQuickFixContext(validationResult);

      const disposable = registerQuickFixProvider();
      const provider = vi.mocked(monaco.languages.registerCodeActionProvider)
        .mock.calls[0][1];

      const mockModel = {
        uri: { path: '/test.rhai' },
        getVersionId: () => 1,
      } as any;

      const mockRange = new monaco.Range(5, 10, 7, 15);
      const mockMarkers = [
        {
          startLineNumber: 5,
          startColumn: 10,
          endLineNumber: 5,
          endColumn: 20,
          message: 'Error 1',
          severity: monaco.MarkerSeverity.Error,
        },
        {
          startLineNumber: 7,
          startColumn: 5,
          endLineNumber: 7,
          endColumn: 15,
          message: 'Error 2',
          severity: monaco.MarkerSeverity.Error,
        },
      ];

      const mockContext = {
        markers: mockMarkers,
      } as any;

      const result = provider.provideCodeActions(
        mockModel,
        mockRange,
        mockContext
      );

      expect(result).toBeDefined();
      expect(result?.actions).toHaveLength(2);
      expect(result?.actions[0].title).toBe('Fix error 1');
      expect(result?.actions[1].title).toBe('Fix error 2');
    });
  });

  describe('updateQuickFixContext', () => {
    it('should update the validation context', () => {
      const validationResult: ValidationResult = {
        errors: [],
        warnings: [],
        hints: [],
        timestamp: new Date().toISOString(),
      };

      updateQuickFixContext(validationResult);

      // Verify by trying to get code actions
      const disposable = registerQuickFixProvider();
      const provider = vi.mocked(monaco.languages.registerCodeActionProvider)
        .mock.calls[0][1];

      const mockModel = {
        uri: { path: '/test.rhai' },
        getVersionId: () => 1,
      } as any;

      const mockRange = new monaco.Range(1, 1, 1, 10);
      const mockContext = {
        markers: [],
      } as any;

      const result = provider.provideCodeActions(
        mockModel,
        mockRange,
        mockContext
      );

      // Should return empty actions (not undefined) since we have a validation result
      expect(result).toBeDefined();
    });

    it('should allow clearing the validation context', () => {
      updateQuickFixContext(null);

      const disposable = registerQuickFixProvider();
      const provider = vi.mocked(monaco.languages.registerCodeActionProvider)
        .mock.calls[0][1];

      const mockModel = {
        uri: { path: '/test.rhai' },
        getVersionId: () => 1,
      } as any;

      const mockRange = new monaco.Range(1, 1, 1, 10);
      const mockContext = {
        markers: [],
      } as any;

      const result = provider.provideCodeActions(
        mockModel,
        mockRange,
        mockContext
      );

      expect(result).toBeUndefined();
    });
  });

  describe('applyQuickFix', () => {
    it('should apply single edit to editor', () => {
      const mockExecuteEdits = vi.fn();
      const mockEditor = {
        getModel: () => ({
          uri: { path: '/test.rhai' },
        }),
        executeEdits: mockExecuteEdits,
      } as any;

      const quickFix: QuickFix = {
        title: 'Fix it',
        description: 'Apply fix',
        edits: [
          {
            startLine: 5,
            startColumn: 10,
            endLine: 5,
            endColumn: 20,
            newText: 'FIXED',
          },
        ],
      };

      applyQuickFix(mockEditor, quickFix);

      expect(mockExecuteEdits).toHaveBeenCalledWith('quick-fix', [
        expect.objectContaining({
          range: expect.any(monaco.Range),
          text: 'FIXED',
          forceMoveMarkers: true,
        }),
      ]);
    });

    it('should apply multiple edits atomically', () => {
      const mockExecuteEdits = vi.fn();
      const mockEditor = {
        getModel: () => ({
          uri: { path: '/test.rhai' },
        }),
        executeEdits: mockExecuteEdits,
      } as any;

      const quickFix: QuickFix = {
        title: 'Fix multiple',
        description: 'Apply multiple fixes',
        edits: [
          {
            startLine: 1,
            startColumn: 1,
            endLine: 1,
            endColumn: 5,
            newText: 'EDIT1',
          },
          {
            startLine: 3,
            startColumn: 10,
            endLine: 3,
            endColumn: 15,
            newText: 'EDIT2',
          },
        ],
      };

      applyQuickFix(mockEditor, quickFix);

      expect(mockExecuteEdits).toHaveBeenCalledTimes(1);
      expect(mockExecuteEdits).toHaveBeenCalledWith('quick-fix', [
        expect.objectContaining({
          text: 'EDIT1',
        }),
        expect.objectContaining({
          text: 'EDIT2',
        }),
      ]);
    });

    it('should handle null model gracefully', () => {
      const mockEditor = {
        getModel: () => null,
        executeEdits: vi.fn(),
      } as any;

      const quickFix: QuickFix = {
        title: 'Fix it',
        description: 'Apply fix',
        edits: [
          {
            startLine: 5,
            startColumn: 10,
            endLine: 5,
            endColumn: 20,
            newText: 'FIXED',
          },
        ],
      };

      // Should not throw
      expect(() => applyQuickFix(mockEditor, quickFix)).not.toThrow();
      expect(mockEditor.executeEdits).not.toHaveBeenCalled();
    });

    it('should support deletion (empty newText)', () => {
      const mockExecuteEdits = vi.fn();
      const mockEditor = {
        getModel: () => ({
          uri: { path: '/test.rhai' },
        }),
        executeEdits: mockExecuteEdits,
      } as any;

      const quickFix: QuickFix = {
        title: 'Remove invalid code',
        description: 'Delete the invalid section',
        edits: [
          {
            startLine: 5,
            startColumn: 10,
            endLine: 5,
            endColumn: 20,
            newText: '',
          },
        ],
      };

      applyQuickFix(mockEditor, quickFix);

      expect(mockExecuteEdits).toHaveBeenCalledWith('quick-fix', [
        expect.objectContaining({
          text: '',
        }),
      ]);
    });
  });
});
