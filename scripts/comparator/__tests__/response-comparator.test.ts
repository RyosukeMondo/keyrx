/**
 * Tests for ResponseComparator
 */

import { ResponseComparator } from '../response-comparator.js';

describe('ResponseComparator', () => {
  let comparator: ResponseComparator;

  beforeEach(() => {
    comparator = new ResponseComparator();
  });

  describe('Primitive comparisons', () => {
    test('should match identical numbers', () => {
      const result = comparator.compare(42, 42);
      expect(result.matches).toBe(true);
      expect(result.diffs).toHaveLength(0);
    });

    test('should detect number mismatch', () => {
      const result = comparator.compare(42, 43);
      expect(result.matches).toBe(false);
      expect(result.diffs).toHaveLength(1);
      expect(result.diffs[0].type).toBe('value-mismatch');
    });

    test('should match identical strings', () => {
      const result = comparator.compare('test', 'test');
      expect(result.matches).toBe(true);
    });

    test('should detect string mismatch', () => {
      const result = comparator.compare('test', 'TEST');
      expect(result.matches).toBe(false);
    });

    test('should match identical booleans', () => {
      const result = comparator.compare(true, true);
      expect(result.matches).toBe(true);
    });
  });

  describe('Object comparisons', () => {
    test('should match identical objects', () => {
      const obj = { name: 'test', value: 42 };
      const result = comparator.compare(obj, obj);
      expect(result.matches).toBe(true);
    });

    test('should detect missing field', () => {
      const actual = { name: 'test' };
      const expected = { name: 'test', value: 42 };
      const result = comparator.compare(actual, expected);
      expect(result.matches).toBe(false);
      expect(result.diffs).toHaveLength(1);
      expect(result.diffs[0].type).toBe('missing');
      expect(result.diffs[0].path).toBe('value');
    });

    test('should detect extra field', () => {
      const actual = { name: 'test', value: 42 };
      const expected = { name: 'test' };
      const result = comparator.compare(actual, expected);
      expect(result.matches).toBe(false);
      expect(result.diffs).toHaveLength(1);
      expect(result.diffs[0].type).toBe('extra');
      expect(result.diffs[0].path).toBe('value');
    });

    test('should detect value mismatch in nested object', () => {
      const actual = { user: { name: 'Alice', age: 30 } };
      const expected = { user: { name: 'Alice', age: 31 } };
      const result = comparator.compare(actual, expected);
      expect(result.matches).toBe(false);
      expect(result.diffs).toHaveLength(1);
      expect(result.diffs[0].path).toBe('user.age');
    });
  });

  describe('Array comparisons', () => {
    test('should match identical arrays', () => {
      const arr = [1, 2, 3];
      const result = comparator.compare(arr, arr);
      expect(result.matches).toBe(true);
    });

    test('should detect array length mismatch', () => {
      const actual = [1, 2];
      const expected = [1, 2, 3];
      const result = comparator.compare(actual, expected);
      expect(result.matches).toBe(false);
    });

    test('should detect element mismatch', () => {
      const actual = [1, 2, 3];
      const expected = [1, 5, 3];
      const result = comparator.compare(actual, expected);
      expect(result.matches).toBe(false);
      expect(result.diffs).toHaveLength(1);
      expect(result.diffs[0].path).toBe('[1]');
    });
  });

  describe('Ignore fields option', () => {
    test('should ignore specified fields', () => {
      const actual = { name: 'test', timestamp: 12345 };
      const expected = { name: 'test', timestamp: 67890 };
      const result = comparator.compare(actual, expected, {
        ignoreFields: ['timestamp'],
      });
      expect(result.matches).toBe(true);
      expect(result.ignoredFields).toContain('timestamp');
    });

    test('should ignore nested fields with suffix match', () => {
      const actual = { user: { id: 1, name: 'Alice' }, post: { id: 2 } };
      const expected = { user: { id: 99, name: 'Alice' }, post: { id: 88 } };
      const result = comparator.compare(actual, expected, {
        ignoreFields: ['*.id'],
      });
      expect(result.matches).toBe(true);
    });
  });

  describe('Ignore whitespace option', () => {
    test('should ignore whitespace when enabled', () => {
      const actual = '  test  ';
      const expected = 'test';
      const result = comparator.compare(actual, expected, {
        ignoreWhitespace: true,
      });
      expect(result.matches).toBe(true);
    });

    test('should not ignore whitespace by default', () => {
      const actual = '  test  ';
      const expected = 'test';
      const result = comparator.compare(actual, expected);
      expect(result.matches).toBe(false);
    });
  });

  describe('Ignore array order option', () => {
    test('should match arrays with different order when enabled', () => {
      const actual = [3, 1, 2];
      const expected = [1, 2, 3];
      const result = comparator.compare(actual, expected, {
        ignoreArrayOrder: true,
      });
      expect(result.matches).toBe(true);
    });

    test('should respect array order by default', () => {
      const actual = [3, 1, 2];
      const expected = [1, 2, 3];
      const result = comparator.compare(actual, expected);
      expect(result.matches).toBe(false);
    });
  });

  describe('Type mismatch detection', () => {
    test('should detect string vs number mismatch', () => {
      const result = comparator.compare('42', 42);
      expect(result.matches).toBe(false);
      expect(result.diffs[0].type).toBe('type-mismatch');
    });

    test('should detect object vs array mismatch', () => {
      const result = comparator.compare({}, []);
      expect(result.matches).toBe(false);
      expect(result.diffs[0].type).toBe('type-mismatch');
    });
  });

  describe('Null and undefined handling', () => {
    test('should match null values', () => {
      const result = comparator.compare(null, null);
      expect(result.matches).toBe(true);
    });

    test('should match undefined values', () => {
      const result = comparator.compare(undefined, undefined);
      expect(result.matches).toBe(true);
    });

    test('should detect null vs undefined mismatch', () => {
      const result = comparator.compare(null, undefined);
      expect(result.matches).toBe(false);
    });
  });

  describe('Circular reference handling', () => {
    test('should detect circular references', () => {
      const actual: any = { name: 'test' };
      actual.self = actual;

      const expected = { name: 'test' };

      const result = comparator.compare(actual, expected);
      expect(result.matches).toBe(false);
      // Should detect circular reference without infinite loop
    });
  });

  describe('Text diff generation', () => {
    test('should generate text diff', () => {
      const expected = { name: 'Alice', age: 30 };
      const actual = { name: 'Bob', age: 30 };

      const diff = ResponseComparator.generateTextDiff(expected, actual);
      expect(diff).toContain('--- Expected');
      expect(diff).toContain('+++ Actual');
      expect(diff).toContain('Alice');
      expect(diff).toContain('Bob');
    });
  });
});
