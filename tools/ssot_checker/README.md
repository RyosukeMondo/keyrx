# SSOT Violation Checker

Static analyzer for detecting Single Source of Truth (SSOT) violations in the KeyRX codebase.

## What It Checks

### 1. Flutter Imports in core_logic
Detects when `core_logic` packages import Flutter dependencies, which violates the architecture principle that core logic should be pure Dart.

### 2. Business Logic in UI Layer
Identifies business logic that should be in `core_logic` but has been implemented in the UI layer:
- Direct FFI bridge usage outside of service wrappers
- Business logic methods (validate, calculate, transform, etc.) in non-widget classes

### 3. Interface Consistency
Verifies that FFI and Web implementations of services:
- Have matching method signatures
- Both implement the same interface
- Use conditional exports correctly

### 4. Duplicate Implementations
Finds classes and constants duplicated between `core_logic` and UI layers.

## Usage

### Run from project root:
```bash
dart run tools/ssot_checker/bin/check.dart
```

### Run with verbose output:
```bash
dart run tools/ssot_checker/bin/check.dart --verbose
```

### Run from different directory:
```bash
dart run tools/ssot_checker/bin/check.dart --path /path/to/keyrx
```

## Exit Codes

- `0`: No violations found
- `1`: Violations detected or error occurred

## Performance

The checker runs in under 200ms on the KeyRX codebase, making it suitable for pre-commit hooks.

## Integration

The SSOT checker can be integrated into:
- Pre-commit hooks (Task 3 in add-web-support spec)
- CI/CD pipelines
- IDE linting workflows

## Example Output

```
❌ SSOT Violations Detected:

━━━ Flutter Imports in core_logic (1) ━━━

[flutterInCoreLogic] packages/core_logic/lib/services/ui_service.dart:1: Flutter import detected in core_logic
  → Remove Flutter dependency, use pure Dart

━━━ Business Logic in UI (2) ━━━

[businessLogicInUI] ui/lib/services/device_service.dart:45: Direct FFI bridge usage detected in UI layer
  → Use services from core_logic instead

[businessLogicInUI] ui/lib/services/flow_validator.dart:71: Business logic detected in UI layer (class: FlowValidator)
  → Move business logic to core_logic service

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Total: 3 violation(s)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```
