# Design Document

## Overview

This design creates a hardened Rhai sandbox with resource limits, capability tiers, and O(1) registry lookup. The core innovation is the `ScriptCapability` system that assigns tiers to functions at compile time, and the `ResourceBudget` that tracks runtime resource consumption. All function calls go through a validation layer.

## Steering Document Alignment

### Technical Standards (tech.md)
- **Security**: No privilege escalation, resource limits
- **Performance**: O(1) lookup, minimal validation overhead
- **Error Handling**: Clear validation errors

### Project Structure (structure.md)
- Sandbox in `core/src/scripting/sandbox/`
- Capabilities in `core/src/scripting/capabilities/`
- Validation in `core/src/scripting/validation/`

## Code Reuse Analysis

### Existing Components to Leverage
- **Rhai Engine**: Built-in limits configuration
- **Existing bindings**: Wrap with validation
- **Error types**: Extend for validation errors

### Integration Points
- **Rhai registration**: Add capability metadata
- **Function calls**: Validate through sandbox
- **Engine**: Configure limits

## Architecture

```mermaid
graph TD
    subgraph "Script Execution"
        SC[Script Call] --> |validate| VL[Validation Layer]
        VL --> |check| CAP[Capability Check]
        CAP --> |allowed| REG[Registry Lookup]
        REG --> |O(1)| FN[Function]
    end

    subgraph "Resource Management"
        FN --> |track| RB[ResourceBudget]
        RB --> |exceeded| TERM[Terminate]
        RB --> |ok| EXEC[Execute]
    end

    subgraph "Capability System"
        CAP --> TIER[Capability Tier]
        TIER --> SAFE[Safe Mode]
        TIER --> STD[Standard Mode]
        TIER --> ADV[Advanced Mode]
    end
```

### Modular Design Principles
- **Defense in Depth**: Multiple safety layers
- **Fail Closed**: Reject on any doubt
- **Observable**: All rejections logged
- **Performant**: Minimal overhead

## Components and Interfaces

### Component 1: ScriptCapability

- **Purpose:** Categorize function security tiers
- **Interfaces:**
  ```rust
  /// Capability tier for script functions.
  #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
  pub enum ScriptCapability {
      /// Safe for any script - no side effects, bounded execution.
      Safe = 0,
      /// Standard operations - may affect engine state.
      Standard = 1,
      /// Advanced operations - system interaction, requires trust.
      Advanced = 2,
      /// Internal only - not exposed to user scripts.
      Internal = 3,
  }

  impl ScriptCapability {
      /// Check if this capability is allowed in given mode.
      pub fn is_allowed_in(&self, mode: ScriptMode) -> bool;

      /// Get human-readable description.
      pub fn description(&self) -> &'static str;
  }

  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  pub enum ScriptMode {
      Safe,      // Only Safe tier
      Standard,  // Safe + Standard tiers
      Full,      // All tiers except Internal
  }
  ```
- **Dependencies:** None
- **Reuses:** Enum patterns

### Component 2: ResourceBudget

- **Purpose:** Track and limit resource consumption
- **Interfaces:**
  ```rust
  /// Resource budget for script execution.
  pub struct ResourceBudget {
      instruction_limit: u64,
      instruction_count: AtomicU64,
      recursion_limit: u32,
      recursion_depth: AtomicU32,
      memory_limit: usize,
      memory_used: AtomicUsize,
      timeout: Duration,
      start_time: Instant,
  }

  impl ResourceBudget {
      pub fn new(config: ResourceConfig) -> Self;

      /// Check if instruction budget exhausted.
      pub fn check_instructions(&self) -> Result<(), ResourceExhausted>;

      /// Increment instruction count.
      pub fn increment_instructions(&self, count: u64) -> Result<(), ResourceExhausted>;

      /// Enter recursion level.
      pub fn enter_recursion(&self) -> Result<RecursionGuard, ResourceExhausted>;

      /// Allocate memory.
      pub fn allocate(&self, bytes: usize) -> Result<(), ResourceExhausted>;

      /// Check timeout.
      pub fn check_timeout(&self) -> Result<(), ResourceExhausted>;

      /// Get current usage.
      pub fn usage(&self) -> ResourceUsage;
  }

  pub struct ResourceConfig {
      pub max_instructions: u64,
      pub max_recursion: u32,
      pub max_memory: usize,
      pub timeout: Duration,
  }

  impl Default for ResourceConfig {
      fn default() -> Self {
          Self {
              max_instructions: 100_000,
              max_recursion: 64,
              max_memory: 1024 * 1024, // 1MB
              timeout: Duration::from_millis(100),
          }
      }
  }

  #[derive(Debug, thiserror::Error)]
  pub enum ResourceExhausted {
      #[error("Instruction limit exceeded ({count}/{limit})")]
      Instructions { count: u64, limit: u64 },
      #[error("Recursion limit exceeded ({depth}/{limit})")]
      Recursion { depth: u32, limit: u32 },
      #[error("Memory limit exceeded ({used}/{limit} bytes)")]
      Memory { used: usize, limit: usize },
      #[error("Script timeout ({elapsed:?}/{timeout:?})")]
      Timeout { elapsed: Duration, timeout: Duration },
  }
  ```
- **Dependencies:** std::sync::atomic
- **Reuses:** Resource tracking patterns

### Component 3: CapabilityRegistry

