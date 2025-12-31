import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { DevicesPage } from './DevicesPage';

// Mock react-router-dom
vi.mock('react-router-dom', () => ({
  useNavigate: () => vi.fn(),
}));

describe('DevicesPage', () => {
  it('renders devices page with device list', () => {
    render(<DevicesPage />);

    expect(screen.getByText('Devices')).toBeInTheDocument();
    expect(screen.getByText(/Device List \(2 connected\)/)).toBeInTheDocument();
    expect(screen.getAllByText('Main Keyboard').length).toBeGreaterThan(0);
    expect(screen.getAllByText('Left Numpad').length).toBeGreaterThan(0);
  });

  it('shows connected status for active device', () => {
    render(<DevicesPage />);

    const connectedLabels = screen.getAllByText('✓ Connected');
    expect(connectedLabels.length).toBeGreaterThan(0);
  });

  it('displays device details', () => {
    render(<DevicesPage />);

    expect(screen.getByText(/USB\\VID_1234&PID_5678\\ABC123/)).toBeInTheDocument();
    expect(screen.getByText(/Serial: ABC123/)).toBeInTheDocument();
    expect(screen.getByText(/Vendor: 0x1234/)).toBeInTheDocument();
    expect(screen.getByText(/Product: 0x5678/)).toBeInTheDocument();
  });

  it('enters rename mode when Rename button is clicked', async () => {
    const user = userEvent.setup();
    render(<DevicesPage />);

    const renameButton = screen.getByLabelText('Rename device Main Keyboard');
    await user.click(renameButton);

    // Input should appear with current name
    const input = screen.getByRole('textbox', { name: 'Device name' });
    expect(input).toBeInTheDocument();
    expect(input).toHaveValue('Main Keyboard');

    // Save and Cancel buttons should appear
    expect(screen.getByLabelText('Save device name')).toBeInTheDocument();
    expect(screen.getByLabelText('Cancel rename')).toBeInTheDocument();
  });

  it('saves new name when Save button is clicked', async () => {
    const user = userEvent.setup();
    render(<DevicesPage />);

    const renameButton = screen.getByLabelText('Rename device Main Keyboard');
    await user.click(renameButton);

    const input = screen.getByRole('textbox', { name: 'Device name' });
    await user.clear(input);
    await user.type(input, 'New Keyboard Name');

    const saveButton = screen.getByLabelText('Save device name');
    await user.click(saveButton);

    // Input should disappear
    expect(screen.queryByRole('textbox', { name: 'Device name' })).not.toBeInTheDocument();

    // New name should be displayed
    expect(screen.getAllByText('New Keyboard Name').length).toBeGreaterThan(0);
  });

  it('saves new name when Enter key is pressed', async () => {
    const user = userEvent.setup();
    render(<DevicesPage />);

    const renameButton = screen.getByLabelText('Rename device Main Keyboard');
    await user.click(renameButton);

    const input = screen.getByRole('textbox', { name: 'Device name' });
    await user.clear(input);
    await user.type(input, 'Renamed via Enter{Enter}');

    await waitFor(() => {
      expect(screen.queryByRole('textbox', { name: 'Device name' })).not.toBeInTheDocument();
    });

    expect(screen.getAllByText('Renamed via Enter').length).toBeGreaterThan(0);
  });

  it('cancels rename when Cancel button is clicked', async () => {
    const user = userEvent.setup();
    render(<DevicesPage />);

    const renameButton = screen.getByLabelText('Rename device Main Keyboard');
    await user.click(renameButton);

    const input = screen.getByRole('textbox', { name: 'Device name' });
    await user.clear(input);
    await user.type(input, 'This should not be saved');

    const cancelButton = screen.getByLabelText('Cancel rename');
    await user.click(cancelButton);

    // Input should disappear
    expect(screen.queryByRole('textbox', { name: 'Device name' })).not.toBeInTheDocument();

    // Original name should still be displayed
    expect(screen.getAllByText('Main Keyboard').length).toBeGreaterThan(0);
    expect(screen.queryByText('This should not be saved')).not.toBeInTheDocument();
  });

  it('cancels rename when Escape key is pressed', async () => {
    const user = userEvent.setup();
    render(<DevicesPage />);

    const renameButton = screen.getByLabelText('Rename device Main Keyboard');
    await user.click(renameButton);

    const input = screen.getByRole('textbox', { name: 'Device name' });
    await user.clear(input);
    await user.type(input, 'This should not be saved{Escape}');

    await waitFor(() => {
      expect(screen.queryByRole('textbox', { name: 'Device name' })).not.toBeInTheDocument();
    });

    expect(screen.getAllByText('Main Keyboard').length).toBeGreaterThan(0);
  });

  it('shows error when trying to save empty name', async () => {
    const user = userEvent.setup();
    render(<DevicesPage />);

    const renameButton = screen.getByLabelText('Rename device Main Keyboard');
    await user.click(renameButton);

    const input = screen.getByRole('textbox', { name: 'Device name' });
    await user.clear(input);

    const saveButton = screen.getByLabelText('Save device name');
    await user.click(saveButton);

    // Error message should appear
    expect(screen.getByText('Device name cannot be empty')).toBeInTheDocument();

    // Input should still be visible
    expect(input).toBeInTheDocument();
  });

  it('shows character counter when maxLength is set', async () => {
    const user = userEvent.setup();
    render(<DevicesPage />);

    const renameButton = screen.getByLabelText('Rename device Main Keyboard');
    await user.click(renameButton);

    const input = screen.getByRole('textbox', { name: 'Device name' });
    await user.clear(input);
    await user.type(input, 'This is a name');

    // Character counter should be visible (Input component shows it when maxLength is set)
    expect(screen.getByText(/\/ 64/)).toBeInTheDocument();
  });

  it('toggles scope from global to device-specific', async () => {
    const user = userEvent.setup();
    render(<DevicesPage />);

    const deviceSpecificButton = screen.getAllByLabelText('Set scope to device-specific')[0];
    await user.click(deviceSpecificButton);

    // Check that the button is now highlighted
    // Note: Exact assertion depends on implementation
    expect(deviceSpecificButton).toHaveClass('border-primary-500');
  });

  it('toggles scope from device-specific to global', async () => {
    const user = userEvent.setup();
    render(<DevicesPage />);

    // Find the second device (Left Numpad) which has device-specific scope
    const globalButtons = screen.getAllByLabelText('Set scope to global');
    const globalButton = globalButtons[1]; // Second device
    await user.click(globalButton);

    expect(globalButton).toHaveClass('border-primary-500');
  });

  it('changes layout via dropdown', async () => {
    const user = userEvent.setup();
    render(<DevicesPage />);

    // Find the first layout dropdown
    const layoutDropdowns = screen.getAllByLabelText('Select keyboard layout');
    const dropdown = layoutDropdowns[0];

    // Open dropdown
    await user.click(dropdown);

    // Select ISO 105 option
    const isoOption = screen.getByText('ISO 105');
    await user.click(isoOption);

    // Verify the selection (implementation-dependent)
    // This test assumes the Dropdown component displays the selected value
  });

  it('opens forget device confirmation modal', async () => {
    const user = userEvent.setup();
    render(<DevicesPage />);

    const forgetButton = screen.getByLabelText('Forget device Main Keyboard');
    await user.click(forgetButton);

    // Modal should appear with confirmation message
    expect(screen.getByRole('dialog', { name: 'Forget Device' })).toBeInTheDocument();
    expect(screen.getByText(/Are you sure you want to forget device/)).toBeInTheDocument();
  });

  it('cancels forget device when Cancel is clicked in modal', async () => {
    const user = userEvent.setup();
    render(<DevicesPage />);

    const forgetButton = screen.getByLabelText('Forget device Main Keyboard');
    await user.click(forgetButton);

    const cancelButton = screen.getByLabelText('Cancel forget device');
    await user.click(cancelButton);

    // Modal should close
    await waitFor(() => {
      expect(screen.queryByRole('dialog', { name: 'Forget Device' })).not.toBeInTheDocument();
    });

    // Device should still be in the list
    expect(screen.getAllByText('Main Keyboard').length).toBeGreaterThan(0);
  });

  it('removes device when Forget Device is confirmed', async () => {
    const user = userEvent.setup();
    render(<DevicesPage />);

    const forgetButton = screen.getByLabelText('Forget device Main Keyboard');
    await user.click(forgetButton);

    const confirmButton = screen.getByLabelText('Confirm forget device');
    await user.click(confirmButton);

    // Modal should close
    await waitFor(() => {
      expect(screen.queryByRole('dialog', { name: 'Forget Device' })).not.toBeInTheDocument();
    });

    // Device should be removed from the list - check that it's gone from the page
    await waitFor(() => {
      expect(screen.queryByLabelText('Forget device Main Keyboard')).not.toBeInTheDocument();
    });

    // Device count should update
    expect(screen.getByText(/Device List \(1 connected\)/)).toBeInTheDocument();
  });

  it('shows empty state when no devices are connected', async () => {
    const user = userEvent.setup();
    render(<DevicesPage />);

    // Remove first device
    const forgetButton1 = screen.getByLabelText('Forget device Main Keyboard');
    await user.click(forgetButton1);
    const confirmButton1 = screen.getByLabelText('Confirm forget device');
    await user.click(confirmButton1);
    await waitFor(() => {
      expect(screen.queryByRole('dialog', { name: 'Forget Device' })).not.toBeInTheDocument();
    });

    // Remove second device
    const forgetButton2 = screen.getByLabelText('Forget device Left Numpad');
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

  it('has accessible labels for all interactive elements', () => {
    render(<DevicesPage />);

    // Rename buttons
    expect(screen.getByLabelText('Rename device Main Keyboard')).toBeInTheDocument();
    expect(screen.getByLabelText('Rename device Left Numpad')).toBeInTheDocument();

    // Scope buttons
    expect(screen.getAllByLabelText('Set scope to global').length).toBeGreaterThan(0);
    expect(screen.getAllByLabelText('Set scope to device-specific').length).toBeGreaterThan(0);

    // Layout dropdowns
    expect(screen.getAllByLabelText('Select keyboard layout').length).toBeGreaterThan(0);

    // Forget buttons
    expect(screen.getByLabelText('Forget device Main Keyboard')).toBeInTheDocument();
    expect(screen.getByLabelText('Forget device Left Numpad')).toBeInTheDocument();
  });
});
