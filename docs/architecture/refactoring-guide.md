# KISS/SLAP Refactoring Guide

**Priority:** P0 and P1 violations from kiss-slap-audit.md

---

## P0-1: keyDefinitions.ts (2064 lines → <500)

### Current Structure
```
keyrx_ui/src/data/keyDefinitions.ts (2064 lines)
├── KeyDefinition interface
├── KEY_DEFINITIONS array with 250+ entries
└── Helper functions
```

### Target Structure
```
keyrx_ui/src/data/keys/
├── index.ts                 (50 lines - re-exports)
├── types.ts                 (30 lines - KeyDefinition interface)
├── letters.ts               (150 lines - A-Z)
├── numbers.ts               (80 lines - 0-9, numpad)
├── modifiers.ts             (100 lines - Shift, Ctrl, Alt, etc.)
├── function-keys.ts         (150 lines - F1-F24)
├── navigation.ts            (100 lines - arrows, home, end, etc.)
├── editing.ts               (80 lines - backspace, delete, insert)
├── media.ts                 (120 lines - play, pause, volume)
├── special.ts               (150 lines - misc keys)
└── utils.ts                 (50 lines - helper functions)
```

### Migration Steps

**Step 1: Create directory structure**
```bash
mkdir -p keyrx_ui/src/data/keys
```

**Step 2: Extract types** (keyrx_ui/src/data/keys/types.ts)
```typescript
export interface KeyDefinition {
  id: string;
  label: string;
  category: 'basic' | 'modifiers' | 'media' | 'macro' | 'layers' | 'special' | 'any';
  subcategory?: string;
  description: string;
  aliases: string[];
  icon?: string;
}
```

**Step 3: Extract letters** (keyrx_ui/src/data/keys/letters.ts)
```typescript
import { KeyDefinition } from './types';

export const LETTER_KEYS: KeyDefinition[] = [
  {
    id: 'A',
    label: 'A',
    category: 'basic',
    subcategory: 'letters',
    description: 'Letter A',
    aliases: ['KC_A', 'VK_A', 'KEY_A'],
  },
  // ... B-Z
];
```

**Step 4: Extract numbers** (keyrx_ui/src/data/keys/numbers.ts)
```typescript
import { KeyDefinition } from './types';

export const NUMBER_KEYS: KeyDefinition[] = [
  // 0-9
];

export const NUMPAD_KEYS: KeyDefinition[] = [
  // Numpad 0-9, +, -, *, /
];
```

**Step 5: Create index** (keyrx_ui/src/data/keys/index.ts)
```typescript
export * from './types';
export { LETTER_KEYS } from './letters';
export { NUMBER_KEYS, NUMPAD_KEYS } from './numbers';
export { MODIFIER_KEYS } from './modifiers';
export { FUNCTION_KEYS } from './function-keys';
export { NAVIGATION_KEYS } from './navigation';
export { EDITING_KEYS } from './editing';
export { MEDIA_KEYS } from './media';
export { SPECIAL_KEYS } from './special';

// Aggregate all keys
export const KEY_DEFINITIONS = [
  ...LETTER_KEYS,
  ...NUMBER_KEYS,
  ...NUMPAD_KEYS,
  ...MODIFIER_KEYS,
  ...FUNCTION_KEYS,
  ...NAVIGATION_KEYS,
  ...EDITING_KEYS,
  ...MEDIA_KEYS,
  ...SPECIAL_KEYS,
];
```

**Step 6: Update imports**
```typescript
// Before
import { KEY_DEFINITIONS } from '@/data/keyDefinitions';

// After
import { KEY_DEFINITIONS, LETTER_KEYS, MODIFIER_KEYS } from '@/data/keys';
```

**Step 7: Run tests**
```bash
cd keyrx_ui
npm test
```

---

## P0-2: main.rs (1995 lines → <500)

### Current Structure
```rust
keyrx_daemon/src/main.rs (1995 lines)
├── CLI argument parsing (300 lines)
├── Daemon initialization (400 lines)
├── Event loop setup (300 lines)
├── IPC server (200 lines)
├── Platform setup (300 lines)
└── Subcommand handlers (495 lines)
```

