/**
 * Integration tests for HomePage
 * Tests dashboard rendering and card interactions
 */

import { describe, it, expect } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import { renderWithProviders } from '../../tests/testUtils';
import userEvent from '@testing-library/user-event';
import { HomePage } from './HomePage';

describe('HomePage - Integration Tests', () => {
  const renderHomePage = () => {
    return renderWithProviders(<HomePage />, {
      wrapWithRouter: true,
      routerInitialEntries: ['/'],
    });
  };

  describe('Dashboard rendering', () => {
    it('renders dashboard heading', async () => {
      renderHomePage();

      await waitFor(() => {
        expect(screen.getByText('Dashboard')).toBeInTheDocument();
      });
    });

    it('renders all three card sections', async () => {
      renderHomePage();

      await waitFor(() => {
        expect(screen.getByText('Active Profile')).toBeInTheDocument();
        expect(screen.getByText(/Connected Devices/)).toBeInTheDocument();
        expect(screen.getByText('Quick Stats')).toBeInTheDocument();
      });
    });
  });

  describe('Active Profile Card', () => {
    it('displays active profile information', async () => {
      renderHomePage();

      await waitFor(() => {
        const activeProfileCard = screen.getByText('Active Profile').closest('div');
        expect(activeProfileCard).toBeTruthy();
      });
    });
  });

  describe('Device List Card', () => {
    it('displays connected devices', async () => {
      renderHomePage();

      await waitFor(() => {
        expect(screen.getByText(/Connected Devices/)).toBeInTheDocument();
      });
    });
  });

  describe('Quick Stats Card', () => {
    it('displays quick statistics', async () => {
      renderHomePage();

      await waitFor(() => {
        expect(screen.getByText('Quick Stats')).toBeInTheDocument();
      });
    });
  });

  describe('Responsive layout', () => {
    it('uses correct responsive spacing classes', async () => {
      const { container } = renderHomePage();
      const mainDiv = container.querySelector('.flex.flex-col');
      expect(mainDiv).toBeTruthy();
      expect(mainDiv?.classList.contains('gap-4')).toBe(true);
    });
  });
});
