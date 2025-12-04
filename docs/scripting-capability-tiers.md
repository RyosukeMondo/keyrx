# Script Function Capability Tiers

This document explains the capability tier assignment for all KeyRx script functions.

## Overview

Each script function is assigned one of four capability tiers based on its security impact:

1. **Safe** - No side effects, bounded execution
2. **Standard** - May affect engine state (keyboard operations)
3. **Advanced** - System interaction, requires trust
4. **Internal** - Not exposed to user scripts

## Tier Assignment Rationale

### Safe Tier Functions

Currently, there are **no Safe tier functions**. Rationale:

- All KeyRx functions modify engine state (remapping, layers, modifiers)
- Even read-only queries (`is_layer_active`, `is_modifier_active`) read engine state
- True "Safe" functions would be pure computations with no I/O

Future candidates for Safe tier:
- String manipulation utilities
- Arithmetic operations
- Key code constant lookups
- Pure validation functions

### Standard Tier Functions (24 total)

All current KeyRx functions are Standard tier because they:

1. **Modify keyboard remapping state** - This is the core purpose of KeyRx
2. **Do not access system resources** - No filesystem, network, clipboard access
3. **Have bounded execution** - All operations complete in deterministic time
4. **Cannot cause resource exhaustion** - No unbounded loops or allocations
5. **Are essential for keyboard customization** - Users expect these to work

#### Debug Functions

| Function | Tier | Rationale |
|----------|------|-----------|
| `print_debug` | Standard | Logging only, no state modification, bounded execution |

#### Remapping Functions

| Function | Tier | Rationale |
|----------|------|-----------|
| `remap` | Standard | Core keyboard remapping, modifies engine state |
| `block` | Standard | Blocks key events, modifies engine state |
| `pass` | Standard | Passes key events through, modifies engine state |
| `tap_hold` | Standard | Tap-hold behavior, modifies engine state |
| `tap_hold_mod` | Standard | Tap-hold with virtual modifiers, modifies engine state |
| `combo` | Standard | Key combination mapping, modifies engine state |

**Security considerations:**
- All remapping functions validate input key codes
- Array sizes are bounded (combo: 2-4 keys)
- No arbitrary code execution
- No access to system resources

#### Layer Functions

| Function | Tier | Rationale |
|----------|------|-----------|
| `layer_define` | Standard | Defines named layers, modifies engine state |
| `layer_map` | Standard | Maps keys within layers, modifies engine state |
| `layer_push` | Standard | Layer stack manipulation, modifies engine state |
| `layer_pop` | Standard | Layer stack manipulation, modifies engine state |
| `layer_toggle` | Standard | Layer activation toggle, modifies engine state |
| `is_layer_active` | Standard | Read-only query, accesses engine state |

**Security considerations:**
- Layer names are validated (no colons, non-empty)
- Stack operations are bounded (max layer depth enforced elsewhere)
- No unbounded recursion
- State changes are deterministic

#### Modifier Functions

| Function | Tier | Rationale |
|----------|------|-----------|
| `define_modifier` | Standard | Defines virtual modifiers, modifies engine state |
| `modifier_on` | Standard | Activates virtual modifiers, modifies engine state |
| `modifier_off` | Standard | Deactivates virtual modifiers, modifies engine state |
| `one_shot` | Standard | One-shot modifier behavior, modifies engine state |
| `is_modifier_active` | Standard | Read-only query, accesses engine state |

**Security considerations:**
- Modifier IDs are validated (0-255 range)
- Maximum modifier count enforced
- No arbitrary modifier creation beyond limits
- State changes are tracked and bounded

#### Timing Functions

| Function | Tier | Rationale |
|----------|------|-----------|
| `set_tap_timeout` | Standard | Configures tap timeout, modifies engine state |
| `set_combo_timeout` | Standard | Configures combo timeout, modifies engine state |
| `set_hold_delay` | Standard | Configures hold delay, modifies engine state |
| `set_eager_tap` | Standard | Configures eager tap mode, modifies engine state |
| `set_permissive_hold` | Standard | Configures permissive hold mode, modifies engine state |
| `set_retro_tap` | Standard | Configures retro tap mode, modifies engine state |

**Security considerations:**
- Timeout values are range-validated (0-5000ms)
- Boolean flags are type-safe
- No unbounded delays or blocking
- Configuration changes are immediate and deterministic

### Advanced Tier Functions

Currently, there are **no Advanced tier functions**.

Future candidates for Advanced tier:
- Clipboard operations (`clipboard_get`, `clipboard_set`)
- System notifications (`notify`)
- External command execution (`exec`, `spawn`)
- File I/O (`read_file`, `write_file`)
- Network requests (`http_get`, `http_post`)
- IPC with other applications

**Why these would require Advanced tier:**
- Access system resources beyond keyboard
- Can leak sensitive information
- Can affect other applications
- Require explicit user trust
- May have unbounded execution time
- Can cause side effects outside KeyRx

### Internal Tier Functions

Currently, there are **no Internal tier functions**.

Future candidates for Internal tier:
- Engine control hooks (pause, resume, shutdown)
- Direct registry manipulation bypassing validation
- Debug-only introspection functions
- Performance profiling hooks
- Low-level memory management

**Why these would be Internal only:**
- Bypass safety checks
- Can break invariants
- Only needed for engine implementation
- Not useful for user scripts
- Could cause crashes or corruption

## Security Model

### Execution Modes

1. **Safe Mode** - Only Safe tier functions
   - Most restrictive
   - Suitable for untrusted scripts
   - Currently: No functions available (none are Safe tier)

2. **Standard Mode** (Default) - Safe + Standard tier functions
   - Balanced security and functionality
   - All keyboard operations allowed
   - No system resource access
   - Recommended for most users

3. **Full Mode** - Safe + Standard + Advanced tier functions
   - All functionality enabled
   - System interaction allowed
   - Requires explicit user trust
   - For advanced users only

### Capability Enforcement

Capability checks happen at:
1. **Function registration** - Functions tagged with capability tier
2. **Script execution** - Mode determines which tiers are allowed
3. **Runtime** - Function calls rejected if capability insufficient

### Future Considerations

As KeyRx evolves, new functions will be categorized based on:

1. **Resource access** - Does it access filesystem, network, clipboard?
2. **Side effects** - Does it modify state outside the keyboard?
3. **Execution bounds** - Is execution time deterministic and bounded?
4. **User trust** - Does it require explicit user permission?
5. **Failure impact** - What's the blast radius of misuse?

**Conservative assignment:**
- When in doubt, assign a higher (more restrictive) tier
- Functions can be downgraded later if proven safe
- Upgrading tiers is a breaking change for safe-mode scripts

## Testing

All capability assignments are tested in:
- `core/src/scripting/sandbox/function_capabilities.rs` - Unit tests
- Future: Integration tests for mode enforcement
- Future: Fuzz tests for capability bypass attempts

## References

- Implementation: `core/src/scripting/sandbox/function_capabilities.rs`
- Capability types: `core/src/scripting/sandbox/capability.rs`
- Function bindings: `core/src/scripting/bindings.rs`
- Spec: `.spec-workflow/specs/rhai-sandbox-hardening/`
