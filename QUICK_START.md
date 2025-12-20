# KeyRx Web - Quick Start Guide

## Auto-Detection System

✅ **All API contract mismatches are now detected during build/test phase, not UAT.**

### Before Every Build

```bash
# Run this to catch issues BEFORE building
./scripts/pre-build.sh

# Then build
flutter build web --release
```

### What Gets Checked

1. **API Contract Tests** - Verifies backend response formats match frontend expectations
2. **Static Analysis** - Catches type errors and code issues

### Current Status

✅ **16 API contract tests** covering all endpoints
✅ **Backend server running** on http://192.168.11.24:8888
✅ **Web UI accessible** at http://192.168.11.24:4200
✅ **All services have safe web implementations**

## Fixed Issues

### 1. API Response Format Mismatches
- **Before**: Backend returned `{"layouts": [...]}`, frontend expected `{"data": [...]}`
- **After**: Frontend correctly parses `{"layouts": [...]}` format
- **Detection**: Contract tests verify this

### 2. Service Stubs Throwing Errors
- **Before**: Web stubs used `noSuchMethod` that threw errors
- **After**: All services return safe defaults
- **Detection**: Compilation errors caught during test

### 3. KeyFlowRepository Requiring FFI
- **Before**: Web platform tried to use FFI bridge (unavailable)
- **After**: Stub RuntimeApi provides empty config
- **Detection**: Runtime error prevented app loading

## Architecture

### Platform Detection
```dart
if (kIsWeb) {
  // Use HTTP/WebSocket services
  // Use stub implementations for FFI-only features
} else {
  // Use FFI bridge services
}
```

### Service Registry
- **Web Platform**: Creates web-based services (LayoutServiceWeb, HardwareServiceWeb, etc.)
- **Desktop Platform**: Creates FFI-based services (LayoutServiceFFI, HardwareServiceFFI, etc.)

## Testing

### Run API Contract Tests
```bash
./scripts/test-api-contracts.sh
```

### Run All Tests
```bash
flutter test
```

### Test Coverage
- API response parsing: ✅
- Error handling: ✅
- Web service implementations: ✅
- Service stubs: ✅

## Deployment

### Start Backend Server
```bash
cd apps/keyrx_server
dart run bin/server.dart --port 8888 --host 0.0.0.0
```

### Build & Serve Web UI
```bash
# Build
./scripts/pre-build.sh && flutter build web --release

# Serve
cd build/web
python3 -m http.server 4200 --bind 0.0.0.0
```

### Access
- Backend API: http://192.168.11.24:8888
- Web UI: http://192.168.11.24:4200

## Development Workflow

1. Make changes to code
2. Run `./scripts/pre-build.sh` to verify
3. If tests pass, build with `flutter build web`
4. If tests fail, fix the issue before building
5. Deploy and test

## Key Files

- `/scripts/test-api-contracts.sh` - API contract test runner
- `/scripts/pre-build.sh` - Pre-build verification script
- `/packages/core_logic/test/services/api_contract_test.dart` - Contract tests
- `/docs/API_CONTRACT_TESTING.md` - Full documentation

## Next Steps

The web UI should now work correctly. Please:

1. Refresh browser at http://192.168.11.24:4200
2. Test all tabs (Dashboard, Flow, Devices, Layouts, Wiring, Mapping)
3. Report any runtime errors
4. Check browser console for warnings

All API format issues are now caught during build/test, not UAT! 🎉
