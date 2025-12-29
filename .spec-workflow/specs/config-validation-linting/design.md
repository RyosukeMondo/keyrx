# Design Document

## Architecture Overview

The Configuration Validation & Linting feature integrates the WASM validation engine into the configuration editor UI, providing real-time feedback as users type. The architecture follows a three-layer pattern: **Editor UI** → **Validation Service** → **WASM Module**.

```
┌─────────────────────────────────────────────────────────────┐
│                         User Types                          │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              Monaco Editor Component                        │
│  - Syntax highlighting                                      │
│  - Error squiggles (red underlines)                        │
│  - Hover tooltips                                          │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼ (debounced 500ms)
┌─────────────────────────────────────────────────────────────┐
│         useConfigValidator Hook                             │
│  - Debounce user input                                      │
│  - Call validator.validate()                                │
│  - Update editor markers                                    │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│            Validator Service                                │
│  (keyrx_ui/src/utils/validator.ts)                         │
│  - Calls WasmCore.loadConfig()                             │
│  - Parses WASM errors                                       │
│  - Returns ValidationResult                                 │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              WASM Module                                    │
│  (keyrx_core/src/wasm.rs)                                  │
│  - load_config(rhaiSource)                                 │
│  - Returns Result<ConfigHandle, Error>                     │
└─────────────────────────────────────────────────────────────┘
```

### Data Flow

1. **User Types** → Editor onChange fires with new content
2. **Debounce** → useConfigValidator waits 500ms after last keystroke
3. **Validate** → validator.validate(content) calls WasmCore.loadConfig()
4. **Parse Errors** → WASM errors converted to Monaco markers
5. **Update UI** → Monaco editor displays error squiggles, validation panel updates counts

## Components and Interfaces

### Component 1: ConfigEditor Component (keyrx_ui/src/components/ConfigEditor.tsx)

**Purpose**: Main configuration editor with Monaco editor integration.

**TypeScript Interface**:
```typescript
interface ConfigEditorProps {
  initialValue?: string;
  onSave: (content: string) => Promise<void>;
  onValidationChange?: (result: ValidationResult) => void;
}

interface ConfigEditorState {
  content: string;
  validationResult: ValidationResult | null;
  isValidating: boolean;
}
```

**Key Methods**:
- `handleEditorChange(value: string)` → Update content and trigger validation
- `handleSave()` → Validate one final time, then call onSave if valid
- `jumpToError(errorIndex: number)` → Move cursor to error location

**Integration Points**:
- Uses `useConfigValidator` hook for validation logic
- Uses `@monaco-editor/react` for Monaco editor
- Displays `ValidationStatusPanel` at bottom

**UI Layout**:
```
+-----------------------------------------------------------+
| Configuration Editor                    [Save] [Test]    |
+-----------------------------------------------------------+
| 1  // Base layer configuration                           |
| 2  layer "base" {                                        |
| 3      map KEY_A to KEY_B                                |
| 4      map KEY_INVALID to KEY_C    <-- ~~~ (red squiggle)|
| 5  }                                                      |
|                                                           |
|                [Hover tooltip on line 4]                 |
|                ┌─────────────────────────────────────┐   |
|                │ Error: Invalid key code             │   |
|                │ Line 4, Col 9                       │   |
|                │ 'KEY_INVALID' is not a valid code   │   |
|                │ Suggestion: Use KEY_A or KEY_B      │   |
|                │ [Quick Fix]                         │   |
|                └─────────────────────────────────────┘   |
|                                                           |
| 42 lines                                  [Validating...] |
+-----------------------------------------------------------+
| Validation Status                                         |
| ❌ 1 Error   ⚠️ 2 Warnings   💡 1 Hint                   |
| [View Errors >]                                           |
+-----------------------------------------------------------+
```

### Component 2: useConfigValidator Hook (keyrx_ui/src/hooks/useConfigValidator.ts)

**Purpose**: React hook that manages validation state and debouncing.

**TypeScript Interface**:
```typescript
interface UseConfigValidatorOptions {
  debounceMs?: number;  // Default 500ms
  enableLinting?: boolean;  // Default true
}

interface UseConfigValidatorReturn {
  validationResult: ValidationResult | null;
  isValidating: boolean;
  validate: (content: string) => Promise<void>;
  clearValidation: () => void;
}

function useConfigValidator(
  options?: UseConfigValidatorOptions
): UseConfigValidatorReturn;
```