### Target Structure
```rust
keyrx_daemon/src/
├── main.rs                  (100 lines - entry point only)
├── cli/
│   ├── parser.rs            (200 lines - clap definitions)
│   └── dispatcher.rs        (150 lines - route to handlers)
├── daemon/
│   ├── runner.rs            (250 lines - daemon lifecycle)
│   └── initialization.rs    (200 lines - setup)
├── ipc/
│   └── server.rs            (250 lines - IPC implementation)
└── platform/
    └── setup.rs             (200 lines - platform initialization)
```

### Migration Steps

**Step 1: Extract CLI parser** (keyrx_daemon/src/cli/parser.rs)
```rust
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "keyrx_daemon")]
#[command(version, about = "OS-level keyboard remapping daemon")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Run { /* ... */ },
    Devices(crate::cli::devices::DevicesArgs),
    Profiles(crate::cli::profiles::ProfilesArgs),
    // ...
}

pub fn parse() -> Cli {
    Cli::parse()
}
```

**Step 2: Extract daemon runner** (keyrx_daemon/src/daemon/runner.rs)
```rust
use crate::platform::Platform;
use crate::daemon::DaemonConfig;

pub fn run(
    platform: Box<dyn Platform>,
    config: DaemonConfig,
) -> Result<(), DaemonError> {
    // Daemon lifecycle logic
}
```

**Step 3: Extract platform setup** (keyrx_daemon/src/platform/setup.rs)
```rust
#[cfg(target_os = "linux")]
pub fn create_platform() -> Box<dyn Platform> {
    Box::new(LinuxPlatform::new())
}

#[cfg(target_os = "windows")]
pub fn create_platform() -> Box<dyn Platform> {
    Box::new(WindowsPlatform::new())
}
```

**Step 4: Simplify main.rs**
```rust
mod cli;
mod daemon;
mod ipc;
mod platform;

fn main() {
    let args = cli::parser::parse();

    match args.command {
        Commands::Run { config, debug, test_mode } => {
            let platform = platform::setup::create_platform();
            let daemon_config = daemon::DaemonConfig { /* ... */ };
            daemon::runner::run(platform, daemon_config)?;
        }
        Commands::Devices(args) => {
            cli::devices::execute(args)?;
        }
        // ... other commands
    }
}
```

**Step 5: Run tests**
```bash
cargo test -p keyrx_daemon
```

---

## P1-1: state.rs (1225 lines → <500)

### Current Structure
```rust
keyrx_core/src/runtime/state.rs (1225 lines)
├── DeviceState struct (300 lines)
├── Modifier operations (200 lines)
├── Lock operations (150 lines)
├── Pressed key tracking (250 lines)
├── Condition evaluation (200 lines)
└── Tests (125 lines)
```

### Target Structure
```rust
keyrx_core/src/runtime/state/
├── mod.rs                   (100 lines - re-exports)
├── device_state.rs          (300 lines - core struct)
├── modifiers.rs             (200 lines - modifier operations)
├── conditions.rs            (250 lines - condition evaluation)
├── pressed_keys.rs          (200 lines - key tracking)
└── tests.rs                 (175 lines - unit tests)
```

### Migration Steps

**Step 1: Create module directory**
```bash
mkdir keyrx_core/src/runtime/state
```

**Step 2: Extract device state** (keyrx_core/src/runtime/state/device_state.rs)
```rust
use arrayvec::ArrayVec;
use bitvec::prelude::*;

pub struct DeviceState {
    modifiers: BitVec<u8, Lsb0>,
    locks: BitVec<u8, Lsb0>,
    tap_hold: TapHoldProcessor<DEFAULT_MAX_PENDING>,
    pressed_keys: ArrayVec<(KeyCode, ArrayVec<KeyCode, MAX_OUTPUT_KEYS_PER_INPUT>), MAX_PRESSED_KEYS>,
}

impl DeviceState {
    pub fn new() -> Self { /* ... */ }
    // Core construction/destruction only
}
```

