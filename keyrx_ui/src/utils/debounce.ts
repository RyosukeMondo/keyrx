/**
 * Debouncing utility for performance optimization
 * Prevents excessive function calls (e.g., search inputs, auto-save)
 */

/**
 * Debounce function - delays execution until after wait time has elapsed
 * @param fn Function to debounce
 * @param wait Delay in milliseconds
 * @returns Debounced function
 */
export function debounce<T extends (...args: unknown[]) => unknown>(
  fn: T,
  wait: number
): (...args: Parameters<T>) => void {
  let timeoutId: ReturnType<typeof setTimeout> | null = null;

  return function debounced(...args: Parameters<T>) {
    if (timeoutId !== null) {
      clearTimeout(timeoutId);
    }

    timeoutId = setTimeout(() => {
      fn(...args);
      timeoutId = null;
    }, wait);
  };
}

/**
 * Throttle function - ensures function is called at most once per wait period
 * @param fn Function to throttle
 * @param wait Minimum time between calls in milliseconds
 * @returns Throttled function
 */
export function throttle<T extends (...args: unknown[]) => unknown>(
  fn: T,
  wait: number
): (...args: Parameters<T>) => void {
  let lastCall = 0;
  let timeoutId: ReturnType<typeof setTimeout> | null = null;

  return function throttled(...args: Parameters<T>) {
    const now = Date.now();
    const timeSinceLastCall = now - lastCall;

    if (timeSinceLastCall >= wait) {
      lastCall = now;
      fn(...args);
    } else {
      // Schedule the call for the remaining time
      if (timeoutId !== null) {
        clearTimeout(timeoutId);
      }
      timeoutId = setTimeout(() => {
        lastCall = Date.now();
        fn(...args);
        timeoutId = null;
      }, wait - timeSinceLastCall);
    }
  };
}

/**
 * Create a debounced version of an async function
 * Automatically cancels pending promises when called again
 */
export function debounceAsync<T extends (...args: unknown[]) => Promise<unknown>>(
  fn: T,
  wait: number
): (...args: Parameters<T>) => Promise<ReturnType<T>> {
  let timeoutId: ReturnType<typeof setTimeout> | null = null;
  let abortController: AbortController | null = null;

  return function debouncedAsync(...args: Parameters<T>): Promise<ReturnType<T>> {
    // Cancel previous timeout and abort previous request
    if (timeoutId !== null) {
      clearTimeout(timeoutId);
    }
    if (abortController !== null) {
      abortController.abort();
    }

    return new Promise((resolve, reject) => {
      timeoutId = setTimeout(async () => {
        try {
          abortController = new AbortController();
          const result = await fn(...args);
          resolve(result as ReturnType<T>);
        } catch (error) {
          if (error instanceof Error && error.name === 'AbortError') {
            // Ignore aborted requests
            return;
          }
          reject(error);
        } finally {
          timeoutId = null;
          abortController = null;
        }
      }, wait);
    });
  };
}
