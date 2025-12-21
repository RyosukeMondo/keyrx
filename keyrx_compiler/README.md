# keyrx_compiler

Rhai-to-binary configuration compiler.

## Purpose

`keyrx_compiler` is a standalone CLI tool that compiles Rhai DSL configuration scripts into static `.krx` binary files. It performs:

- **Rhai parsing**: Parse configuration scripts written in Rhai DSL
- **MPHF generation**: Generate Minimal Perfect Hash Functions for O(1) key lookup
- **DFA generation**: Create Deterministic Finite Automaton state machines
- **Binary serialization**: Output `.krx` files using rkyv zero-copy format

## Dependencies

- `rhai`: Embedded scripting language for configuration DSL
- `serde`: Intermediate serialization before rkyv conversion
- `clap`: CLI argument parsing with derive macros

## Usage

```bash
# Compile a Rhai configuration to .krx binary
keyrx_compiler config.rhai -o config.krx

# Verbose output
keyrx_compiler config.rhai -o config.krx --verbose

# Display help
keyrx_compiler --help
```

## Example Configuration

```rhai
// Example Rhai configuration (syntax to be defined)
// This will be compiled to a static .krx binary
```

## Testing

Run tests:
```bash
cargo test --package keyrx_compiler
```

Run integration tests:
```bash
cargo test --test '*' --package keyrx_compiler
```
