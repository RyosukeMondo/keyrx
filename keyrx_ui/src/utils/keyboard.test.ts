/**
 * Keyboard Navigation Utilities Tests
 */

import { describe, it, expect } from 'vitest';
import {
  Keys,
  isFocusable,
  getFocusableElements,
  isClickKey,
  focusVisibleClass,
} from './keyboard';

describe('Keys constants', () => {
  it('should have correct key values', () => {
    expect(Keys.ENTER).toBe('Enter');
    expect(Keys.ESCAPE).toBe('Escape');
    expect(Keys.SPACE).toBe(' ');
    expect(Keys.TAB).toBe('Tab');
    expect(Keys.ARROW_UP).toBe('ArrowUp');
    expect(Keys.ARROW_DOWN).toBe('ArrowDown');
    expect(Keys.ARROW_LEFT).toBe('ArrowLeft');
    expect(Keys.ARROW_RIGHT).toBe('ArrowRight');
  });
});

describe('isFocusable', () => {
  it('should return true for focusable elements', () => {
    const button = document.createElement('button');
    expect(isFocusable(button)).toBe(true);

    const link = document.createElement('a');
    link.href = '#';
    expect(isFocusable(link)).toBe(true);

    const input = document.createElement('input');
    expect(isFocusable(input)).toBe(true);
  });

  it('should return false for disabled elements', () => {
    const button = document.createElement('button');
    button.disabled = true;
    expect(isFocusable(button)).toBe(false);
  });

  it('should return false for non-focusable elements', () => {
    const div = document.createElement('div');
    expect(isFocusable(div)).toBe(false);

    const span = document.createElement('span');
    expect(isFocusable(span)).toBe(false);
  });

  it('should return true for elements with tabindex', () => {
    const div = document.createElement('div');
    div.tabIndex = 0;
    expect(isFocusable(div)).toBe(true);
  });

  it('should return false for elements with tabindex -1', () => {
    const div = document.createElement('div');
    div.tabIndex = -1;
    expect(isFocusable(div)).toBe(false);
  });
});

describe('getFocusableElements', () => {
  it('should return all focusable elements in container', () => {
    const container = document.createElement('div');
    const button1 = document.createElement('button');
    const button2 = document.createElement('button');
    const link = document.createElement('a');
    link.href = '#';
    const disabledButton = document.createElement('button');
    disabledButton.disabled = true;

    container.appendChild(button1);
    container.appendChild(button2);
    container.appendChild(link);
    container.appendChild(disabledButton);

    const focusable = getFocusableElements(container);
    expect(focusable).toHaveLength(3);
    expect(focusable).toContain(button1);
    expect(focusable).toContain(button2);
    expect(focusable).toContain(link);
    expect(focusable).not.toContain(disabledButton);
  });

  it('should return empty array for container with no focusable elements', () => {
    const container = document.createElement('div');
    const div = document.createElement('div');
    container.appendChild(div);

    const focusable = getFocusableElements(container);
    expect(focusable).toHaveLength(0);
  });
});

describe('isClickKey', () => {
  it('should return true for Enter key', () => {
    const event = new KeyboardEvent('keydown', { key: 'Enter' });
    expect(isClickKey(event as any)).toBe(true);
  });

  it('should return true for Space key', () => {
    const event = new KeyboardEvent('keydown', { key: ' ' });
    expect(isClickKey(event as any)).toBe(true);
  });

  it('should return false for other keys', () => {
    const event = new KeyboardEvent('keydown', { key: 'a' });
    expect(isClickKey(event as any)).toBe(false);
  });
});

describe('focusVisibleClass', () => {
  it('should contain focus outline classes', () => {
    expect(focusVisibleClass).toContain('focus:outline');
    expect(focusVisibleClass).toContain('focus:outline-2');
    expect(focusVisibleClass).toContain('focus:outline-primary-500');
  });
});
