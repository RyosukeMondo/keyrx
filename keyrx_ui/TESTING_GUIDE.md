# KeyRx UI Testing Guide

This guide covers all testing utilities and best practices for the KeyRx UI project.

## Table of Contents

- [Overview](#overview)
- [Test Factories](#test-factories)
- [Enhanced Render Helpers](#enhanced-render-helpers)
- [User Interaction Utilities](#user-interaction-utilities)
- [Async Testing Utilities](#async-testing-utilities)
- [Assertion Helpers](#assertion-helpers)
- [Storybook Integration](#storybook-integration)
- [Visual Regression Testing](#visual-regression-testing)
- [Best Practices](#best-practices)

## Overview

The KeyRx UI test suite uses industry-standard tools for comprehensive testing:

- **Vitest** - Fast unit test runner with coverage
- **React Testing Library** - Component testing with accessibility focus
- **@faker-js/faker** - Realistic test data generation
- **jest-websocket-mock** - WebSocket testing
- **MSW** - API mocking
- **Storybook** - Component development and visual testing
- **Chromatic** - Visual regression testing

## Test Factories

Test factories generate realistic mock data using faker-js. All factories support partial overrides.

### Basic Usage

```typescript
import { createProfile, createDevice, seed } from '../tests/factories';

test('renders profile', () => {
  const profile = createProfile();
  // profile has realistic random data
});

test('renders active profile', () => {
  const profile = createProfile({ isActive: true });
  // Specific properties can be overridden
});
```

### Deterministic Testing

For visual regression and snapshot tests, use seeding:

```typescript
import { seed } from '../tests/factories';

beforeEach(() => {
  seed(12345); // Same data every time
});

test('snapshot test', () => {
  const profile = createProfile();
  expect(profile).toMatchSnapshot();
});
```

### Available Factories

#### Profiles

```typescript
import {
  createProfile,
  createProfiles,
  createActivationResult,
} from '../tests/factories';

// Single profile
const profile = createProfile({
  name: 'Gaming',
  isActive: true,
  deviceCount: 2,
  keyCount: 24,
});

// Multiple profiles
const profiles = createProfiles(5);

// Activation result
const result = createActivationResult({
  success: true,
  compiledSize: 2048,
});
```

#### Devices

```typescript
import { createDevice, createDevices } from '../tests/factories';

// Single device
const device = createDevice({
  isConnected: true,
  layoutPreset: 'ANSI_104',
});

// Multiple devices
const devices = createDevices(3, { scope: 'global' });
```

#### Key Events & Metrics

```typescript
import {
  createKeyEvent,
  createLatencyMetrics,
  createDaemonState,
} from '../tests/factories';

const event = createKeyEvent({
  eventType: 'press',
  keyCode: 'KEY_A',
});

const metrics = createLatencyMetrics({
  avg: 125,
  p95: 245,
});

const state = createDaemonState({
  activeLayer: 'gaming',
  modifiers: ['MD_00'],
});
```

#### RPC Messages

```typescript
import {
  createQueryMessage,
  createCommandMessage,
  createResponse,
  createConnectedMessage,
} from '../tests/factories';

const query = createQueryMessage('get_profiles');
const cmd = createCommandMessage('activate_profile', { name: 'gaming' });
const response = createResponse('req-123', { success: true });
const connected = createConnectedMessage();
```

## Enhanced Render Helpers

### renderWithProviders

Renders components with necessary context providers:

```typescript
import { renderWithProviders } from '../tests/testUtils';

test('renders component', () => {
  const { getByText } = renderWithProviders(<MyComponent />);
  expect(getByText('Hello')).toBeInTheDocument();
});

// With routing
renderWithProviders(<ProfilesPage />, {
  wrapWithRouter: true,
  routerInitialEntries: ['/profiles'],
});

// Without WASM (for simple components)
renderWithProviders(<Button />, {
  wrapWithWasm: false,
});
```

### renderPage

Shorthand for pages that need routing:

```typescript
import { renderPage } from '../tests/testUtils';

test('renders page', () => {
  renderPage(<ProfilesPage />);
  // Automatically includes router
});
```

### renderPure

For testing presentational components in isolation:

```typescript
import { renderPure } from '../tests/testUtils';

test('renders pure component', () => {
  renderPure(<Button onClick={() => {}}>Click me</Button>);
  // No providers, faster tests
});
```

## User Interaction Utilities

### setupUser

Consistent user interaction setup:

```typescript
import { setupUser } from '../tests/testUtils';

test('user interactions', async () => {
  const user = setupUser();
  const button = screen.getByRole('button');

  await user.click(button);
  await user.type(input, 'hello');
});
```

### Helper Functions

```typescript
import {
  typeIntoField,
  clickElement,
  selectOption,
} from '../tests/testUtils';

// Type into field (clears first)
await typeIntoField(screen.getByLabelText('Name'), 'John Doe');

// Click element
await clickElement(screen.getByRole('button', { name: 'Save' }));

// Select option
await selectOption(screen.getByLabelText('Layout'), 'ANSI_104');
```

## Async Testing Utilities

### waitForElement

Wait for elements to appear:

```typescript
import { waitForElement } from '../tests/testUtils';

test('waits for element', async () => {
  renderWithProviders(<AsyncComponent />);

  const result = await waitForElement(
    () => screen.getByText('Loaded!'),
    { timeout: 5000 }
  );
});
```

### waitForLoadingToFinish

Wait for loading states:

```typescript
import { waitForLoadingToFinish } from '../tests/testUtils';

test('waits for loading', async () => {
  renderWithProviders(<DataComponent />);

  await waitForLoadingToFinish(); // Default: 'Loading...'
  await waitForLoadingToFinish('Fetching...', 3000); // Custom text and timeout

  expect(screen.getByText('Data loaded')).toBeInTheDocument();
});
```

### waitForData

Combined loading + data assertion:

```typescript
import { waitForData } from '../tests/testUtils';

test('waits for data', async () => {
  renderWithProviders(<ProfilesPage />);

  await waitForData(() => screen.getByText('Gaming Profile'));
});
```

## Assertion Helpers

### assertAccessibleName

Verify accessible names (ARIA labels):

```typescript
import { assertAccessibleName } from '../tests/testUtils';

test('has accessible name', () => {
  renderWithProviders(<IconButton />);
  const button = screen.getByRole('button');

  assertAccessibleName(button, 'Close dialog');
});
```

### assertFieldError

Verify form validation:

```typescript
import { assertFieldError } from '../tests/testUtils';

test('shows validation error', async () => {
  const { getByLabelText } = renderWithProviders(<Form />);
  await submitInvalidForm();

  assertFieldError(getByLabelText('Email'), 'Invalid email address');
});
```

### assertVisibleAndAccessible

Comprehensive visibility check:

```typescript
import { assertVisibleAndAccessible } from '../tests/testUtils';

test('element is visible and accessible', () => {
  renderWithProviders(<Modal />);
  const dialog = screen.getByRole('dialog');

  assertVisibleAndAccessible(dialog);
  // Checks: visible, in DOM, not aria-hidden
});
```

## Storybook Integration

### Writing Stories

Create stories in `*.stories.tsx` files:

```typescript
import type { Meta, StoryObj } from '@storybook/react';
import { MyComponent } from './MyComponent';
import { createProfile } from '../../tests/factories';

const meta = {
  title: 'Components/MyComponent',
  component: MyComponent,
  tags: ['autodocs'],
} satisfies Meta<typeof MyComponent>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  args: {
    profile: createProfile(),
  },
};

export const Active: Story = {
  args: {
    profile: createProfile({ isActive: true }),
  },
};
```

### Using Factories in Stories

Leverage test factories for consistent data:

```typescript
import { seed, createProfiles } from '../../tests/factories';

// Deterministic data for visual regression
seed(42);

export const MultipleProfiles: Story = {
  args: {
    profiles: createProfiles(5),
  },
};
```

### Running Storybook

```bash
# Start Storybook dev server
npm run storybook

# Build static Storybook
npm run build-storybook
```

## Visual Regression Testing

### Chromatic Integration

Stories automatically become visual tests with Chromatic:

```typescript
export const VisualBaseline: Story = {
  args: {
    profile: createProfile({
      name: 'Baseline',
      isActive: true,
    }),
  },
  parameters: {
    chromatic: {
      // Test at multiple viewports
      viewports: [320, 768, 1024, 1920],
      // Delay capture for animations
      delay: 300,
    },
  },
};
```

### Running Chromatic

```bash
# Run visual regression tests (requires Chromatic account)
npx chromatic --project-token=<your-token>
```

## Best Practices

### 1. Use Factories Instead of Hardcoded Data

```typescript
// ❌ Bad
const profile = {
  name: 'test',
  createdAt: '2024-01-01',
  // ... hardcoded values
};

// ✅ Good
const profile = createProfile({
  name: 'test', // Only override what matters for the test
});
```

### 2. Prefer User-Centric Queries

```typescript
// ❌ Bad
const button = container.querySelector('.btn-primary');

// ✅ Good
const button = screen.getByRole('button', { name: 'Save profile' });
```

### 3. Use Async Utilities for Dynamic Content

```typescript
// ❌ Bad
expect(screen.getByText('Loaded')).toBeInTheDocument(); // May fail if not loaded yet

// ✅ Good
await waitForData(() => screen.getByText('Loaded'));
```

### 4. Test Accessibility

```typescript
test('is accessible', async () => {
  const { container } = renderWithProviders(<MyComponent />);

  const results = await runA11yAudit(container);
  expect(results).toHaveNoViolations();

  assertVisibleAndAccessible(screen.getByRole('button'));
});
```

### 5. Use Deterministic Seeds for Snapshots

```typescript
import { seed } from '../tests/factories';

beforeEach(() => {
  seed(12345); // Consistent across runs
});

test('matches snapshot', () => {
  const profile = createProfile();
  expect(profile).toMatchSnapshot();
});
```

### 6. Organize Tests by Feature

```
src/
  components/
    Button.tsx
    Button.test.tsx        # Unit tests
    Button.stories.tsx     # Storybook stories
    Button.a11y.test.tsx   # Accessibility tests
```

### 7. Use Debug Utilities When Stuck

```typescript
import { logDOMTree, logAccessibilityTree } from '../tests/testUtils';

test('debug test', () => {
  renderWithProviders(<MyComponent />);

  logDOMTree(); // See current DOM
  logAccessibilityTree(); // See accessibility tree
});
```

## Coverage Requirements

- **Overall**: ≥80% code coverage
- **Critical Paths**: ≥90% code coverage
- **Accessibility**: Zero WCAG 2.2 Level AA violations

Run coverage:

```bash
npm test -- --coverage
```

## CI/CD Integration

All tests run automatically on:

- Pull requests
- Pushes to main branch
- Pre-commit hooks

Quality gates enforce:

- ✅ All tests passing
- ✅ Coverage thresholds met
- ✅ Accessibility compliance
- ✅ No linting errors

## Resources

- [React Testing Library Docs](https://testing-library.com/docs/react-testing-library/intro/)
- [Vitest Docs](https://vitest.dev/)
- [Faker.js Docs](https://fakerjs.dev/)
- [Storybook Docs](https://storybook.js.org/docs)
- [Chromatic Docs](https://www.chromatic.com/docs/)
- [WCAG 2.2 Guidelines](https://www.w3.org/WAI/WCAG22/quickref/)

## Need Help?

- Check existing tests in `src/**/*.test.tsx` for examples
- Review component stories in `src/**/*.stories.tsx`
- Read WebSocket testing guide in `tests/WEBSOCKET_TESTING.md`
- See test infrastructure improvements in `TEST_INFRASTRUCTURE_IMPROVEMENTS.md`
