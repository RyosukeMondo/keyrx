# Tasks: Fix Code Formatting

- [x] 1. Run Prettier on all files
  - Run `npm run format` to auto-format all 174 files
  - _Prompt: Role: Code Quality Engineer | Task: Implement the task for spec fix-code-formatting. First run spec-workflow-guide, then implement: Run `npm run format` to apply Prettier to all files. Verify formatting applied correctly. | Restrictions: No code logic changes | Success: Prettier applied, formatting consistent | After: 1) Mark [-], 2) log-implementation, 3) Mark [x]_

- [x] 2. Verify formatting and run tests
  - Run `npm run format:check` and `npm test`
  - _Prompt: Role: QA Engineer | Task: Implement the task for spec fix-code-formatting. First run spec-workflow-guide, then implement: Run `npm run format:check` (should pass 0 violations). Run `npm test` (all pass). Review git diff for formatting-only changes. Fix any issues. | Restrictions: Only formatting changes allowed | Success: Format check passes, tests pass, clean diff | After: 1) Mark [-], 2) log-implementation, 3) Mark [x]_
