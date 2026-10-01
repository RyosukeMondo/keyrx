import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { SwapKeysPanel } from './SwapKeysPanel';
import type { SVGKeyData } from '@/utils/kle-parser';

const keys: SVGKeyData[] = [
  {
    code: 'KC_CAPS',
    label: 'Caps',
    x: 0,
    y: 0,
    w: 1.5,
    h: 1,
    shape: 'standard',
  },
  {
    code: 'KC_LCTL',
    label: 'Ctrl',
    x: 0,
    y: 1,
    w: 1.5,
    h: 1,
    shape: 'standard',
  },
  { code: 'KC_A', label: 'A', x: 1.5, y: 0, w: 1, h: 1, shape: 'standard' },
];

describe('SwapKeysPanel', () => {
  it('swaps Caps Lock and Ctrl in one click', async () => {
    const onSwap = vi.fn();
    render(
      <SwapKeysPanel layoutKeys={keys} onSwap={onSwap} onCancel={vi.fn()} />
    );
    await userEvent.click(
      screen.getByRole('button', { name: 'Caps Lock ↔ Ctrl' })
    );
    expect(onSwap).toHaveBeenCalledWith('VK_CapsLock', 'VK_LCtrl');
  });

  it('swaps any two different keys, with friendly option names', async () => {
    const user = userEvent.setup();
    const onSwap = vi.fn();
    render(
      <SwapKeysPanel layoutKeys={keys} onSwap={onSwap} onCancel={vi.fn()} />
    );
    const apply = screen.getByRole('button', { name: 'Swap them' });
    expect(apply).toBeDisabled();

    await user.selectOptions(screen.getByLabelText('First key'), 'Caps Lock');
    await user.selectOptions(screen.getByLabelText('Second key'), 'Caps Lock');
    expect(apply).toBeDisabled(); // same key twice is not a swap

    await user.selectOptions(screen.getByLabelText('Second key'), 'A');
    expect(apply).toBeEnabled();
    await user.click(apply);
    expect(onSwap).toHaveBeenCalledWith('VK_CapsLock', 'VK_A');
  });

  it('can be cancelled', async () => {
    const onCancel = vi.fn();
    render(
      <SwapKeysPanel layoutKeys={keys} onSwap={vi.fn()} onCancel={onCancel} />
    );
    await userEvent.click(screen.getByRole('button', { name: 'Cancel' }));
    expect(onCancel).toHaveBeenCalled();
  });
});
