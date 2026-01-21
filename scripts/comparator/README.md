# Response Comparator & Validation Reporter

Deep equality comparison library and test result reporter for API test validation.

## Components

1. **ResponseComparator** - Deep equality checking with detailed diffs
2. **ValidationReporter** - Human-readable and JSON report generation

## Features

- Deep equality checking for nested objects and arrays
- Configurable field ignoring (timestamps, IDs, etc.)
- Semantic array comparison (order-independent)
- Whitespace normalization for strings
- Circular reference detection
- Detailed diff paths for debugging
- No external dependencies

## Usage

```typescript
import { ResponseComparator } from './response-comparator.js';

const comparator = new ResponseComparator();

// Basic comparison
const result = comparator.compare(
  { name: 'Alice', age: 30 },
  { name: 'Alice', age: 31 }
);

console.log(result.matches); // false
console.log(result.diffs);
// [
//   {
//     path: 'age',
//     type: 'value-mismatch',
//     expected: 31,
//     actual: 30,
//     description: 'Expected 31 but got 30'
//   }
// ]
```

## Options

### ignoreFields

Ignore specific fields during comparison. Supports exact match, suffix match, and wildcard patterns.

```typescript
const result = comparator.compare(
  { name: 'test', timestamp: 12345, user: { id: 1 } },
  { name: 'test', timestamp: 67890, user: { id: 2 } },
  {
    ignoreFields: ['timestamp', '*.id'] // Ignore timestamp and all id fields
  }
);
// result.matches = true
```

### ignoreArrayOrder

Enable semantic array comparison (order-independent).

```typescript
const result = comparator.compare(
  [3, 1, 2],
  [1, 2, 3],
  { ignoreArrayOrder: true }
);
// result.matches = true
```

### ignoreWhitespace

Normalize whitespace in string comparisons.

```typescript
const result = comparator.compare(
  '  test  ',
  'test',
  { ignoreWhitespace: true }
);
// result.matches = true
```

### maxDepth

Set maximum depth for nested comparisons (default: 50).

```typescript
const result = comparator.compare(
  deeplyNestedObject,
  anotherDeepObject,
  { maxDepth: 10 }
);
```

## Diff Types

- `missing` - Field exists in expected but not in actual
- `extra` - Field exists in actual but not in expected
- `type-mismatch` - Field types don't match (e.g., string vs number)
- `value-mismatch` - Values don't match

## Text Diff Generation

Generate human-readable text diffs:

```typescript
const diff = ResponseComparator.generateTextDiff(
  { name: 'Alice', age: 30 },
  { name: 'Bob', age: 30 }
);

console.log(diff);
// --- Expected
// +++ Actual
//
//   {
// -   "name": "Alice",
// +   "name": "Bob",
//     "age": 30
//   }
```

## API Reference

### `ResponseComparator.compare(actual, expected, options?): ComparisonResult`

Compare two values and return detailed diff.

**Parameters:**
- `actual: unknown` - Actual value from API response
- `expected: unknown` - Expected value from test database
- `options?: ComparisonOptions` - Comparison options

**Returns:** `ComparisonResult`
- `matches: boolean` - Whether values match
- `diffs: Diff[]` - List of differences
- `ignoredFields: string[]` - Fields ignored during comparison

### `ResponseComparator.generateTextDiff(expected, actual): string`

Generate text diff for display purposes.

**Parameters:**
- `expected: unknown` - Expected value
- `actual: unknown` - Actual value

**Returns:** `string` - Unified diff format

## Examples

### API Response Validation

```typescript
import { ResponseComparator } from './response-comparator.js';
import { expectedResults } from '../fixtures/expected-results.js';

const comparator = new ResponseComparator();

// Compare API response with expected result
const result = comparator.compare(
  apiResponse.data,
  expectedResults['/api/status'].success,
  {
    ignoreFields: ['timestamp', 'uptime_ms'],
    ignoreArrayOrder: false,
  }
);

if (!result.matches) {
  console.error('Response validation failed:');
  result.diffs.forEach(diff => {
    console.error(`  ${diff.path}: ${diff.description}`);
  });
}
```

### Nested Object Comparison

