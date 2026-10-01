import { describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { renderWithProviders } from '../../../tests/testUtils';
import { UseCaseGuide } from './UseCaseGuide';

const renderGuide = (hasDevice = true) => {
  const callbacks = {
    onStartSwap: vi.fn(),
    onStartSimple: vi.fn(),
    onStartCommandPad: vi.fn(),
    onStartAdvanced: vi.fn(),
  };
  renderWithProviders(<UseCaseGuide hasDevice={hasDevice} {...callbacks} />);
  return callbacks;
};

describe('UseCaseGuide', () => {
  it('offers paths for light, gadget, and advanced users', () => {
    renderGuide();

    expect(
      screen.getByRole('button', { name: /Change a few keys/i })
    ).toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: /Make a command pad/i })
    ).toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: /Build a full layout/i })
    ).toBeInTheDocument();
  });

  it('offers a one-step "Swap two keys" path', async () => {
    const user = userEvent.setup();
    const callbacks = renderGuide();
    await user.click(screen.getByRole('button', { name: /Swap two keys/i }));
    expect(callbacks.onStartSwap).toHaveBeenCalledOnce();
  });

  it('routes each path to its setup action', async () => {
    const user = userEvent.setup();
    const callbacks = renderGuide();

    await user.click(
      screen.getByRole('button', { name: /Change a few keys/i })
    );
    await user.click(
      screen.getByRole('button', { name: /Make a command pad/i })
    );
    await user.click(
      screen.getByRole('button', { name: /Build a full layout/i })
    );

    expect(callbacks.onStartSimple).toHaveBeenCalledOnce();
    expect(callbacks.onStartCommandPad).toHaveBeenCalledOnce();
    expect(callbacks.onStartAdvanced).toHaveBeenCalledOnce();
  });

  it('explains why command-pad setup is unavailable without a device', () => {
    renderGuide(false);

    const commandPad = screen.getByRole('button', {
      name: /Make a command pad/i,
    });
    expect(commandPad).toBeDisabled();
    expect(screen.getByText('Connect a device first')).toBeInTheDocument();
  });
});
