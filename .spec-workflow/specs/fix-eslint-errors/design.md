# Design: Fix ESLint Errors

## Error Categories

### 1. `@typescript-eslint/no-explicit-any` (638 instances)
**Strategy**: Replace progressively
- Function parameters: Use proper interface types
- API responses: Use Zod schemas or interface types
- Event handlers: Use `React.MouseEvent`, `React.KeyboardEvent`
- Unknown data: Use `unknown` + type guards instead of `any`

### 2. `no-console` (~40 instances)
**Strategy**: Remove or guard
- Remove debug console.log statements
- Keep console.error/warn in error handlers
- Wrap in `if (import.meta.env.DEV)` for dev-only logs

### 3. `@typescript-eslint/no-unused-vars`
**Strategy**: Clean or mark
- Remove genuinely unused variables/imports
- Prefix intentionally unused with `_` (e.g., `_unusedParam`)

## Approach
1. Run `npm run lint` to get full error list
2. Group errors by file
3. Fix systematically file-by-file
4. Prioritize high-impact files (many errors)
5. Run tests after each file to catch breakage early
