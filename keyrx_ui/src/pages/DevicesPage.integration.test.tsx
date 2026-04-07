/**
 * Integration tests for DevicesPage
 * Tests complete user flows with component's internal mock data
 *
 * NOTE: These tests verify UI interactions and state changes.
 * Once API integration is complete (stores connected to backend),
 * update these tests to use MSW for full end-to-end testing.
 */

import { describe, it, expect, vi } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import { renderWithProviders } from '../../tests/testUtils';
import userEvent from '@testing-library/user-event';
import { DevicesPage } from './DevicesPage';

// Mock react-router-dom
vi.mock('react-router-dom', () => ({
  useNavigate: () => vi.fn(),
}));

describe('DevicesPage - Integration Tests', () => {
  describe('Device rename flow', () => {
    it('successfully renames device', async () => {
      const user = userEvent.setup();
      renderWithProviders(<DevicesPage />);

      // Wait for page to render (uses internal mock data)
      await waitFor(() => {
        expect(screen.getByText('Test Keyboard 1')).toBeInTheDocument();
      });

      // Click rename button for "Test Keyboard 1"
      const renameButton = screen.getByLabelText('Rename Test Keyboard 1');
      await user.click(renameButton);

      // Find the input field
      const input = screen.getByRole('textbox', { name: 'Device name' });
      expect(input).toHaveValue('Test Keyboard 1');

      // Change the name
      await user.clear(input);
      await user.type(input, 'My Gaming Keyboard');

      // Save the change
      const saveButton = screen.getByLabelText('Save');
      await user.click(saveButton);

      // Verify name changes
      await waitFor(() => {
        expect(screen.getByText('My Gaming Keyboard')).toBeInTheDocument();
        expect(screen.queryByText('Test Keyboard 1')).not.toBeInTheDocument();
      });
    });

    it('handles rename with Enter key', async () => {
      const user = userEvent.setup();
      renderWithProviders(<DevicesPage />);

      await waitFor(() => {
        expect(screen.getByText('Test Keyboard 1')).toBeInTheDocument();
      });

      const renameButton = screen.getByLabelText('Rename Test Keyboard 1');
      await user.click(renameButton);

      const input = screen.getByRole('textbox', { name: 'Device name' });
      await user.clear(input);
      await user.type(input, 'New Name{Enter}');

      await waitFor(() => {
        expect(screen.getByText('New Name')).toBeInTheDocument();
      });
    });

    it('cancels rename on Escape key', async () => {
      const user = userEvent.setup();
      renderWithProviders(<DevicesPage />);

      await waitFor(() => {
        expect(screen.getByText('Test Keyboard 1')).toBeInTheDocument();
      });

      const renameButton = screen.getByLabelText('Rename Test Keyboard 1');
      await user.click(renameButton);

      const input = screen.getByRole('textbox', { name: 'Device name' });
      await user.clear(input);
      await user.type(input, 'This will be cancelled{Escape}');

      // Original name should still be there
      await waitFor(() => {
        expect(screen.getByText('Test Keyboard 1')).toBeInTheDocument();
      });
    });

    it('cancels rename on Cancel button click', async () => {
      const user = userEvent.setup();
      renderWithProviders(<DevicesPage />);

      await waitFor(() => {
        expect(screen.getByText('Test Keyboard 1')).toBeInTheDocument();
      });

      const renameButton = screen.getByLabelText('Rename Test Keyboard 1');
      await user.click(renameButton);

      const input = screen.getByRole('textbox', { name: 'Device name' });
      await user.clear(input);
      await user.type(input, 'This will be cancelled');

      const cancelButton = screen.getByLabelText('Cancel');
      await user.click(cancelButton);

      // Original name should still be there
      expect(screen.getByText('Test Keyboard 1')).toBeInTheDocument();
    });

    it('validates empty device name', async () => {
      const user = userEvent.setup();
      renderWithProviders(<DevicesPage />);

      await waitFor(() => {
        expect(screen.getByText('Test Keyboard 1')).toBeInTheDocument();
      });

      const renameButton = screen.getByLabelText('Rename Test Keyboard 1');
      await user.click(renameButton);

      const input = screen.getByRole('textbox', { name: 'Device name' });
      await user.clear(input);

      // Try to save empty name
      const saveButton = screen.getByLabelText('Save');
      await user.click(saveButton);

      // Should show validation error and remain in edit mode
      expect(
        screen.getByRole('textbox', { name: 'Device name' })
      ).toBeInTheDocument();
    });
  });

  describe('Forget device flow', () => {
    it('shows confirmation modal before forgetting device', async () => {
      const user = userEvent.setup();
      renderWithProviders(<DevicesPage />);

      await waitFor(() => {
        expect(screen.getByText('Test Keyboard 1')).toBeInTheDocument();
      });

      const forgetButton = screen.getByLabelText('Permanently forget Test Keyboard 1');
      await user.click(forgetButton);

      // Modal should appear
      await waitFor(() => {
        expect(
          screen.getByText(/Are you sure you want to forget device/i)
        ).toBeInTheDocument();
      });
    });

    it('successfully forgets device on confirmation', async () => {
      const user = userEvent.setup();
      renderWithProviders(<DevicesPage />);

      await waitFor(() => {
        expect(screen.getByText('Test Keyboard 1')).toBeInTheDocument();
      });

      const forgetButton = screen.getByLabelText('Permanently forget Test Keyboard 1');
      await user.click(forgetButton);

      // Wait for modal and click confirm
      await waitFor(() => {
        expect(screen.getByLabelText('Confirm forget device')).toBeInTheDocument();
      });

      const confirmButton = screen.getByLabelText('Confirm forget device');
      await user.click(confirmButton);

      // Device should be removed from list
      await waitFor(() => {
        expect(screen.queryByText('Test Keyboard 1')).not.toBeInTheDocument();
      });

      // Other device should still be there
      expect(screen.getByText('Test Keyboard 2')).toBeInTheDocument();
    });

    it('cancels forget operation on Cancel button', async () => {
      const user = userEvent.setup();
      renderWithProviders(<DevicesPage />);

      await waitFor(() => {
        expect(screen.getAllByText('Test Keyboard 1').length).toBeGreaterThan(0);
      });

      const forgetButton = screen.getByLabelText('Permanently forget Test Keyboard 1');
      await user.click(forgetButton);

      // Wait for modal to appear, use aria-label to avoid matching other Cancel buttons
      await waitFor(() => {
        expect(screen.getByLabelText('Cancel forget device')).toBeInTheDocument();
      });

      const cancelButton = screen.getByLabelText('Cancel forget device');
      await user.click(cancelButton);

      // Modal should close and device should still be there
      await waitFor(() => {
        expect(
          screen.queryByText(/Are you sure you want to forget device/i)
        ).not.toBeInTheDocument();
      });

      expect(screen.getAllByText('Test Keyboard 1').length).toBeGreaterThan(0);
    });
  });

  describe('Layout selector flow', () => {
    it('changes keyboard layout preset', async () => {
      const user = userEvent.setup();
      renderWithProviders(<DevicesPage />);

      await waitFor(() => {
        expect(screen.getAllByText('Test Keyboard 1').length).toBeGreaterThan(0);
      });

      // Find the device layout selector (HeadlessUI Listbox button)
      // DeviceRow's LayoutDropdown has aria-label="Layout"
      const layoutButtons = screen.getAllByLabelText('Layout');
      const firstLayoutButton = layoutButtons[0];

      // Default should show ANSI 104
      expect(firstLayoutButton).toHaveTextContent('ANSI 104');

      // Click to open the dropdown
      await user.click(firstLayoutButton);

      // Select ISO 105 option
      const isoOption = await screen.findByRole('option', {
        name: /ISO 105/i,
      });
      await user.click(isoOption);

      // Verify the button text changed
      await waitFor(() => {
        expect(firstLayoutButton).toHaveTextContent('ISO 105');
      });
    });
  });

  describe('Multiple devices interaction', () => {
    it('can rename multiple devices in sequence', async () => {
      const user = userEvent.setup();
      renderWithProviders(<DevicesPage />);

      await waitFor(() => {
        expect(screen.getByText('Test Keyboard 1')).toBeInTheDocument();
        expect(screen.getByText('Test Keyboard 2')).toBeInTheDocument();
      });

      // Rename first device
      const renameButton1 = screen.getByLabelText(
        'Rename Test Keyboard 1'
      );
      await user.click(renameButton1);

      const input1 = screen.getByRole('textbox', { name: 'Device name' });
      await user.clear(input1);
      await user.type(input1, 'Primary Keyboard{Enter}');

      await waitFor(() => {
        expect(screen.getByText('Primary Keyboard')).toBeInTheDocument();
      });

      // Rename second device
      const renameButton2 = screen.getByLabelText('Rename Test Keyboard 2');
      await user.click(renameButton2);

      const input2 = screen.getByRole('textbox', { name: 'Device name' });
      await user.clear(input2);
      await user.type(input2, 'Number Pad{Enter}');

      await waitFor(() => {
        expect(screen.getByText('Number Pad')).toBeInTheDocument();
      });

      // Both should be present
      expect(screen.getByText('Primary Keyboard')).toBeInTheDocument();
      expect(screen.getByText('Number Pad')).toBeInTheDocument();
    });
  });
});
