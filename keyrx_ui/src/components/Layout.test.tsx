import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { BrowserRouter } from 'react-router-dom';
import userEvent from '@testing-library/user-event';
import { Layout } from './Layout';

const renderWithRouter = (ui: React.ReactElement) => {
  return render(ui, { wrapper: BrowserRouter });
};

describe('Layout', () => {
  it('renders children correctly', () => {
    renderWithRouter(
      <Layout>
        <div>Test Content</div>
      </Layout>
    );

    expect(screen.getByText('Test Content')).toBeInTheDocument();
  });

  it('renders mobile header with brand name', () => {
    renderWithRouter(
      <Layout>
        <div>Content</div>
      </Layout>
    );

    const brandElements = screen.getAllByText('KeyRx2');
    expect(brandElements.length).toBeGreaterThan(0);
  });

  it('renders hamburger menu button on mobile', () => {
    renderWithRouter(
      <Layout>
        <div>Content</div>
      </Layout>
    );

    const menuButton = screen.getByLabelText('Toggle navigation menu');
    expect(menuButton).toBeInTheDocument();
  });

  it('toggles sidebar when hamburger button is clicked', async () => {
    const user = userEvent.setup();

    renderWithRouter(
      <Layout>
        <div>Content</div>
      </Layout>
    );

    const menuButton = screen.getByLabelText('Toggle navigation menu');
    expect(menuButton).toHaveAttribute('aria-expanded', 'false');

    await user.click(menuButton);
    expect(menuButton).toHaveAttribute('aria-expanded', 'true');

    await user.click(menuButton);
    expect(menuButton).toHaveAttribute('aria-expanded', 'false');
  });

  it('closes mobile sidebar when backdrop is clicked', async () => {
    const user = userEvent.setup();

    renderWithRouter(
      <Layout>
        <div>Content</div>
      </Layout>
    );

    const menuButton = screen.getByLabelText('Toggle navigation menu');

    // Open sidebar
    await user.click(menuButton);
    expect(menuButton).toHaveAttribute('aria-expanded', 'true');

    // Click backdrop
    const backdrop = document.querySelector('.bg-black\\/50');
    if (backdrop) {
      await user.click(backdrop as HTMLElement);
      expect(menuButton).toHaveAttribute('aria-expanded', 'false');
    }
  });

  it('renders BottomNav component', () => {
    renderWithRouter(
      <Layout>
        <div>Content</div>
      </Layout>
    );

    const bottomNav = screen.getByRole('navigation', {
      name: 'Mobile bottom navigation',
    });
    expect(bottomNav).toBeInTheDocument();
  });

  it('renders desktop sidebar with brand', () => {
    renderWithRouter(
      <Layout>
        <div>Content</div>
      </Layout>
    );

    // Should have brand name in desktop sidebar
    const brandElements = screen.getAllByText('KeyRx2');
    expect(brandElements.length).toBeGreaterThan(0);
  });

  it('has correct responsive padding for main content', () => {
    const { container } = renderWithRouter(
      <Layout>
        <div>Content</div>
      </Layout>
    );

    const main = container.querySelector('main');
    expect(main).toHaveClass('pt-16', 'md:pt-0'); // Top padding for mobile header
    expect(main).toHaveClass('pb-16', 'md:pb-0'); // Bottom padding for mobile nav
    expect(main).toHaveClass('md:ml-64'); // Left margin for desktop sidebar
  });

  it('desktop sidebar is visible on md+ screens', () => {
    const { container } = renderWithRouter(
      <Layout>
        <div>Content</div>
      </Layout>
    );

    const desktopSidebar = container.querySelector('.hidden.md\\:block');
    expect(desktopSidebar).toBeInTheDocument();
    expect(desktopSidebar).toHaveClass('w-64');
  });

  it('mobile header is hidden on md+ screens', () => {
    const { container } = renderWithRouter(
      <Layout>
        <div>Content</div>
      </Layout>
    );

    const mobileHeader = container.querySelector('header.md\\:hidden');
    expect(mobileHeader).toBeInTheDocument();
  });

  it('has correct z-index stacking', () => {
    const { container } = renderWithRouter(
      <Layout>
        <div>Content</div>
      </Layout>
    );

    const mobileHeader = container.querySelector('header.md\\:hidden');
    expect(mobileHeader).toHaveClass('z-40');

    const desktopSidebar = container.querySelector('.hidden.md\\:block');
    expect(desktopSidebar).toHaveClass('z-30');
  });

  it('hamburger icon changes when sidebar is open', async () => {
    const user = userEvent.setup();

    renderWithRouter(
      <Layout>
        <div>Content</div>
      </Layout>
    );

    const menuButton = screen.getByLabelText('Toggle navigation menu');
    const svg = menuButton.querySelector('svg');

    // Initially shows hamburger (3 lines)
    let path = svg?.querySelector('path');
    expect(path).toHaveAttribute('d', 'M4 6h16M4 12h16M4 18h16');

    // After click, shows X
    await user.click(menuButton);
    path = svg?.querySelector('path');
    expect(path).toHaveAttribute('d', 'M6 18L18 6M6 6l12 12');
  });

  it('has accessible focus management', () => {
    renderWithRouter(
      <Layout>
        <div>Content</div>
      </Layout>
    );

    const menuButton = screen.getByLabelText('Toggle navigation menu');
    expect(menuButton).toHaveClass('focus:outline', 'focus:outline-2', 'focus:outline-primary-500');
  });

  it('has minimum height for full viewport', () => {
    const { container } = renderWithRouter(
      <Layout>
        <div>Content</div>
      </Layout>
    );

    const root = container.querySelector('.min-h-screen');
    expect(root).toBeInTheDocument();
  });

  it('uses dark theme colors', () => {
    const { container } = renderWithRouter(
      <Layout>
        <div>Content</div>
      </Layout>
    );

    const root = container.querySelector('.bg-slate-900');
    expect(root).toBeInTheDocument();
    expect(root).toHaveClass('text-slate-100');
  });
});