**Key Implementation**:
```typescript
const useConfigValidator = (options = {}) => {
  const [result, setResult] = useState<ValidationResult | null>(null);
  const [isValidating, setIsValidating] = useState(false);

  const debouncedValidate = useMemo(
    () => debounce(async (content: string) => {
      setIsValidating(true);
      try {
        const validationResult = await validator.validate(content);
        setResult(validationResult);
      } catch (error) {
        setResult({ errors: [parseWasmError(error)], warnings: [], hints: [] });
      } finally {
        setIsValidating(false);
      }
    }, options.debounceMs || 500),
    [options.debounceMs]
  );

  return { validationResult: result, isValidating, validate: debouncedValidate, clearValidation: () => setResult(null) };
};
```

### Component 3: Validator Service (keyrx_ui/src/utils/validator.ts)

**Purpose**: Core validation logic that wraps WASM module.

**TypeScript Interface**:
```typescript
export interface ValidationResult {
  errors: ValidationError[];
  warnings: ValidationWarning[];
  hints: ValidationHint[];
  stats?: ConfigStats;
}

export interface ValidationError {
  line: number;
  column: number;
  endLine?: number;
  endColumn?: number;
  message: string;
  code: string;  // e.g., "SYNTAX_ERROR", "UNDEFINED_LAYER"
  suggestion?: string;
  quickFix?: QuickFix;
}

export interface ValidationWarning {
  line: number;
  column: number;
  message: string;
  code: string;  // e.g., "UNUSED_LAYER"
}

export interface ValidationHint {
  line: number;
  column: number;
  message: string;
  code: string;  // e.g., "NAMING_INCONSISTENCY"
}

export interface QuickFix {
  title: string;
  edits: TextEdit[];
}

export interface TextEdit {
  range: { startLine: number, startColumn: number, endLine: number, endColumn: number };
  newText: string;
}

export interface ConfigStats {
  lineCount: number;
  mappingCount: number;
  layerCount: number;
}

class ConfigValidator {
  async validate(rhaiSource: string): Promise<ValidationResult> {
    // Implementation
  }

  private parseWasmError(wasmError: any): ValidationError[] {
    // Convert WASM JsValue errors to ValidationError format
  }

  private runLintingRules(configHandle: ConfigHandle): ValidationWarning[] {
    // Run optional linting rules (unused layers, naming conventions)
  }
}

export const validator = new ConfigValidator();
```

### Component 4: ValidationStatusPanel (keyrx_ui/src/components/ValidationStatusPanel.tsx)

**Purpose**: Bottom panel showing error/warning counts with quick navigation.

**TypeScript Interface**:
```typescript
interface ValidationStatusPanelProps {
  validationResult: ValidationResult | null;
  isValidating: boolean;
  onErrorClick: (error: ValidationError) => void;
  onWarningClick: (warning: ValidationWarning) => void;
}
```

**UI Layout**:
```
+-----------------------------------------------------------+
| Validation Status                                         |
+-----------------------------------------------------------+
| ❌ 2 Errors   ⚠️ 3 Warnings   💡 1 Hint                  |
|                                                           |
| Errors:                                                   |
|  • Line 4: Invalid key code 'KEY_INVALID'      [Jump]    |
|  • Line 12: Missing semicolon after statement  [Jump]    |
|                                                           |
| Warnings:                                                 |
|  • Line 5: Layer 'debug' is defined but never used        |
|  • Line 8: Modifier 'MD_00' not activated in any mapping  |
|  • Line 20: Consider using snake_case for layer names     |
|                                                           |
| Hints:                                                    |
|  • Configuration exceeds 500 lines. Consider splitting.   |
+-----------------------------------------------------------+
```

### Component 5: Monaco Editor Integration (keyrx_ui/src/utils/monacoConfig.ts)

**Purpose**: Configure Monaco editor for Rhai syntax highlighting and error markers.

