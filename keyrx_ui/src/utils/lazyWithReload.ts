/**
 * Recover from a stale tab.
 *
 * Page chunks are content-hashed (`DevicesPage-<hash>.js`). A tab opened
 * before the daemon was upgraded still points at the old names, so switching
 * tab fails with "Failed to fetch dynamically imported module". The fix is to
 * reload once (the new index.html names the new chunks). A sessionStorage
 * stamp stops a genuinely broken deployment from reload-looping.
 */
import { lazy, type ComponentType, type LazyExoticComponent } from 'react';

const RELOAD_STAMP = 'keyrx.chunkReloadAt';
/** A second chunk failure within this window is a real error, not a stale tab. */
const RELOAD_COOLDOWN_MS = 30_000;

/** Whether `error` is the browser failing to load a code-split chunk. */
export function isChunkLoadError(error: unknown): boolean {
  const message = error instanceof Error ? error.message : String(error ?? '');
  return /dynamically imported module|importing a module script failed|ChunkLoadError|Loading chunk .* failed/i.test(
    message
  );
}

/**
 * Reloads the page unless it already did so moments ago.
 * Returns whether a reload was started.
 */
export function reloadOnceForNewBuild(
  now: number = Date.now(),
  reload: () => void = () => window.location.reload()
): boolean {
  try {
    const last = window.sessionStorage.getItem(RELOAD_STAMP);
    if (last !== null && now - Number(last) < RELOAD_COOLDOWN_MS) return false;
    window.sessionStorage.setItem(RELOAD_STAMP, String(now));
  } catch {
    return false; // no storage: cannot guard against a loop, so do not reload
  }
  reload();
  return true;
}

/** `React.lazy` that reloads once when its chunk is gone (stale tab). */
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function lazyWithReload<T extends ComponentType<any>>(
  factory: () => Promise<{ default: T }>
): LazyExoticComponent<T> {
  return lazy(async () => {
    try {
      return await factory();
    } catch (error) {
      if (isChunkLoadError(error) && reloadOnceForNewBuild()) {
        // The page is reloading: never settle, so no error flashes meanwhile.
        return new Promise<{ default: T }>(() => undefined);
      }
      throw error;
    }
  });
}

/** Also covers Vite's own preload helper (`vite:preloadError`). */
export function installStaleBuildRecovery(): void {
  window.addEventListener('vite:preloadError', (event) => {
    if (reloadOnceForNewBuild()) event.preventDefault();
  });
}
