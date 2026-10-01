import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { LayerSwitcher } from './LayerSwitcher';

describe('LayerSwitcher', () => {
  it('lists only Base and the used layers by default', () => {
    render(
      <LayerSwitcher
        activeLayer="base"
        availableLayers={['base', 'md-00']}
        onLayerChange={vi.fn()}
      />
    );
    expect(screen.getAllByRole('option').map((o) => o.textContent)).toEqual([
      'Base',
      'MD_00',
    ]);
  });

  it('reveals all 256 layers behind the Advanced toggle', async () => {
    const user = userEvent.setup();
    render(
      <LayerSwitcher
        activeLayer="base"
        availableLayers={['base']}
        onLayerChange={vi.fn()}
      />
    );
    const toggle = screen.getByRole('button', { name: 'All layers' });
    expect(toggle).toHaveAttribute('aria-expanded', 'false');
    await user.click(toggle);
    expect(screen.getAllByRole('option')).toHaveLength(257);
    expect(
      screen.getByRole('button', { name: 'Fewer layers' })
    ).toHaveAttribute('aria-expanded', 'true');
  });

  it('is one Tab stop; arrows move focus and Enter selects', async () => {
    const onLayerChange = vi.fn();
    const user = userEvent.setup();
    render(
      <LayerSwitcher
        activeLayer="base"
        availableLayers={['base', 'md-00', 'md-01']}
        onLayerChange={onLayerChange}
      />
    );
    const options = screen.getAllByRole('option');
    expect(options.filter((o) => o.tabIndex === 0)).toHaveLength(1);
    await user.tab();
    expect(document.activeElement).toBe(options[0]);
    await user.keyboard('{ArrowDown}{ArrowDown}');
    expect(document.activeElement).toBe(options[2]);
    await user.keyboard('{Enter}');
    expect(onLayerChange).toHaveBeenCalledWith('md-01');
  });
});
