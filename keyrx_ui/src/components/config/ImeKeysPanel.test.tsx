import { afterEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen } from '@testing-library/react';
import { ImeKeysPanel } from './ImeKeysPanel';
import { setLocale } from '@/i18n';
import type { SVGKeyData } from '@/utils/kle-parser';

const key = (code: string): SVGKeyData =>
  ({ code, label: code, x: 0, y: 0, w: 1, h: 1 }) as SVGKeyData;
const keys = ['KC_CAPS', 'KC_RALT', 'KC_LALT', 'KC_SPC'].map(key);

describe('ImeKeysPanel', () => {
  afterEach(() => setLocale('en'));

  it('toggle style maps one key to Zenkaku/Hankaku (Grave)', () => {
    const onApply = vi.fn();
    render(
      <ImeKeysPanel layoutKeys={keys} onApply={onApply} onCancel={vi.fn()} />
    );
    fireEvent.click(screen.getByRole('button', { name: 'Set IME keys' }));
    expect(onApply).toHaveBeenCalledWith([['VK_CapsLock', 'VK_Zenkaku']]);
  });

  it('pair style maps an on key to Henkan and an off key to Muhenkan', () => {
    const onApply = vi.fn();
    render(
      <ImeKeysPanel layoutKeys={keys} onApply={onApply} onCancel={vi.fn()} />
    );
    fireEvent.click(
      screen.getByRole('radio', { name: /Separate on and off keys/ })
    );
    fireEvent.click(screen.getByRole('button', { name: 'Set IME keys' }));
    expect(onApply).toHaveBeenCalledWith([
      ['VK_RAlt', 'VK_Henkan'],
      ['VK_LAlt', 'VK_Muhenkan'],
    ]);
  });

  it('refuses the same key for on and off', () => {
    render(
      <ImeKeysPanel layoutKeys={keys} onApply={vi.fn()} onCancel={vi.fn()} />
    );
    fireEvent.click(
      screen.getByRole('radio', { name: /Separate on and off keys/ })
    );
    fireEvent.change(screen.getByLabelText(/Key for IME off/), {
      target: { value: 'VK_RAlt' },
    });
    expect(screen.getByRole('button', { name: 'Set IME keys' })).toBeDisabled();
  });

  it('is offered in Japanese as IME オン/オフ', () => {
    setLocale('ja');
    render(
      <ImeKeysPanel layoutKeys={keys} onApply={vi.fn()} onCancel={vi.fn()} />
    );
    expect(
      screen.getByRole('heading', { name: 'IME オン/オフ' })
    ).toBeInTheDocument();
  });
});
