# keyrx_core

Platform-agnostic keyboard remapping logic library.

## Purpose

`keyrx_core` contains the core remapping engine that is OS-agnostic and WASM-compatible. It implements:

- **Configuration loading**: Zero-copy deserialization of `.krx` binary files using rkyv
- **Key lookup**: O(1) lookup using Minimal Perfect Hash Functions (MPHF) via boomphf
- **DFA state machine**: Tap/hold behavior and key sequence processing
- **Extended state**: 255 modifiers + 255 locks using fixedbitset
- **Keyboard simulation**: Browser-based testing via WASM

## Architecture

This crate is `no_std` to ensure it can be compiled to any target, including:
- Linux daemon (x86_64-unknown-linux-gnu)
- Windows daemon (x86_64-pc-windows-msvc)
- Browser WASM (wasm32-unknown-unknown)

## Dependencies

- `rkyv`: Zero-copy deserialization
- `boomphf`: MPHF generation (CHD algorithm)
- `fixedbitset`: Compact bitset for state management
- `arrayvec`: Fixed-capacity vectors (no heap allocation)

## Usage

```rust
use keyrx_core::config;
use keyrx_core::simulator;

// Load configuration
// Simulate key processing
// (Implementation to follow)
```

## Testing

Run tests:
```bash
cargo test --package keyrx_core
```

Run benchmarks:
```bash
cargo bench --package keyrx_core
```

Run fuzz tests:
```bash
cd keyrx_core/fuzz
cargo fuzz run fuzz_target_1
```
