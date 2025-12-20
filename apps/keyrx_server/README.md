# KeyRx Backend API Server

Backend HTTP API server for KeyRx web platform support.

## Features

- REST API endpoints for all KeyRx services
- CORS support for web clients
- Real-time event broadcasting via WebSocket (coming soon)
- Direct integration with core_logic services

## Usage

```bash
# Install dependencies
dart pub get

# Run server (default port 8080)
dart run bin/server.dart

# Run on custom port
dart run bin/server.dart --port 3000

# Run on custom host
dart run bin/server.dart --host localhost --port 3000

# Show help
dart run bin/server.dart --help
```

## API Endpoints

### Hardware Profiles
- `GET /api/hardware/profiles` - List all hardware profiles
- `POST /api/hardware/profiles` - Save hardware profile
- `DELETE /api/hardware/profiles/<id>` - Delete hardware profile

### Keymaps
- `GET /api/keymaps` - List all keymaps
- `POST /api/keymaps` - Save keymap
- `DELETE /api/keymaps/<id>` - Delete keymap

### Layouts
- `GET /api/layouts` - List all virtual layouts
- `POST /api/layouts` - Save layout
- `DELETE /api/layouts/<id>` - Delete layout

### Devices
- `GET /api/devices` - Get all devices
- `POST /api/devices/virtual` - Add virtual device
- `DELETE /api/devices/virtual/<key>` - Remove virtual device
- `POST /api/devices/<key>/toggle` - Toggle device enabled state
- `POST /api/devices/<key>/toggle-remap` - Toggle remap enabled state
- `POST /api/devices/<key>/assign-profile` - Assign profile to device
- `POST /api/devices/<key>/set-label` - Set user label for device
- `POST /api/devices/refresh` - Refresh device list

### Profiles
- `GET /api/profiles` - List all profile IDs
- `GET /api/profiles/<id>` - Get specific profile
- `POST /api/profiles` - Save profile
- `DELETE /api/profiles/<id>` - Delete profile
- `GET /api/profiles/compatible/<layoutType>` - Find compatible profiles
- `POST /api/profiles/refresh` - Refresh profile list

### Runtime
- `GET /api/runtime/config` - Get runtime configuration
- `POST /api/runtime/slots/add` - Add slot
- `POST /api/runtime/slots/remove` - Remove slot
- `POST /api/runtime/slots/reorder` - Reorder slot
- `POST /api/runtime/slots/set-active` - Set slot active state

### Tests
- `POST /api/tests/discover` - Discover tests in script
- `POST /api/tests/run` - Run tests

### Scripts
- `POST /api/scripts/save` - Save script file
- `POST /api/scripts/load` - Load script file

## Architecture

The server wraps core_logic services with a REST API layer:
- Uses shelf for HTTP server
- Uses shelf_router for routing
- Zero business logic - delegates to core_logic services
- CORS enabled for web client access