- **Purpose:** O(1) lookup of function capabilities
- **Interfaces:**
  ```rust
  /// Registry mapping functions to capabilities.
  pub struct CapabilityRegistry {
      by_name: HashMap<String, FunctionCapability>,
      by_keycode: HashMap<KeyCode, Vec<FunctionCapability>>,
  }

  #[derive(Debug, Clone)]
  pub struct FunctionCapability {
      pub name: String,
      pub capability: ScriptCapability,
      pub validator: Option<Box<dyn InputValidator>>,
      pub description: String,
  }

  impl CapabilityRegistry {
      pub fn new() -> Self;

      /// Register a function with capability.
      pub fn register(&mut self, cap: FunctionCapability);

      /// Get capability for function name - O(1).
      pub fn get(&self, name: &str) -> Option<&FunctionCapability>;

      /// Get functions for KeyCode - O(1).
      pub fn for_keycode(&self, key: KeyCode) -> &[FunctionCapability];

      /// Check if function is allowed in mode.
      pub fn is_allowed(&self, name: &str, mode: ScriptMode) -> bool;

      /// Get all functions for capability tier.
      pub fn by_tier(&self, tier: ScriptCapability) -> Vec<&FunctionCapability>;
  }
  ```
- **Dependencies:** HashMap
- **Reuses:** Registry patterns

### Component 4: InputValidator

- **Purpose:** Validate function inputs
- **Interfaces:**
  ```rust
  /// Trait for input validation.
  pub trait InputValidator: Send + Sync {
      fn validate(&self, args: &[Dynamic]) -> Result<(), ValidationError>;
  }

  #[derive(Debug, thiserror::Error)]
  pub enum ValidationError {
      #[error("Wrong number of arguments: expected {expected}, got {got}")]
      WrongArgCount { expected: usize, got: usize },
      #[error("Invalid argument type at position {position}: expected {expected}, got {got}")]
      InvalidType { position: usize, expected: String, got: String },
      #[error("Invalid value at position {position}: {message}")]
      InvalidValue { position: usize, message: String },
      #[error("Validation failed: {message}")]
      Custom { message: String },
  }

  /// Common validators
  pub struct RangeValidator {
      pub position: usize,
      pub min: i64,
      pub max: i64,
  }

  pub struct TypeValidator {
      pub expected_types: Vec<(&'static str, TypeId)>,
  }

  pub struct KeyCodeValidator;

  impl InputValidator for RangeValidator { ... }
  impl InputValidator for TypeValidator { ... }
  impl InputValidator for KeyCodeValidator { ... }
  ```
- **Dependencies:** rhai::Dynamic
- **Reuses:** Validation patterns

### Component 5: ScriptSandbox

- **Purpose:** Unified sandbox for script execution
- **Interfaces:**
  ```rust
  /// Sandboxed script execution environment.
  pub struct ScriptSandbox {
      engine: Engine,
      capabilities: CapabilityRegistry,
      mode: ScriptMode,
      resource_config: ResourceConfig,
  }

  impl ScriptSandbox {
      pub fn new(mode: ScriptMode) -> Self;

      /// Configure resource limits.
      pub fn with_resources(mut self, config: ResourceConfig) -> Self;

      /// Register a function with capability.
      pub fn register_fn<A, R, F>(&mut self, name: &str, cap: ScriptCapability, f: F)
      where
          F: Fn(A) -> R + Send + Sync + 'static;

      /// Execute script with sandbox.
      pub fn execute(&self, script: &str) -> Result<Dynamic, ScriptError>;

      /// Execute function call with validation.
      pub fn call_fn(
          &self,
          name: &str,
          args: &[Dynamic],
      ) -> Result<Dynamic, ScriptError>;
  }

  #[derive(Debug, thiserror::Error)]
  pub enum ScriptError {
      #[error("Capability denied: {function} requires {required:?}, mode is {mode:?}")]
      CapabilityDenied {
          function: String,
          required: ScriptCapability,
          mode: ScriptMode,
      },
      #[error("Resource exhausted: {0}")]
      ResourceExhausted(#[from] ResourceExhausted),
      #[error("Validation failed: {0}")]
      ValidationFailed(#[from] ValidationError),
      #[error("Execution error: {0}")]
      Execution(#[from] Box<rhai::EvalAltResult>),
  }
  ```
- **Dependencies:** Rhai, CapabilityRegistry, ResourceBudget
- **Reuses:** Sandbox patterns

## Data Models

### ResourceUsage
```rust
#[derive(Debug, Clone, Serialize)]
pub struct ResourceUsage {
    pub instructions: u64,
    pub max_instructions: u64,
    pub recursion_depth: u32,
    pub max_recursion: u32,
    pub memory_used: usize,
    pub max_memory: usize,
    pub elapsed: Duration,
    pub timeout: Duration,
}
```

### ScriptMetrics
```rust
#[derive(Debug, Clone, Serialize)]
pub struct ScriptMetrics {
    pub calls: u64,
    pub errors: u64,
    pub capability_denials: u64,
    pub resource_exhaustions: u64,
    pub avg_execution_time: Duration,
}
```

## Error Handling

### Error Scenarios

1. **Capability denied**
   - **Handling:** Return ScriptError::CapabilityDenied
   - **User Impact:** Script fails with clear message

2. **Resource exhausted**
   - **Handling:** Terminate script, return error
   - **User Impact:** Script stops, key passes through

3. **Invalid input**
   - **Handling:** Return validation error
   - **User Impact:** Script fails with helpful message

## Testing Strategy

### Unit Testing
- Test each capability tier
- Test resource limits
- Test validation rules

### Fuzz Testing
- Fuzz script inputs
- Test resource exhaustion
- Test capability bypass attempts

### Performance Testing
- Benchmark registry lookup
- Measure validation overhead
- Test under load
