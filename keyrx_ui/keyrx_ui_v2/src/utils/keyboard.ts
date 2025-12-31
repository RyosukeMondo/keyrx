/**
 * Keyboard Navigation Utilities
 *
 * Helper functions and hooks for keyboard navigation,
 * accessibility, and focus management.
 */

import { useEffect, useCallback, RefObject } from 'react';

/**
 * Common keyboard keys
 */
export const Keys = {
  ENTER: 'Enter',
  ESCAPE: 'Escape',
  SPACE: ' ',
  TAB: 'Tab',
  ARROW_UP: 'ArrowUp',
  ARROW_DOWN: 'ArrowDown',
  ARROW_LEFT: 'ArrowLeft',
  ARROW_RIGHT: 'ArrowRight',
  HOME: 'Home',
  END: 'End',
  PAGE_UP: 'PageUp',
  PAGE_DOWN: 'PageDown',
} as const;

/**
 * Check if an element is focusable
 */
export function isFocusable(element: HTMLElement): boolean {
  const focusableSelectors = [
    'a[href]',
    'button:not([disabled])',
    'textarea:not([disabled])',
    'input:not([disabled])',
    'select:not([disabled])',
    '[tabindex]:not([tabindex="-1"])',
  ];

  return focusableSelectors.some((selector) => element.matches(selector));
}

/**
 * Get all focusable elements within a container
 */
export function getFocusableElements(
  container: HTMLElement
): HTMLElement[] {
  const selector = [
    'a[href]',
    'button:not([disabled])',
    'textarea:not([disabled])',
    'input:not([disabled])',
    'select:not([disabled])',
    '[tabindex]:not([tabindex="-1"])',
  ].join(', ');

  return Array.from(container.querySelectorAll<HTMLElement>(selector));
}

/**
 * Focus the first focusable element in a container
 */
export function focusFirst(container: HTMLElement): void {
  const elements = getFocusableElements(container);
  elements[0]?.focus();
}

/**
 * Focus the last focusable element in a container
 */
export function focusLast(container: HTMLElement): void {
  const elements = getFocusableElements(container);
  elements[elements.length - 1]?.focus();
}

/**
 * Hook to handle Escape key
 */
export function useEscapeKey(onEscape: () => void, enabled = true): void {
  useEffect(() => {
    if (!enabled) return;

    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === Keys.ESCAPE) {
        onEscape();
      }
    };

    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [onEscape, enabled]);
}

/**
 * Hook to handle arrow key navigation in a list
 */
export function useArrowNavigation(
  containerRef: RefObject<HTMLElement>,
  options: {
    orientation?: 'vertical' | 'horizontal';
    loop?: boolean;
    onEnter?: (index: number) => void;
  } = {}
): void {
  const { orientation = 'vertical', loop = true, onEnter } = options;

  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    const handleKeyDown = (event: KeyboardEvent) => {
      const focusableElements = getFocusableElements(container);
      const currentIndex = focusableElements.findIndex(
        (el) => el === document.activeElement
      );

      if (currentIndex === -1) return;

      const isVertical = orientation === 'vertical';
      const nextKey = isVertical ? Keys.ARROW_DOWN : Keys.ARROW_RIGHT;
      const prevKey = isVertical ? Keys.ARROW_UP : Keys.ARROW_LEFT;

      let nextIndex = currentIndex;

      if (event.key === nextKey) {
        event.preventDefault();
        nextIndex = currentIndex + 1;
        if (nextIndex >= focusableElements.length) {
          nextIndex = loop ? 0 : focusableElements.length - 1;
        }
      } else if (event.key === prevKey) {
        event.preventDefault();
        nextIndex = currentIndex - 1;
        if (nextIndex < 0) {
          nextIndex = loop ? focusableElements.length - 1 : 0;
        }
      } else if (event.key === Keys.HOME) {
        event.preventDefault();
        nextIndex = 0;
      } else if (event.key === Keys.END) {
        event.preventDefault();
        nextIndex = focusableElements.length - 1;
      } else if (event.key === Keys.ENTER && onEnter) {
        event.preventDefault();
        onEnter(currentIndex);
        return;
      }

      if (nextIndex !== currentIndex) {
        focusableElements[nextIndex]?.focus();
      }
    };

    container.addEventListener('keydown', handleKeyDown);
    return () => container.removeEventListener('keydown', handleKeyDown);
  }, [containerRef, orientation, loop, onEnter]);
}

/**
 * Hook to trap focus within a container (for modals, dialogs)
 */
export function useFocusTrap(
  containerRef: RefObject<HTMLElement>,
  enabled = true
): void {
  useEffect(() => {
    if (!enabled) return;

    const container = containerRef.current;
    if (!container) return;

    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key !== Keys.TAB) return;

      const focusableElements = getFocusableElements(container);
      const firstElement = focusableElements[0];
      const lastElement = focusableElements[focusableElements.length - 1];

      if (event.shiftKey) {
        // Shift + Tab
        if (document.activeElement === firstElement) {
          event.preventDefault();
          lastElement?.focus();
        }
      } else {
        // Tab
        if (document.activeElement === lastElement) {
          event.preventDefault();
          firstElement?.focus();
        }
      }
    };

    container.addEventListener('keydown', handleKeyDown);
    return () => container.removeEventListener('keydown', handleKeyDown);
  }, [containerRef, enabled]);
}

/**
 * Hook to restore focus when component unmounts
 */
export function useRestoreFocus(enabled = true): () => void {
  const previousElement = useCallback(() => {
    if (!enabled) return;
    const activeElement = document.activeElement as HTMLElement;
    return activeElement;
  }, [enabled]);

  return useCallback(() => {
    const element = previousElement();
    if (element && enabled) {
      element.focus();
    }
  }, [previousElement, enabled]);
}

/**
 * Create keyboard event handler
 */
export function createKeyboardHandler(
  handlers: Record<string, (event: React.KeyboardEvent) => void>
) {
  return (event: React.KeyboardEvent) => {
    const handler = handlers[event.key];
    if (handler) {
      handler(event);
    }
  };
}

/**
 * Check if an event should trigger a button click
 * (Space or Enter keys)
 */
export function isClickKey(event: React.KeyboardEvent): boolean {
  return event.key === Keys.ENTER || event.key === Keys.SPACE;
}

/**
 * Prevent default for click keys
 */
export function preventDefaultForClickKeys(
  event: React.KeyboardEvent
): void {
  if (isClickKey(event)) {
    event.preventDefault();
  }
}

/**
 * CSS class for focus-visible outline
 */
export const focusVisibleClass =
  'focus:outline focus:outline-2 focus:outline-primary-500 focus:outline-offset-2';

/**
 * CSS class for focus-visible outline (within parent)
 */
export const focusVisibleWithinClass =
  'focus-within:outline focus-within:outline-2 focus-within:outline-primary-500 focus-within:outline-offset-2';
