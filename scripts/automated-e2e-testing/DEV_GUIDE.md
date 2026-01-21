# Developer Guide: Automated E2E Testing

A practical guide for contributing to the automated E2E testing system. Learn how to add new test cases, update expected results, and write custom fix strategies.

## Table of Contents

- [Adding Test Cases](#adding-test-cases)
- [Updating Expected Results](#updating-expected-results)
- [Writing Fix Strategies](#writing-fix-strategies)
- [Running Tests Locally](#running-tests-locally)
- [Best Practices](#best-practices)

## Adding Test Cases

### Step 1: Define Test Case Structure

Test cases follow the **TestCase** interface:

```typescript
interface TestCase {
  id: string;                    // Unique identifier (e.g., "status-healthy")
  name: string;                  // Human-readable name
  endpoint: string;              // API endpoint being tested
  scenario: string;              // Scenario description
  setup: (client: ApiClient) => Promise<any>;
  execute: (client: ApiClient, context: any) => Promise<any>;
  assert: (actual: any, expected: any) => ValidationResult;
  cleanup?: (client: ApiClient, context: any) => Promise<void>;
}
```

### Step 2: Create Your Test

Add to `scripts/test-cases/api-tests.ts`:

```typescript
import { TestCase } from './test-utils';
import { ApiClient } from '../api-client/client';

const createProfileTest: TestCase = {
  id: 'profiles-create',
  name: 'POST /api/profiles - Create new profile',
  endpoint: '/api/profiles',
  scenario: 'create',

  // Setup: Prepare test environment
  async setup(client: ApiClient) {
    // Ensure profile doesn't exist (cleanup from previous run)
    try {
      await client.deleteProfile('test-profile');
    } catch {
      // Profile doesn't exist, that's fine
    }

    return { profileName: 'test-profile' };
  },

  // Execute: Call the API
  async execute(client: ApiClient, context: any) {
    const response = await client.createProfile(context.profileName);
    return response;
  },

  // Assert: Validate the response
  assert(actual: any, expected: any) {
    if (actual.status !== 200) {
      return {
        passed: false,
        message: `Expected status 200, got ${actual.status}`,
      };
    }

    if (actual.data.name !== 'test-profile') {
      return {
        passed: false,
        message: `Expected name 'test-profile', got '${actual.data.name}'`,
        diff: [{
          path: 'data.name',
          expected: 'test-profile',
          actual: actual.data.name,
        }],
      };
    }

    return { passed: true };
  },

  // Cleanup: Remove test data (always runs)
  async cleanup(client: ApiClient, context: any) {
    try {
      await client.deleteProfile(context.profileName);
    } catch {
      // Best effort cleanup
    }
  },
};

// Add to test suite
export function getAllTestCases(): TestCase[] {
  return [
    // ... existing tests
    createProfileTest,
  ];
}
```

### Step 3: Handle Edge Cases

```typescript
// Test: Duplicate profile creation (should fail)
const createDuplicateProfileTest: TestCase = {
  id: 'profiles-create-duplicate',
  name: 'POST /api/profiles - Reject duplicate profile',
  endpoint: '/api/profiles',
  scenario: 'duplicate',

  async setup(client: ApiClient) {
    // Create profile first
    await client.createProfile('duplicate-test');
    return { profileName: 'duplicate-test' };
  },

  async execute(client: ApiClient, context: any) {
    try {
      // Try to create same profile again
      const response = await client.createProfile(context.profileName);
      return { status: response.status, error: null };
    } catch (error) {
      // Expected to fail
      return { status: error.response?.status || 500, error: error.message };
    }
  },

  assert(actual: any, expected: any) {
    // Should get 409 Conflict
    if (actual.status !== 409) {
      return {
        passed: false,
        message: `Expected 409 Conflict, got ${actual.status}`,
      };
    }

    return { passed: true };
  },

  async cleanup(client: ApiClient, context: any) {
    await client.deleteProfile(context.profileName);
  },
};
```

### Step 4: Test Isolation Best Practices

**DO**:
```typescript
// ✅ Use unique test data
const testId = `test-${Date.now()}`;
const profileName = `profile-${testId}`;

// ✅ Clean up in both setup and cleanup
async setup(client) {
  await client.deleteProfile('test-profile'); // Remove if exists
  return {};
}

async cleanup(client, context) {
  await client.deleteProfile('test-profile'); // Always remove
}

// ✅ Handle errors gracefully
try {
  await client.deleteProfile(name);
} catch {
  // Ignore - profile may not exist
}
```

**DON'T**:
```typescript
// ❌ Hard-coded data (conflicts with other tests)
const profileName = 'test';

// ❌ No cleanup
cleanup: undefined

// ❌ Shared global state
let globalProfile = null; // Tests aren't isolated
```

## Updating Expected Results

The expected results database defines what responses to expect from the API.

### When to Update

Update `scripts/fixtures/expected-results.json` when:

1. **API Contract Changes**: New fields, renamed fields, different types
2. **Default Values Change**: Updated default profiles, devices, etc.
3. **Error Messages Change**: Updated validation messages
4. **Status Codes Change**: Different HTTP status codes

### How to Update

#### Option 1: Manual Edit

Edit `expected-results.json`:

```json
{
  "version": "1.0",
  "endpoints": {
    "/api/profiles": {
      "scenarios": {
        "list": {
          "status": 200,
          "body": {
            "profiles": [
              {
                "name": "default",
                "active": true,
                "layers": []
              }
            ]
          }
        }
      }
    }
  }
}
```

#### Option 2: Auto-Fix

If auto-fix detects a schema mismatch, it will suggest updating the expected results:

```
Test: GET /api/profiles
Status: FAIL
Issue: Schema mismatch - field 'active' expected boolean, got string

Auto-fix suggestion:
  Update expected-results.json:
    endpoints./api/profiles.scenarios.list.body.profiles[0].active
    Change: true (boolean) → "true" (string)
```

You can then manually apply the suggested change.

### Validation

After updating, verify:

```bash
# Run tests to ensure expectations match
npm run test:e2e:api

# Check test results
# All tests should pass
```

## Writing Fix Strategies

Fix strategies automatically resolve common test failures.

### Step 1: Understand the FixStrategy Interface

```typescript
interface FixStrategy {
  // Can this strategy fix this issue?
  canFix(issue: Issue): boolean;

  // Apply the fix
  apply(issue: Issue, context: FixContext): Promise<FixResult>;
}

interface FixContext {
  daemon: DaemonFixture;
  apiClient: ApiClient;
  testCase: TestCase;
}

interface FixResult {
  success: boolean;      // Did the fix work?
  message: string;       // Human-readable description
  retry: boolean;        // Should we retry the test?
}
```

### Step 2: Implement a Simple Strategy

Example: Retry transient failures

```typescript
import { FixStrategy, Issue, FixContext, FixResult } from './fix-strategies';

export class RetryTestStrategy implements FixStrategy {
  private maxRetries = 3;
  private retryCount = new Map<string, number>();

  canFix(issue: Issue): boolean {
    // Only fix transient network errors
    return issue.type === 'network' && issue.description.includes('timeout');
  }

  async apply(issue: Issue, context: FixContext): Promise<FixResult> {
    const testId = context.testCase.id;
    const retries = this.retryCount.get(testId) || 0;

    if (retries >= this.maxRetries) {
      return {
        success: false,
        message: `Max retries (${this.maxRetries}) exceeded`,
        retry: false,
      };
    }

    // Wait with exponential backoff
    const waitMs = Math.pow(2, retries) * 1000;
    await new Promise(resolve => setTimeout(resolve, waitMs));

    this.retryCount.set(testId, retries + 1);

    return {
      success: true,
      message: `Retrying after ${waitMs}ms (attempt ${retries + 1}/${this.maxRetries})`,
      retry: true,
    };
  }
}
```

### Step 3: Implement a Complex Strategy

Example: Restart daemon on connection failure

```typescript
export class RestartDaemonStrategy implements FixStrategy {
  canFix(issue: Issue): boolean {
    return issue.type === 'network' &&
           (issue.description.includes('ECONNREFUSED') ||
            issue.description.includes('Connection refused'));
  }

  async apply(issue: Issue, context: FixContext): Promise<FixResult> {
    try {
      console.log('  Stopping daemon...');
      await context.daemon.stop();

      console.log('  Waiting 2 seconds...');
      await new Promise(resolve => setTimeout(resolve, 2000));

      console.log('  Starting daemon...');
      await context.daemon.start();
      await context.daemon.waitUntilReady(30000);

      return {
        success: true,
        message: 'Daemon restarted successfully',
        retry: true,
      };
    } catch (error) {
      return {
        success: false,
        message: `Failed to restart daemon: ${error.message}`,
        retry: false,
      };
    }
  }
}
```

### Step 4: Register Your Strategy

Add to `scripts/auto-fix/fix-strategies.ts`:

```typescript
export function createFixRegistry(): FixStrategyRegistry {
  const registry = new FixStrategyRegistry();

  // Existing strategies
  registry.register(new RestartDaemonStrategy());
  registry.register(new UpdateExpectedResultStrategy());
  registry.register(new ReseedFixtureStrategy());

  // Your new strategy
  registry.register(new RetryTestStrategy());

  return registry;
}
```

### Strategy Best Practices

**DO**:
```typescript
// ✅ Idempotent - safe to run multiple times
async apply(issue, context) {
  // Check current state first
  if (await context.daemon.isHealthy()) {
    return { success: true, message: 'Already healthy', retry: true };
  }
  // Apply fix
}

// ✅ Detailed logging
console.log('  Applying fix: Restart daemon');
console.log('  Reason: Connection refused');

// ✅ Graceful error handling
try {
  await risky operation();
} catch (error) {
  return { success: false, message: error.message, retry: false };
}
```

**DON'T**:
```typescript
// ❌ Modify code (only config/fixtures allowed)
fs.writeFileSync('src/api.ts', newCode);

// ❌ Infinite loops (track retry counts)
while (true) { retry(); }

// ❌ Silent failures
try {
  await fix();
} catch {
  // Ignoring error - this hides issues!
}
```

## Running Tests Locally

### Basic Testing

```bash
# Run all tests
npm run test:e2e:api

# Run with auto-fix
npm run test:e2e:api:fix

# Generate reports
npm run test:e2e:api:report
npm run test:e2e:api:html
```

### Debugging Tests

```bash
# Run specific test by grepping logs
npm run test:e2e:api 2>&1 | grep "profiles-create"

# Enable verbose output (modify executor config)
const executor = createExecutor({ verbose: true });

# Check daemon logs
cat scripts/logs/daemon-*.log

# Use Node debugger
node --inspect-brk $(which tsx) scripts/automated-e2e-test.ts
```

### Testing Your Changes

```bash
# 1. Make changes to test cases or fix strategies

# 2. Run tests without auto-fix to see failures
npm run test:e2e:api

# 3. Enable auto-fix to test your strategy
npm run test:e2e:api:fix

# 4. Check HTML report for details
npm run test:e2e:api:html
open keyrx_ui/test-report.html

# 5. Verify metrics recorded
npm run test:e2e:api:metrics
```

### Integration Testing

```bash
# Test CI workflow locally (requires act)
gh act pull_request -W .github/workflows/e2e-auto.yml

# Or push to branch and create PR
git push origin feature/my-test-changes
gh pr create
```

## Best Practices

### Test Design

1. **One Concept Per Test**: Each test should verify one specific behavior
   ```typescript
   // ✅ Good: One concept
   testProfileCreation()
   testProfileCreationWithInvalidName()

   // ❌ Bad: Multiple concepts
   testAllProfileOperations()
   ```

2. **Deterministic**: Tests should always produce the same result
   ```typescript
   // ✅ Deterministic
   const timestamp = 1234567890;
   const name = 'test-profile';

   // ❌ Non-deterministic
   const timestamp = Date.now(); // Changes every run
   const name = Math.random().toString(); // Random
   ```

3. **Fast**: Aim for < 1 second per test
   ```typescript
   // ✅ Fast: Direct API call
   await client.getStatus();

   // ❌ Slow: Unnecessary wait
   await new Promise(r => setTimeout(r, 5000));
   await client.getStatus();
   ```

4. **Isolated**: Tests should not depend on each other
   ```typescript
   // ✅ Isolated: Sets up its own data
   async setup() {
     await client.createProfile('test');
     return { profile: 'test' };
   }

   // ❌ Dependent: Assumes data from previous test
   async setup() {
     return { profile: 'test' }; // Assumes 'test' exists
   }
   ```

### Error Handling

1. **Expect Errors**: Some tests intentionally trigger errors
   ```typescript
   try {
     await client.deleteProfile('nonexistent');
     return { status: 200, error: null };
   } catch (error) {
     return { status: error.response?.status || 500, error };
   }
   ```

2. **Meaningful Messages**: Help developers understand failures
   ```typescript
   // ✅ Helpful
   return {
     passed: false,
     message: `Expected profile 'default' to be active, but found inactive`,
   };

   // ❌ Vague
   return {
     passed: false,
     message: 'Test failed',
   };
   ```

3. **Best-Effort Cleanup**: Don't fail tests due to cleanup errors
   ```typescript
   async cleanup(client, context) {
     try {
       await client.deleteProfile(context.profile);
     } catch (error) {
       console.warn(`Cleanup failed: ${error.message}`);
       // Don't throw - cleanup is best effort
     }
   }
   ```

### Performance

1. **Minimize API Calls**: Batch when possible
   ```typescript
   // ✅ Efficient: One call
   const profiles = await client.getProfiles();
   const hasDefault = profiles.some(p => p.name === 'default');

   // ❌ Inefficient: Multiple calls
   for (const name of ['default', 'test', 'custom']) {
     await client.getProfile(name);
   }
   ```

2. **Parallel Setup**: Use `Promise.all` when order doesn't matter
   ```typescript
   // ✅ Parallel
   await Promise.all([
     client.createProfile('test1'),
     client.createProfile('test2'),
     client.createProfile('test3'),
   ]);

   // ❌ Sequential (slower)
   await client.createProfile('test1');
   await client.createProfile('test2');
   await client.createProfile('test3');
   ```

3. **Caching**: Reuse expensive computations
   ```typescript
   // Cache API schema validation
   let cachedSchema: Schema | null = null;

   function getSchema() {
     if (!cachedSchema) {
       cachedSchema = loadSchemaFromFile();
     }
     return cachedSchema;
   }
   ```

### Code Quality

1. **Type Safety**: Use TypeScript types everywhere
   ```typescript
   // ✅ Typed
   async execute(client: ApiClient, context: { profileName: string }) {
     return await client.getProfile(context.profileName);
   }

   // ❌ Untyped
   async execute(client: any, context: any) {
     return await client.getProfile(context.profileName);
   }
   ```

2. **DRY**: Extract common test utilities
   ```typescript
   // ✅ Reusable helper
   async function ensureProfileExists(client: ApiClient, name: string) {
     try {
       await client.getProfile(name);
     } catch {
       await client.createProfile(name);
     }
   }

   // Use in multiple tests
   await ensureProfileExists(client, 'test-profile');
   ```

3. **Documentation**: Comment complex logic
   ```typescript
   // Retry with exponential backoff to handle transient network issues
   // Max 3 attempts: 1s, 2s, 4s delays
   const waitMs = Math.pow(2, retries) * 1000;
   await new Promise(resolve => setTimeout(resolve, waitMs));
   ```

## Testing Checklist

Before submitting your changes:

- [ ] All tests pass locally
- [ ] New tests have setup/cleanup
- [ ] Tests are deterministic (no random data, no timing dependencies)
- [ ] Tests run in < 1 second each
- [ ] Fix strategies are idempotent
- [ ] Fix strategies log their actions
- [ ] Error messages are helpful
- [ ] Code follows TypeScript strict mode
- [ ] No hardcoded credentials or secrets
- [ ] Documentation updated (README, DEV_GUIDE)

## Additional Resources

- **README**: [README.md](./README.md) - System overview and usage
- **Example Test**: [examples/example-test.ts](./examples/example-test.ts) - Complete template
- **API Client**: `scripts/api-client/client.ts` - Available API methods
- **Test Utilities**: `scripts/test-cases/test-utils.ts` - Helper functions
- **Issue Tracker**: Report bugs and request features

## Questions?

If you're stuck or have questions:

1. Check the [README](./README.md) and this guide
2. Look at [example-test.ts](./examples/example-test.ts)
3. Review existing tests in `scripts/test-cases/api-tests.ts`
4. Ask in the project's discussion forum or issue tracker
