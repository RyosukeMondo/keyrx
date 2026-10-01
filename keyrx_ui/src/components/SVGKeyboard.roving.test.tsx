import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { SVGKeyboard, describeKey, type SVGKey } from './SVGKeyboard';
import type { KeyMapping } from '@/types';

const keys: SVGKey[] = [
  { code: 'KC_CAPS', label: 'Caps', x: 0, y: 0, w: 1.5, h: 1 },
  { code: 'KC_A', label: 'A', x: 1.5, y: 0, w: 1, h: 1 },
  { code: 'KC_S', label: 'S', x: 2.5, y: 0, w: 1, h: 1 },
  { code: 'KC_LCTL', label: 'Ctrl', x: 0, y: 1, w: 1.5, h: 1 },
];

const tabStops = () =>
  screen
    .getAllByRole('button')
    .filter((b) => b.getAttribute('tabindex') === '0');

describe('SVGKeyboard keyboard navigation', () => {
  it('is a single Tab stop (roving tabindex)', () => {
    render(
      <SVGKeyboard keys={keys} keyMappings={new Map()} onKeyClick={vi.fn()} />
    );
    expect(screen.getAllByRole('button')).toHaveLength(4);
    expect(tabStops()).toHaveLength(1);
  });

  it('moves focus and the tab stop with the arrow keys', async () => {
    const user = userEvent.setup();
    render(
      <SVGKeyboard keys={keys} keyMappings={new Map()} onKeyClick={vi.fn()} />
    );
    await user.tab();
    expect(document.activeElement).toHaveAttribute(
      'data-key-code',
      'VK_CapsLock'
    );
    await user.keyboard('{ArrowRight}');
    expect(document.activeElement).toHaveAttribute('data-key-code', 'VK_A');
    expect(tabStops()).toHaveLength(1);
    expect(tabStops()[0]).toBe(document.activeElement);
    await user.keyboard('{ArrowDown}');
    expect(document.activeElement).toHaveAttribute('data-key-code', 'VK_LCtrl');
    await user.keyboard('{ArrowUp}{End}');
    expect(document.activeElement).toHaveAttribute('data-key-code', 'VK_S');
  });

  it('selects the focused key with Enter and Space (normalized VK_ code)', async () => {
    const onKeyClick = vi.fn();
    const user = userEvent.setup();
    render(
      <SVGKeyboard
        keys={keys}
        keyMappings={new Map()}
        onKeyClick={onKeyClick}
      />
    );
    await user.tab();
    await user.keyboard('{Enter}');
    await user.keyboard('{ArrowRight}');
    await user.keyboard(' ');
    expect(onKeyClick).toHaveBeenNthCalledWith(1, 'VK_CapsLock');
    expect(onKeyClick).toHaveBeenNthCalledWith(2, 'VK_A');
  });

  it('starts the tab stop on the selected key and marks it pressed', () => {
    render(
      <SVGKeyboard
        keys={keys}
        keyMappings={new Map()}
        onKeyClick={vi.fn()}
        selectedKeyCode="VK_S"
      />
    );
    expect(tabStops()[0]).toHaveAttribute('data-key-code', 'VK_S');
    expect(tabStops()[0]).toHaveAttribute('aria-pressed', 'true');
  });

  it('has no tab stops in simulator (read-only) mode', () => {
    render(
      <SVGKeyboard
        keys={keys}
        keyMappings={new Map()}
        onKeyClick={vi.fn()}
        simulatorMode
      />
    );
    expect(screen.queryAllByRole('button')).toHaveLength(0);
    expect(screen.getAllByRole('img')).toHaveLength(4);
  });
});

describe('describeKey', () => {
  it('uses DSL names, not QMK codes', () => {
    expect(describeKey('KC_CAPS')).toBe('CapsLock, not remapped');
    const simple: KeyMapping = { type: 'simple', tapAction: 'VK_LCtrl' };
    expect(describeKey('KC_CAPS', simple)).toBe('CapsLock, acts as LCtrl');
    const th: KeyMapping = {
      type: 'tap_hold',
      tapAction: 'VK_Space',
      holdAction: 'MD_00',
      threshold: 200,
    };
    expect(describeKey('KC_SPC', th)).toBe(
      'Space, tap for Space, hold for MD_00 after 200 ms'
    );
  });
});
