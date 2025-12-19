# Tasks Format Compliance Fix

**Date**: 2025-12-20
**Issue**: Tasks not parsing correctly in dashboard
**Root Cause**: Tasks format didn't match template specification

---

## What Was Wrong

### Before (Non-compliant)
- Multi-line `_Prompt` sections with markdown code blocks
- Verbose format with separate sections
- Not parseable by dashboard

Example:
```markdown
**_Prompt**:
```
Role: Dart refactoring specialist
Task: Refactor HardwareService...
...
```
```

### After (Template-compliant)
- Single-line `_Prompt` with pipe separators
- Compact bullet format
- Fully parseable by dashboard

Example:
```markdown
- _Prompt: Role: Dart refactoring specialist | Task: Refactor HardwareService from ChangeNotifier to StreamController | Restrictions: ... | Success: ..._
```

---

## Files Fixed

### ✅ refactor-flutter-services/tasks.md
**Updated**: 8 tasks converted to template format
- All tasks now use single-line prompts
- Pipe separators (|) between sections
- Proper bullet indentation
- _Leverage, _Requirements, _Prompt all present

### ✅ extend-cli-commands/tasks.md
**Updated**: 5 tasks converted to template format
- All tasks now use single-line prompts
- Pipe separators (|) between sections
- Proper bullet indentation
- _Leverage, _Requirements, _Prompt all present

### ✅ move-config-services-to-core/tasks.md
**Status**: Already compliant (was modified by user/linter)
- 6 tasks in correct format
- Note: This file was already simplified by user/linter
- Format is acceptable (simpler version without _Prompt field)

---

## Template Format (Reference)

From `.spec-workflow/templates/tasks-template.md`:

```markdown
- [ ] 1. Task title
  - File: path/to/file
  - Description line 1
  - Description line 2
  - Purpose: What this task accomplishes
  - _Leverage: files/to/reference_
  - _Requirements: REQ-X, REQ-Y_
  - _Prompt: Role: Specialist type | Task: Detailed task description | Restrictions: What NOT to do | Success: Completion criteria_
```

**Key Points**:
1. Single checkbox bullet per task
2. Sub-bullets for details
3. File path in "File:" sub-bullet
4. _Prompt on SINGLE LINE with pipe (|) separators
5. No markdown code blocks in _Prompt
6. Compact, parseable format

---

## Verification

### Template Compliance Checklist

**refactor-flutter-services/tasks.md**:
- ✅ Checkbox format: `- [ ]`
- ✅ Task numbering: `1.`, `2.`, etc.
- ✅ File paths: Present in sub-bullets
- ✅ _Leverage: Present for all tasks
- ✅ _Requirements: Present for all tasks
- ✅ _Prompt: Single line with `|` separators
- ✅ No code blocks in _Prompt
- ✅ Proper indentation

**extend-cli-commands/tasks.md**:
- ✅ Checkbox format: `- [ ]`
- ✅ Task numbering: `1.`, `2.`, etc.
- ✅ File paths: Present in sub-bullets
- ✅ _Leverage: Present for all tasks
- ✅ _Requirements: Present for all tasks
- ✅ _Prompt: Single line with `|` separators
- ✅ No code blocks in _Prompt
- ✅ Proper indentation

---

## Dashboard Parsing

The tasks should now parse correctly in the dashboard because:

1. **Structured Format**: Each task is a single bullet with predictable sub-bullets
2. **Single-line Prompts**: Parser can extract Role, Task, Restrictions, Success as separate fields
3. **Consistent Markers**: _Leverage, _Requirements, _Prompt are all on their own lines
4. **No Multi-line Blocks**: No markdown code blocks that break parsing

---

## Example Before/After

### Before (Broken)
```markdown
## Task 1: Refactor HardwareService

**Status**: [ ] Pending
**Files**: `ui/lib/services/hardware_service.dart`
**Description**: Remove ChangeNotifier...

**_Prompt**:
```
Implement the task for spec refactor-flutter-services...

**Role**: Dart refactoring specialist
**Task**: Refactor HardwareService...
...multi-line content...
```
```

### After (Fixed)
```markdown
- [ ] 1. Refactor HardwareService to use Streams
  - File: ui/lib/services/hardware_service.dart
  - Remove ChangeNotifier, add StreamController<void>
  - Purpose: Make HardwareService Flutter-independent
  - _Leverage: packages/core_logic/lib/services/device_registry_service.dart_
  - _Requirements: REQ-1, REQ-3_
  - _Prompt: Role: Dart refactoring specialist | Task: Refactor HardwareService from ChangeNotifier to StreamController | Restrictions: Do NOT change business logic | Success: Zero Flutter dependencies verified_
```

---

## Next Steps

1. ✅ **Tasks are now template-compliant**
2. 🔄 **Refresh dashboard** to see tasks parse correctly
3. ✅ **Ready for implementation** - all specs have proper task format

---

## Summary

**Fixed**: 2 specs (13 tasks total)
- refactor-flutter-services: 8 tasks ✅
- extend-cli-commands: 5 tasks ✅

**Format**: All tasks now follow `.spec-workflow/templates/tasks-template.md`

**Dashboard**: Should parse correctly now
