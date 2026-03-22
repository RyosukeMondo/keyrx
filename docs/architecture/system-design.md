# KeyRx System Design

## Crate Dependency Graph

```mermaid
graph BT
    CORE["keyrx_core<br/>(no_std library)"]
    COMPILER["keyrx_compiler<br/>(CLI binary)"]
    DAEMON["keyrx_daemon<br/>(daemon binary)"]
    UI["keyrx_ui<br/>(React + WASM)"]

    COMPILER -->|"depends on"| CORE
    DAEMON -->|"depends on"| CORE
    DAEMON -->|"depends on"| COMPILER
    UI -->|"WASM build of"| CORE
    DAEMON -->|"embeds dist/"| UI

    style CORE fill:#f96,stroke:#333
    style DAEMON fill:#6f9,stroke:#333
```

- **keyrx_core**: no_std library, shared by all consumers
- **keyrx_compiler**: used by keyrx_daemon for runtime profile compilation
- **keyrx_daemon**: embeds keyrx_ui dist/ at compile time via `include_dir!`
- **keyrx_ui**: compiled to WASM from keyrx_core for browser-side simulation

## Configuration Pipeline

```mermaid
sequenceDiagram
    participant User
    participant Rhai as .rhai file
    participant Builders as keyrx_core::parser::builders
    participant Compiler as keyrx_compiler
    participant KRX as .krx binary
    participant Daemon as keyrx_daemon

    User->>Rhai: Write config
    User->>Daemon: POST /api/profiles/{name}/activate
    Daemon->>Compiler: compile_file(rhai, krx)
    Compiler->>Builders: build_map(), build_tap_hold(), etc.
    Builders-->>Compiler: BaseKeyMapping variants
    Compiler->>KRX: rkyv serialize + SHA256
    Daemon->>KRX: rkyv zero-copy deserialize
    Daemon->>Daemon: Install hook, start remapping
```

## Key Event Flow (Windows)

```mermaid
graph LR
    KB["Physical<br/>Keyboard"] -->|"scan code"| HOOK["Low-level<br/>Hook"]
    HOOK -->|"intercept"| LOOKUP["Mapping<br/>Lookup"]
    LOOKUP -->|"layer check"| STATE["Modifier<br/>State"]
    STATE -->|"active layer"| LOOKUP
    LOOKUP -->|"BaseKeyMapping"| PROCESS["Event<br/>Processor"]
    PROCESS -->|"TapHold?"| TAP["Tap-Hold<br/>State Machine"]
    PROCESS -->|"Simple/Modified"| INJECT["SendInput<br/>(scan code)"]
    TAP -->|"tap/hold resolved"| INJECT
    INJECT -->|"remapped key"| OS["OS / App"]

    style KB fill:#ddd,stroke:#333
    style OS fill:#ddd,stroke:#333
    style HOOK fill:#f96,stroke:#333
```

## Binary Format Stability

```mermaid
graph TD
    ENUM["BaseKeyMapping<br/>#[repr(u8)]"]
    ENUM --> S["Simple = 0"]
    ENUM --> M["Modifier = 1"]
    ENUM --> L["Lock = 2"]
    ENUM --> TH["TapHold = 3"]
    ENUM --> MO["ModifiedOutput = 4"]
    ENUM --> HO["HoldOnly = 5"]
    ENUM --> NEXT["NewVariant = 6<br/>(future)"]

    style ENUM fill:#ff9,stroke:#333
    style NEXT fill:#eee,stroke:#999,stroke-dasharray: 5 5
```

Each variant has an **explicit discriminant value** (`= N`). Source ordering is irrelevant to binary format. New variants get the next unused ID. Enforced by `test_base_key_mapping_discriminant_stability`.

## Parser SSOT Architecture

