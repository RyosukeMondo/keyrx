import { beforeEach, describe, expect, it, vi } from 'vitest';
import { isChunkLoadError, reloadOnceForNewBuild } from './lazyWithReload';

describe('stale-tab recovery', () => {
  beforeEach(() => window.sessionStorage.clear());

  it('recognises the browsers chunk-load failures, and only those', () => {
    for (const message of [
      'Failed to fetch dynamically imported module: http://x/assets/DevicesPage-abc.js',
      'error loading dynamically imported module',
      'Importing a module script failed.',
      'Loading chunk 7 failed.',
    ]) {
      expect(isChunkLoadError(new TypeError(message)), message).toBe(true);
    }
    expect(isChunkLoadError(new Error('Cannot read properties of undefined'))).toBe(false);
    expect(isChunkLoadError(undefined)).toBe(false);
  });

  it('reloads once, then refuses to loop within the cooldown', () => {
    const reload = vi.fn();
    expect(reloadOnceForNewBuild(1_000, reload)).toBe(true);
    expect(reloadOnceForNewBuild(5_000, reload)).toBe(false);
    expect(reload).toHaveBeenCalledTimes(1);
    // A much later failure is a new stale tab, not a loop.
    expect(reloadOnceForNewBuild(1_000 + 60_000, reload)).toBe(true);
    expect(reload).toHaveBeenCalledTimes(2);
  });

  it('does not reload when it cannot remember that it did', () => {
    const reload = vi.fn();
    const spy = vi
      .spyOn(Storage.prototype, 'getItem')
      .mockImplementation(() => {
        throw new Error('blocked');
      });
    expect(reloadOnceForNewBuild(1, reload)).toBe(false);
    expect(reload).not.toHaveBeenCalled();
    spy.mockRestore();
  });
});
