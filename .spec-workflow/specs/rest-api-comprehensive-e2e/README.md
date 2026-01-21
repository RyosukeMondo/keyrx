# REST API Comprehensive E2E Testing

## Quick Start

```bash
# Fix dependencies
npm install

# Run tests
npm run test:e2e:auto --prefix keyrx_ui

# Or from root
make e2e-auto
```

## Overview

Comprehensive REST API testing that exercises ALL daemon features (40+ endpoints) via JSON-based communication. No browser/JavaScript required - pure API-driven feature validation.

## Current Status

**Coverage:** 35% → Target: 100%
- ✅ 14/40+ endpoints tested (need 26 more)
- ✅ 20/65+ test cases (need 45 more)
- ❌ Tests currently broken (missing `zod` dependency)
- ❌ WebSocket not tested
- ❌ Macro recorder not tested
- ❌ Simulator not tested

## Spec Documents

- **[requirements.md](./requirements.md)** - Complete requirements (40+ endpoints, 65+ tests)
- **[design.md](./design.md)** - Architecture and implementation design
- **[tasks.md](./tasks.md)** - 24 tasks across 6 phases (3-4 days)

## Test Categories

| Category | Endpoints | Tests | Status |
|----------|-----------|-------|--------|
| Health & Metrics | 7 | 7 | ⚠️ 57% (4/7) |
| Device Management | 7 | 10 | ⚠️ 30% (3/10) |
| Profile Management | 10 | 15 | ⚠️ 60% (9/15) |
| Config & Layers | 5 | 8 | ❌ 0% (0/8) |
| Layouts | 2 | 3 | ⚠️ 33% (1/3) |
| Macro Recorder | 4 | 6 | ❌ 0% (0/6) |
| Simulator | 2 | 5 | ❌ 0% (0/5) |
| WebSocket | 1 | 5 | ❌ 0% (0/5) |
| Workflows | Multiple | 6 | ⚠️ 33% (2/6) |
| **Total** | **40+** | **65+** | **31% (20/65)** |

## Implementation Phases

### Phase 1: Fix Infrastructure (CRITICAL)
**Time:** 2-3 hours
**Status:** 🔴 Blocking

- [ ] Install missing dependencies (zod, axios, ws, etc.)
- [ ] Fix broken tests (assertions, cleanup, race conditions)
- [ ] Fix daemon route typo (`/profiles:name/config`)

### Phase 2: Add Missing Endpoints
**Time:** 1-2 days
**Status:** 🟡 Ready after Phase 1

- [ ] Health & Metrics (3 endpoints)
- [ ] Device Management (4 endpoints)
- [ ] Profile Management (3 endpoints)
- [ ] Config & Layers (5 endpoints)
- [ ] Layouts (1 endpoint)
- [ ] Macro Recorder (4 endpoints)
- [ ] Simulator (2 endpoints)

### Phase 3: Feature Workflows
**Time:** 1 day
**Status:** 🟡 Depends on Phase 2

- [ ] Profile lifecycle workflows (3 tests)
- [ ] Device management workflows (1 test)
- [ ] Config & mapping workflows (1 test)
- [ ] Macro recording workflows (1 test)
- [ ] Simulator workflows (1 test)

### Phase 4: WebSocket Testing
**Time:** 4-6 hours
**Status:** 🟡 Ready after Phase 1

- [ ] WebSocket client utility
- [ ] Connection & subscription tests
- [ ] Event notification tests
- [ ] Reconnection resilience tests

### Phase 5: CI Integration
**Time:** 2-3 hours
**Status:** 🟡 Depends on Phase 1-4

- [ ] Update GitHub Actions workflow
- [ ] Add test failure notifications
- [ ] Enhance reporting (category breakdown, timing)

### Phase 6: Documentation
**Time:** 2-3 hours
**Status:** 🟢 Can start anytime

- [ ] Update README with all endpoints
- [ ] Update developer guide
- [ ] Add troubleshooting guide
- [ ] Create example tests

## Missing Endpoints (Critical Gaps)

### Macro Recorder (0% coverage)
- POST /api/macros/start-recording
- POST /api/macros/stop-recording
- GET /api/macros/recorded-events
- POST /api/macros/clear

### Simulator (0% coverage)
- POST /api/simulator/events
- POST /api/simulator/reset

### Config & Layers (0% coverage)
- GET /api/config
- PUT /api/config
- POST /api/config/key-mappings
- DELETE /api/config/key-mappings/:id
- GET /api/layers

### Device Advanced Features (0% coverage)
- PUT /api/devices/:id/name
- PUT/GET /api/devices/:id/layout
- DELETE /api/devices/:id

## Success Criteria

### Coverage
- ✅ 100% endpoint coverage (40+/40+)
- ✅ ≥65 test cases
- ✅ 100% test pass rate
- ✅ WebSocket tested

### Quality
- ✅ < 3 minute execution time
- ✅ 0 flaky tests
- ✅ All files < 500 lines
- ✅ Clear error messages with diffs

### CI/CD
- ✅ Tests run on GitHub Actions
- ✅ Artifacts uploaded
- ✅ PR comments
- ✅ Workflow fails on failure

## Quick Commands

```bash
# Install dependencies
npm install

# Run all tests
npm run test:e2e:auto --prefix keyrx_ui

# Run specific category
npm run test:e2e:auto -- --filter="health"

# Generate HTML report
npm run test:e2e:auto:report

# Run in CI mode
npm run test:e2e:auto -- --ci
```

## Related Specs

- **automated-api-e2e-testing** - Original partial implementation (35% coverage)
- **e2e-playwright-testing** - Browser UI testing (separate from this spec)
- **api-contract-testing** - API schema validation

## Next Steps

1. **Fix dependencies** - Install zod, axios, ws, commander
2. **Fix broken tests** - Run test suite, fix failures
3. **Add missing endpoints** - Start with macros and simulator (critical gaps)
4. **Add WebSocket tests** - Real-time event validation
5. **Update CI** - Integrate into GitHub Actions
6. **Documentation** - Complete README, developer guide, troubleshooting

## Contact

For questions or issues with this spec:
- Create issue in repo
- Reference spec name: `rest-api-comprehensive-e2e`