```typescript
const actual = {
  user: {
    id: 123,
    name: 'Alice',
    profile: {
      age: 30,
      city: 'NYC'
    }
  }
};

const expected = {
  user: {
    id: 456, // Different ID (will be ignored)
    name: 'Alice',
    profile: {
      age: 30,
      city: 'SF' // Different city
    }
  }
};

const result = comparator.compare(actual, expected, {
  ignoreFields: ['*.id']
});

// result.matches = false
// result.diffs = [
//   {
//     path: 'user.profile.city',
//     type: 'value-mismatch',
//     expected: 'SF',
//     actual: 'NYC',
//     description: 'Expected "SF" but got "NYC"'
//   }
// ]
```

## ValidationReporter Usage

Generate formatted test reports for console output and JSON export.

### Human-Readable Reports

```typescript
import { ValidationReporter } from './validation-reporter.js';

const reporter = new ValidationReporter();

const suiteResult = {
  name: 'API Tests',
  total: 10,
  passed: 8,
  failed: 2,
  skipped: 0,
  errors: 0,
  duration: 5000,
  results: [
    {
      id: 'test-1',
      name: 'GET /api/status',
      status: 'pass',
      duration: 500,
    },
    {
      id: 'test-2',
      name: 'GET /api/devices',
      status: 'fail',
      duration: 600,
      comparison: {
        matches: false,
        diffs: [
          {
            path: 'data.count',
            type: 'value-mismatch',
            expected: 5,
            actual: 3,
            description: 'Expected 5 but got 3'
          }
        ],
        ignoredFields: []
      },
      expected: { data: { count: 5 } },
      actual: { data: { count: 3 } }
    }
  ]
};

// Print to console
console.log(reporter.formatHuman(suiteResult));

// Output:
// ═══════════════════════════════════════════════════════════════════════════
//   Test Suite Results: API Tests
// ═══════════════════════════════════════════════════════════════════════════
//
// Summary:
//   Total:    10 tests
//   ✓ Passed:  8
//   ✗ Failed:  2
//   Duration: 5.00s
//   Pass Rate: 80.0%
//
// Test Results:
//
// ✓ PASS GET /api/status (500ms)
//
// ✗ FAIL GET /api/devices (600ms)
//   Differences:
//     Path: data.count
//     Type: value-mismatch
//     - Expected: 5
//     + Actual:   3
// ...
```

### JSON Reports

```typescript
// Generate JSON report
const jsonReport = reporter.formatJson(suiteResult);

console.log(JSON.stringify(jsonReport, null, 2));

// Output:
// {
//   "version": "1.0",
//   "timestamp": "2024-01-01T00:00:00.000Z",
//   "suite": "API Tests",
//   "summary": {
//     "total": 10,
//     "passed": 8,
//     "failed": 2,
//     "skipped": 0,
//     "errors": 0,
//     "duration": 5000,
//     "passRate": 80.0
//   },
//   "results": [
//     {
//       "id": "test-1",
//       "name": "GET /api/status",
//       "status": "pass",
//       "duration": 500
//     },
//     {
//       "id": "test-2",
//       "name": "GET /api/devices",
//       "status": "fail",
//       "duration": 600,
//       "diffs": [
//         {
//           "path": "data.count",
//           "type": "value-mismatch",
//           "expected": 5,
//           "actual": 3,
//           "description": "Expected 5 but got 3"
//         }
//       ]
//     }
//   ]
// }

// Write to file
await reporter.writeJsonReport(suiteResult, './test-results.json');
```

### Color Output

The reporter automatically detects whether to use colors:

- Disabled if `NO_COLOR` environment variable is set
- Disabled in CI environments (unless GitHub Actions or GitLab CI)
- Enabled if stdout is a TTY

```bash
# Disable colors
NO_COLOR=1 node test-runner.js

# Force colors in CI (if supported)
GITHUB_ACTIONS=true node test-runner.js
```

### Test Status Types

- `pass` - Test passed ✓
- `fail` - Test failed (comparison mismatch) ✗
- `skip` - Test skipped ○
- `error` - Test error (exception/timeout) ⚠

## Implementation Notes

### ResponseComparator

- Uses `WeakSet` for circular reference detection
- Depth limit prevents stack overflow on deeply nested structures
- Field ignoring supports patterns: `'field'`, `'*.field'`, `'parent.child'`
- Array comparison can be order-dependent or order-independent
- Handles special types: `Date`, `null`, `undefined`
- No external dependencies for maximum portability

### ValidationReporter

- ANSI color codes for terminal output
- Respects `NO_COLOR` and `CI` environment variables
- Diff output limited to prevent excessive console spam
- JSON reports are machine-parseable for CI integration
- Duration formatting: ms, seconds, minutes
- Pass rate calculation with color-coded display
