/**
 * Keyboard Shortcuts Hook
 *
 * Global keyboard shortcuts for the application.
 * Provides common shortcuts like:
 * - Ctrl/Cmd + S: Save
 * - Ctrl/Cmd + K: Search
 * - Ctrl/Cmd + /: Toggle help
 * - Escape: Close dialogs/modals
 */

import { useEffect, useCallback } from 'react';

export interface KeyboardShortcut {
  key: string;
  ctrlKey?: boolean;
  shiftKey?: boolean;
  altKey?: boolean;
  metaKey?: boolean;
  handler: (event: KeyboardEvent) => void;
  description: string;
  preventDefault?: boolean;
}

/**
 * Check if keyboard event matches shortcut definition
 */
function matchesShortcut(
  event: KeyboardEvent,
  shortcut: KeyboardShortcut
): boolean {
  const keyMatches =
    event.key.toLowerCase() === shortcut.key.toLowerCase();
  const ctrlMatches = !!event.ctrlKey === !!shortcut.ctrlKey;
  const shiftMatches = !!event.shiftKey === !!shortcut.shiftKey;
  const altMatches = !!event.altKey === !!shortcut.altKey;
  const metaMatches = !!event.metaKey === !!shortcut.metaKey;

  return (
    keyMatches && ctrlMatches && shiftMatches && altMatches && metaMatches
  );
}

/**
 * Hook to register keyboard shortcuts
 */
export function useKeyboardShortcuts(
  shortcuts: KeyboardShortcut[],
  enabled = true
): void {
  const handleKeyDown = useCallback(
    (event: KeyboardEvent) => {
      // Don't trigger shortcuts when typing in inputs
      const target = event.target as HTMLElement;
      const isInput =
        target.tagName === 'INPUT' ||
        target.tagName === 'TEXTAREA' ||
        target.isContentEditable;

      if (isInput && !event.ctrlKey && !event.metaKey) {
        return;
      }

      for (const shortcut of shortcuts) {
        if (matchesShortcut(event, shortcut)) {
          if (shortcut.preventDefault !== false) {
            event.preventDefault();
          }
          shortcut.handler(event);
          break;
        }
      }
    },
    [shortcuts]
  );

  useEffect(() => {
    if (!enabled) return;

    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [handleKeyDown, enabled]);
}

/**
 * Format shortcut for display
 */
export function formatShortcut(shortcut: KeyboardShortcut): string {
  const parts: string[] = [];

  const isMac = navigator.platform.toLowerCase().includes('mac');

  if (shortcut.ctrlKey && !isMac) {
    parts.push('Ctrl');
  }
  if (shortcut.metaKey || (shortcut.ctrlKey && isMac)) {
    parts.push('⌘');
  }
  if (shortcut.shiftKey) {
    parts.push('Shift');
  }
  if (shortcut.altKey) {
    parts.push(isMac ? 'Option' : 'Alt');
  }

  // Capitalize key for display
  const key = shortcut.key.length === 1 ? shortcut.key.toUpperCase() : shortcut.key;
  parts.push(key);

  return parts.join('+');
}

/**
 * Common keyboard shortcuts
 */
export const CommonShortcuts = {
  save: (handler: () => void): KeyboardShortcut => ({
    key: 's',
    ctrlKey: true,
    handler: () => handler(),
    description: 'Save changes',
  }),

  search: (handler: () => void): KeyboardShortcut => ({
    key: 'k',
    ctrlKey: true,
    handler: () => handler(),
    description: 'Open search',
  }),

  help: (handler: () => void): KeyboardShortcut => ({
    key: '/',
    ctrlKey: true,
    handler: () => handler(),
    description: 'Show help',
  }),

  escape: (handler: () => void): KeyboardShortcut => ({
    key: 'Escape',
    handler: () => handler(),
    description: 'Close dialog',
  }),

  refresh: (handler: () => void): KeyboardShortcut => ({
    key: 'r',
    ctrlKey: true,
    handler: () => handler(),
    description: 'Refresh data',
    preventDefault: true,
  }),

  undo: (handler: () => void): KeyboardShortcut => ({
    key: 'z',
    ctrlKey: true,
    handler: () => handler(),
    description: 'Undo',
  }),

  redo: (handler: () => void): KeyboardShortcut => ({
    key: 'z',
    ctrlKey: true,
    shiftKey: true,
    handler: () => handler(),
    description: 'Redo',
  }),

  toggleSidebar: (handler: () => void): KeyboardShortcut => ({
    key: 'b',
    ctrlKey: true,
    handler: () => handler(),
    description: 'Toggle sidebar',
  }),

  newItem: (handler: () => void): KeyboardShortcut => ({
    key: 'n',
    ctrlKey: true,
    handler: () => handler(),
    description: 'Create new item',
  }),

  delete: (handler: () => void): KeyboardShortcut => ({
    key: 'Delete',
    handler: () => handler(),
    description: 'Delete selected item',
  }),
};
