# Tasks: Fix ESLint Errors

- [x] 1. Generate ESLint error report
  - Run `npm run lint -- --format json > eslint-report.json`
  - Analyze error distribution by file and rule
  - _Prompt: Role: Code Quality Analyst | Task: Implement the task for spec fix-eslint-errors. First run spec-workflow-guide, then implement: Generate ESLint report (npm run lint -- --format json). Analyze errors by file and rule. Identify top 10 files with most errors. Create prioritized fixing plan. | Restrictions: Just analysis, no code changes | Success: Report generated, errors categorized, top files identified | After: 1) Mark [-], 2) log-implementation, 3) Mark [x]_

- [x] 2. Fix `any` types in API/types files
  - Files: src/api/*.ts, src/types/*.ts
  - Replace `any` with proper types using Zod or interfaces
  - _Leverage: existing type definitions_
  - _Prompt: Role: TypeScript Developer | Task: Implement the task for spec fix-eslint-errors. First run spec-workflow-guide, then implement: Fix all `any` types in src/api/ and src/types/ files. Replace with proper interfaces, Zod schemas, or `unknown` + type guards. Run tests after each file. | Restrictions: Maintain type safety, don't break APIs, tests must pass | Success: All `any` in api/types files replaced, tests pass, TypeScript compiles | After: 1) Mark [-], 2) log-implementation, 3) Mark [x]_

- [x] 3. Fix `any` types in components
  - Files: src/components/*.tsx
  - Replace with React event types, prop interfaces
  - _Leverage: React type definitions_
  - _Prompt: Role: React TypeScript Developer | Task: Implement the task for spec fix-eslint-errors. First run spec-workflow-guide, then implement: Fix all `any` types in src/components/. Use React.MouseEvent, React.KeyboardEvent, etc. Define proper prop interfaces. Run tests after changes. | Restrictions: Don't break component APIs, tests must pass | Success: All `any` in components replaced, tests pass, no type errors | After: 1) Mark [-], 2) log-implementation, 3) Mark [x]_

- [x] 4. Fix `any` types in hooks and utils
  - Files: src/hooks/*.ts, src/utils/*.ts
  - Replace with proper types or generics
  - _Prompt: Role: TypeScript Developer | Task: Implement the task for spec fix-eslint-errors. First run spec-workflow-guide, then implement: Fix all `any` types in hooks and utils. Use generics where appropriate, proper types otherwise. Run tests. | Restrictions: Maintain hook/util APIs, tests must pass | Success: All `any` fixed, tests pass, no type errors | After: 1) Mark [-], 2) log-implementation, 3) Mark [x]_

- [x] 5. Remove or guard console statements
  - Remove debug console.log, guard necessary logs
  - _Prompt: Role: Code Quality Engineer | Task: Implement the task for spec fix-eslint-errors. First run spec-workflow-guide, then implement: Fix all `no-console` violations. Remove debug logs. Wrap dev-only logs in `if (import.meta.env.DEV)`. Keep error/warn in error handlers. | Restrictions: Don't remove useful error logging | Success: No console violations, appropriate logging preserved | After: 1) Mark [-], 2) log-implementation, 3) Mark [x]_

- [x] 6. Fix unused variables and imports
  - Remove unused or prefix with `_`
  - _Prompt: Role: Code Quality Engineer | Task: Implement the task for spec fix-eslint-errors. First run spec-workflow-guide, then implement: Fix all unused variable/import violations. Remove genuinely unused. Prefix intentionally unused with `_`. | Restrictions: Don't break functionality | Success: No unused var violations, code clean | After: 1) Mark [-], 2) log-implementation, 3) Mark [x]_

- [x] 7. Fix remaining ESLint errors
  - Fix any other error types found
  - _Prompt: Role: Code Quality Engineer | Task: Implement the task for spec fix-eslint-errors. First run spec-workflow-guide, then implement: Fix all remaining ESLint errors (triple-slash references, etc). Run `npm run lint` until 0 errors. | Restrictions: Fix properly, don't disable rules | Success: ESLint 0 errors/warnings | After: 1) Mark [-], 2) log-implementation, 3) Mark [x]_

- [x] 8. Run full test suite and verify
  - Ensure all tests pass, TypeScript compiles
  - _Prompt: Role: QA Engineer | Task: Implement the task for spec fix-eslint-errors. First run spec-workflow-guide, then implement: Run full test suite (npm test). Run TypeScript compiler (tsc --noEmit). Verify all pass. Fix any breakage. | Restrictions: All tests must pass | Success: Tests pass, TypeScript compiles, ESLint clean | After: 1) Mark [-], 2) log-implementation, 3) Mark [x]_
