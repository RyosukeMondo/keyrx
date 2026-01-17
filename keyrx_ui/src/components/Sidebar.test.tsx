import { describe, it, expect, vi } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithProviders } from '../../tests/testUtils';
import { BrowserRouter } from 'react-router-dom';
import userEvent from '@testing-library/user-event';
import { Sidebar } from './Sidebar';

const renderWithRouter = (ui: React.ReactElement, { route = '/' } = {}) => {
  window.history.pushState({}, 'Test page', route);
  return renderWithProviders(ui, { wrapper: BrowserRouter });
};

describe('Sidebar', () => {
  it('renders all navigation items', () => {
    renderWithRouter(<Sidebar />);

    expect(screen.getByText('Home')).toBeInTheDocument();
    expect(screen.getByText('Devices')).toBeInTheDocument();
    expect(screen.getByText('Profiles')).toBeInTheDocument();
    expect(screen.getByText('Config')).toBeInTheDocument();
    expect(screen.getByText('Metrics')).toBeInTheDocument();
    expect(screen.getByText('Simulator')).toBeInTheDocument();
  });

  it('has proper ARIA labels on navigation items', () => {
    renderWithRouter(<Sidebar />);

    expect(screen.getByLabelText('Navigate to Home page')).toBeInTheDocument();
    expect(
      screen.getByLabelText('Navigate to Devices page')
    ).toBeInTheDocument();
    expect(
      screen.getByLabelText('Navigate to Profiles page')
    ).toBeInTheDocument();
    expect(
      screen.getByLabelText('Navigate to Configuration page')
    ).toBeInTheDocument();
    expect(
      screen.getByLabelText('Navigate to Metrics page')
    ).toBeInTheDocument();
    expect(
      screen.getByLabelText('Navigate to Simulator page')
    ).toBeInTheDocument();
  });

  it('has proper ARIA label on sidebar element', () => {
    renderWithRouter(<Sidebar />);

    expect(
      screen.getByLabelText('Main navigation sidebar')
    ).toBeInTheDocument();
  });

  it('highlights active navigation item', () => {
    renderWithRouter(<Sidebar />, { route: '/devices' });

    const devicesLink = screen.getByLabelText('Navigate to Devices page');
    expect(devicesLink).toHaveClass('bg-primary-600');
    expect(devicesLink).toHaveClass('text-white');
  });

  it('non-active items have hover states', () => {
    renderWithRouter(<Sidebar />, { route: '/' });

    const profilesLink = screen.getByLabelText('Navigate to Profiles page');
    expect(profilesLink).toHaveClass('text-slate-300');
    expect(profilesLink).toHaveClass('hover:bg-slate-700');
  });

  it('shows active indicator for current page', () => {
    renderWithRouter(<Sidebar />, { route: '/config' });

    const configLink = screen.getByLabelText('Navigate to Configuration page');
    // Active indicator is a white rounded bar
    expect(
      configLink.querySelector('.bg-white.rounded-full')
    ).toBeInTheDocument();
  });

  it('calls onClose when navigation item is clicked', async () => {
    const user = userEvent.setup();
    const onClose = vi.fn();

    renderWithRouter(<Sidebar onClose={onClose} />);

    const homeLink = screen.getByLabelText('Navigate to Home page');
    await user.click(homeLink);

    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it('calls onClose when Escape key is pressed', async () => {
    const user = userEvent.setup();
    const onClose = vi.fn();

    renderWithRouter(<Sidebar onClose={onClose} />);

    // Focus on a nav item first, then press Escape
    const homeLink = screen.getByLabelText('Navigate to Home page');
    homeLink.focus();
    await user.keyboard('{Escape}');

    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it('does not call onClose when onClose prop is not provided', async () => {
    const user = userEvent.setup();

    renderWithRouter(<Sidebar />);

    const sidebar = screen.getByLabelText('Main navigation sidebar');
    sidebar.focus();

    // Should not throw error when Escape is pressed without onClose
    await user.keyboard('{Escape}');
  });

  it('applies custom className when provided', () => {
    renderWithRouter(<Sidebar className="custom-class" />);

    const sidebar = screen.getByLabelText('Main navigation sidebar');
    expect(sidebar).toHaveClass('custom-class');
  });

  it('is visible when isOpen is true', () => {
    renderWithRouter(<Sidebar isOpen={true} />);

    const sidebar = screen.getByLabelText('Main navigation sidebar');
    expect(sidebar).toHaveClass('translate-x-0');
  });

  it('is hidden on mobile when isOpen is false', () => {
    renderWithRouter(<Sidebar isOpen={false} />);

    const sidebar = screen.getByLabelText('Main navigation sidebar');
    expect(sidebar).toHaveClass('-translate-x-full');
    expect(sidebar).toHaveClass('md:translate-x-0');
  });

  it('renders version information in footer', () => {
    renderWithRouter(<Sidebar />);

    expect(screen.getByText('KeyRx v2.0.0')).toBeInTheDocument();
  });

  it('has focus visible styles on navigation items', () => {
    renderWithRouter(<Sidebar />);

    const homeLink = screen.getByLabelText('Navigate to Home page');
    expect(homeLink).toHaveClass('focus:outline');
    expect(homeLink).toHaveClass('focus:outline-2');
    expect(homeLink).toHaveClass('focus:outline-primary-500');
  });

  it('is keyboard navigable with Tab', async () => {
    const user = userEvent.setup();

    renderWithRouter(<Sidebar />);

    // Tab through all navigation items
    await user.tab();
    expect(screen.getByLabelText('Navigate to Home page')).toHaveFocus();

    await user.tab();
    expect(screen.getByLabelText('Navigate to Devices page')).toHaveFocus();

    await user.tab();
    expect(screen.getByLabelText('Navigate to Profiles page')).toHaveFocus();
  });

  it('activates link on Enter key press', async () => {
    const user = userEvent.setup();
    const onClose = vi.fn();

    renderWithRouter(<Sidebar onClose={onClose} />);

    // Tab to first link and press Enter
    await user.tab();
    await user.keyboard('{Enter}');

    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it('renders all icons correctly', () => {
    renderWithRouter(<Sidebar />);

    // Check that all nav items have an icon (svg element)
    const navLinks = screen.getAllByRole('link');
    expect(navLinks).toHaveLength(6);

    navLinks.forEach((link) => {
      const svg = link.querySelector('svg');
      expect(svg).toBeInTheDocument();
      expect(svg).toHaveAttribute('aria-hidden', 'true');
    });
  });

  it('has smooth transition animation', () => {
    renderWithRouter(<Sidebar />);

    const sidebar = screen.getByLabelText('Main navigation sidebar');
    expect(sidebar).toHaveClass('transition-transform');
    expect(sidebar).toHaveClass('duration-300');
    expect(sidebar).toHaveClass('ease-in-out');
  });

  it('highlights Config when on /config route', () => {
    renderWithRouter(<Sidebar />, { route: '/config' });

    const configLink = screen.getByLabelText('Navigate to Configuration page');
    expect(configLink).toHaveClass('bg-primary-600');
    expect(configLink).toHaveClass('text-white');
  });

  it('highlights Config when on /profiles/:name/config route', () => {
    renderWithRouter(<Sidebar />, { route: '/profiles/default/config' });

    const configLink = screen.getByLabelText('Navigate to Configuration page');
    expect(configLink).toHaveClass('bg-primary-600');
    expect(configLink).toHaveClass('text-white');
    expect(
      configLink.querySelector('.bg-white.rounded-full')
    ).toBeInTheDocument();
  });

  it('highlights Config when on /profiles/:name/config route with different profile names', () => {
    renderWithRouter(<Sidebar />, { route: '/profiles/gaming/config' });

    const configLink = screen.getByLabelText('Navigate to Configuration page');
    expect(configLink).toHaveClass('bg-primary-600');
    expect(configLink).toHaveClass('text-white');
  });

  it('does not highlight Config when on /profiles route (without /config)', () => {
    renderWithRouter(<Sidebar />, { route: '/profiles' });

    const configLink = screen.getByLabelText('Navigate to Configuration page');
    expect(configLink).not.toHaveClass('bg-primary-600');
    expect(configLink).toHaveClass('text-slate-300');
  });
});
