import { afterEach, describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { CurrentMappingsSummary } from './CurrentMappingsSummary';
import { setLocale } from '@/i18n';
import type { KeyMapping } from '@/types';

const mappings = new Map<string, KeyMapping>([
  ['VK_CapsLock', { type: 'simple', tapAction: 'VK_LCtrl' }],
  [
    'VK_Space',
    {
      type: 'tap_hold',
      tapAction: 'VK_Space',
      holdAction: 'MD_00',
      threshold: 180,
    },
  ],
  ['VK_Muhenkan', { type: 'simple', tapAction: 'VK_Escape' }],
]);

const renderSummary = () =>
  render(
    <CurrentMappingsSummary
      keyMappings={mappings}
      onEditMapping={vi.fn()}
      onClearMapping={vi.fn()}
    />
  );

describe('CurrentMappingsSummary', () => {
  afterEach(() => setLocale('en'));

  it('has exactly one "Current Mappings" heading', () => {
    renderSummary();
    expect(
      screen.getAllByRole('heading', { name: /Current Mappings/ })
    ).toHaveLength(1);
  });

  it('shows human key names, never VK_ codes', () => {
    const { container } = renderSummary();
    expect(container.textContent).not.toMatch(/VK_/);
    expect(screen.getByText('Caps Lock')).toBeInTheDocument();
    expect(screen.getByText('Ctrl')).toBeInTheDocument();
    expect(
      screen.getByLabelText('Remove mapping for Caps Lock')
    ).toBeInTheDocument();
  });

  it('speaks Japanese with the English key name beside JIS keys', () => {
    setLocale('ja');
    renderSummary();
    expect(
      screen.getByRole('heading', { name: /現在の割り当て/ })
    ).toBeInTheDocument();
    expect(screen.getByText('無変換')).toBeInTheDocument();
    expect(screen.getByText('Muhenkan')).toBeInTheDocument();
  });
});
