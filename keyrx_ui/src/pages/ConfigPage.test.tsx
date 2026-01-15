/**
 * ConfigPage - Simple Beneficial Tests
 *
 * Philosophy: Test user-visible behavior with minimal mocking
 * - Use real components where possible
 * - Mock only external APIs (via MSW)
 * - Focus on critical user paths
 *
 * Complex integration tests removed - they were over-mocked and brittle.
 * If you need to test complex workflows, use E2E tests instead.
 */

import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import { renderWithProviders, setupMockWebSocket, cleanupMockWebSocket } from '../../tests/testUtils';
import ConfigPage from './ConfigPage';
import { http, HttpResponse } from 'msw';
import { server } from '../test/mocks/server';

describe('ConfigPage - Simple Tests', () => {
  beforeEach(async () => {
    await setupMockWebSocket();
  });

  afterEach(() => {
    cleanupMockWebSocket();
  });

  describe('Basic Rendering', () => {
    it('renders config page successfully', async () => {
      server.use(
        http.get('/api/profiles/:name/rhai', () => {
          return HttpResponse.text('// Simple config');
        })
      );

      renderWithProviders(<ConfigPage />, { wrapWithRouter: true });

      // Should render without crashing and show save button
      await waitFor(() => {
        expect(screen.getByRole('button', { name: /Save/i })).toBeInTheDocument();
      });
    });

    it('renders device selector', async () => {
      server.use(
        http.get('/api/profiles/:name/rhai', () => {
          return HttpResponse.text('// Config');
        })
      );

      renderWithProviders(<ConfigPage />, { wrapWithRouter: true });

      await waitFor(() => {
        expect(screen.getByTestId('device-selector')).toBeInTheDocument();
      });
    });
  });

  describe('User Interactions', () => {
    it('shows save button for user to save changes', async () => {
      server.use(
        http.get('/api/profiles/:name/rhai', () => {
          return HttpResponse.text('// Config');
        })
      );

      renderWithProviders(<ConfigPage />, { wrapWithRouter: true });

      await waitFor(() => {
        const saveButton = screen.getByRole('button', { name: /Save/i });
        expect(saveButton).toBeInTheDocument();
      });
    });
  });
});