```mermaid
graph TB
    subgraph "keyrx_core (always available, no feature gate)"
        VALIDATORS["parser::validators<br/>(key name parsing)"]
        BUILDERS["parser::builders<br/>(mapping creation)"]
    end

    subgraph "keyrx_core (wasm feature)"
        CORE_PARSER["parser::functions/*<br/>(Rhai registration)<br/>spin::Mutex"]
    end

    subgraph "keyrx_compiler (std)"
        COMP_PARSER["parser::functions/*<br/>(Rhai registration)<br/>std::Mutex"]
    end

    CORE_PARSER -->|"calls"| BUILDERS
    COMP_PARSER -->|"calls"| BUILDERS
    BUILDERS -->|"calls"| VALIDATORS

    style BUILDERS fill:#6f9,stroke:#333
    style VALIDATORS fill:#6f9,stroke:#333
```

Both parsers exist because keyrx_core uses `spin::Mutex` (no_std) and keyrx_compiler uses `std::Mutex`. The **validation and mapping creation logic** lives in shared `builders` module — only the Rhai engine registration is duplicated.

## Build Pipeline & Staleness Enforcement

```mermaid
graph TD
    subgraph "make build (correct order)"
        W["1. WASM Build<br/>wasm-pack"] --> U["2. UI Build<br/>Vite + version inject"]
        U --> D["3. Daemon Build<br/>cargo + include_dir!"]
    end

    subgraph "build.rs checks"
        D --> CHECK_WASM{"keyrx_core/src<br/>newer than WASM?"}
        D --> CHECK_UI{"keyrx_ui/src<br/>newer than dist?"}
        CHECK_WASM -->|"yes"| FAIL_WASM["FAIL: Stale WASM"]
        CHECK_UI -->|"yes"| FAIL_UI["FAIL: Stale UI"]
        CHECK_WASM -->|"no"| OK["Embed & compile"]
        CHECK_UI -->|"no"| OK
    end

    SKIP["KEYRX_SKIP_FRONTEND_CHECK=1"] -.->|"bypass"| OK

    style FAIL_WASM fill:#f66,stroke:#333
    style FAIL_UI fill:#f66,stroke:#333
    style OK fill:#6f9,stroke:#333
```

## Tap-Hold & Hold-Only State Machine

```mermaid
stateDiagram-v2
    [*] --> Idle
    Idle --> Pending: Key Press

    Pending --> Hold: Timeout exceeded
    Pending --> Tap: Key Release (< threshold)
    Pending --> Hold: Other key pressed (permissive hold)

    Hold --> Idle: Key Release

    state Tap {
        [*] --> EmitTap: tap_hold
        [*] --> Suppress: hold_only
    }

    state Hold {
        [*] --> ActivateModifier
        ActivateModifier --> DeactivateModifier: Release
    }
```

- **tap_hold**: tap emits configured key, hold activates modifier layer
- **hold_only**: tap does nothing (suppressed), hold activates modifier layer

## Version Flow

```mermaid
graph LR
    CARGO["Cargo.toml<br/>version = '1.0.0'<br/>(SSOT)"]

    CARGO -->|"build.rs reads"| RUST_VER["env!(CARGO_PKG_VERSION)<br/>+ BUILD_DATE + GIT_HASH"]
    CARGO -->|"vite.config.ts reads"| VITE_VER["__APP_VERSION__<br/>+ __BUILD_TIME__<br/>+ __GIT_COMMIT__"]
    CARGO -->|"sync-version.sh"| PKG_JSON["package.json"]
    CARGO -->|"Makefile extracts"| NSIS["NSIS /DVERSION"]

    RUST_VER --> DAEMON["Daemon<br/>health API + logs"]
    VITE_VER --> UI_DISPLAY["UI sidebar<br/>+ about dialog"]
    PKG_JSON -->|"build.rs validates"| FAIL{"Mismatch?<br/>FAIL build"}

    style CARGO fill:#ff9,stroke:#333
    style FAIL fill:#f66,stroke:#333
```

No intermediate generated files. Version is injected at build time directly from Cargo.toml.
