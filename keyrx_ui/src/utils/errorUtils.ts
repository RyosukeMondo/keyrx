/**
 * Error handling utilities
 *
 * Provides consistent error message extraction across the application.
 */

/**
 * Extract a human-readable error message from any error type
 *
 * Handles:
 * - Error instances
 * - Error-like objects with message property
 * - Objects with nested error structures
 * - Primitive values
 *
 * @param error - Any error value
 * @param fallback - Fallback message if extraction fails
 * @returns Human-readable error message
 *
 * @example
 * ```ts
 * try {
 *   await apiCall();
 * } catch (err) {
 *   const message = getErrorMessage(err, 'Operation failed');
 *   setError(message);
 * }
 * ```
 */
export function getErrorMessage(
  error: unknown,
  fallback = 'An error occurred'
): string {
  // Handle null/undefined, but allow false/0/"" as valid values
  if (error === null || error === undefined) {
    return fallback;
  }

  // Standard Error instance
  if (error instanceof Error) {
    return error.message;
  }

  // Error-like object with message
  if (typeof error === 'object' && error !== null && 'message' in error) {
    const message = (error as { message: unknown }).message;

    // If message is a string, return it
    if (typeof message === 'string') {
      return message;
    }

    // If message is an object, try to stringify it
    if (typeof message === 'object' && message !== null) {
      return JSON.stringify(message);
    }
  }

  // String error
  if (typeof error === 'string') {
    return error;
  }

  // Try to stringify objects
  if (typeof error === 'object') {
    try {
      return JSON.stringify(error);
    } catch {
      return fallback;
    }
  }

  // Fallback for primitives
  return String(error);
}

/**
 * Format error for user display
 *
 * Ensures error messages are user-friendly and safe to display
 *
 * @param error - Any error value
 * @param context - Optional context prefix (e.g., "Failed to save profile")
 * @returns Formatted error message
 */
export function formatErrorForDisplay(
  error: unknown,
  context?: string
): string {
  const message = getErrorMessage(error);

  if (context) {
    // Avoid duplicate context if message already starts with it
    if (message.toLowerCase().startsWith(context.toLowerCase())) {
      return message;
    }
    return `${context}: ${message}`;
  }

  return message;
}

// ---------------------------------------------------------------------------
// User-facing daemon / compiler errors
// ---------------------------------------------------------------------------

/** Levenshtein edit distance (iterative, O(n*m)), case-insensitive. */
export function editDistance(a: string, b: string): number {
  const x = a.toLowerCase();
  const y = b.toLowerCase();
  let prev = Array.from({ length: y.length + 1 }, (_, j) => j);
  for (let i = 1; i <= x.length; i++) {
    const row = [i];
    for (let j = 1; j <= y.length; j++) {
      row[j] = Math.min(
        prev[j] + 1,
        row[j - 1] + 1,
        prev[j - 1] + (x[i - 1] === y[j - 1] ? 0 : 1)
      );
    }
    prev = row;
  }
  return prev[y.length];
}

/** Largest edit distance at which a candidate still looks like a typo fix. */
const maxSuggestionDistance = (word: string) =>
  Math.min(2, Math.max(1, Math.floor(word.length / 3)));

/**
 * Daemon/compiler errors append "Did you mean one of these? - F - O - F1" for
 * unknown key names. Those lists are often unrelated to the typo ("Foo" ->
 * F, O, F1), so keep only candidates that are a small edit away and drop the
 * whole hint when none qualifies.
 */
function refineSuggestions(message: string): string {
  const hint = message.match(
    /\s*Did you mean one of these\?\s*((?:\s*-\s*[^\n-][^\n]*)+)/
  );
  if (!hint) return message;
  const word = message.match(/Unknown key name: '([^']+)'/)?.[1];
  const candidates = [...hint[1].matchAll(/-\s*([^\n]+)/g)].map((m) =>
    m[1].trim()
  );
  const near = word
    ? candidates.filter(
        (c) => editDistance(word, c) <= maxSuggestionDistance(word)
      )
    : [];
  const replacement = near.length ? ` Did you mean ${near.join(' or ')}?` : '';
  return message.replace(hint[0], replacement);
}

const PATH_WITH_POSITION =
  /(?:[A-Za-z]:)?[\\/](?:[^\s:'"\\/]+[\\/])*[^\s:'"\\/]+\.(?:rhai|krx)(?:\.tmp)?:(\d+)(?::\d+)?:?/g;
const BARE_PATH =
  /(?<![\w'"])(?:[A-Za-z]:\\|\/)(?:[^\s:'"\\/]+[\\/])+[^\s:'"\\/]+/g;

/**
 * Turn a raw daemon/compiler error into something safe and readable for a
 * toast or a diagnostic: no absolute or temp-file paths, no wrapper noise,
 * no duplicated "Syntax error" prefixes, and only plausible typo suggestions.
 */
export function friendlyErrorMessage(raw: string): string {
  let m = raw
    .replace(/^Failed to [^:]+:\s*/i, '')
    .replace(/^Compilation error:\s*/i, '')
    .replace(/^Compilation failed:\s*/i, '')
    .replace(PATH_WITH_POSITION, 'Line $1:')
    .replace(BARE_PATH, 'a file')
    .replace(/(?:(?:Syntax|Parse) error:\s*)+(?:Runtime error:\s*)?/gi, 'Syntax error: ')
    .replace(/\s*\(line \d+, position \d+\)/g, '')
    .replace(/\s*Help: Check your Rhai script syntax[^\n]*/g, '');
  m = refineSuggestions(m);
  m = m
    .replace(/\s*\n\s*/g, ' ')
    .replace(/\s{2,}/g, ' ')
    .trim();
  return m.length > 240 ? `${m.slice(0, 237)}…` : m;
}

/** `getErrorMessage` + `friendlyErrorMessage`: the text to show to a person. */
export function userFacingError(error: unknown, fallback?: string): string {
  return friendlyErrorMessage(getErrorMessage(error, fallback));
}
