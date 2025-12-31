/**
 * Integration tests for DevicesPage
 * Tests complete user flows with component's internal mock data
 *
 * NOTE: These tests verify UI interactions and state changes.
 * Once API integration is complete (stores connected to backend),
 * update these tests to use MSW for full end-to-end testing.
 */

import { describe, it, expect, vi } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
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
      render(<DevicesPage />);

      // Wait for page to render (uses internal mock data)
      await waitFor(() => {
        expect(screen.getByText('Main Keyboard')).toBeInTheDocument();
      });

      // Click rename button for "Main Keyboard"
      const renameButton = screen.getByLabelText('Rename device Main Keyboard');
      await user.click(renameButton);

      // Find the input field
      const input = screen.getByRole('textbox', { name: 'Device name' });
      expect(input).toHaveValue('Main Keyboard');

      // Change the name
      await user.clear(input);
      await user.type(input, 'My Gaming Keyboard');

      // Save the change
      const saveButton = screen.getByLabelText('Save device name');
      await user.click(saveButton);

      // Verify name changes
      await waitFor(() => {
        expect(screen.getByText('My Gaming Keyboard')).toBeInTheDocument();
        expect(screen.queryByText('Main Keyboard')).not.toBeInTheDocument();
      });
    });

    it('handles rename with Enter key', async () => {
      const user = userEvent.setup();
      render(<DevicesPage />);

      await waitFor(() => {
        expect(screen.getByText('Main Keyboard')).toBeInTheDocument();
      });

      const renameButton = screen.getByLabelText('Rename device Main Keyboard');
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
      render(<DevicesPage />);

      await waitFor(() => {
        expect(screen.getByText('Main Keyboard')).toBeInTheDocument();
      });

      const renameButton = screen.getByLabelText('Rename device Main Keyboard');
      await user.click(renameButton);

      const input = screen.getByRole('textbox', { name: 'Device name' });
      await user.clear(input);
      await user.type(input, 'This will be cancelled{Escape}');

      // Original name should still be there
      await waitFor(() => {
        expect(screen.getByText('Main Keyboard')).toBeInTheDocument();
      });
    });

    it('cancels rename on Cancel button click', async () => {
      const user = userEvent.setup();
      render(<DevicesPage />);

      await waitFor(() => {
        expect(screen.getByText('Main Keyboard')).toBeInTheDocument();
      });

      const renameButton = screen.getByLabelText('Rename device Main Keyboard');
      await user.click(renameButton);

      const input = screen.getByRole('textbox', { name: 'Device name' });
      await user.clear(input);
      await user.type(input, 'This will be cancelled');

      const cancelButton = screen.getByLabelText('Cancel rename');
      await user.click(cancelButton);

      // Original name should still be there
      expect(screen.getByText('Main Keyboard')).toBeInTheDocument();
    });

    it('validates empty device name', async () => {
      const user = userEvent.setup();
      render(<DevicesPage />);

      await waitFor(() => {
        expect(screen.getByText('Main Keyboard')).toBeInTheDocument();
      });

      const renameButton = screen.getByLabelText('Rename device Main Keyboard');
      await user.click(renameButton);

      const input = screen.getByRole('textbox', { name: 'Device name' });
      await user.clear(input);

      // Try to save empty name
      const saveButton = screen.getByLabelText('Save device name');
      await user.click(saveButton);

      // Should show validation error
      // Original name should still be there
      expect(screen.getByText('Main Keyboard')).toBeInTheDocument();
    });
  });

  describe('Scope toggle flow', () => {
    it('successfully changes device scope', async () => {
      const user = userEvent.setup();
      render(<DevicesPage />);

      await waitFor(() => {
        expect(screen.getByText('Main Keyboard')).toBeInTheDocument();
      });

      // Find the scope selectors
      const scopeSelectors = screen.getAllByRole('combobox', {
        name: /Scope selector/i,
      });
      const firstScopeSelector = scopeSelectors[0];

      // Verify initial scope is global
      expect(firstScopeSelector).toHaveValue('global');

      // Change to device-specific
      await user.selectOptions(firstScopeSelector, 'device-specific');

      // Verify change
      await waitFor(() => {
        expect(firstScopeSelector).toHaveValue('device-specific');
      });
    });
  });

  describe('Forget device flow', () => {
    it('shows confirmation modal before forgetting device', async () => {
      const user = userEvent.setup();
      render(<DevicesPage />);

      await waitFor(() => {
        expect(screen.getByText('Main Keyboard')).toBeInTheDocument();
      });

      const forgetButton = screen.getByLabelText('Forget device Main Keyboard');
      await user.click(forgetButton);

      // Modal should appear
      await waitFor(() => {
        expect(
          screen.getByText(/Are you sure you want to forget this device/i)
        ).toBeInTheDocument();
      });
    });

    it('successfully forgets device on confirmation', async () => {
      const user = userEvent.setup();
      render(<DevicesPage />);

      await waitFor(() => {
        expect(screen.getByText('Main Keyboard')).toBeInTheDocument();
      });

      const forgetButton = screen.getByLabelText('Forget device Main Keyboard');
      await user.click(forgetButton);

      // Wait for modal and click confirm
      await waitFor(() => {
        expect(screen.getByText('Confirm')).toBeInTheDocument();
      });

      const confirmButton = screen.getByText('Confirm');
      await user.click(confirmButton);

      // Device should be removed from list
      await waitFor(() => {
        expect(screen.queryByText('Main Keyboard')).not.toBeInTheDocument();
      });

      // Other device should still be there
      expect(screen.getByText('Left Numpad')).toBeInTheDocument();
    });

    it('cancels forget operation on Cancel button', async () => {
      const user = userEvent.setup();
      render(<DevicesPage />);

      await waitFor(() => {
        expect(screen.getByText('Main Keyboard')).toBeInTheDocument();
      });

      const forgetButton = screen.getByLabelText('Forget device Main Keyboard');
      await user.click(forgetButton);

      await waitFor(() => {
        expect(screen.getByText('Cancel')).toBeInTheDocument();
      });

      const cancelButton = screen.getByText('Cancel');
      await user.click(cancelButton);

      // Modal should close and device should still be there
      await waitFor(() => {
        expect(
          screen.queryByText(/Are you sure you want to forget this device/i)
        ).not.toBeInTheDocument();
      });

      expect(screen.getByText('Main Keyboard')).toBeInTheDocument();
    });
  });

  describe('Layout selector flow', () => {
    it('changes keyboard layout preset', async () => {
      const user = userEvent.setup();
      render(<DevicesPage />);

      await waitFor(() => {
        expect(screen.getByText('Main Keyboard')).toBeInTheDocument();
      });

      // Find layout selector dropdown (multiple devices may have them)
      const layoutSelectors = screen.getAllByRole('combobox', {
        name: /Keyboard Layout/i,
      });
      const firstLayoutSelector = layoutSelectors[0];

      // Default should be ANSI_104
      expect(firstLayoutSelector).toHaveValue('ANSI_104');

      // Change to ISO_105
      await user.selectOptions(firstLayoutSelector, 'ISO_105');

      // Verify change
      expect(firstLayoutSelector).toHaveValue('ISO_105');
    });
  });

  describe('Multiple devices interaction', () => {
    it('can rename multiple devices in sequence', async () => {
      const user = userEvent.setup();
      render(<DevicesPage />);

      await waitFor(() => {
        expect(screen.getByText('Main Keyboard')).toBeInTheDocument();
        expect(screen.getByText('Left Numpad')).toBeInTheDocument();
      });

      // Rename first device
      const renameButton1 = screen.getByLabelText('Rename device Main Keyboard');
      await user.click(renameButton1);

      const input1 = screen.getByRole('textbox', { name: 'Device name' });
      await user.clear(input1);
      await user.type(input1, 'Primary Keyboard{Enter}');

      await waitFor(() => {
        expect(screen.getByText('Primary Keyboard')).toBeInTheDocument();
      });

      // Rename second device
      const renameButton2 = screen.getByLabelText('Rename device Left Numpad');
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
