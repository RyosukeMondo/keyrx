# FFI Versioning and Breaking Change Policy

This document describes the versioning strategy and breaking change policy for KeyRx FFI (Foreign Function Interface) contracts.

## Table of Contents

- [Overview](#overview)
- [Semantic Versioning](#semantic-versioning)
- [Breaking vs. Compatible Changes](#breaking-vs-compatible-changes)
- [Breaking Change Workflow](#breaking-change-workflow)
- [CI/CD Integration](#cicd-integration)
- [Best Practices](#best-practices)

## Overview

KeyRx uses FFI contracts to define the interface between the Rust core and the Dart UI. These contracts are versioned using semantic versioning to ensure backward compatibility and clear communication of changes.

The FFI breaking change detection system automatically analyzes contract modifications and enforces version bump requirements when breaking changes are introduced.

## Semantic Versioning

We follow [Semantic Versioning 2.0.0](https://semver.org/) for FFI contracts:

```
MAJOR.MINOR.PATCH
```

- **MAJOR**: Breaking changes that require UI code updates
- **MINOR**: Backward-compatible additions (new functions, optional parameters)
- **PATCH**: Non-breaking changes (documentation, internal improvements)

### Version Location

The FFI version is tracked in `core/Cargo.toml`:

```toml
[package]
name = "keyrx_core"
version = "0.3.0"  # ← FFI version
```

## Breaking vs. Compatible Changes

### Major (Breaking) Changes

Changes that **require** client code updates:

- ❌ **Removing a function**
  ```diff
  - keyrx_engine_get_state()
  ```

- ❌ **Changing parameter types**
  ```diff
  - fn set_volume(level: u8)
  + fn set_volume(level: f32)
  ```

- ❌ **Changing return types**
  ```diff
  - fn get_state() -> String
  + fn get_state() -> Result<String>
  ```

- ❌ **Removing parameters**
  ```diff
  - fn create_device(name: String, config: String)
  + fn create_device(name: String)
  ```

- ❌ **Adding required parameters**
  ```diff
  - fn create_device(name: String)
  + fn create_device(name: String, config: String)  // config is required
  ```

- ❌ **Changing function domain**
  ```diff
  - domain: "engine"
  + domain: "runtime"
  ```

- ❌ **Making optional parameter required**
  ```diff
  - parameter: { name: "config", required: false }
  + parameter: { name: "config", required: true }
  ```

### Minor (Compatible) Changes

Backward-compatible additions:

- ✅ **Adding new functions**
  ```diff
  + keyrx_engine_get_metrics()
  ```

- ✅ **Adding optional parameters**
  ```diff
  - fn create_device(name: String)
  + fn create_device(name: String, config: Option<String>)
  ```

- ✅ **Making required parameter optional**
  ```diff
  - parameter: { name: "config", required: true }
  + parameter: { name: "config", required: false }
  ```

- ✅ **Adding new error types**
  ```diff
  + errors: ["DeviceNotFound"]
  ```

### Patch Changes

Non-breaking improvements:

- ✅ **Updating documentation**
  ```diff
  - description: "Get state"
  + description: "Get the current engine state as JSON"
  ```

- ✅ **Renaming parameters** (if type unchanged)
  ```diff
  - parameter: { name: "cfg", type: "string" }
  + parameter: { name: "config", type: "string" }
  ```

- ✅ **Updating metadata**
  ```diff
  - version: "1.0.0"
  + version: "1.0.1"
  ```

## Breaking Change Workflow

### 1. Detect Breaking Changes

Run the detection tool locally:

```bash
# Compare against previous commit
just diff-ffi-contracts

# Compare against main branch
just diff-ffi-contracts main

# Output JSON format
just diff-ffi-contracts HEAD~1 json

# Output Markdown format
just diff-ffi-contracts HEAD~1 markdown
```

### 2. Handle Breaking Changes

If breaking changes are detected, you have two options:

#### Option A: Bump Version (Recommended)

Update the version in `core/Cargo.toml`:

```toml
[package]
version = "1.0.0"  # Major bump for breaking changes
```

Follow these rules:
- **Major breaking changes**: Bump MAJOR version (1.2.3 → 2.0.0)
- **Minor compatible additions**: Bump MINOR version (1.2.3 → 1.3.0)
- **Patch non-breaking changes**: Bump PATCH version (1.2.3 → 1.2.4)

#### Option B: Allow Breaking Changes (PR Label)

For intentional breaking changes that are acknowledged but don't warrant an immediate version bump:

1. Add the `allow-breaking` label to your PR
2. Document the breaking changes in the PR description
3. Include a migration guide for UI developers

### 3. CI Validation

The CI pipeline automatically:

1. Detects FFI contract changes
2. Classifies changes (major/minor/patch)
3. Checks if version was bumped (for breaking changes)
4. Posts a comment on PRs with breaking changes
5. Blocks PRs if breaking changes without version bump or `allow-breaking` label

## CI/CD Integration

### GitHub Actions Workflow

The `.github/workflows/ffi-breaking-changes.yml` workflow runs on:
- Every PR that modifies `core/src/ffi/**`
- Every push to `main` that modifies FFI contracts

### PR Comments

When breaking changes are detected, the CI posts a comment:

```markdown
## ⚠️ FFI Breaking Changes Detected

Found **3** breaking change(s) in FFI contracts.

❌ **Action Required**: Either bump the version in `core/Cargo.toml` or add the `allow-breaking` label to this PR.

<details>
<summary>Breaking Changes Report</summary>

### ⚠️ MAJOR BREAKING CHANGES (3)

- engine.ffi-contract.json: function_removed - Function removed: get_legacy_state
- runtime.ffi-contract.json: param_type_changed - add_slot.priority: uint32 -> int32
- config.ffi-contract.json: return_type_changed - get_config: string -> Result<string>

</details>
```

### Quality Gates

PRs are blocked if:
- Breaking changes detected **AND**
- Version not bumped **AND**
- No `allow-breaking` label

## Best Practices

### 1. Avoid Breaking Changes

Prefer backward-compatible changes:

```rust
// ❌ Breaking: changing signature
#[ffi_export(domain = "engine")]
fn set_volume(level: f32) { /* ... */ }

// ✅ Compatible: add new function
#[ffi_export(domain = "engine")]
fn set_volume_precise(level: f32) { /* ... */ }

// Keep old function deprecated
#[deprecated(since = "2.0.0", note = "Use set_volume_precise instead")]
#[ffi_export(domain = "engine")]
fn set_volume(level: u8) { /* ... */ }
```

### 2. Use Optional Parameters

Make new parameters optional when possible:

```rust
// ✅ Compatible: optional parameter
#[ffi_export(domain = "engine")]
fn create_device(
    name: String,
    config: Option<String>,  // ← optional, backward compatible
) -> Result<String> {
    /* ... */
}
```

### 3. Deprecation Path

For breaking changes, provide a deprecation path:

1. Mark old function as deprecated
2. Add new function with improved signature
3. Update documentation with migration guide
4. Remove deprecated function in next major version

### 4. Document Breaking Changes

Always document breaking changes in:
- PR description
- CHANGELOG.md
- Migration guide (for major versions)
- Implementation log

### 5. Test Before Bumping

Before bumping version:
1. Run `just diff-ffi-contracts` to verify changes
2. Test UI with new contracts
3. Update UI code if needed
4. Verify all tests pass

## Examples

### Example 1: Adding Optional Parameter (Minor)

```diff
# contract change
  "parameters": [
    {"name": "name", "type": "string", "required": true},
+   {"name": "priority", "type": "uint32", "required": false}
  ]
```

**Action**: Bump MINOR version (1.2.0 → 1.3.0)

### Example 2: Removing Function (Major)

```diff
# contract change
- {
-   "name": "get_legacy_config",
-   "rust_name": "keyrx_config_get_legacy",
-   ...
- }
```

**Action**: Bump MAJOR version (1.2.0 → 2.0.0) + migration guide

### Example 3: Updating Documentation (Patch)

```diff
# contract change
  "description": {
-   "Get config"
+   "Get the current runtime configuration as JSON"
  }
```

**Action**: Bump PATCH version (1.2.0 → 1.2.1)

## Troubleshooting

### False Positives

If the tool reports false positives:
1. Review the contract changes carefully
2. Check if JSON structure changed unexpectedly
3. Verify parameter types and names
4. Report the issue with examples

### CI Blocking Legitimate Changes

If CI blocks legitimate changes:
1. Add the `allow-breaking` label
2. Document why breaking changes are acceptable
3. Include migration instructions in PR

### Version Bump Not Detected

If you bumped the version but CI still complains:
1. Verify the version in `core/Cargo.toml` was changed
2. Check that the commit includes the version change
3. Ensure the base reference is correct (usually `main`)

## Related Documentation

- [FFI Development Guide](./FFI_DEVELOPMENT.md)
- [AI Agent Guide](./AI_AGENT_GUIDE.md)
- [Contract Schema](./.spec-workflow/steering/structure.md)
- [Module Organization](./FFI_MODULE_CONVENTIONS.md)

## Scripts and Tools

- `scripts/detect_ffi_breaking_changes.sh` - Breaking change detection tool
- `just diff-ffi-contracts` - Justfile recipe for detection
- `.github/workflows/ffi-breaking-changes.yml` - CI workflow

## Support

For questions or issues:
1. Check this documentation first
2. Review recent PRs with breaking changes
3. Ask in team chat or open a discussion
4. Report bugs in issue tracker