**Step 3: Extract modifiers** (keyrx_core/src/runtime/state/modifiers.rs)
```rust
use super::device_state::DeviceState;

impl DeviceState {
    pub fn set_modifier(&mut self, id: u8) -> bool { /* ... */ }
    pub fn clear_modifier(&mut self, id: u8) -> bool { /* ... */ }
    pub fn is_modifier_active(&self, id: u8) -> bool { /* ... */ }
    pub fn toggle_lock(&mut self, id: u8) -> bool { /* ... */ }
    pub fn is_lock_active(&self, id: u8) -> bool { /* ... */ }
}
```

**Step 4: Extract conditions** (keyrx_core/src/runtime/state/conditions.rs)
```rust
use super::device_state::DeviceState;
use crate::config::Condition;

impl DeviceState {
    pub fn evaluate_condition(&self, condition: &Condition) -> bool { /* ... */ }
    fn evaluate_item(&self, item: &ConditionItem) -> bool { /* ... */ }
}
```

**Step 5: Create module** (keyrx_core/src/runtime/state/mod.rs)
```rust
mod device_state;
mod modifiers;
mod conditions;
mod pressed_keys;

pub use device_state::DeviceState;

#[cfg(test)]
mod tests;
```

**Step 6: Update imports**
```rust
// Before
use crate::runtime::state::DeviceState;

// After (same, thanks to re-export)
use crate::runtime::state::DeviceState;
```

---

## P1-2: config.rs CLI handlers (927 lines → <300)

### Current Structure
```rust
keyrx_daemon/src/cli/config.rs (927 lines)
├── Command definitions (200 lines)
├── execute_inner dispatcher (100 lines)
├── set_key handler (150 lines)
├── set_tap_hold handler (150 lines)
├── set_macro handler (150 lines)
├── Other handlers (177 lines)
```

### Target Structure
```rust
keyrx_daemon/src/cli/config/
├── mod.rs                   (100 lines - public API)
├── types.rs                 (150 lines - command types)
├── handlers/
│   ├── mod.rs               (50 lines)
│   ├── set_key.rs           (150 lines)
│   ├── set_tap_hold.rs      (150 lines)
│   ├── set_macro.rs         (150 lines)
│   ├── get_key.rs           (100 lines)
│   └── validate.rs          (100 lines)
```

### Migration Steps

**Step 1: Extract command types** (keyrx_daemon/src/cli/config/types.rs)
```rust
use clap::{Args, Subcommand};

#[derive(Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigCommands,

    #[arg(long, global = true)]
    pub json: bool,
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    SetKey { /* ... */ },
    SetTapHold { /* ... */ },
    // ...
}
```

**Step 2: Extract handlers** (keyrx_daemon/src/cli/config/handlers/set_key.rs)
```rust
use crate::config::ProfileManager;
use crate::cli::config::types::*;

pub fn handle_set_key(
    key: String,
    target: String,
    layer: String,
    profile: Option<String>,
    profile_manager: &ProfileManager,
) -> Result<SetKeyOutput, ConfigError> {
    // Pure handler logic - no CLI parsing
}
```

**Step 3: Create dispatcher** (keyrx_daemon/src/cli/config/mod.rs)
```rust
mod types;
mod handlers;

pub use types::{ConfigArgs, ConfigCommands};

pub fn execute(args: ConfigArgs, config_dir: Option<PathBuf>) -> Result<(), DaemonError> {
    let profile_manager = ProfileManager::new(config_dir)?;

    match args.command {
        ConfigCommands::SetKey { key, target, layer, profile } => {
            handlers::set_key::handle_set_key(key, target, layer, profile, &profile_manager)?;
        }
        ConfigCommands::SetTapHold { /* ... */ } => {
            handlers::set_tap_hold::handle(/* ... */)?;
        }
        // ...
    }

    Ok(())
}
```

---

## P1-3: profile_manager.rs (870 lines → <400)

