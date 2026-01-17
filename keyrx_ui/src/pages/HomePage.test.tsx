import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { screen } from '@testing-library/react';
import {
  renderWithProviders,
  setupMockWebSocket,
  cleanupMockWebSocket,
} from '../../tests/testUtils';
import { BrowserRouter } from 'react-router-dom';
import { HomePage } from './HomePage';

const renderWithRouter = (component: React.ReactElement) => {
  return renderWithProviders(<BrowserRouter>{component}</BrowserRouter>);
};

describe('HomePage', () => {
  beforeEach(async () => {
    await setupMockWebSocket();
  });

  afterEach(() => {
    cleanupMockWebSocket();
  });
  it('renders dashboard heading', () => {
    renderWithRouter(<HomePage />);
    expect(screen.getByText('Dashboard')).toBeInTheDocument();
  });

  it.skip('renders all three card sections - SKIP: requires MSW API mocking', () => {
    renderWithRouter(<HomePage />);

    // Check for card headings
    expect(screen.getByText('Active Profile')).toBeInTheDocument();
    expect(screen.getByText(/Connected Devices/)).toBeInTheDocument();
    expect(screen.getByText('Quick Stats')).toBeInTheDocument();
  });

  it('uses correct responsive spacing classes', () => {
    const { container } = renderWithRouter(<HomePage />);
    const mainDiv = container.querySelector('.flex.flex-col');
    expect(mainDiv).toBeTruthy();
    // HomePage uses responsive gap classes (gap-4 md:gap-6 lg:gap-8)
    expect(mainDiv?.classList.contains('gap-4')).toBe(true);
  });

  it('renders heading with correct accessibility level', () => {
    renderWithRouter(<HomePage />);
    const heading = screen.getByRole('heading', { level: 1 });
    expect(heading).toHaveTextContent('Dashboard');
  });
});
