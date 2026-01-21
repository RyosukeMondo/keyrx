# Response Comparator

Deep equality comparison library for API test validation with detailed diff output.

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

## Implementation Notes

- Uses `WeakSet` for circular reference detection
- Depth limit prevents stack overflow on deeply nested structures
- Field ignoring supports patterns: `'field'`, `'*.field'`, `'parent.child'`
- Array comparison can be order-dependent or order-independent
- Handles special types: `Date`, `null`, `undefined`
- No external dependencies for maximum portability
