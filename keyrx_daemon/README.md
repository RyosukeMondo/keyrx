# keyrx_daemon

OS-level keyboard interception daemon with embedded web server.

## Purpose

`keyrx_daemon` is the runtime component of KeyRx that:
- Intercepts keyboard events at the OS level (Linux via evdev, Windows via low-level hooks)
- Applies remapping rules using the compiled configuration from `keyrx_compiler`
- Provides a web interface for monitoring and configuration
- Serves the React UI built by `keyrx_ui`

## Features

### Platform-Specific Input Handling

- **Linux**: Uses `evdev` for reading input devices and `uinput` for emitting remapped events
- **Windows**: Uses low-level keyboard hooks from `windows-sys`
- Feature-gated to compile only relevant platform code

### Web Server (Default Feature)

- Built with `axum` for REST API and static file serving
- WebSocket support for real-time event streaming
- Serves the compiled UI from `ui_dist/` directory

## Build Features

The daemon supports multiple feature flags:

```bash
# Default build (web server only, no platform code)
cargo build --bin keyrx_daemon

# Linux build with input interception
cargo build --bin keyrx_daemon --features linux

# Windows build with input interception
cargo build --bin keyrx_daemon --features windows

# Web-only build (testing/development)
cargo build --bin keyrx_daemon --features web
```

## Usage

```bash
# Run daemon with default settings
cargo run --bin keyrx_daemon

# Run with custom config file
cargo run --bin keyrx_daemon -- --config path/to/config.bin

# Run in headless mode (no browser launch)
cargo run --bin keyrx_daemon -- --headless

# Run with debug logging
cargo run --bin keyrx_daemon -- --log-level debug
```

## Architecture

```
keyrx_daemon/
├── src/
│   ├── main.rs              # Entry point and CLI
│   ├── platform/            # Platform-specific input handling
│   │   ├── mod.rs          # Platform abstraction
│   │   ├── linux.rs        # Linux evdev/uinput implementation
│   │   └── windows.rs      # Windows hooks implementation
│   └── web/                 # Web server components
│       ├── mod.rs          # Server setup and routing
│       ├── api.rs          # REST API endpoints
│       ├── ws.rs           # WebSocket handlers
│       └── static_files.rs # UI file serving
└── ui_dist/                 # Embedded UI files (from keyrx_ui build)
```

## Dependencies

- `keyrx_core`: Core remapping logic
- `axum`: Web framework (optional, enabled by default)
- `tower-http`: HTTP middleware for static files and CORS (optional)
- `tokio`: Async runtime (optional, for web server)
- `evdev`: Linux input device reading (optional, Linux only)
- `uinput`: Linux input device emulation (optional, Linux only)
- `nix`: Unix system calls (optional, Linux only)
- `windows-sys`: Windows API bindings (optional, Windows only)

## Development

The daemon is designed to be run via the automation scripts:

```bash
# Build daemon
./scripts/build.sh

# Launch daemon with default config
./scripts/launch.sh

# Launch in debug mode
./scripts/launch.sh --debug

# Launch with custom config
./scripts/launch.sh --config path/to/config.bin
```

## Testing

Unit tests are included for each module:

```bash
cargo test --bin keyrx_daemon
```

Integration tests verify platform-specific functionality in `tests/integration/`.

## Security Considerations

The daemon requires elevated privileges to intercept keyboard events:
- **Linux**: Requires access to `/dev/input/eventX` and `/dev/uinput` (typically via udev rules or root)
- **Windows**: Requires administrator privileges for low-level keyboard hooks

Always verify the integrity of configuration files before loading them into the daemon.

## License

See the workspace LICENSE file for licensing information.
