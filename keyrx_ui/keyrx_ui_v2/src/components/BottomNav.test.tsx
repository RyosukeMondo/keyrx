import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { BrowserRouter, MemoryRouter } from 'react-router-dom';
import { BottomNav } from './BottomNav';

describe('BottomNav', () => {
  it('renders all navigation items', () => {
    render(
      <BrowserRouter>
        <BottomNav />
      </BrowserRouter>
    );

    expect(screen.getByText('Home')).toBeInTheDocument();
    expect(screen.getByText('Devices')).toBeInTheDocument();
    expect(screen.getByText('Profiles')).toBeInTheDocument();
    expect(screen.getByText('Config')).toBeInTheDocument();
    expect(screen.getByText('Metrics')).toBeInTheDocument();
  });

  it('has correct accessibility attributes', () => {
    render(
      <BrowserRouter>
        <BottomNav />
      </BrowserRouter>
    );

    const nav = screen.getByRole('navigation', {
      name: 'Mobile bottom navigation',
    });
    expect(nav).toBeInTheDocument();

    const homeLink = screen.getByLabelText('Navigate to Home page');
    expect(homeLink).toBeInTheDocument();
  });

  it('highlights active route', () => {
    render(
      <MemoryRouter initialEntries={['/devices']}>
        <BottomNav />
      </MemoryRouter>
    );

    const devicesLink = screen.getByLabelText('Navigate to Devices page');
    expect(devicesLink).toHaveClass('text-primary-500');
  });

  it('applies custom className', () => {
    const { container } = render(
      <BrowserRouter>
        <BottomNav className="custom-class" />
      </BrowserRouter>
    );

    const nav = container.querySelector('nav');
    expect(nav).toHaveClass('custom-class');
  });

  it('has touch targets >= 44px (h-16 = 64px)', () => {
    render(
      <BrowserRouter>
        <BottomNav />
      </BrowserRouter>
    );

    const homeLink = screen.getByLabelText('Navigate to Home page');
    expect(homeLink).toHaveClass('h-16');
  });

  it('is fixed at bottom with correct z-index', () => {
    const { container } = render(
      <BrowserRouter>
        <BottomNav />
      </BrowserRouter>
    );

    const nav = container.querySelector('nav');
    expect(nav).toHaveClass('fixed', 'bottom-0', 'left-0', 'right-0');
    expect(nav).toHaveStyle({ zIndex: 'var(--z-fixed)' });
  });

  it('is hidden on medium screens and above (md:hidden)', () => {
    const { container } = render(
      <BrowserRouter>
        <BottomNav />
      </BrowserRouter>
    );

    const nav = container.querySelector('nav');
    expect(nav).toHaveClass('md:hidden');
  });

  it('fills icon for active state', () => {
    const { container } = render(
      <MemoryRouter initialEntries={['/profiles']}>
        <BottomNav />
      </MemoryRouter>
    );

    const profilesLink = screen.getByLabelText('Navigate to Profiles page');
    const icon = profilesLink.querySelector('svg');
    expect(icon).toHaveClass('fill-current');
  });

  it('makes active label semibold', () => {
    render(
      <MemoryRouter initialEntries={['/config']}>
        <BottomNav />
      </MemoryRouter>
    );

    const configText = screen.getByText('Config');
    expect(configText).toHaveClass('font-semibold');
  });

  it('has focus visible styles', () => {
    render(
      <BrowserRouter>
        <BottomNav />
      </BrowserRouter>
    );

    const homeLink = screen.getByLabelText('Navigate to Home page');
    expect(homeLink).toHaveClass(
      'focus:outline',
      'focus:outline-2',
      'focus:outline-primary-500'
    );
  });
});
