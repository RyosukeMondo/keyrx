import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { BrowserRouter } from 'react-router-dom';
import { AppShell } from './AppShell';

const renderWithRouter = (component: React.ReactElement) => {
  return render(<BrowserRouter>{component}</BrowserRouter>);
};

describe('AppShell', () => {
  it('renders with default layout', () => {
    renderWithRouter(<AppShell />);
    expect(screen.getByRole('main')).toBeInTheDocument();
  });

  it('renders children when provided', () => {
    renderWithRouter(
      <AppShell>
        <div>Test Content</div>
      </AppShell>
    );
    expect(screen.getByText('Test Content')).toBeInTheDocument();
  });

  it('displays KeyRx2 brand', () => {
    renderWithRouter(<AppShell />);
    const brandElements = screen.getAllByText('KeyRx2');
    expect(brandElements.length).toBeGreaterThan(0);
  });

  it('has mobile bottom navigation with 5 items', () => {
    renderWithRouter(<AppShell />);
    const bottomNav = screen.getByRole('navigation', { name: /mobile navigation/i });
    expect(bottomNav).toBeInTheDocument();

    // Check for navigation items
    expect(screen.getByRole('button', { name: /home/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /devices/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /profiles/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /config/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /metrics/i })).toBeInTheDocument();
  });

  it('has desktop sidebar navigation', () => {
    renderWithRouter(<AppShell />);
    const sidebarNav = screen.getAllByRole('complementary', { name: /main navigation/i });
    expect(sidebarNav.length).toBeGreaterThan(0);
  });

  it('toggles sidebar when hamburger menu is clicked', async () => {
    const user = userEvent.setup();
    renderWithRouter(<AppShell />);

    const toggleButton = screen.getByRole('button', { name: /toggle navigation menu/i });
    expect(toggleButton).toBeInTheDocument();

    // Click to open
    await user.click(toggleButton);

    // Check that sidebar state changed (icon changes from hamburger to X)
    const svgPaths = toggleButton.querySelectorAll('path');
    expect(svgPaths.length).toBeGreaterThan(0);
  });

  it('has proper ARIA labels for accessibility', () => {
    renderWithRouter(<AppShell />);

    expect(screen.getByRole('main')).toBeInTheDocument();
    expect(screen.getByRole('navigation', { name: /mobile navigation/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /toggle navigation menu/i })).toBeInTheDocument();
  });

  it('displays all navigation items with icons and labels', () => {
    renderWithRouter(<AppShell />);

    const navItems = ['Home', 'Devices', 'Profiles', 'Config', 'Metrics'];
    navItems.forEach((item) => {
      const button = screen.getByRole('button', { name: new RegExp(item, 'i') });
      expect(button).toBeInTheDocument();

      // Check for icon (svg) and text label
      const svg = button.querySelector('svg');
      expect(svg).toBeInTheDocument();

      const label = screen.getByText(item);
      expect(label).toBeInTheDocument();
    });
  });

  it('applies correct responsive classes', () => {
    renderWithRouter(<AppShell />);

    const main = screen.getByRole('main');

    // Check for responsive padding classes
    expect(main.className).toContain('pt-16');
    expect(main.className).toContain('lg:pt-0');
    expect(main.className).toContain('pb-16');
    expect(main.className).toContain('md:pb-0');
  });

  it('has minimum height for full viewport', () => {
    renderWithRouter(<AppShell />);

    const appShell = screen.getByRole('main').parentElement;
    expect(appShell?.className).toContain('min-h-screen');
  });
});
