import { useCallback, useEffect, useRef, useState } from 'react';

const MAX_HISTORY = 100;

interface HistorySource {
  state: string;
  getCode: () => string;
  /** Replace the whole document (also re-projects it onto the visual editor). */
  loadServerConfig: (code: string) => void;
}

export interface CodeHistory {
  undo: () => void;
  redo: () => void;
  canUndo: boolean;
  canRedo: boolean;
  /** Forget everything (profile switch, fresh load). */
  reset: (code: string) => void;
}

interface Stacks {
  past: string[];
  future: string[];
  current: string | null;
}

const EMPTY: Stacks = { past: [], future: [], current: null };

/** True when the key event belongs to a text control with its own undo. */
export function hasNativeUndo(target: EventTarget | null): boolean {
  const el = target as HTMLElement | null;
  if (!el?.tagName) return false;
  return (
    el.tagName === 'INPUT' ||
    el.tagName === 'TEXTAREA' ||
    el.isContentEditable ||
    !!el.closest?.('.monaco-editor')
  );
}

/**
 * Undo/redo over the profile document the editors share. Visual edits and
 * code edits both end up as a new document, so tracking the document covers
 * both. Ctrl+Z / Ctrl+Shift+Z (and Ctrl+Y) are handled globally except inside
 * text controls and Monaco, which keep their own undo.
 */
export function useCodeHistory(
  source: HistorySource,
  enabled: boolean
): CodeHistory {
  const [stacks, setStacks] = useState<Stacks>(EMPTY);
  // Latest stacks for handlers/effects; only ever read outside render.
  const latest = useRef<Stacks>(EMPTY);
  const commit = useCallback((next: Stacks) => {
    latest.current = next;
    setStacks(next);
  }, []);

  const code = source.getCode();
  const settled = source.state === 'idle';

  // Record every settled document change that did not come from undo/redo.
  useEffect(() => {
    if (!settled) return;
    const { past, current } = latest.current;
    if (current === null) {
      commit({ past, future: [], current: code });
    } else if (code !== current) {
      commit({
        past: [...past, current].slice(-MAX_HISTORY),
        future: [],
        current: code,
      });
    }
  }, [code, settled, commit]);

  const { loadServerConfig } = source;
  const jump = useCallback(
    (direction: 'undo' | 'redo') => {
      const { past, future, current } = latest.current;
      const from = direction === 'undo' ? past : future;
      const target = from[from.length - 1];
      if (target === undefined || current === null) return;
      const rest = from.slice(0, -1);
      const other = [...(direction === 'undo' ? future : past), current];
      commit(
        direction === 'undo'
          ? { past: rest, future: other, current: target }
          : { past: other, future: rest, current: target }
      );
      loadServerConfig(target);
    },
    [commit, loadServerConfig]
  );

  const undo = useCallback(() => jump('undo'), [jump]);
  const redo = useCallback(() => jump('redo'), [jump]);
  const reset = useCallback(
    (fresh: string) => commit({ past: [], future: [], current: fresh }),
    [commit]
  );

  useEffect(() => {
    if (!enabled) return;
    const onKey = (event: KeyboardEvent) => {
      if (!(event.ctrlKey || event.metaKey) || event.altKey) return;
      if (hasNativeUndo(event.target)) return;
      const key = event.key.toLowerCase();
      const isRedo = (key === 'z' && event.shiftKey) || key === 'y';
      const isUndo = key === 'z' && !event.shiftKey;
      if (!isUndo && !isRedo) return;
      event.preventDefault();
      if (isRedo) redo();
      else undo();
    };
    document.addEventListener('keydown', onKey);
    return () => document.removeEventListener('keydown', onKey);
  }, [enabled, undo, redo]);

  return {
    undo,
    redo,
    reset,
    canUndo: stacks.past.length > 0,
    canRedo: stacks.future.length > 0,
  };
}
