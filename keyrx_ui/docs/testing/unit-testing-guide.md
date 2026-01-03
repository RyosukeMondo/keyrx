# Unit Testing Guide

This guide explains how to write unit tests for the KeyRx UI frontend using Vitest, React Testing Library, and MSW (Mock Service Worker) for WebSocket mocking.

## Table of Contents

- [Quick Start](#quick-start)
- [Test Structure](#test-structure)
- [Testing Components](#testing-components)
- [WebSocket Mocking with MSW](#websocket-mocking-with-msw)
- [Common Patterns](#common-patterns)
- [When to Use Unit vs Integration Tests](#when-to-use-unit-vs-integration-tests)
- [Best Practices](#best-practices)
- [Troubleshooting](#troubleshooting)

## Quick Start

### Running Tests

```bash
# Run unit tests (fast, default)
npm test

# Run unit tests in watch mode
npm run test:watch

# Run unit tests with coverage
npm run test:coverage

# Run specific test file
npm test -- ActiveProfileCard.test.tsx
```

### Basic Test Template

```typescript
import { describe, it, expect, beforeEach } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import { renderWithProviders } from '../../tests/testUtils';
import { MyComponent } from './MyComponent';

describe('MyComponent', () => {
  beforeEach(() => {
    // Setup runs before each test
  });

  it('renders component', () => {
    renderWithProviders(<MyComponent />);
    expect(screen.getByText('Hello')).toBeInTheDocument();
  });
});
```

## Test Structure

### File Naming Convention

- **Unit tests**: Place next to component: `MyComponent.test.tsx`
- **Location**: Same directory as the component being tested
- **Pattern**: `[ComponentName].test.tsx` or `[ComponentName].test.ts`

### Test Organization

```typescript
describe('ComponentName', () => {
  // Setup and mocks
  const mockData = { /* ... */ };

  beforeEach(() => {
    // Reset state before each test
  });

  // Group related tests
  describe('rendering', () => {
    it('renders loading state', () => { /* ... */ });
    it('renders empty state', () => { /* ... */ });
    it('renders with data', () => { /* ... */ });
  });

  describe('user interactions', () => {
    it('handles button click', async () => { /* ... */ });
    it('submits form', async () => { /* ... */ });
  });

  describe('WebSocket updates', () => {
    it('updates on state change', async () => { /* ... */ });
  });
});
```

## Testing Components

### Using `renderWithProviders`

The `renderWithProviders` helper wraps your component with necessary test providers:

```typescript
import { renderWithProviders } from '../../tests/testUtils';

// Basic usage (includes React Query and WASM providers)
renderWithProviders(<MyComponent />);

// With router (for components using useNavigate, useParams, etc.)
renderWithProviders(<MyComponent />, {
  wrapWithRouter: true
});

// With custom initial route
renderWithProviders(<MyComponent />, {
  wrapWithRouter: true,
  routerInitialEntries: ['/profiles/Gaming/config']
});

// Disable WASM provider (rare cases)
renderWithProviders(<MyComponent />, {
  wrapWithWasm: false
});
```

### Testing User Interactions

```typescript
import userEvent from '@testing-library/user-event';

it('handles button click', async () => {
  const user = userEvent.setup();
  renderWithProviders(<MyComponent />);

  const button = screen.getByRole('button', { name: 'Save' });
  await user.click(button);

  expect(screen.getByText('Saved!')).toBeInTheDocument();
});

it('handles text input', async () => {
  const user = userEvent.setup();
  renderWithProviders(<MyComponent />);

  const input = screen.getByRole('textbox', { name: 'Profile name' });
  await user.type(input, 'Gaming');

  expect(input).toHaveValue('Gaming');
});
```

### Testing Async Behavior

Use `waitFor` for async assertions:

```typescript
import { waitFor } from '@testing-library/react';

it('updates after async operation', async () => {
  renderWithProviders(<MyComponent />);

  // Trigger async operation
  const button = screen.getByRole('button', { name: 'Load' });
  await userEvent.click(button);

  // Wait for async update
  await waitFor(() => {
    expect(screen.getByText('Loaded!')).toBeInTheDocument();
  });
});

// Use findBy queries (built-in waitFor)
it('shows data after load', async () => {
  renderWithProviders(<MyComponent />);

  // findBy* queries automatically wait
  const heading = await screen.findByRole('heading', { name: 'Data' });
  expect(heading).toBeInTheDocument();
});
```

## WebSocket Mocking with MSW

Our tests use MSW (Mock Service Worker) for automatic WebSocket mocking. No manual setup required!

### Available WebSocket Helpers

Import from `@/test/mocks/websocketHelpers`:

```typescript
import {
  setDaemonState,
  sendLatencyUpdate,
  sendKeyEvent,
  sendServerMessage,
  waitForWebSocketConnection
} from '@/test/mocks/websocketHelpers';
```

### Simulating Daemon State Changes

```typescript
import { setDaemonState } from '@/test/mocks/websocketHelpers';

it('updates when profile changes', async () => {
  renderWithProviders(<ActiveProfileCard />);

  // Simulate daemon activating a profile
  setDaemonState({ activeProfile: 'Gaming' });

  // Wait for component to update
  await waitFor(() => {
    expect(screen.getByText('Gaming')).toBeInTheDocument();
  });
});

it('updates when layer changes', async () => {
  renderWithProviders(<LayerIndicator />);

  // Simulate layer change
  setDaemonState({ layer: 'fn' });

  await waitFor(() => {
    expect(screen.getByText('fn')).toBeInTheDocument();
  });
});

it('shows modifier state', async () => {
  renderWithProviders(<ModifierDisplay />);

  // Simulate modifier press
  setDaemonState({
    modifiers: ['MD_00', 'MD_01'],
    layer: 'shift'
  });

  await waitFor(() => {
    expect(screen.getByText(/2 modifiers active/)).toBeInTheDocument();
  });
});
```

### Simulating Latency Updates

```typescript
import { sendLatencyUpdate } from '@/test/mocks/websocketHelpers';

it('displays latency metrics', async () => {
  renderWithProviders(<QuickStatsCard />);

  // Simulate good performance (values in microseconds)
  sendLatencyUpdate({
    min: 200,
    avg: 500,
    max: 1000,
    p95: 800,
    p99: 950
  });

  await waitFor(() => {
    expect(screen.getByText(/0.50ms/)).toBeInTheDocument(); // avg
  });
});

it('shows latency warning', async () => {
  renderWithProviders(<LatencyMonitor />);

  // Simulate high latency
  sendLatencyUpdate({
    min: 5000,
    avg: 15000,
    max: 50000,
    p95: 30000,
    p99: 45000
  });

  await waitFor(() => {
    expect(screen.getByText(/High latency/)).toBeInTheDocument();
  });
});
```

### Simulating Key Events

```typescript
import { sendKeyEvent } from '@/test/mocks/websocketHelpers';

it('displays key events', async () => {
  renderWithProviders(<KeyEventMonitor />);

  // Simulate key press
  sendKeyEvent({
    keyCode: 'KEY_A',
    eventType: 'press',
    input: 'KEY_A',
    output: 'KEY_B', // Remapped
    latency: 500
  });

  await waitFor(() => {
    expect(screen.getByText(/KEY_A → KEY_B/)).toBeInTheDocument();
  });
});
```

### Custom Server Messages

For advanced scenarios, use `sendServerMessage`:

```typescript
import { sendServerMessage } from '@/test/mocks/websocketHelpers';

it('handles custom daemon state', async () => {
  renderWithProviders(<MyComponent />);

  sendServerMessage('daemon-state', {
    modifiers: ['MD_CUSTOM'],
    locks: ['LK_CUSTOM'],
    layer: 'custom-layer'
  });

  await waitFor(() => {
    expect(screen.getByText('custom-layer')).toBeInTheDocument();
  });
});
```

## Common Patterns

### Testing Loading States

```typescript
it('renders loading state', () => {
  renderWithProviders(<MyComponent loading={true} />);

  const loadingElements = screen.getAllByRole('status');
  const hasAnimatePulse = loadingElements.some((el) =>
    el.classList.contains('animate-pulse')
  );
  expect(hasAnimatePulse).toBe(true);
});
```

### Testing Empty States

```typescript
it('renders empty state when no data', () => {
  renderWithProviders(<MyComponent />);

  expect(screen.getByText(/No data available/)).toBeInTheDocument();
  expect(screen.queryByRole('table')).not.toBeInTheDocument();
});
```

### Testing Error States

```typescript
it('displays error message', () => {
  renderWithProviders(<MyComponent error="Failed to load" />);

  expect(screen.getByText(/Failed to load/)).toBeInTheDocument();
  expect(screen.getByRole('alert')).toBeInTheDocument();
});
```

### Mocking Router Navigation

```typescript
import { vi } from 'vitest';

const mockNavigate = vi.fn();

vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom');
  return {
    ...actual,
    useNavigate: () => mockNavigate,
  };
});

it('navigates on button click', async () => {
  const user = userEvent.setup();
  renderWithProviders(<MyComponent />, { wrapWithRouter: true });

  const button = screen.getByRole('button', { name: 'Go to profiles' });
  await user.click(button);

  expect(mockNavigate).toHaveBeenCalledWith('/profiles');
});
```

### Testing Accessibility

```typescript
it('renders with proper aria labels', () => {
  renderWithProviders(<MyComponent profile={mockProfile} />);

  const editButton = screen.getByRole('button', {
    name: 'Edit profile Gaming'
  });
  expect(editButton).toBeInTheDocument();
});

it('renders icon with accessibility label', () => {
  renderWithProviders(<MyComponent />);

  const icon = screen.getByRole('img', { name: 'Profile icon' });
  expect(icon).toBeInTheDocument();
});

it('keyboard navigation works', async () => {
  const user = userEvent.setup();
  renderWithProviders(<MyComponent />);

  const button = screen.getByRole('button');
  await user.tab(); // Focus first element

  expect(button).toHaveFocus();

  await user.keyboard('{Enter}');
  // Verify action triggered
});
```

### Testing Conditional Rendering

```typescript
it('shows edit button only when profile exists', () => {
  const { rerender } = renderWithProviders(<MyComponent />);

  // No profile - no edit button
  expect(screen.queryByRole('button', { name: /Edit/ })).not.toBeInTheDocument();

  // With profile - edit button appears
  rerender(<MyComponent profile={mockProfile} />);
  expect(screen.getByRole('button', { name: /Edit/ })).toBeInTheDocument();
});
```

### Testing Component Updates

```typescript
it('updates when props change', async () => {
  const { rerender } = renderWithProviders(
    <MyComponent profile={{ name: 'Gaming' }} />
  );

  expect(screen.getByText('Gaming')).toBeInTheDocument();

  // Update props
  rerender(<MyComponent profile={{ name: 'Work' }} />);

  expect(screen.getByText('Work')).toBeInTheDocument();
  expect(screen.queryByText('Gaming')).not.toBeInTheDocument();
});
```

## When to Use Unit vs Integration Tests

### Use Unit Tests When:

✅ **Testing a single component in isolation**
- Example: Button, Input, Card, Icon components
- Focus: Component renders correctly, handles props, emits events

✅ **Testing pure logic functions**
- Example: Utility functions, formatters, validators
- Focus: Correct output for given input

✅ **Testing component behavior with mocked dependencies**
- Example: Component with WebSocket updates
- Focus: Component responds correctly to external events

✅ **Testing user interactions on single component**
- Example: Button clicks, form input, dropdown selection
- Focus: Event handlers called, state updated

✅ **Fast feedback is critical**
- Unit tests run in <5 seconds
- Ideal for TDD (Test-Driven Development)

### Use Integration Tests When:

❌ **Testing multiple components working together**
- Example: Full page with multiple interactive cards
- Use: Integration test with `ConfigPage.integration.test.tsx`

❌ **Testing complete user workflows**
- Example: Load profile → Edit → Validate → Save
- Use: Integration test covering entire flow

❌ **Testing router navigation between pages**
- Example: Dashboard → Profile detail → Config editor
- Use: Integration test with real routing

❌ **Testing API calls and data fetching**
- Example: Load profiles from API, handle errors, retry logic
- Use: Integration test with MSW HTTP mocking

❌ **Testing WebSocket subscription lifecycle**
- Example: Connect → Subscribe → Receive updates → Unsubscribe
- Use: Integration test verifying full connection flow

### Decision Tree

```
Does the test involve multiple components?
├─ Yes → Integration Test
└─ No → Continue
    │
    Does the test verify a complete user workflow?
    ├─ Yes → Integration Test
    └─ No → Continue
        │
        Does the test need real routing/navigation?
        ├─ Yes → Integration Test
        └─ No → Continue
            │
            Is the test focused on a single component's behavior?
            ├─ Yes → Unit Test ✓
            └─ No → Consider E2E test
```

## Best Practices

### 1. Use Semantic Queries

Prefer queries that resemble how users interact:

```typescript
// Good - Accessible by role and name
screen.getByRole('button', { name: 'Save' });
screen.getByRole('textbox', { name: 'Profile name' });

// Avoid - Relies on implementation details
screen.getByTestId('save-button');
screen.getByClassName('input-field');
```

### 2. Avoid Testing Implementation Details

```typescript
// Bad - Testing internal state
expect(component.state.count).toBe(5);

// Good - Testing user-visible behavior
expect(screen.getByText('Count: 5')).toBeInTheDocument();
```

### 3. Keep Tests Independent

```typescript
// Bad - Tests depend on each other
let sharedState;

it('test 1', () => {
  sharedState = 'value';
});

it('test 2', () => {
  expect(sharedState).toBe('value'); // Fragile!
});

// Good - Each test is independent
it('test 1', () => {
  const state = 'value';
  // Test with state
});

it('test 2', () => {
  const state = 'value';
  // Test with state
});
```

### 4. Use `beforeEach` for Common Setup

```typescript
describe('MyComponent', () => {
  let mockNavigate;

  beforeEach(() => {
    mockNavigate = vi.fn();
    // Other setup
  });

  it('test 1', () => {
    // mockNavigate is fresh for each test
  });

  it('test 2', () => {
    // mockNavigate is fresh for each test
  });
});
```

### 5. Test User-Visible Behavior, Not Code Structure

```typescript
// Bad - Testing that function was called
expect(mockHandleClick).toHaveBeenCalled();

// Good - Testing that visible result occurred
expect(screen.getByText('Success!')).toBeInTheDocument();
```

### 6. Use Descriptive Test Names

```typescript
// Bad
it('works', () => { /* ... */ });

// Good
it('displays profile name when profile is active', () => { /* ... */ });
it('navigates to config page when edit button is clicked', async () => { /* ... */ });
it('shows error message when profile load fails', () => { /* ... */ });
```

### 7. Async Testing Best Practices

```typescript
// Bad - No waiting, will fail randomly
setDaemonState({ activeProfile: 'Gaming' });
expect(screen.getByText('Gaming')).toBeInTheDocument(); // Might fail!

// Good - Wait for async update
setDaemonState({ activeProfile: 'Gaming' });
await waitFor(() => {
  expect(screen.getByText('Gaming')).toBeInTheDocument();
});

// Better - Use findBy (built-in waitFor)
setDaemonState({ activeProfile: 'Gaming' });
expect(await screen.findByText('Gaming')).toBeInTheDocument();
```

### 8. Clean Up After Tests

MSW and React Testing Library handle most cleanup automatically, but for custom resources:

```typescript
import { afterEach } from 'vitest';

afterEach(() => {
  // Clear mocks
  vi.clearAllMocks();

  // Clean up subscriptions or listeners
  cleanup();
});
```

## Troubleshooting

### Tests Timing Out

**Problem**: Test hangs and times out

**Solutions**:
```typescript
// 1. Check if async operation is missing await
await waitFor(() => {
  expect(screen.getByText('Result')).toBeInTheDocument();
});

// 2. Increase timeout for slow operations
await waitFor(() => {
  expect(screen.getByText('Result')).toBeInTheDocument();
}, { timeout: 10000 }); // 10 seconds

// 3. Use findBy queries (automatically wait)
expect(await screen.findByText('Result')).toBeInTheDocument();
```

### "Unable to find element" Errors

**Problem**: `Unable to find an element with the text: "..."`

**Solutions**:
```typescript
// 1. Check if element is rendered with different text
screen.debug(); // Print current DOM

// 2. Use flexible matchers
screen.getByText(/partial match/i); // Case-insensitive regex

// 3. Wait for async rendering
await screen.findByText('Text'); // Wait for element to appear

// 4. Check if element is inside a different container
const container = screen.getByRole('dialog');
within(container).getByText('Text');
```

### Act Warnings

**Problem**: `Warning: An update to Component inside a test was not wrapped in act(...)`

**Solutions**:
```typescript
// 1. Use async queries and await properly
await waitFor(() => {
  expect(screen.getByText('Updated')).toBeInTheDocument();
});

// 2. For user interactions, always await
await user.click(button);
await user.type(input, 'text');

// 3. WebSocket updates should be awaited
setDaemonState({ activeProfile: 'Gaming' });
await waitFor(() => {
  expect(screen.getByText('Gaming')).toBeInTheDocument();
});
```

### WebSocket Events Not Triggering Updates

**Problem**: Component doesn't update when WebSocket events are sent

**Solutions**:
```typescript
// 1. Ensure component is rendered before sending events
renderWithProviders(<MyComponent />);
setDaemonState({ activeProfile: 'Gaming' });
await waitFor(() => { /* ... */ });

// 2. Check component is subscribed to correct channel
// Component must use useUnifiedApi or similar hook

// 3. Verify event data format matches expected type
setDaemonState({
  modifiers: [], // Array, not undefined
  locks: [],     // Array, not undefined
  layer: 'base', // String
  activeProfile: 'Gaming'
});

// 4. Wait for WebSocket connection (rare)
await waitForWebSocketConnection();
setDaemonState({ activeProfile: 'Gaming' });
```

### Router Errors

**Problem**: `useNavigate() may be used only in the context of a <Router> component`

**Solution**:
```typescript
// Wrap with router
renderWithProviders(<MyComponent />, { wrapWithRouter: true });
```

**Problem**: `useParams() may be used only in the context of a <Route> component`

**Solution**:
```typescript
// Provide route parameters via initial entries
renderWithProviders(<MyComponent />, {
  wrapWithRouter: true,
  routerInitialEntries: ['/profiles/Gaming/config']
});
```

## Example: Complete Component Test

Here's a complete example testing `ActiveProfileCard`:

```typescript
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import { renderWithProviders } from '../../tests/testUtils';
import userEvent from '@testing-library/user-event';
import { ActiveProfileCard } from './ActiveProfileCard';
import { setDaemonState } from '@/test/mocks/websocketHelpers';

const mockNavigate = vi.fn();

vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom');
  return {
    ...actual,
    useNavigate: () => mockNavigate,
  };
});

describe('ActiveProfileCard', () => {
  const mockProfile = {
    name: 'Gaming',
    layers: 5,
    mappings: 127,
    modifiedAt: '2 hours ago',
  };

  beforeEach(() => {
    mockNavigate.mockClear();
  });

  describe('rendering', () => {
    it('renders loading state', () => {
      renderWithProviders(<ActiveProfileCard loading={true} />, {
        wrapWithRouter: true
      });

      const loadingElements = screen.getAllByRole('status');
      const hasAnimatePulse = loadingElements.some((el) =>
        el.classList.contains('animate-pulse')
      );
      expect(hasAnimatePulse).toBe(true);
    });

    it('renders empty state when no profile', () => {
      renderWithProviders(<ActiveProfileCard />, { wrapWithRouter: true });

      expect(screen.getByText('Active Profile')).toBeInTheDocument();
      expect(
        screen.getByText(/No profile is currently active/)
      ).toBeInTheDocument();
    });

    it('renders profile data correctly', () => {
      renderWithProviders(<ActiveProfileCard profile={mockProfile} />, {
        wrapWithRouter: true
      });

      expect(screen.getByText('Gaming')).toBeInTheDocument();
      expect(screen.getByText('• 5 Layers')).toBeInTheDocument();
      expect(screen.getByText('• Modified: 2 hours ago')).toBeInTheDocument();
      expect(screen.getByText('• 127 key mappings')).toBeInTheDocument();
    });
  });

  describe('user interactions', () => {
    it('navigates to profiles page when manage button clicked', async () => {
      const user = userEvent.setup();
      renderWithProviders(<ActiveProfileCard />, { wrapWithRouter: true });

      const button = screen.getByRole('button', {
        name: 'Go to profiles page',
      });
      await user.click(button);

      expect(mockNavigate).toHaveBeenCalledWith('/profiles');
    });

    it('navigates to config page when edit button clicked', async () => {
      const user = userEvent.setup();
      renderWithProviders(<ActiveProfileCard profile={mockProfile} />, {
        wrapWithRouter: true
      });

      const editButton = screen.getByRole('button', {
        name: 'Edit profile Gaming',
      });
      await user.click(editButton);

      expect(mockNavigate).toHaveBeenCalledWith('/profiles/Gaming/config');
    });
  });

  describe('WebSocket updates', () => {
    it('updates when daemon activates profile', async () => {
      renderWithProviders(<ActiveProfileCard />, { wrapWithRouter: true });

      // Simulate daemon activating profile
      setDaemonState({ activeProfile: 'Gaming' });

      await waitFor(() => {
        expect(screen.getByText('Gaming')).toBeInTheDocument();
      });
    });
  });

  describe('accessibility', () => {
    it('renders profile icon with accessibility label', () => {
      renderWithProviders(<ActiveProfileCard profile={mockProfile} />, {
        wrapWithRouter: true
      });

      const icon = screen.getByRole('img', { name: 'Profile icon' });
      expect(icon).toBeInTheDocument();
    });

    it('renders edit button with proper aria-label', () => {
      renderWithProviders(<ActiveProfileCard profile={mockProfile} />, {
        wrapWithRouter: true
      });

      const editButton = screen.getByRole('button', {
        name: 'Edit profile Gaming',
      });
      expect(editButton).toBeInTheDocument();
    });
  });
});
```

## Additional Resources

- **Integration Testing Guide**: `integration-testing-guide.md` - For multi-component tests
- **Vitest Documentation**: https://vitest.dev/
- **React Testing Library**: https://testing-library.com/react
- **MSW Documentation**: https://mswjs.io/
- **Testing Library Queries**: https://testing-library.com/docs/queries/about

## Getting Help

If you encounter issues not covered here:

1. Check the troubleshooting section above
2. Review existing tests in the codebase for examples
3. Consult the React Testing Library documentation
4. Ask in team chat or create an issue