### Current Structure
```rust
keyrx_daemon/src/config/profile_manager.rs (870 lines)
├── ProfileManager struct (100 lines)
├── CRUD operations (300 lines)
├── Compilation logic (150 lines)
├── Activation logic (200 lines)
├── Persistence (120 lines)
```

### Target Structure
```rust
keyrx_daemon/src/config/profile/
├── manager.rs               (200 lines - core manager)
├── repository.rs            (250 lines - file I/O)
├── compiler_service.rs      (150 lines - compilation)
├── activation_service.rs    (200 lines - activation logic)
```

### Migration Steps

**Step 1: Extract repository** (keyrx_daemon/src/config/profile/repository.rs)
```rust
pub struct ProfileRepository {
    profiles_dir: PathBuf,
}

impl ProfileRepository {
    pub fn scan(&self) -> Result<Vec<ProfileMetadata>, ProfileError> { /* ... */ }
    pub fn load(&self, name: &str) -> Result<ProfileMetadata, ProfileError> { /* ... */ }
    pub fn save(&self, profile: &ProfileMetadata) -> Result<(), ProfileError> { /* ... */ }
    pub fn delete(&self, name: &str) -> Result<(), ProfileError> { /* ... */ }
}
```

**Step 2: Extract compiler service** (keyrx_daemon/src/config/profile/compiler_service.rs)
```rust
pub struct ProfileCompilerService {
    compiler: ProfileCompiler,
}

impl ProfileCompilerService {
    pub fn compile(&self, rhai_path: &Path, krx_path: &Path) -> Result<(), CompilationError> {
        // Pure compilation logic
    }
}
```

**Step 3: Extract activation service** (keyrx_daemon/src/config/profile/activation_service.rs)
```rust
pub struct ActivationService {
    active_profile: Arc<RwLock<Option<String>>>,
    activation_lock: Arc<Mutex<()>>,
}

impl ActivationService {
    pub fn activate(&self, name: &str) -> Result<ActivationResult, ProfileError> {
        // Pure activation logic
    }

    pub fn get_active(&self) -> Option<String> { /* ... */ }
}
```

**Step 4: Simplify manager** (keyrx_daemon/src/config/profile/manager.rs)
```rust
pub struct ProfileManager {
    repository: ProfileRepository,
    compiler: ProfileCompilerService,
    activation: ActivationService,
}

impl ProfileManager {
    pub fn new(config_dir: PathBuf) -> Result<Self, ProfileError> {
        Ok(Self {
            repository: ProfileRepository::new(config_dir.clone()),
            compiler: ProfileCompilerService::new(),
            activation: ActivationService::new(),
        })
    }

    // Delegate to services
    pub fn create_profile(&self, name: &str, template: ProfileTemplate) -> Result<(), ProfileError> {
        self.repository.create(name, template)
    }

    pub fn activate_profile(&self, name: &str) -> Result<ActivationResult, ProfileError> {
        let metadata = self.repository.load(name)?;
        self.compiler.compile(&metadata.rhai_path, &metadata.krx_path)?;
        self.activation.activate(name)
    }
}
```

---

## Testing Strategy

After each refactoring:

1. **Run existing tests**
```bash
cargo test --workspace
cd keyrx_ui && npm test
```

2. **Check file sizes**
```bash
find . -name "*.rs" -exec wc -l {} + | sort -rn | head -20
```

3. **Run linters**
```bash
cargo clippy --workspace -- -D warnings
cargo fmt --check
```

4. **Verify builds**
```bash
make build
make verify
```

---

## Success Criteria

- [ ] All source files <500 code lines
- [ ] All functions <50 lines
- [ ] No cyclomatic complexity >10
- [ ] No nesting depth >3
- [ ] Clear single level of abstraction per function
- [ ] All existing tests pass
- [ ] No new clippy warnings

---

## Rollback Plan

If refactoring causes issues:

1. **Create backup branch**
```bash
git checkout -b refactor-backup
git checkout main
```

2. **Revert specific file**
```bash
git checkout HEAD~1 -- path/to/file
```

3. **Run tests to confirm rollback**
```bash
cargo test --workspace
```