**Key Functions**:
```typescript
export function registerRhaiLanguage() {
  monaco.languages.register({ id: 'rhai' });
  monaco.languages.setMonarchTokensProvider('rhai', rhaiSyntax);
  monaco.languages.setLanguageConfiguration('rhai', rhaiConfig);
}

export function updateEditorMarkers(
  editor: monaco.editor.IStandaloneCodeEditor,
  validationResult: ValidationResult
) {
  const model = editor.getModel();
  if (!model) return;

  const markers: monaco.editor.IMarkerData[] = [
    ...validationResult.errors.map(err => ({
      severity: monaco.MarkerSeverity.Error,
      startLineNumber: err.line,
      startColumn: err.column,
      endLineNumber: err.endLine || err.line,
      endColumn: err.endColumn || err.column + 10,
      message: err.message,
      code: err.code,
    })),
    ...validationResult.warnings.map(warn => ({
      severity: monaco.MarkerSeverity.Warning,
      startLineNumber: warn.line,
      startColumn: warn.column,
      endLineNumber: warn.line,
      endColumn: warn.column + 10,
      message: warn.message,
      code: warn.code,
    })),
  ];

  monaco.editor.setModelMarkers(model, 'config-validator', markers);
}

export function registerQuickFixProvider() {
  monaco.languages.registerCodeActionProvider('rhai', {
    provideCodeActions: (model, range, context) => {
      // Provide "Quick Fix" actions for errors with suggestions
    }
  });
}
```

## Data Models

### ValidationResult Type

```typescript
export type ValidationResult = {
  errors: ValidationError[];      // Must fix before applying
  warnings: ValidationWarning[];  // Should review but can ignore
  hints: ValidationHint[];        // Optional suggestions
  stats?: ConfigStats;            // Line count, mapping count, etc.
};
```

### ValidationError Type

```typescript
export type ValidationError = {
  line: number;          // 1-based line number
  column: number;        // 1-based column number
  endLine?: number;      // Optional end position for multi-line errors
  endColumn?: number;
  message: string;       // Human-readable error description
  code: string;          // Machine-readable error code (e.g., "SYNTAX_ERROR")
  suggestion?: string;   // Optional fix suggestion
  quickFix?: QuickFix;   // Optional automated fix
};
```

### QuickFix Type

```typescript
export type QuickFix = {
  title: string;         // e.g., "Replace with 'gaming'"
  edits: TextEdit[];     // Automated edits to apply
};

export type TextEdit = {
  range: { startLine: number, startColumn: number, endLine: number, endColumn: number };
  newText: string;
};
```

## Error Handling

### WASM Validation Errors

```typescript
async function validate(rhaiSource: string): Promise<ValidationResult> {
  try {
    // Try to load config via WASM
    const configHandle = await WasmCore.loadConfig(rhaiSource);

    // If successful, run linting rules
    const warnings = runLintingRules(configHandle);

    return {
      errors: [],
      warnings,
      hints: [],
      stats: { lineCount: rhaiSource.split('\n').length, mappingCount: 0, layerCount: 0 }
    };
  } catch (wasmError) {
    // Parse WASM error into ValidationError format
    const errors = parseWasmError(wasmError);
    return { errors, warnings: [], hints: [] };
  }
}

function parseWasmError(wasmError: any): ValidationError[] {
  // WASM errors come as JsValue with message like:
  // "Parse error at line 4, column 9: Invalid key code 'KEY_INVALID'"
  const match = wasmError.message.match(/line (\d+), column (\d+): (.+)/);
  if (match) {
    return [{
      line: parseInt(match[1]),
      column: parseInt(match[2]),
      message: match[3],
      code: 'WASM_ERROR',
    }];
  }

  // Fallback for unparseable errors
  return [{
    line: 1,
    column: 1,
    message: wasmError.message || 'Unknown validation error',
    code: 'UNKNOWN_ERROR',
  }];
}
```

### Graceful Degradation

If WASM module fails to initialize:

```typescript
function useConfigValidator() {
  const [wasmAvailable, setWasmAvailable] = useState(false);

  useEffect(() => {
    WasmCore.init()
      .then(() => setWasmAvailable(true))
      .catch(() => {
        console.error('WASM validation unavailable');
        setWasmAvailable(false);
      });
  }, []);

  const validate = async (content: string) => {
    if (!wasmAvailable) {
      return {
        errors: [{
          line: 1,
          column: 1,
          message: 'Validation unavailable. Please reload the page.',
          code: 'WASM_UNAVAILABLE'
        }],
        warnings: [],
        hints: []
      };
    }
    // ... normal validation
  };

  return { validate, wasmAvailable };
}
```

## Testing Strategy

### Unit Tests

**validator.test.ts**:
- Test validation of valid Rhai configs (0 errors)
- Test validation of invalid syntax (parse errors with line numbers)
- Test validation of semantic errors (undefined layers, invalid key codes)
- Test WASM error parsing (extract line/column from error messages)
- Test linting rules (unused layers, naming conventions)

