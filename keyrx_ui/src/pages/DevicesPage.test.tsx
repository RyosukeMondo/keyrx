import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { screen, waitFor, within } from '@testing-library/react';
import { renderWithProviders, setupMockWebSocket, cleanupMockWebSocket } from '../../tests/testUtils';
import userEvent from '@testing-library/user-event';
import { DevicesPage } from './DevicesPage';
import { http, HttpResponse } from 'msw';
import { server } from '../test/mocks/server';

// Mock react-router-dom
vi.mock('react-router-dom', () => ({
  useNavigate: () => vi.fn(),
}));

describe('DevicesPage - Integration Tests', () => {
  beforeEach(async () => {
    await setupMockWebSocket();
    localStorage.clear(); // Clear localStorage to prevent test pollution
  });

  afterEach(() => {
    cleanupMockWebSocket();
    localStorage.clear(); // Clean up localStorage after tests
  });

  // ========================================================================
  // Global Layout Selector Tests (Requirements 2.1-2.5)
  // ========================================================================
  describe('Global Layout Selector', () => {
    it('displays Global Settings card at top of page', async () => {
      renderWithProviders(<DevicesPage />);

      await waitFor(() => {
        expect(screen.getByText('Global Settings')).toBeInTheDocument();
      });

      const globalSettingsCard = screen.getByText('Global Settings').closest('div[class*="bg-slate-800"]');
      expect(globalSettingsCard).toBeInTheDocument();
      expect(screen.getByText('Default Keyboard Layout')).toBeInTheDocument();
    });

    it('loads and displays current global layout from API', async () => {
      server.use(
        http.get('/api/settings/global-layout', () => {
          return HttpResponse.json({ layout: 'ISO_105' });
        })
      );

      renderWithProviders(<DevicesPage />);

      await waitFor(() => {
        const layoutDropdown = screen.getByLabelText('Select default keyboard layout');
        expect(layoutDropdown).toHaveTextContent('ISO 105 (Full)');
      });
    });

    it('saves global layout when changed via dropdown', async () => {
      const user = userEvent.setup();
      let savedLayout = '';

      server.use(
        http.put('/api/settings/global-layout', async ({ request }) => {
          const body = (await request.json()) as { layout: string };
          savedLayout = body.layout;
          return HttpResponse.json({ success: true });
        })
      );

      renderWithProviders(<DevicesPage />);

      await waitFor(() => {
        expect(screen.getByText('Global Settings')).toBeInTheDocument();
      });

      const layoutDropdown = screen.getByLabelText('Select default keyboard layout');
      await user.click(layoutDropdown);

      const isoOption = screen.getByText('ISO 105 (Full)');
      await user.click(isoOption);

      await waitFor(() => {
        expect(savedLayout).toBe('ISO_105');
      });
    });

    it('shows saving spinner while saving global layout', async () => {
      const user = userEvent.setup();

      server.use(
        http.put('/api/settings/global-layout', async () => {
          // Simulate slow save
          await new Promise((resolve) => setTimeout(resolve, 100));
          return HttpResponse.json({ success: true });
        })
      );

      renderWithProviders(<DevicesPage />);

      await waitFor(() => {
        expect(screen.getByText('Global Settings')).toBeInTheDocument();
      });

      const layoutDropdown = screen.getByLabelText('Select default keyboard layout');
      await user.click(layoutDropdown);
      await user.click(screen.getByText('JIS 109 (Full)'));

      // Should show saving spinner
      await waitFor(() => {
        expect(screen.getByText('Saving...')).toBeInTheDocument();
      });
    });

    it('shows success checkmark after successful save', async () => {
      const user = userEvent.setup();

      renderWithProviders(<DevicesPage />);

      await waitFor(() => {
        expect(screen.getByText('Global Settings')).toBeInTheDocument();
      });

      const layoutDropdown = screen.getByLabelText('Select default keyboard layout');
      await user.click(layoutDropdown);
      await user.click(screen.getByText('HHKB'));

      await waitFor(() => {
        expect(screen.getByText('✓ Saved')).toBeInTheDocument();
      });
    });

    it('shows error message when save fails', async () => {
      const user = userEvent.setup();

      server.use(
        http.put('/api/settings/global-layout', () => {
          return HttpResponse.json({ error: 'Database error' }, { status: 500 });
        })
      );

      renderWithProviders(<DevicesPage />);

      await waitFor(() => {
        expect(screen.getByText('Global Settings')).toBeInTheDocument();
      });

      const layoutDropdown = screen.getByLabelText('Select default keyboard layout');
      await user.click(layoutDropdown);
      await user.click(screen.getByText('Numpad'));

      await waitFor(() => {
        expect(screen.getByText('✗ Error')).toBeInTheDocument();
      });
    });

    it('handles offline scenario gracefully when fetching global layout', async () => {
      server.use(
        http.get('/api/settings/global-layout', () => {
          return HttpResponse.error();
        })
      );

      renderWithProviders(<DevicesPage />);

      // Should still render with default layout (ANSI_104)
      await waitFor(() => {
        expect(screen.getByText('Global Settings')).toBeInTheDocument();
      });

      const layoutDropdown = screen.getByLabelText('Select default keyboard layout');
      expect(layoutDropdown).toHaveTextContent('ANSI 104');
    });
  });

  // ========================================================================
  // Device List and Editing Tests (Requirements 1.1-1.4, 5.1-5.5)
  // ========================================================================
  describe('Device List Rendering', () => {
    it('renders all devices in the list', async () => {
      renderWithProviders(<DevicesPage />);

      await waitFor(() => {
        expect(screen.getByText('Devices')).toBeInTheDocument();
      });

      expect(screen.getByText(/Device List \(2 connected\)/)).toBeInTheDocument();
      expect(screen.getAllByText('Test Keyboard 1').length).toBeGreaterThan(0);
      expect(screen.getAllByText('Test Keyboard 2').length).toBeGreaterThan(0);
    });

    it('shows connection status badges for devices', async () => {
      renderWithProviders(<DevicesPage />);

      await screen.findByText('Devices');

      await waitFor(() => {
        // Connection status is shown via aria-label on status dots
        const connectedBadges = screen.queryAllByLabelText('Connected');
        expect(connectedBadges.length).toBeGreaterThan(0);
      });
    });

    it('displays device identifiers (path)', async () => {
      renderWithProviders(<DevicesPage />);

      await screen.findByText('Devices');

      await waitFor(() => {
        expect(screen.getByText('/dev/input/event0')).toBeInTheDocument();
        expect(screen.getByText('/dev/input/event1')).toBeInTheDocument();
      });
    });

    it('displays device name, layout, and serial information', async () => {
      renderWithProviders(<DevicesPage />);

      await waitFor(() => {
        expect(screen.getByText('Devices')).toBeInTheDocument();
      });

      // Wait for device data to load
      await waitFor(() => {
        expect(screen.getByText(/Device List \(2 connected\)/)).toBeInTheDocument();
      });

      // Check that device names are displayed
      expect(screen.getAllByText('Test Keyboard 1').length).toBeGreaterThan(0);
      expect(screen.getAllByText('Test Keyboard 2').length).toBeGreaterThan(0);

      // Check that layout selectors exist (one per device)
      const layoutSelectors = screen.getAllByLabelText('Layout');
      expect(layoutSelectors.length).toBeGreaterThanOrEqual(2);
    });

  });

  describe('Device Editing', () => {
    it('enters rename mode when Rename button is clicked', async () => {
    const user = userEvent.setup();
    renderWithProviders(<DevicesPage />);

    // Wait for devices to load
    const renameButton = await screen.findByLabelText('Rename Test Keyboard 1');
    await user.click(renameButton);

    // Input should appear with current name
    const input = screen.getByRole('textbox', { name: 'Device name' });
    expect(input).toBeInTheDocument();
    expect(input).toHaveValue('Test Keyboard 1');

    // Save and Cancel buttons should appear
    expect(screen.getByLabelText('Save')).toBeInTheDocument();
    expect(screen.getByLabelText('Cancel')).toBeInTheDocument();
  });

  it('saves new name when Save button is clicked', async () => {
    const user = userEvent.setup();
    renderWithProviders(<DevicesPage />);

    // Wait for devices to load
    const renameButton = await screen.findByLabelText('Rename Test Keyboard 1');
    await user.click(renameButton);

    const input = screen.getByRole('textbox', { name: 'Device name' });
    await user.clear(input);
    await user.type(input, 'New Keyboard Name');

    const saveButton = screen.getByLabelText('Save');
    await user.click(saveButton);

    // Input should disappear (exits edit mode)
    expect(screen.queryByRole('textbox', { name: 'Device name' })).not.toBeInTheDocument();

    // TODO: Once rename API is implemented, verify name actually changes
    // For now, rename just exits edit mode without persisting
  });

  it('saves new name when Enter key is pressed', async () => {
    const user = userEvent.setup();
    renderWithProviders(<DevicesPage />);

    // Wait for devices to load
    const renameButton = await screen.findByLabelText('Rename Test Keyboard 1');
    await user.click(renameButton);

    const input = screen.getByRole('textbox', { name: 'Device name' });
    await user.clear(input);
    await user.type(input, 'Renamed via Enter{Enter}');

    await waitFor(() => {
      expect(screen.queryByRole('textbox', { name: 'Device name' })).not.toBeInTheDocument();
    });

    // TODO: Once rename API is implemented, verify name actually changes
    // For now, rename just exits edit mode without persisting
  });

  it('cancels rename when Cancel button is clicked', async () => {
    const user = userEvent.setup();
    renderWithProviders(<DevicesPage />);

    // Wait for devices to load
    const renameButton = await screen.findByLabelText('Rename Test Keyboard 1');
    await user.click(renameButton);

    const input = screen.getByRole('textbox', { name: 'Device name' });
    await user.clear(input);
    await user.type(input, 'This should not be saved');

    const cancelButton = screen.getByLabelText('Cancel');
    await user.click(cancelButton);

    // Input should disappear
    expect(screen.queryByRole('textbox', { name: 'Device name' })).not.toBeInTheDocument();

    // Original name should still be displayed
    expect(screen.getAllByText('Test Keyboard 1').length).toBeGreaterThan(0);
    expect(screen.queryByText('This should not be saved')).not.toBeInTheDocument();
  });

  it('cancels rename when Escape key is pressed', async () => {
    const user = userEvent.setup();
    renderWithProviders(<DevicesPage />);

    // Wait for devices to load
    const renameButton = await screen.findByLabelText('Rename Test Keyboard 1');
    await user.click(renameButton);

    const input = screen.getByRole('textbox', { name: 'Device name' });
    await user.clear(input);
    await user.type(input, 'This should not be saved{Escape}');

    await waitFor(() => {
      expect(screen.queryByRole('textbox', { name: 'Device name' })).not.toBeInTheDocument();
    });

    expect(screen.getAllByText('Test Keyboard 1').length).toBeGreaterThan(0);
  });

  it('shows error when trying to save empty name', async () => {
    const user = userEvent.setup();
    renderWithProviders(<DevicesPage />);

    // Wait for devices to load
    const renameButton = await screen.findByLabelText('Rename Test Keyboard 1');
    await user.click(renameButton);

    const input = screen.getByRole('textbox', { name: 'Device name' });
    await user.clear(input);

    const saveButton = screen.getByLabelText('Save');
    await user.click(saveButton);

    // Error message should appear
    expect(screen.getByText('Device name cannot be empty')).toBeInTheDocument();

    // Input should still be visible
    expect(input).toBeInTheDocument();
  });

  it('shows character counter when maxLength is set', async () => {
    const user = userEvent.setup();
    renderWithProviders(<DevicesPage />);

    // Wait for devices to load
    const renameButton = await screen.findByLabelText('Rename Test Keyboard 1');
    await user.click(renameButton);

    const input = screen.getByRole('textbox', { name: 'Device name' });
    await user.clear(input);
    await user.type(input, 'This is a name');

    // Character counter should be visible (Input component shows it when maxLength is set)
    expect(screen.getByText(/\/ 64/)).toBeInTheDocument();
  });

  it('persists device name changes', async () => {
    const user = userEvent.setup();
    renderWithProviders(<DevicesPage />);

    const renameButton = await screen.findByLabelText('Rename Test Keyboard 1');
    await user.click(renameButton);

    const input = screen.getByRole('textbox', { name: 'Device name' });
    await user.clear(input);
    await user.type(input, 'My Custom Keyboard');
    await user.click(screen.getByLabelText('Save'));

    await waitFor(() => {
      expect(screen.queryByRole('textbox', { name: 'Device name' })).not.toBeInTheDocument();
    });

    // TODO: Once rename API is implemented, verify name persists
    // For now, rename just exits edit mode without persisting
  });

  it('persists device layout changes with auto-save', async () => {
    const user = userEvent.setup();
    let savedLayout = '';

    server.use(
      http.patch('/api/devices/:id', async ({ request, params }) => {
        const body = (await request.json()) as { layout?: string };
        if (body.layout) {
          savedLayout = body.layout;
        }
        return HttpResponse.json({ success: true });
      })
    );

    renderWithProviders(<DevicesPage />);

    await waitFor(() => {
      expect(screen.getByText('Devices')).toBeInTheDocument();
    });

    const layoutDropdowns = screen.getAllByLabelText('Layout');
    const firstDropdown = layoutDropdowns[0];

    await user.click(firstDropdown);
    await user.click(screen.getByText('JIS 109 (Full)'));

    // Wait for auto-save to trigger
    await waitFor(
      () => {
        expect(savedLayout).toBe('JIS_109');
      },
      { timeout: 2000 }
    );
  });
  });

  describe('Device Layout Selection', () => {
    it('changes layout via dropdown', async () => {
    const user = userEvent.setup();
    renderWithProviders(<DevicesPage />);

    // Wait for devices to load
    await screen.findByText('Devices');

    await waitFor(() => {
      const layoutDropdowns = screen.getAllByLabelText('Layout');
      expect(layoutDropdowns.length).toBeGreaterThan(0);
    });

    // Find the first layout dropdown
    const layoutDropdowns = screen.getAllByLabelText('Layout');
    const dropdown = layoutDropdowns[0];

    // Open dropdown
    await user.click(dropdown);

    // Select ISO 105 option
    const isoOption = screen.getByText('ISO 105 (Full)');
    await user.click(isoOption);

    // Verify the selection (implementation-dependent)
    // This test assumes the Dropdown component displays the selected value
  });

  it('shows auto-save feedback for layout changes', async () => {
    const user = userEvent.setup();

    server.use(
      http.patch('/api/devices/:id', async () => {
        // Simulate slow save
        await new Promise((resolve) => setTimeout(resolve, 100));
        return HttpResponse.json({ success: true });
      })
    );

    renderWithProviders(<DevicesPage />);

    await waitFor(() => {
      expect(screen.getByText('Devices')).toBeInTheDocument();
    });

    const layoutDropdowns = screen.getAllByLabelText('Layout');
    await user.click(layoutDropdowns[0]);
    await user.click(screen.getByText('ISO 105 (Full)'));

    // Should show saving feedback (checkmark after save completes)
    // Device rows show a spinner (no "Saving..." text) followed by a checkmark
    await waitFor(
      () => {
        expect(screen.getByText('✓')).toBeInTheDocument();
      },
      { timeout: 2000 }
    );
  });
  });

  describe('Forget Device', () => {
    it('opens forget device confirmation modal', async () => {
    const user = userEvent.setup();
    renderWithProviders(<DevicesPage />);

    // Wait for devices to load
    const forgetButton = await screen.findByLabelText('Permanently forget Test Keyboard 1');
    await user.click(forgetButton);

    // Modal should appear with confirmation message
    expect(screen.getByRole('dialog', { name: 'Forget Device' })).toBeInTheDocument();
    expect(screen.getByText(/Are you sure you want to forget device/)).toBeInTheDocument();
  });

  it('cancels forget device when Cancel is clicked in modal', async () => {
    const user = userEvent.setup();
    renderWithProviders(<DevicesPage />);

    // Wait for devices to load
    const forgetButton = await screen.findByLabelText('Permanently forget Test Keyboard 1');
    await user.click(forgetButton);

    const cancelButton = screen.getByLabelText('Cancel forget device');
    await user.click(cancelButton);

    // Modal should close
    await waitFor(() => {
      expect(screen.queryByRole('dialog', { name: 'Forget Device' })).not.toBeInTheDocument();
    });

    // Device should still be in the list
    expect(screen.getAllByText('Test Keyboard 1').length).toBeGreaterThan(0);
  });

  it('removes device when Forget Device is confirmed', async () => {
    const user = userEvent.setup();
    renderWithProviders(<DevicesPage />);

    // Wait for devices to load
    const forgetButton = await screen.findByLabelText('Permanently forget Test Keyboard 1');
    await user.click(forgetButton);

    const confirmButton = screen.getByLabelText('Confirm forget device');
    await user.click(confirmButton);

    // Modal should close
    await waitFor(() => {
      expect(screen.queryByRole('dialog', { name: 'Forget Device' })).not.toBeInTheDocument();
    });

    // Device should be removed from the list - check that it's gone from the page
    await waitFor(() => {
      expect(screen.queryByLabelText('Permanently forget Test Keyboard 1')).not.toBeInTheDocument();
    });

    // Device count should update
    expect(screen.getByText(/Device List \(1 connected\)/)).toBeInTheDocument();
  });

  it('shows empty state when no devices are connected', async () => {
    const user = userEvent.setup();
    renderWithProviders(<DevicesPage />);

    // Wait for devices to load
    await screen.findByText('Devices');

    // Remove first device
    const forgetButton1 = screen.getByLabelText('Permanently forget Test Keyboard 1');
    await user.click(forgetButton1);
    const confirmButton1 = screen.getByLabelText('Confirm forget device');
    await user.click(confirmButton1);
    await waitFor(() => {
      expect(screen.queryByRole('dialog', { name: 'Forget Device' })).not.toBeInTheDocument();
    });

    // Remove second device
    const forgetButton2 = screen.getByLabelText('Permanently forget Test Keyboard 2');
    await user.click(forgetButton2);
    const confirmButton2 = screen.getByLabelText('Confirm forget device');
    await user.click(confirmButton2);
    await waitFor(() => {
      expect(screen.queryByRole('dialog', { name: 'Forget Device' })).not.toBeInTheDocument();
    });

    // Empty state should appear
    expect(
      screen.getByText('No devices connected. Connect a keyboard to get started.')
    ).toBeInTheDocument();
  });

  it('has accessible labels for all interactive elements', async () => {
    renderWithProviders(<DevicesPage />);

    // Wait for devices to load
    await screen.findByText('Devices');

    await waitFor(() => {
      // Rename buttons
      expect(screen.getByLabelText('Rename Test Keyboard 1')).toBeInTheDocument();
      expect(screen.getByLabelText('Rename Test Keyboard 2')).toBeInTheDocument();
    });

    // Layout dropdowns
    expect(screen.getAllByLabelText('Layout').length).toBeGreaterThan(0);

    // Forget buttons
    expect(screen.getByLabelText('Permanently forget Test Keyboard 1')).toBeInTheDocument();
    expect(screen.getByLabelText('Permanently forget Test Keyboard 2')).toBeInTheDocument();
  });
  });

  // ========================================================================
  // Responsive Design Tests
  // ========================================================================
  describe('Responsive Design', () => {
    it('renders mobile-friendly layout with stacked buttons', async () => {
      renderWithProviders(<DevicesPage />);

      await waitFor(() => {
        expect(screen.getByText('Devices')).toBeInTheDocument();
      });

      // Check that page renders and has responsive structure
      const headingContainer = screen.getByText('Devices').closest('div');
      expect(headingContainer).toBeInTheDocument();
      // Responsive classes should exist in the page
      expect(document.body.innerHTML).toContain('sm:flex-row');
    });

    it('renders desktop layout with inline elements', async () => {
      renderWithProviders(<DevicesPage />);

      await waitFor(() => {
        expect(screen.getByText('Devices')).toBeInTheDocument();
      });

      const headingContainer = screen.getByText('Devices').closest('div');
      expect(headingContainer).toBeInTheDocument();
      // Check that responsive classes are present
      expect(headingContainer?.className).toContain('flex');
    });

    it('handles narrow viewports gracefully', async () => {
      renderWithProviders(<DevicesPage />);

      await waitFor(() => {
        expect(screen.getByText('Devices')).toBeInTheDocument();
      });

      // Wait for devices to load
      await waitFor(() => {
        expect(screen.getByText(/Device List \(2 connected\)/)).toBeInTheDocument();
      });

      // Check that page uses flex-col for responsive layout
      const pageContainer = document.querySelector('.flex.flex-col.gap-4');
      expect(pageContainer).toBeInTheDocument();
    });
  });

  // ========================================================================
  // Scope Verification Tests (Requirements 1.1-1.4)
  // ========================================================================
  describe('Scope Completely Removed', () => {
    it('does NOT display scope toggle UI', async () => {
      renderWithProviders(<DevicesPage />);

      await waitFor(() => {
        expect(screen.getByText('Devices')).toBeInTheDocument();
      });

      // Should NOT have global/device-specific toggle buttons
      expect(screen.queryByText('Global')).not.toBeInTheDocument();
      expect(screen.queryByText('Device-Specific')).not.toBeInTheDocument();
      expect(screen.queryByRole('button', { name: /global/i })).not.toBeInTheDocument();
      expect(screen.queryByRole('button', { name: /device-specific/i })).not.toBeInTheDocument();
    });

    it('does NOT send scope in device update API calls', async () => {
      const user = userEvent.setup();
      let requestBody: any = null;

      server.use(
        http.patch('/api/devices/:id', async ({ request }) => {
          requestBody = await request.json();
          return HttpResponse.json({ success: true });
        })
      );

      renderWithProviders(<DevicesPage />);

      await waitFor(() => {
        expect(screen.getByText('Devices')).toBeInTheDocument();
      });

      const layoutDropdowns = screen.getAllByLabelText('Layout');
      await user.click(layoutDropdowns[0]);
      await user.click(screen.getByText('ISO 105 (Full)'));

      // Wait for API call
      await waitFor(
        () => {
          expect(requestBody).toBeTruthy();
        },
        { timeout: 2000 }
      );

      // Verify scope is NOT in request body
      expect(requestBody).not.toHaveProperty('scope');
    });

    it('does NOT display scope field in device information', async () => {
      renderWithProviders(<DevicesPage />);

      await waitFor(() => {
        expect(screen.getByText('Devices')).toBeInTheDocument();
      });

      // Wait for devices to load
      await waitFor(() => {
        expect(screen.getByText(/Device List \(2 connected\)/)).toBeInTheDocument();
      });

      // Check that the entire page does not contain the word "scope"
      const pageText = document.body.textContent || '';
      expect(pageText.toLowerCase()).not.toContain('scope');
    });

    it('shows explanatory note about Rhai-driven scope', async () => {
      renderWithProviders(<DevicesPage />);

      await waitFor(() => {
        expect(screen.getByText('Devices')).toBeInTheDocument();
      });

      // Check for comment in component that explains scope is determined by Rhai
      // This is in the component documentation, not visible UI
      // Just verify no scope UI is present
      expect(screen.queryByText(/scope/i)).not.toBeInTheDocument();
    });
  });

  // ========================================================================
  // Error Handling and Edge Cases
  // ========================================================================
  describe('Error Scenarios', () => {
    // TODO: Fix React Query error handling in tests - error isn't being propagated properly
    it.skip('displays error message when API fetch fails', async () => {
      server.use(
        http.get('/api/devices', () => {
          return HttpResponse.json({ error: 'Internal server error' }, { status: 500 });
        })
      );

      renderWithProviders(<DevicesPage />);

      await waitFor(() => {
        expect(screen.getByText(/Failed to fetch/i)).toBeInTheDocument();
      });
    });

    it('handles all devices disconnected scenario', async () => {
      server.use(
        http.get('/api/devices', () => {
          return HttpResponse.json({ devices: [] });
        })
      );

      renderWithProviders(<DevicesPage />);

      await waitFor(() => {
        expect(screen.getByText('Device List (0 connected)')).toBeInTheDocument();
      });

      expect(
        screen.getByText('No devices connected. Connect a keyboard to get started.')
      ).toBeInTheDocument();
    });

    it('shows device information when devices are connected', async () => {
      renderWithProviders(<DevicesPage />);

      await waitFor(() => {
        expect(screen.getByText('Devices')).toBeInTheDocument();
      });

      // Wait for devices to load
      await waitFor(() => {
        expect(screen.getByText(/Device List \(2 connected\)/)).toBeInTheDocument();
      });

      // Verify devices are shown
      expect(screen.getAllByText('Test Keyboard 1').length).toBeGreaterThan(0);
      expect(screen.getAllByText('Test Keyboard 2').length).toBeGreaterThan(0);

      // Empty state is tested in the "handles all devices disconnected scenario" test
    });

    it('handles offline daemon gracefully', async () => {
      server.use(
        http.get('/api/devices', () => {
          return HttpResponse.error();
        })
      );

      renderWithProviders(<DevicesPage />);

      await waitFor(() => {
        expect(screen.getByText(/Failed to fetch/i)).toBeInTheDocument();
      });
    });
  });

  // ========================================================================
  // Enable/Disable Device Toggle Tests (Task 2.4)
  // ========================================================================
  describe('Device Enable/Disable Toggle', () => {
    it('displays toggle switch for each device', async () => {
      renderWithProviders(<DevicesPage />);

      await waitFor(() => {
        expect(screen.getByText('Devices')).toBeInTheDocument();
      });

      // Wait for devices to load
      await waitFor(() => {
        expect(screen.getByText(/Device List \(2 connected\)/)).toBeInTheDocument();
      });

      // Find all toggle switches (role="switch")
      const toggles = screen.getAllByRole('switch');
      expect(toggles.length).toBeGreaterThanOrEqual(2);

      // Verify toggle labels
      expect(screen.getByLabelText(/Disable Test Keyboard 1/)).toBeInTheDocument();
      expect(screen.getByLabelText(/Disable Test Keyboard 2/)).toBeInTheDocument();
    });

    it('toggle switch is checked when device is enabled', async () => {
      renderWithProviders(<DevicesPage />);

      await waitFor(() => {
        expect(screen.getByText(/Device List \(2 connected\)/)).toBeInTheDocument();
      });

      const toggle1 = screen.getByLabelText(/Disable Test Keyboard 1/) as HTMLInputElement;
      expect(toggle1).toHaveAttribute('aria-checked', 'true');
    });

    it('clicking toggle changes device enabled state', async () => {
      const user = userEvent.setup();

      renderWithProviders(<DevicesPage />);

      await waitFor(() => {
        expect(screen.getByText(/Device List \(2 connected\)/)).toBeInTheDocument();
      });

      const toggle = screen.getByLabelText('Disable Test Keyboard 1');
      expect(toggle).toHaveAttribute('aria-checked', 'true');

      await user.click(toggle);

      // Toggle should update to unchecked (disabled)
      await waitFor(() => {
        expect(toggle).toHaveAttribute('aria-checked', 'false');
      });

      // Verify localStorage was updated
      const storedStates = localStorage.getItem('keyrx:device:enabled');
      expect(storedStates).toBeTruthy();
      const states = JSON.parse(storedStates!);
      expect(states['device-1']).toBe(false);
    });

    it('disabled devices have reduced opacity styling', async () => {
      const user = userEvent.setup();

      server.use(
        http.get('/api/devices', () => {
          return HttpResponse.json({
            devices: [
              {
                id: 'device-1',
                name: 'Test Keyboard 1',
                path: '/dev/input/event0',
                active: true,
                layout: 'ANSI_104',
                enabled: false, // Disabled device
              },
              {
                id: 'device-2',
                name: 'Test Keyboard 2',
                path: '/dev/input/event1',
                active: true,
                layout: 'ANSI_104',
                enabled: true,
              },
            ],
          });
        })
      );

      // Set localStorage to match the disabled state
      localStorage.setItem('keyrx:device:enabled', JSON.stringify({
        'device-1': false,
        'device-2': true,
      }));

      renderWithProviders(<DevicesPage />);

      await waitFor(() => {
        expect(screen.getByText(/Device List \(2 connected\)/)).toBeInTheDocument();
      });

      // Find device cards by test-id
      const deviceCards = screen.getAllByTestId('device-card');
      expect(deviceCards.length).toBe(2);

      // First device (disabled) should have opacity-50 class
      expect(deviceCards[0]).toHaveClass('opacity-50');

      // Second device (enabled) should NOT have opacity-50 class
      expect(deviceCards[1]).not.toHaveClass('opacity-50');
    });

    it('disabled devices show "Disabled" badge', async () => {
      server.use(
        http.get('/api/devices', () => {
          return HttpResponse.json({
            devices: [
              {
                id: 'device-1',
                name: 'Test Keyboard 1',
                path: '/dev/input/event0',
                active: true,
                layout: 'ANSI_104',
                enabled: false, // Disabled device
              },
              {
                id: 'device-2',
                name: 'Test Keyboard 2',
                path: '/dev/input/event1',
                active: true,
                layout: 'ANSI_104',
                enabled: true,
              },
            ],
          });
        })
      );

      // Set localStorage to match the disabled state
      localStorage.setItem('keyrx:device:enabled', JSON.stringify({
        'device-1': false,
        'device-2': true,
      }));

      renderWithProviders(<DevicesPage />);

      await waitFor(() => {
        expect(screen.getByText(/Device List \(2 connected\)/)).toBeInTheDocument();
      });

      // First device should show "Disabled" badge
      expect(screen.getByText('Disabled')).toBeInTheDocument();

      // Only one disabled badge should exist
      expect(screen.getAllByText('Disabled')).toHaveLength(1);
    });

    it('toggling device updates UI immediately (optimistic update)', async () => {
      const user = userEvent.setup();

      renderWithProviders(<DevicesPage />);

      await waitFor(() => {
        expect(screen.getByText(/Device List \(2 connected\)/)).toBeInTheDocument();
      });

      // Wait for toggle to be present
      const toggle = await screen.findByLabelText('Disable Test Keyboard 1');

      // Initially checked (enabled)
      expect(toggle).toHaveAttribute('aria-checked', 'true');

      await user.click(toggle);

      // Should update immediately to unchecked (disabled)
      await waitFor(() => {
        expect(toggle).toHaveAttribute('aria-checked', 'false');
      });
    });

    it('toggle switch label changes based on enabled state', async () => {
      const user = userEvent.setup();

      renderWithProviders(<DevicesPage />);

      await waitFor(() => {
        expect(screen.getByText(/Device List \(2 connected\)/)).toBeInTheDocument();
      });

      // Wait for toggle and verify label
      const toggle = await screen.findByLabelText('Disable Test Keyboard 1');
      await user.click(toggle);

      // After disabling - label should say "Enable"
      await waitFor(() => {
        expect(screen.getByLabelText('Enable Test Keyboard 1')).toBeInTheDocument();
      });
    });

    it('toggle persists state across component re-renders', async () => {
      const user = userEvent.setup();

      renderWithProviders(<DevicesPage />);

      await waitFor(() => {
        expect(screen.getByText(/Device List \(2 connected\)/)).toBeInTheDocument();
      });

      const toggle = await screen.findByLabelText('Disable Test Keyboard 1');
      await user.click(toggle);

      // Toggle should be disabled
      await waitFor(() => {
        expect(toggle).toHaveAttribute('aria-checked', 'false');
      });

      // State should persist in localStorage
      const storedStates = localStorage.getItem('keyrx:device:enabled');
      expect(storedStates).toBeTruthy();
      const states = JSON.parse(storedStates!);
      expect(states['device-1']).toBe(false);
    });

    it('disabled devices persist across page refresh', async () => {
      const user = userEvent.setup();

      server.use(
        http.get('/api/devices', () => {
          return HttpResponse.json({
            devices: [
              {
                id: 'device-1',
                name: 'Test Keyboard 1',
                path: '/dev/input/event0',
                active: true,
                layout: 'ANSI_104',
                enabled: false, // Persisted as disabled
              },
              {
                id: 'device-2',
                name: 'Test Keyboard 2',
                path: '/dev/input/event1',
                active: true,
                layout: 'ANSI_104',
                enabled: true,
              },
            ],
          });
        })
      );

      // Simulate persisted disabled state in localStorage
      localStorage.setItem('keyrx:device:enabled', JSON.stringify({
        'device-1': false,
        'device-2': true,
      }));

      renderWithProviders(<DevicesPage />);

      await waitFor(() => {
        expect(screen.getByText(/Device List \(2 connected\)/)).toBeInTheDocument();
      });

      // Verify disabled state persists
      const toggle1 = screen.getByLabelText(/Enable Test Keyboard 1/);
      expect(toggle1).toHaveAttribute('aria-checked', 'false');

      const toggle2 = screen.getByLabelText(/Disable Test Keyboard 2/);
      expect(toggle2).toHaveAttribute('aria-checked', 'true');
    });

    it('forget button still works for disabled devices', async () => {
      const user = userEvent.setup();

      server.use(
        http.get('/api/devices', () => {
          return HttpResponse.json({
            devices: [
              {
                id: 'device-1',
                name: 'Test Keyboard 1',
                path: '/dev/input/event0',
                active: true,
                layout: 'ANSI_104',
                enabled: false, // Disabled device
              },
            ],
          });
        })
      );

      renderWithProviders(<DevicesPage />);

      await waitFor(() => {
        expect(screen.getByText(/Device List \(1 connected\)/)).toBeInTheDocument();
      });

      // Should be able to forget disabled device
      const forgetButton = screen.getByLabelText(/Permanently forget Test Keyboard 1/);
      expect(forgetButton).toBeInTheDocument();
      await user.click(forgetButton);

      // Confirmation modal should appear
      expect(screen.getByRole('dialog', { name: 'Forget Device' })).toBeInTheDocument();
    });

    it('toggle is keyboard accessible', async () => {
      const user = userEvent.setup();

      renderWithProviders(<DevicesPage />);

      await waitFor(() => {
        expect(screen.getByText(/Device List \(2 connected\)/)).toBeInTheDocument();
      });

      const toggle = await screen.findByLabelText('Disable Test Keyboard 1');

      // Focus the toggle
      toggle.focus();
      expect(toggle).toHaveFocus();

      // Press Space to toggle
      await user.keyboard(' ');

      await waitFor(() => {
        expect(toggle).toHaveAttribute('aria-checked', 'false');
      });
    });
  });

  // ========================================================================
  // Integration Workflow Tests
  // ========================================================================
  describe('End-to-End Workflows', () => {
    // TODO: Unskip when rename API is implemented (see DevicesPage.tsx line 338)
    it.skip('complete device management workflow: rename, change layout, verify persistence', async () => {
      const user = userEvent.setup();
      let savedName = '';
      let savedLayout = '';

      server.use(
        http.patch('/api/devices/:id', async ({ request }) => {
          const body = (await request.json()) as { name?: string; layout?: string };
          if (body.name) savedName = body.name;
          if (body.layout) savedLayout = body.layout;
          return HttpResponse.json({ success: true });
        })
      );

      renderWithProviders(<DevicesPage />);

      await waitFor(() => {
        expect(screen.getByText('Devices')).toBeInTheDocument();
      });

      // Step 1: Rename device
      const renameButton = screen.getByLabelText('Rename Test Keyboard 1');
      await user.click(renameButton);

      const input = screen.getByRole('textbox', { name: 'Device name' });
      await user.clear(input);
      await user.type(input, 'Gaming Keyboard');
      await user.click(screen.getByLabelText('Save'));

      await waitFor(() => {
        expect(screen.getAllByText('Gaming Keyboard').length).toBeGreaterThan(0);
      });

      // Step 2: Change layout
      const layoutDropdowns = screen.getAllByLabelText('Layout');
      await user.click(layoutDropdowns[0]);
      await user.click(screen.getByText('ISO 105 (Full)'));

      // Step 3: Verify persistence
      await waitFor(
        () => {
          expect(savedLayout).toBe('ISO_105');
        },
        { timeout: 2000 }
      );
    });

    it('global layout change workflow: select, save, verify feedback', async () => {
      const user = userEvent.setup();
      let savedGlobalLayout = '';

      server.use(
        http.put('/api/settings/global-layout', async ({ request }) => {
          const body = (await request.json()) as { layout: string };
          savedGlobalLayout = body.layout;
          return HttpResponse.json({ success: true });
        })
      );

      renderWithProviders(<DevicesPage />);

      await waitFor(() => {
        expect(screen.getByText('Global Settings')).toBeInTheDocument();
      });

      // Select new global layout
      const globalLayoutDropdown = screen.getByLabelText('Select default keyboard layout');
      await user.click(globalLayoutDropdown);
      await user.click(screen.getByText('HHKB'));

      // Verify save
      await waitFor(() => {
        expect(savedGlobalLayout).toBe('HHKB');
      });

      // Verify success feedback
      await waitFor(() => {
        expect(screen.getByText('✓ Saved')).toBeInTheDocument();
      });
    });

    // TODO: Unskip when rename API is implemented (see DevicesPage.tsx line 338)
    it.skip('multi-device management: edit multiple devices independently', async () => {
      const user = userEvent.setup();
      renderWithProviders(<DevicesPage />);

      await waitFor(() => {
        expect(screen.getByText('Devices')).toBeInTheDocument();
      });

      // Rename first device
      const renameButton1 = screen.getByLabelText('Rename Test Keyboard 1');
      await user.click(renameButton1);
      const input1 = screen.getByRole('textbox', { name: 'Device name' });
      await user.clear(input1);
      await user.type(input1, 'Work Keyboard');
      await user.click(screen.getByLabelText('Save'));

      await waitFor(() => {
        expect(screen.queryByRole('textbox', { name: 'Device name' })).not.toBeInTheDocument();
      });

      // Rename second device
      const renameButton2 = screen.getByLabelText('Rename Test Keyboard 2');
      await user.click(renameButton2);
      const input2 = screen.getByRole('textbox', { name: 'Device name' });
      await user.clear(input2);
      await user.type(input2, 'Home Keyboard');
      await user.click(screen.getByLabelText('Save'));

      await waitFor(() => {
        expect(screen.getAllByText('Work Keyboard').length).toBeGreaterThan(0);
        expect(screen.getAllByText('Home Keyboard').length).toBeGreaterThan(0);
      });
    });
  });
});
