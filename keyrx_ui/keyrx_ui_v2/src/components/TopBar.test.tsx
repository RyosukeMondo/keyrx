import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { TopBar } from './TopBar';

describe('TopBar', () => {
  it('renders logo and title', () => {
    render(<TopBar />);

    // Logo should be present
    expect(screen.getByLabelText('KeyRx2 Logo')).toBeInTheDocument();

    // Title should be present
    expect(screen.getByText('KeyRx2 Configuration')).toBeInTheDocument();
  });

  it('renders settings and help buttons', () => {
    render(<TopBar />);

    // Settings button should be present
    expect(
      screen.getByRole('button', { name: /open settings/i })
    ).toBeInTheDocument();

    // Help button should be present
    expect(
      screen.getByRole('button', { name: /help and documentation/i })
    ).toBeInTheDocument();
  });

  it('calls onSettingsClick when settings button is clicked', async () => {
    const user = userEvent.setup();
    const onSettingsClick = vi.fn();

    render(<TopBar onSettingsClick={onSettingsClick} />);

    const settingsButton = screen.getByRole('button', {
      name: /open settings/i,
    });
    await user.click(settingsButton);

    expect(onSettingsClick).toHaveBeenCalledTimes(1);
  });

  it('calls onHelpClick when help button is clicked', async () => {
    const user = userEvent.setup();
    const onHelpClick = vi.fn();

    render(<TopBar onHelpClick={onHelpClick} />);

    const helpButton = screen.getByRole('button', {
      name: /help and documentation/i,
    });
    await user.click(helpButton);

    expect(onHelpClick).toHaveBeenCalledTimes(1);
  });

  it('has correct ARIA role for header', () => {
    const { container } = render(<TopBar />);

    const header = container.querySelector('header');
    expect(header).toHaveAttribute('role', 'banner');
  });

  it('applies custom className', () => {
    const { container } = render(<TopBar className="custom-class" />);

    const header = container.querySelector('header');
    expect(header).toHaveClass('custom-class');
  });

  it('hides title on mobile screens', () => {
    render(<TopBar />);

    const title = screen.getByText('KeyRx2 Configuration');

    // Title should have hidden class for mobile
    expect(title).toHaveClass('hidden');
    expect(title).toHaveClass('md:block');
  });

  it('buttons have hover states', () => {
    render(<TopBar />);

    const settingsButton = screen.getByRole('button', {
      name: /open settings/i,
    });
    const helpButton = screen.getByRole('button', {
      name: /help and documentation/i,
    });

    // Both buttons should have ghost variant
    expect(settingsButton).toHaveClass('text-slate-300');
    expect(settingsButton).toHaveClass('hover:text-slate-100');
    expect(helpButton).toHaveClass('text-slate-300');
    expect(helpButton).toHaveClass('hover:text-slate-100');
  });

  it('renders without crashing when callbacks are not provided', () => {
    expect(() => render(<TopBar />)).not.toThrow();
  });
});
