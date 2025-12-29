# Requirements: MPHF Lookup System

## Introduction

The Minimal Perfect Hash Function (MPHF) Lookup System is a performance-critical enhancement to the KeyRx runtime. It replaces the current `HashMap`-based key lookup with a deterministic, constant-time (O(1)) lookup mechanism generated at compile-time. This ensures that the system meets its hard latency requirement of <1ms (target <100μs) even with thousands of remapping rules, by providing sub-50ns lookup speeds with zero hash collisions and optimal cache locality.

## Alignment with Product Vision

- **Sub-Millisecond Latency**: Guarantees O(1) lookup performance regardless of configuration size.
- **Complete Determinism**: The hash function is generated at compile-time and is identical across all platforms for a given configuration.
- **Zero-Cost Abstractions**: Moves the overhead of hash table construction from runtime to the compiler.
- **AI-First Verification**: Enables consistent performance benchmarking and formal validation of lookup costs.

## Requirements

### Requirement 1: Compile-Time MPHF Generation

**User Story:** As a compiler, I want to generate a minimal perfect hash function for all input keys in a configuration, so that the runtime can perform lookups without collisions.

#### Acceptance Criteria

1. THE compiler SHALL identify all unique input `KeyCode`s across all device configurations.
2. THE compiler SHALL generate an MPHF using the CHD (Compress, Hash and Displace) algorithm (via `boomphf`).
3. THE generated MPHF SHALL be "minimal", meaning the hash range is exactly equal to the number of input keys (no gaps in the resulting array).
4. THE generation process SHALL be deterministic: same input keys → same MPHF parameters.

### Requirement 2: O(1) Runtime Lookup

**User Story:** As the remapping daemon, I want to resolve input keys to their mapping indices in constant time, so that I can maintain sub-microsecond processing latency.

#### Acceptance Criteria

1. THE runtime SHALL use the generated MPHF to calculate the index of a `KeyCode` in the remapping table.
2. THE lookup SHALL be truly O(1) with zero collisions (guaranteed by MPHF).
3. THE lookup SHALL handle unmapped keys by verifying the key at the hashed index matches the input key (since MPHFs may map non-members to valid indices).
4. THE lookup performance SHALL be <50ns for most keys on target hardware.

### Requirement 3: Binary Serialization Integration

**User Story:** As a developer, I want the MPHF parameters to be serialized into the `.krx` binary, so that the daemon can load and use them without regeneration.

#### Acceptance Criteria

1. THE `.krx` format SHALL include the necessary parameters for the MPHF (e.g., hash seeds, displacement values).
2. THE parameters SHALL be serialized using `rkyv` for zero-copy access.
3. THE `KeyLookup` structure in `keyrx_core` SHALL be compatible with the serialized parameters.

## Non-Functional Requirements

### Code Architecture and Modularity
- **Separation of Concerns**: The generation logic (compiler) and evaluation logic (core) shall be clearly separated.
- **No Heap Allocation**: The runtime lookup implementation in `keyrx_core` shall not require heap allocation (compatible with `no_std`).

### Performance
- **Lookup Latency**: Sub-50ns average lookup time.
- **Binary Size**: The overhead of MPHF parameters should be proportional to the number of keys (typically <8 bytes per key).

### Reliability
- **Safety**: The implementation shall avoid `unsafe` code where possible, or clearly document and justify any necessary unsafe optimizations.
- **Robustness**: The system must handle cases with 0, 1, or 10,000+ keys gracefully.

### Usability
- **Transparency**: The compiler should report the efficiency/quality of the generated MPHF in debug/verbose mode.
