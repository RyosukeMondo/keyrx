# E2E Test Fixes - Design

## Architecture

### API Mock Layer
```
e2e/fixtures/
├── api-mocks.ts       # Core API mocking (profiles, devices)
├── dashboard-mocks.ts # Dashboard/WebSocket mocking
└── config-mocks.ts    # Config editor mocking
```

### Test Structure
Each test file uses `setupApiMocks()` in `beforeEach` to intercept API calls.

### Component Testability
Components expose `data-testid` attributes for reliable selection:
- `data-testid="connection-banner"` - Dashboard connection status
- `data-testid="state-indicator-panel"` - Daemon state display
- `data-testid="metrics-chart"` - Latency chart
- `data-testid="event-timeline"` - Event log
- `data-testid="keyboard-visualizer"` - Visual editor
- `data-testid="code-editor"` - Monaco editor
- `data-profile="name"` - Profile cards
- `data-testid="device-card"` - Device cards

### Mock Response Format
All mocks return schema-compliant responses matching `src/api/schemas.ts`.
