import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { SaveReviewModal, describeChange } from './SaveReviewModal';

vi.mock('./ProfileDiffView', () => ({
  ProfileDiffView: () => <div data-testid="rhai-diff" />,
}));

const original = 'device_start("*");\ndevice_end();\n';
const modified = `device_start("*");
  map("VK_CapsLock", "VK_LCtrl");
  map("VK_LCtrl", "VK_CapsLock");
device_end();
`;

describe('SaveReviewModal', () => {
  it('shows the change in plain language and hides the Rhai behind Details', async () => {
    const user = userEvent.setup();
    render(
      <SaveReviewModal
        open
        original={original}
        modified={modified}
        onCancel={vi.fn()}
        onConfirm={vi.fn()}
      />
    );
    const list = screen.getByTestId('save-review-changes');
    expect(list).toHaveTextContent('Caps Lock → Ctrl');
    expect(list).toHaveTextContent('Ctrl → Caps Lock');
    expect(screen.queryByTestId('rhai-diff')).toBeNull();

    const details = screen.getByRole('button', { name: 'Details' });
    expect(details).toHaveAttribute('aria-expanded', 'false');
    await user.click(details);
    expect(await screen.findByTestId('rhai-diff')).toBeInTheDocument();
    expect(details).toHaveAttribute('aria-expanded', 'true');
  });

  it('confirms and cancels', async () => {
    const user = userEvent.setup();
    const onConfirm = vi.fn();
    const onCancel = vi.fn();
    render(
      <SaveReviewModal
        open
        original={original}
        modified={modified}
        onCancel={onCancel}
        onConfirm={onConfirm}
      />
    );
    await user.click(screen.getByRole('button', { name: 'Save changes' }));
    expect(onConfirm).toHaveBeenCalled();
    await user.click(screen.getByRole('button', { name: 'Cancel' }));
    expect(onCancel).toHaveBeenCalled();
  });

  it('falls back to the raw diff when the summary cannot be computed', async () => {
    render(
      <SaveReviewModal
        open
        original={original}
        modified={'device_start("x");'}
        onCancel={vi.fn()}
        onConfirm={vi.fn()}
      />
    );
    expect(await screen.findByTestId('rhai-diff')).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Details' })).toBeNull();
  });
});

describe('describeChange', () => {
  it('adds layer and device context', () => {
    expect(
      describeChange({
        kind: 'added',
        from: 'H',
        to: 'Left',
        layer: 'MD_00',
        device: 'usb*',
      })
    ).toBe('H → Left (on layer MD_00) (only on usb*)');
    expect(describeChange({ kind: 'removed', from: 'A', to: '' })).toBe(
      'A is back to normal'
    );
  });
});
