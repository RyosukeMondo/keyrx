# Integration Testing Guide for KeyRx UI

This guide covers integration testing patterns for the KeyRx UI frontend using Vitest, React Testing Library, and MSW (Mock Service Worker) for WebSocket mocking.

## Table of Contents

- [What are Integration Tests?](#what-are-integration-tests)
- [When to Write Integration Tests](#when-to-write-integration-tests)
- [Test Infrastructure](#test-infrastructure)
- [Writing Integration Tests](#writing-integration-tests)
- [Full Page Testing](#full-page-testing)
- [WebSocket State Management](#websocket-state-management)
- [Multi-Component Interactions](#multi-component-interactions)
- [Async Patterns and Best Practices](#async-patterns-and-best-practices)
- [Common Patterns](#common-patterns)
- [Troubleshooting](#troubleshooting)

## What are Integration Tests?

Integration tests verify that multiple components work together correctly. Unlike unit tests that isolate a single component, integration tests:

- Test complete user workflows (e.g., full page interactions)
- Verify component composition and communication
- Include router context, state management, and API calls
- Test WebSocket real-time updates across components
- Validate full features from user perspective

**Example**: Testing the ConfigPage with keyboard visualizer, layer selector, and WebSocket updates working together.

## When to Write Integration Tests

Use integration tests when:

✅ **Testing Full Pages**: Complete page workflows with multiple components
✅ **Testing Multi-Step Flows**: User journeys spanning multiple actions (e.g., click key → open dialog → configure → save)
✅ **Testing WebSocket Integration**: Real-time state updates affecting multiple components
✅ **Testing Router-Dependent Components**: Components using `useParams`, `useNavigate`, etc.
✅ **Testing Component Interactions**: Parent-child communication, event bubbling, context propagation

Don't use integration tests when:

❌ Single component in isolation (use unit tests)
❌ Pure logic functions (use unit tests)
❌ Simple render tests (use unit tests)

**Decision Tree**:

```
Does the test involve multiple components?
  ├─ YES: Does it test a full user workflow?
  │   ├─ YES: Integration Test ✅
  │   └─ NO: Unit test if components are loosely coupled
  └─ NO: Unit Test
```

## Test Infrastructure

Integration tests use the same infrastructure as unit tests, but with additional setup:

### Test Location

Integration tests are stored in:
- `src/pages/__integration__/` - Page-level integration tests
- `tests/integration/` - Cross-cutting integration tests

### Running Integration Tests

```bash
# Run all integration tests
npm run test:integration

# Watch mode
npm run test:integration:watch

# Run specific test file
npm run test:integration -- ConfigPage.integration.test.tsx
```

### Configuration

Integration tests use `vitest.integration.config.ts`:
- Longer timeouts (30s for tests, 10s for hooks)
- Includes `**/__integration__/**` and `tests/integration/**`
- Excludes unit tests, E2E, and performance tests

## Writing Integration Tests

### Basic Structure

```typescript
import { describe, it, expect, beforeEach } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import { renderWithProviders, setDaemonState } from '../../../tests/testUtils';
import userEvent from '@testing-library/user-event';
import { ConfigPage } from './ConfigPage';

describe('ConfigPage - Integration Tests', () => {
  // Helper to render page with router context
  const renderConfigPage = (profile = 'default') => {
    return renderWithProviders(<ConfigPage profileName={profile} />, {
      wrapWithRouter: true,
      routerInitialEntries: ['/config'],
    });
  };

  beforeEach(() => {
    // Set up initial WebSocket daemon state
    setDaemonState({
      activeProfile: 'default',
      layer: 'base',
      modifiers: [],
      locks: [],
      connected: true,
    });
  });

  it('completes full configuration workflow', async () => {
    const user = userEvent.setup();
    renderConfigPage();

    // Wait for page to load
    await waitFor(() => {
      expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
    });

    // Test multi-step workflow...
  });
});
```

## Full Page Testing

### Rendering Pages with Router Context

Many pages require React Router context. Use `renderWithProviders` with router options:

```typescript
// Basic page with router
renderWithProviders(<ConfigPage />, {
  wrapWithRouter: true,
  routerInitialEntries: ['/config'],
});

// Page with route parameters
renderWithProviders(<ProfileDetailPage />, {
  wrapWithRouter: true,
  routerInitialEntries: ['/profiles/gaming'],
  routerPath: '/profiles/:name',
});
```

**Router Options**:
- `wrapWithRouter: true` - Wraps component with `<MemoryRouter>`
- `routerInitialEntries` - Array of route paths (initial location)
- `routerPath` - Route path pattern (for `useParams`)

### Testing Complete User Workflows

Integration tests should follow realistic user journeys:

```typescript
it('configures simple key remap', async () => {
  const user = userEvent.setup();
  renderConfigPage();

  // Step 1: Wait for page load
  await waitFor(() => {
    expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
  });

  // Step 2: Click a key to configure
  const keyButtons = screen.getAllByRole('button', {
    name: /Key [A-Z0-9]/i,
  });
  await user.click(keyButtons[0]);

  // Step 3: Verify dialog opens
  await waitFor(() => {
    expect(screen.getByRole('dialog')).toBeInTheDocument();
  });

  // Step 4: Select action type
  const actionTypeSelector = screen.getByRole('combobox', {
    name: /Action Type/i,
  });
  await user.selectOptions(actionTypeSelector, 'simple');

  // Step 5: Select target key
  const targetKeySelector = screen.getByRole('combobox', {
    name: /Target Key/i,
  });
  await user.selectOptions(targetKeySelector, 'KEY_B');

  // Step 6: Save configuration
  const saveButton = screen.getByRole('button', { name: /Save/i });
  await user.click(saveButton);

  // Step 7: Verify dialog closes
  await waitFor(() => {
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
  });
});
```

**Key Points**:
- Use `userEvent.setup()` for realistic user interactions
- Use semantic queries (`getByRole`, `getByLabelText`) for accessibility
- Use `waitFor()` for async operations (dialog open, state updates)
- Test the complete flow, not individual steps in isolation

### Testing Loading and Error States

Integration tests should verify loading and error handling:

```typescript
it('shows loading state while fetching config', async () => {
  // Set loading state via store
  const store = useConfigStore.getState();
  store.loading = true;

  renderConfigPage();

  // Verify loading indicator appears
  expect(screen.getByRole('status', { name: /Loading/i })).toBeInTheDocument();
});

it('displays error message when fetch fails', async () => {
  const store = useConfigStore.getState();
  store.error = 'Failed to load configuration';

  renderConfigPage();

  // Verify error message displays
  expect(
    screen.getByText(/Failed to load configuration/i)
  ).toBeInTheDocument();
});
```

## WebSocket State Management

Integration tests can simulate WebSocket events to test real-time updates.

### Setting Daemon State

Use `setDaemonState()` to simulate daemon state changes:

```typescript
import { setDaemonState } from '../../tests/testUtils';

beforeEach(() => {
  // Set initial daemon state
  setDaemonState({
    activeProfile: 'default',
    layer: 'base',
    modifiers: [],
    locks: [],
    connected: true,
  });
});

it('updates display when activeProfile changes', async () => {
  renderComponent();

  // Simulate profile change via WebSocket
  setDaemonState({ activeProfile: 'gaming' });

  // Verify component updates
  await waitFor(() => {
    expect(screen.getByText('gaming')).toBeInTheDocument();
  });
});
```

### Simulating Latency Updates

Use `sendLatencyUpdate()` to simulate latency metrics:

```typescript
import { sendLatencyUpdate } from '../../tests/testUtils';

it('displays updated latency metrics', async () => {
  renderComponent();

  // Subscribe to latency channel (done automatically by component)

  // Simulate latency update
  sendLatencyUpdate({
    min: 100,
    avg: 500,
    max: 2000,
    p95: 1500,
    p99: 1800,
  });

  // Verify metrics display
  await waitFor(() => {
    expect(screen.getByText(/500.*ms/i)).toBeInTheDocument();
  });
});
```

### Simulating Key Events

Use `sendKeyEvent()` to simulate key press/release events:

```typescript
import { sendKeyEvent } from '../../tests/testUtils';

it('displays key events in real-time', async () => {
  renderComponent();

  // Simulate key press
  sendKeyEvent({
    keyCode: 'KEY_A',
    eventType: 'press',
    input: 'KEY_A',
    output: 'KEY_B',
    latency: 500,
  });

  // Verify event appears
  await waitFor(() => {
    expect(screen.getByText(/KEY_A.*→.*KEY_B/i)).toBeInTheDocument();
  });
});
```

### WebSocket Connection Lifecycle

Test WebSocket connection and disconnection:

```typescript
it('handles WebSocket disconnection gracefully', async () => {
  renderComponent();

  // Component starts connected
  await waitFor(() => {
    expect(screen.queryByText(/Disconnected/i)).not.toBeInTheDocument();
  });

  // Simulate disconnection
  setDaemonState({ connected: false });

  // Verify disconnected state displays
  await waitFor(() => {
    expect(screen.getByText(/Disconnected/i)).toBeInTheDocument();
  });
});
```

## Multi-Component Interactions

Integration tests verify that multiple components communicate correctly.

### Parent-Child Communication

Test that parent components pass props and handle events from children:

```typescript
it('switches between layers', async () => {
  const user = userEvent.setup();
  renderConfigPage();

  await waitFor(() => {
    const layerSelector = screen.queryByRole('combobox', {
      name: /Layer/i,
    });
    expect(layerSelector).toBeInTheDocument();
  });

  // LayerSelector (child) updates ConfigPage (parent) state
  const layerSelector = screen.getByRole('combobox', {
    name: /Layer/i,
  });

  const options = layerSelector.querySelectorAll('option');
  if (options.length > 1) {
    await user.selectOptions(layerSelector, options[1].value);

    // Verify parent state updates
    expect(layerSelector).toHaveValue(options[1].value);

    // Verify KeyboardVisualizer (sibling) updates to show new layer
    const keyButtons = screen.getAllByRole('button', {
      name: /Key [A-Z0-9]/i,
    });
    expect(keyButtons.length).toBeGreaterThan(0);
  }
});
```

### Event Bubbling and Delegation

Test that events propagate correctly through component tree:

```typescript
it('opens dialog when key is clicked', async () => {
  const user = userEvent.setup();
  renderConfigPage();

  await waitFor(() => {
    expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
  });

  // KeyboardVisualizer (child) triggers dialog in ConfigPage (parent)
  const keyButtons = screen.getAllByRole('button', {
    name: /Key [A-Z0-9]/i,
  });

  await user.click(keyButtons[0]);

  // Verify dialog opens (managed by parent)
  await waitFor(() => {
    expect(screen.getByRole('dialog')).toBeInTheDocument();
  });
});
```

### Context Propagation

Test that context values propagate correctly:

```typescript
it('DeviceScopeToggle receives real devices from API', async () => {
  renderConfigPage();

  await waitFor(() => {
    expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
  });

  // DeviceScopeToggle should receive devices from QueryClient context
  expect(screen.getByText(/Mapping Scope/i)).toBeInTheDocument();
  expect(screen.getByText('Global')).toBeInTheDocument();
  expect(screen.getByText('Device-Specific')).toBeInTheDocument();
});
```

## Async Patterns and Best Practices

Integration tests often involve async operations. Follow these patterns:

### Using waitFor()

`waitFor()` is essential for async assertions:

```typescript
// ✅ Good: Wait for async state update
await waitFor(() => {
  expect(screen.getByText('Loaded')).toBeInTheDocument();
});

// ❌ Bad: Immediate assertion (race condition)
expect(screen.getByText('Loaded')).toBeInTheDocument();
```

**waitFor() Options**:
```typescript
await waitFor(
  () => {
    expect(screen.getByText('Data')).toBeInTheDocument();
  },
  {
    timeout: 5000,  // Max wait time (default: 1000ms)
    interval: 100,  // Check interval (default: 50ms)
  }
);
```

### Using findBy Queries

`findBy` queries combine `getBy` + `waitFor`:

```typescript
// ✅ Good: findBy waits automatically
const element = await screen.findByText('Loaded');
expect(element).toBeInTheDocument();

// Equivalent to:
await waitFor(() => {
  expect(screen.getByText('Loaded')).toBeInTheDocument();
});
```

**When to use**:
- Use `findBy` when you expect element to appear after async operation
- Use `waitFor` + `getBy` when you need custom wait logic or multiple assertions

### Handling User Events

Always use `userEvent.setup()` for realistic interactions:

```typescript
const user = userEvent.setup();

// Click
await user.click(button);

// Type
await user.type(input, 'text');

// Select option
await user.selectOptions(select, 'value');

// Keyboard shortcuts
await user.keyboard('{Escape}');

// Hover
await user.hover(element);
```

**Important**: All `userEvent` methods are async and must be awaited.

### Cleaning Up State

Reset state in `beforeEach` to ensure test isolation:

```typescript
beforeEach(() => {
  // Reset store state
  const store = useConfigStore.getState();
  store.config = null;
  store.loading = false;
  store.error = null;

  // Reset WebSocket state
  setDaemonState({
    activeProfile: 'default',
    layer: 'base',
    modifiers: [],
    locks: [],
    connected: true,
  });
});
```

## Common Patterns

### Testing Form Validation

```typescript
it('validates threshold value', async () => {
  const user = userEvent.setup();
  renderConfigPage();

  await waitFor(() => {
    expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
  });

  const keyButtons = screen.getAllByRole('button', {
    name: /Key [A-Z0-9]/i,
  });

  await user.click(keyButtons[0]);

  await waitFor(() => {
    expect(screen.getByRole('dialog')).toBeInTheDocument();
  });

  // Select tap/hold action
  const actionTypeSelector = screen.getByRole('combobox', {
    name: /Action Type/i,
  });
  await user.selectOptions(actionTypeSelector, 'tap_hold');

  // Try invalid threshold
  const thresholdInput = screen.getByRole('spinbutton', {
    name: /Threshold/i,
  });
  await user.clear(thresholdInput);
  await user.type(thresholdInput, '-100');

  // Verify validation error
  await waitFor(() => {
    expect(
      screen.getByText(/Threshold must be positive/i)
    ).toBeInTheDocument();
  });
});
```

### Testing Conditional Rendering

```typescript
it('device selector only visible in device-specific mode', async () => {
  const user = userEvent.setup();
  renderConfigPage();

  await waitFor(() => {
    expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
  });

  // Initially hidden (global mode)
  expect(screen.queryByText(/Select Device/i)).not.toBeInTheDocument();

  // Switch to device-specific mode
  const deviceSpecificButton = screen.getByRole('button', {
    name: /Device-Specific/i,
  });
  await user.click(deviceSpecificButton);

  // Now visible
  await waitFor(() => {
    expect(screen.getByText(/Select Device/i)).toBeInTheDocument();
  });
});
```

### Testing Keyboard Interactions

```typescript
it('closes dialog on Escape key', async () => {
  const user = userEvent.setup();
  renderConfigPage();

  await waitFor(() => {
    expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
  });

  const keyButtons = screen.getAllByRole('button', {
    name: /Key [A-Z0-9]/i,
  });

  await user.click(keyButtons[0]);

  await waitFor(() => {
    expect(screen.getByRole('dialog')).toBeInTheDocument();
  });

  // Press Escape
  await user.keyboard('{Escape}');

  // Verify dialog closes
  await waitFor(() => {
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
  });
});
```

### Testing Drag-and-Drop

```typescript
it('supports keyboard-only drag-and-drop (WCAG 2.2 AA)', async () => {
  const user = userEvent.setup();
  renderConfigPage();

  await waitFor(() => {
    expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
  });

  // Find draggable key
  const dragKey = screen.getByRole('button', { name: /^A$/i });

  // Focus element
  dragKey.focus();
  expect(dragKey).toHaveFocus();

  // Space to grab
  await user.keyboard('{Space}');

  // Verify accessibility attributes
  expect(dragKey).toHaveAttribute('aria-label');
});
```

## Troubleshooting

### Element Not Found

**Problem**: `Unable to find element with text "..."`

**Solutions**:
```typescript
// 1. Use waitFor() for async rendering
await waitFor(() => {
  expect(screen.getByText('Text')).toBeInTheDocument();
});

// 2. Use findBy (combines getBy + waitFor)
const element = await screen.findByText('Text');

// 3. Check if element is in document
const element = screen.queryByText('Text');
if (element) {
  expect(element).toBeInTheDocument();
}

// 4. Debug what's actually rendered
screen.debug(); // Prints entire DOM
```

### Act() Warnings

**Problem**: `Warning: An update to Component inside a test was not wrapped in act(...)`

**Solutions**:
```typescript
// 1. Use waitFor() for state updates
await waitFor(() => {
  expect(screen.getByText('Updated')).toBeInTheDocument();
});

// 2. Await all user interactions
await user.click(button);  // Don't forget await!

// 3. Wait for async operations to complete
await waitFor(() => {
  expect(mockFn).toHaveBeenCalled();
});
```

### Router Errors

**Problem**: `useParams() may be used only in the context of a <Router> component`

**Solutions**:
```typescript
// Wrap component with router
renderWithProviders(<Component />, {
  wrapWithRouter: true,
  routerInitialEntries: ['/path'],
});

// For components using useParams
renderWithProviders(<Component />, {
  wrapWithRouter: true,
  routerInitialEntries: ['/profiles/gaming'],
  routerPath: '/profiles/:name',
});
```

### WebSocket State Not Updating

**Problem**: Component doesn't update after `setDaemonState()`

**Solutions**:
```typescript
// 1. Ensure component subscribes to channel
// Component must call useUnifiedApi() or similar hook

// 2. Use waitFor() for state propagation
setDaemonState({ activeProfile: 'gaming' });

await waitFor(() => {
  expect(screen.getByText('gaming')).toBeInTheDocument();
});

// 3. Check WebSocket subscription in component
// Verify component calls subscribe() for 'daemon-state' channel
```

### Slow Tests

**Problem**: Integration tests take too long

**Solutions**:
```typescript
// 1. Reduce wait times (but keep realistic)
await waitFor(
  () => {
    expect(screen.getByText('Data')).toBeInTheDocument();
  },
  { timeout: 1000 }  // Reduce from default 5000ms
);

// 2. Mock expensive operations
vi.mock('../../utils/expensiveCalculation', () => ({
  calculate: vi.fn(() => 'mocked result'),
}));

// 3. Use separate test categories
// Run integration tests separately: npm run test:integration
```

### Flaky Tests

**Problem**: Tests pass sometimes, fail other times

**Solutions**:
```typescript
// 1. Add explicit waits for async operations
await waitFor(() => {
  expect(screen.getByText('Data')).toBeInTheDocument();
});

// 2. Reset state properly in beforeEach
beforeEach(() => {
  resetWebSocketState();
  useConfigStore.getState().reset();
});

// 3. Avoid relying on timing
// Use waitFor() instead of setTimeout()

// 4. Clean up after tests
afterEach(() => {
  vi.clearAllMocks();
});
```

## Summary

Integration tests verify that multiple components work together correctly. Key principles:

✅ **Test Complete Workflows**: Full user journeys, not isolated components
✅ **Use Semantic Queries**: `getByRole`, `getByLabelText` for accessibility
✅ **Wait for Async Operations**: `waitFor()`, `findBy` queries
✅ **Simulate Real User Behavior**: `userEvent.setup()` for interactions
✅ **Test WebSocket Integration**: `setDaemonState()`, `sendLatencyUpdate()`
✅ **Clean Up State**: Reset stores and WebSocket state in `beforeEach`

For more examples, see:
- `src/pages/__integration__/ConfigPage.integration.test.tsx` - Full page testing
- `tests/integration/websocket-msw.test.ts` - WebSocket infrastructure testing
- `src/hooks/__tests__/useUnifiedApi.test.ts` - Hook testing with WebSocket

For unit testing patterns, see [unit-testing-guide.md](./unit-testing-guide.md).