**useConfigValidator.test.ts**:
- Test debouncing (validate only after 500ms idle)
- Test validation state updates (isValidating flag)
- Test error handling (WASM crashes)

### Integration Tests

**ConfigEditor.test.tsx**:
- Test Monaco editor integration (markers appear on errors)
- Test hover tooltips (error messages displayed)
- Test keyboard shortcuts (F8 to jump to next error)
- Test Quick Fix actions (apply suggested fixes)

### E2E Tests

**config-validation.spec.ts**:
- Load editor → type invalid config → verify error squiggle appears
- Hover over error → verify tooltip shows message
- Click Quick Fix → verify edit applied
- Fix error → verify squiggle disappears
- Click "Apply Configuration" with errors → verify button disabled

## Performance Considerations

### Debouncing

Use 500ms debounce to avoid excessive WASM calls:

```typescript
const debouncedValidate = useMemo(
  () => debounce(async (content: string) => {
    await validate(content);
  }, 500),
  []
);
```

### Web Worker (Future Optimization)

For very large configs (>2000 lines), run validation in Web Worker to avoid blocking UI:

```typescript
// Future: validator-worker.ts
self.addEventListener('message', async (event) => {
  const { content } = event.data;
  const result = await validate(content);
  self.postMessage({ result });
});
```

### Incremental Validation (Future)

Only re-validate changed lines instead of entire file (requires Monaco's incremental parser).

## Dependencies

### Existing Infrastructure

- **WASM Module** (wasm-simulation-integration): load_config function
- **WasmCore API** (keyrx_ui/src/wasm/core.ts): TypeScript wrapper
- **React 18**: Hooks (useState, useEffect, useMemo)

### New Dependencies

- `@monaco-editor/react@^4.6.0` - Monaco editor React component
- `monaco-editor@^0.45.0` - Monaco editor core
- `lodash.debounce@^4.0.8` - Debounce utility

### Version Constraints

- TypeScript 5.0+ (for strict mode)
- React 18+ (for concurrent features)

## Code Quality Metrics

- **File Size Limits**:
  - validator.ts: ≤300 lines
  - useConfigValidator.ts: ≤150 lines
  - ConfigEditor.tsx: ≤400 lines
  - ValidationStatusPanel.tsx: ≤200 lines
- **Function Size**: ≤50 lines per function
- **Test Coverage**: ≥90% for validator.ts, ≥80% for UI components
- **TypeScript Strict Mode**: Enabled (no `any` types)
- **Accessibility**: 0 axe violations, WCAG 2.1 AA compliant

## Architecture Decisions

### Why Monaco Editor Instead of CodeMirror?

**Decision**: Use Monaco editor for configuration editing.

**Rationale**:
- Built-in TypeScript/JavaScript support (familiar to developers)
- Rich API for markers, hover providers, code actions
- Used by VSCode (proven scalability)
- Better WASM integration (can run language servers in Web Workers)

**Trade-offs**:
- Larger bundle size (~3MB) vs CodeMirror (~500KB)
- Accepted because config editor is not on critical path

### Why Debounce Instead of On-Blur Validation?

**Decision**: Validate after 500ms idle time instead of on blur.

**Rationale**:
- Users see errors faster (don't need to unfocus field)
- Matches IDE behavior (VSCode, IntelliJ)
- Research shows [inline validation reduces errors](https://www.smashingmagazine.com/2022/09/inline-validation-web-forms-ux/)

**Trade-offs**:
- More WASM calls (mitigated by debouncing)
- Accepted because validation is fast (<100ms)

### Why Reuse WASM Module Instead of Separate Validator?

**Decision**: Reuse load_config function from WASM module for validation.

**Rationale**:
- 100% parity with daemon validation (same parser, same errors)
- No code duplication
- Errors include exact line numbers from parser

**Trade-offs**:
- Tightly coupled to WASM module (if WASM fails, validation fails)
- Accepted because graceful degradation handles WASM failures

## Sources

This design incorporates UI/UX patterns from:
- [A Complete Guide To Live Validation UX — Smashing Magazine](https://www.smashingmagazine.com/2022/09/inline-validation-web-forms-ux/)
- [Error handling - UX design patterns](https://medium.com/design-bootcamp/error-handling-ux-design-patterns-c2a5bbae5f8d)
- [Inline Validation UX — Smart Interface Design Patterns](https://smart-interface-design-patterns.com/articles/inline-validation-ux/)
