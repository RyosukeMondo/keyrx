# Module Splitting Guide

This guide explains how to use the `module_splitter` tool to automatically split large Rust modules into smaller, more manageable files.

## Table of Contents

- [Overview](#overview)
- [Why Split Modules?](#why-split-modules)
- [Installation](#installation)
- [Usage](#usage)
  - [Analyze Command](#analyze-command)
  - [Preview Command](#preview-command)
  - [Apply Command](#apply-command)
- [Grouping Strategies](#grouping-strategies)
- [Best Practices](#best-practices)
- [Examples](#examples)
- [Troubleshooting](#troubleshooting)

## Overview

The `module_splitter` tool is a Rust-based utility that automatically analyzes and splits large Rust modules into smaller, focused files. It uses intelligent grouping strategies to organize functions logically while ensuring each resulting module stays under a configurable size limit (default: 500 lines).

## Why Split Modules?

Large modules (>500 lines) create several problems:

1. **AI Context Window Limitations**: Large files exceed LLM context windows, reducing AI code generation accuracy
2. **Reduced Comprehension**: Difficult for developers to understand and navigate
3. **Merge Conflicts**: More likely when multiple developers work on the same large file
4. **Slower Compilation**: Large modules can slow down incremental compilation

**Target**: Keep all modules under 500 lines of code (excluding comments and blank lines)

## Installation

The tool is part of the KeyRx workspace and will be built automatically when you build the project:

```bash
cargo build --package module_splitter
```

Alternatively, use the convenience wrapper script:

```bash
./scripts/split_ffi_module.sh --help
```

## Usage

The tool provides three main commands:

### Analyze Command

Analyze a module to see its current size and statistics:

```bash
# Using the Rust tool directly
cargo run --package module_splitter -- analyze <module-file> [--format text|json]

# Using the wrapper script
./scripts/split_ffi_module.sh analyze core/src/ffi/exports_engine.rs
```

**Output:**
```
Analyzing core/src/ffi/exports_engine.rs

Module Statistics:
  Total lines: 400
  Code lines: 323
  Functions: 15
  Types: 5
  Imports: 12

✓ Module is within size limit (323 <= 500)
```

**JSON Output:**
```bash
./scripts/split_ffi_module.sh analyze core/src/ffi/exports_engine.rs --format json
```

### Preview Command

Preview how the module would be split without making any changes:

```bash
# Using the wrapper script
./scripts/split_ffi_module.sh preview core/src/ffi/exports_engine.rs

# With custom max size
./scripts/split_ffi_module.sh preview core/src/ffi/exports_engine.rs --max-size 400
```

**Output:**
```
Analyzing core/src/ffi/exports_engine.rs

Planning Generating split plan...

Split Plan:
  Will create 3 new modules:

  1. engine (150 lines, 5 functions)
     Functions:
       - start_engine
       - stop_engine
       - pause_engine
       - resume_engine
       - restart_engine

  2. state (120 lines, 4 functions)
     Functions:
       - get_state
       - get_status
       - get_metrics
       - get_config

  3. script (100 lines, 6 functions)
     Functions:
       - load_script
       - unload_script
       - reload_script
       - validate_script
       - list_scripts
       - get_script_info

  Total estimated lines: 370
  Original code lines: 323
```

### Apply Command

Apply the split and create new module files:

```bash
# With confirmation prompt
./scripts/split_ffi_module.sh apply core/src/ffi/exports_engine.rs

# Skip confirmation
./scripts/split_ffi_module.sh apply core/src/ffi/exports_engine.rs -y

# With custom max size
./scripts/split_ffi_module.sh apply core/src/ffi/exports_engine.rs --max-size 400 -y
```

**What happens:**

1. Creates a backup of the original file (`exports_engine.rs.backup`)
2. Creates a new directory (e.g., `core/src/ffi/engine/`)
3. Generates individual module files (e.g., `engine.rs`, `state.rs`, `script.rs`)
4. Creates a `mod.rs` file that re-exports all functions
5. Preserves all imports, attributes, and documentation

**Output:**
```
Split Complete!
  Created 3 files:
    ✓ core/src/ffi/engine/engine.rs
    ✓ core/src/ffi/engine/state.rs
    ✓ core/src/ffi/engine/script.rs
  Created mod.rs: core/src/ffi/engine/mod.rs

Next Steps:
  1. Review the generated files
  2. Run 'cargo check' to verify compilation
  3. Run tests to ensure functionality is preserved
  4. Update imports in dependent modules
```

## Grouping Strategies

The tool uses intelligent grouping strategies in order of preference:

### 1. By Domain (FFI Functions)

If the module contains functions with `#[ffi_export(domain = "...")]` annotations, they will be grouped by domain:

```rust
#[ffi_export(domain = "engine")]
pub fn start_engine() -> Result<()> { }

#[ffi_export(domain = "engine")]
pub fn stop_engine() -> Result<()> { }

#[ffi_export(domain = "runtime")]
pub fn get_state() -> Result<State> { }
```

Result: Two groups - `engine` and `runtime`

### 2. By Prefix (Standard Functions)

If no FFI annotations are found, functions are grouped by their name prefix:

```rust
pub fn get_state() -> State { }
pub fn get_config() -> Config { }
pub fn set_state(state: State) { }
pub fn set_config(config: Config) { }
```

Result: Two groups - `get` and `set`

### 3. Automatic Splitting (Oversized Groups)

If a group exceeds the max size limit, it will be automatically split into multiple numbered subgroups:

- `engine_1.rs`
- `engine_2.rs`
- etc.

## Best Practices

### Before Splitting

1. **Ensure tests pass**: Run `cargo test` to verify current functionality
2. **Commit your changes**: Ensure you have a clean git state
3. **Review the preview**: Always preview before applying

### After Splitting

1. **Verify compilation**: Run `cargo check` or `cargo build`
2. **Run tests**: Ensure all tests still pass
3. **Update dependent modules**: Update `use` statements in other files that import from the split module
4. **Review generated code**: Check that the split makes logical sense
5. **Update documentation**: If necessary, update module-level docs

### Naming Conventions

The tool automatically sanitizes module names:

- Removes `exports_` prefix: `exports_engine.rs` → `engine/`
- Converts to lowercase: `FooBar` → `foobar`
- Replaces hyphens/spaces with underscores: `foo-bar` → `foo_bar`

## Examples

### Example 1: Split a Large FFI Module

```bash
# 1. Analyze the module
./scripts/split_ffi_module.sh analyze core/src/ffi/exports_engine.rs

# 2. Preview the split
./scripts/split_ffi_module.sh preview core/src/ffi/exports_engine.rs

# 3. Apply the split
./scripts/split_ffi_module.sh apply core/src/ffi/exports_engine.rs -y

# 4. Verify compilation
cargo check

# 5. Run tests
cargo test
```

### Example 2: Custom Max Size

For extremely large modules or stricter limits:

```bash
./scripts/split_ffi_module.sh apply core/src/api.rs --max-size 300 -y
```

### Example 3: Programmatic Usage (JSON Output)

For integration with other tools or scripts:

```bash
./scripts/split_ffi_module.sh analyze core/src/ffi/exports_engine.rs --format json | jq .
```

## Troubleshooting

### Issue: "Module is already within size limit"

**Cause**: The module is already under the max size threshold.

**Solution**: The module doesn't need splitting. If you still want to split it, lower the `--max-size` value.

### Issue: "Failed to parse module"

**Cause**: The Rust file contains syntax errors.

**Solution**: Fix syntax errors first with `cargo check`, then retry.

### Issue: "Imports resolve incorrectly after split"

**Cause**: The generated modules may need additional imports, or other files need to update their import paths.

**Solution**:
1. Check the generated `mod.rs` for re-exports
2. Update dependent modules to import from the new path
3. Run `cargo check` to find missing imports

### Issue: "Tests fail after split"

**Cause**: Module visibility or import paths may have changed.

**Solution**:
1. Check if any functions became private unintentionally
2. Update test imports to use the new module structure
3. Ensure `mod.rs` properly re-exports all public items

## Integration with Pre-commit Hooks

The module size enforcement can be integrated with pre-commit hooks:

```bash
# In .githooks/pre-commit
for file in $(git diff --cached --name-only --diff-filter=ACM | grep '\.rs$'); do
    # Check module size
    code_lines=$(tokei "$file" --output json | jq '.Rust.code')
    if [ "$code_lines" -gt 500 ]; then
        echo "Error: $file exceeds 500 lines ($code_lines lines)"
        echo "Please split it using: ./scripts/split_ffi_module.sh preview $file"
        exit 1
    fi
done
```

## Development

### Running Tests

```bash
cargo test --package module_splitter
```

### Building Release Version

```bash
cargo build --package module_splitter --release
```

The release binary will be at `core/tools/module_splitter/target/release/module_splitter`.

## See Also

- [FFI Development Guide](./FFI_DEVELOPMENT.md)
- [AI Agent Directives](../.keyrx/agent.md)
- [Module Size Analysis](../.spec-workflow/analysis/module-sizes.json)
