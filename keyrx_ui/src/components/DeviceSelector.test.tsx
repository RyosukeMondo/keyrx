import { describe, it, expect, vi } from 'vitest';
import { renderPure, screen, waitFor } from '../../tests/testUtils';
import userEvent from '@testing-library/user-event';
import { DeviceSelector, Device } from './DeviceSelector';

describe('DeviceSelector', () => {
  const mockDevices: Device[] = [
    {
      id: 'device-1',
      name: 'Keyboard 1',
      serial: 'SN12345',
      connected: true,
      layout: 'ANSI_104',
    },
    {
      id: 'device-2',
      name: 'Keyboard 2',
      serial: 'SN67890',
      connected: false,
      layout: 'ISO_105',
    },
    {
      id: 'device-3',
      name: 'Keyboard 3',
      connected: true,
      layout: 'JIS_109',
    },
  ];

  const defaultProps = {
    devices: mockDevices,
    selectedDevices: [],
    globalSelected: false,
    onSelectionChange: vi.fn(),
  };

  describe('Rendering', () => {
    it('renders device list correctly', () => {
      renderPure(<DeviceSelector {...defaultProps} />);

      expect(screen.getByText('Device Selection')).toBeInTheDocument();
      expect(screen.getByText('Keyboard 1')).toBeInTheDocument();
      expect(screen.getByText('Keyboard 2')).toBeInTheDocument();
      expect(screen.getByText('Keyboard 3')).toBeInTheDocument();
    });

    it('renders global checkbox when showGlobalOption is true', () => {
      renderPure(<DeviceSelector {...defaultProps} showGlobalOption={true} />);

      const globalCheckbox = screen.getByLabelText(
        'Apply configuration globally to all devices'
      );
      expect(globalCheckbox).toBeInTheDocument();
      expect(screen.getByText('Global (All Devices)')).toBeInTheDocument();
    });

    it('does not render global checkbox when showGlobalOption is false', () => {
      renderPure(<DeviceSelector {...defaultProps} showGlobalOption={false} />);

      expect(
        screen.queryByLabelText('Apply configuration globally to all devices')
      ).not.toBeInTheDocument();
    });

    it('renders device serial numbers when present', () => {
      renderPure(<DeviceSelector {...defaultProps} />);

      expect(screen.getByText('Serial: SN12345')).toBeInTheDocument();
      expect(screen.getByText('Serial: SN67890')).toBeInTheDocument();
    });

    it('renders device layouts when present', () => {
      renderPure(<DeviceSelector {...defaultProps} />);

      expect(screen.getByText('Layout: ANSI_104')).toBeInTheDocument();
      expect(screen.getByText('Layout: ISO_105')).toBeInTheDocument();
      expect(screen.getByText('Layout: JIS_109')).toBeInTheDocument();
    });

    it('renders empty state when no devices', () => {
      renderPure(<DeviceSelector {...defaultProps} devices={[]} />);

      expect(
        screen.getByText('No devices detected. Connect a keyboard to get started.')
      ).toBeInTheDocument();
    });

    it('renders multi-select info text by default', () => {
      renderPure(<DeviceSelector {...defaultProps} />);

      expect(
        screen.getByText(
          'Select one or more devices to configure. Device-specific mappings will be generated in Rhai script.'
        )
      ).toBeInTheDocument();
    });

    it('renders single-select info text when multiSelect is false', () => {
      renderPure(<DeviceSelector {...defaultProps} multiSelect={false} />);

      expect(screen.getByText('Select a device to configure.')).toBeInTheDocument();
    });
  });

  describe('Connection Status Badges', () => {
    it('shows connected badge for connected devices', () => {
      renderPure(<DeviceSelector {...defaultProps} />);

      const connectedBadges = screen.getAllByText('Connected');
      expect(connectedBadges).toHaveLength(2); // device-1 and device-3

      // Verify badge has correct styling (green)
      connectedBadges.forEach((badge) => {
        expect(badge).toHaveClass('bg-green-900/30', 'text-green-300');
      });
    });

    it('shows disconnected badge for disconnected devices', () => {
      renderPure(<DeviceSelector {...defaultProps} />);

      const disconnectedBadge = screen.getByText('Disconnected');
      expect(disconnectedBadge).toBeInTheDocument();

      // Verify badge has correct styling (gray)
      expect(disconnectedBadge).toHaveClass('bg-gray-700/30', 'text-gray-400');
    });

    it('does not show badge when connected property is undefined', () => {
      const devicesWithoutConnection: Device[] = [
        { id: 'device-1', name: 'Keyboard 1' },
      ];
      renderPure(
        <DeviceSelector {...defaultProps} devices={devicesWithoutConnection} />
      );

      expect(screen.queryByText('Connected')).not.toBeInTheDocument();
      expect(screen.queryByText('Disconnected')).not.toBeInTheDocument();
    });
  });

  describe('No Selection Warning', () => {
    it('shows warning when no device and no global selected', () => {
      renderPure(<DeviceSelector {...defaultProps} />);

      const warning = screen.getByRole('alert');
      expect(warning).toHaveTextContent(
        '⚠ Select at least one device or global to configure'
      );
      expect(warning).toHaveAttribute('aria-live', 'polite');
    });

    it('hides warning when global is selected', () => {
      renderPure(<DeviceSelector {...defaultProps} globalSelected={true} />);

      expect(screen.queryByRole('alert')).not.toBeInTheDocument();
    });

    it('hides warning when devices are selected', () => {
      renderPure(
        <DeviceSelector {...defaultProps} selectedDevices={['device-1']} />
      );

      expect(screen.queryByRole('alert')).not.toBeInTheDocument();
    });

    it('hides warning when both global and devices are selected', () => {
      renderPure(
        <DeviceSelector
          {...defaultProps}
          globalSelected={true}
          selectedDevices={['device-1']}
        />
      );

      expect(screen.queryByRole('alert')).not.toBeInTheDocument();
    });
  });

  describe('Multi-device Selection', () => {
    it('allows selecting multiple devices when multiSelect is true', async () => {
      const user = userEvent.setup();
      const onSelectionChange = vi.fn();

      renderPure(
        <DeviceSelector
          {...defaultProps}
          multiSelect={true}
          onSelectionChange={onSelectionChange}
        />
      );

      // Select first device
      const checkbox1 = screen.getByLabelText('Select device Keyboard 1');
      await user.click(checkbox1);
      expect(onSelectionChange).toHaveBeenCalledWith(['device-1'], false);

      // Select second device (should add to array)
      onSelectionChange.mockClear();
      const checkbox2 = screen.getByLabelText('Select device Keyboard 2');
      await user.click(checkbox2);
      expect(onSelectionChange).toHaveBeenCalledWith(['device-2'], false);
    });

    it('deselects device when clicking checked checkbox', async () => {
      const user = userEvent.setup();
      const onSelectionChange = vi.fn();

      renderPure(
        <DeviceSelector
          {...defaultProps}
          selectedDevices={['device-1', 'device-2']}
          onSelectionChange={onSelectionChange}
        />
      );

      const checkbox1 = screen.getByLabelText('Select device Keyboard 1');
      await user.click(checkbox1);

      expect(onSelectionChange).toHaveBeenCalledWith(['device-2'], false);
    });

    it('only allows single device when multiSelect is false', async () => {
      const user = userEvent.setup();
      const onSelectionChange = vi.fn();

      renderPure(
        <DeviceSelector
          {...defaultProps}
          multiSelect={false}
          onSelectionChange={onSelectionChange}
        />
      );

      const checkbox1 = screen.getByLabelText('Select device Keyboard 1');
      await user.click(checkbox1);

      // Should call with single device in array, not adding to existing
      expect(onSelectionChange).toHaveBeenCalledWith(['device-1'], false);
    });

    it('displays selected devices as checked', () => {
      renderPure(
        <DeviceSelector
          {...defaultProps}
          selectedDevices={['device-1', 'device-3']}
        />
      );

      const checkbox1 = screen.getByLabelText('Select device Keyboard 1');
      const checkbox2 = screen.getByLabelText('Select device Keyboard 2');
      const checkbox3 = screen.getByLabelText('Select device Keyboard 3');

      expect(checkbox1).toBeChecked();
      expect(checkbox2).not.toBeChecked();
      expect(checkbox3).toBeChecked();
    });
  });

  describe('Global Checkbox', () => {
    it('toggles global selection independently of device selection', async () => {
      const user = userEvent.setup();
      const onSelectionChange = vi.fn();

      renderPure(
        <DeviceSelector
          {...defaultProps}
          selectedDevices={['device-1']}
          onSelectionChange={onSelectionChange}
        />
      );

      const globalCheckbox = screen.getByLabelText(
        'Apply configuration globally to all devices'
      );
      await user.click(globalCheckbox);

      // Global should toggle while preserving device selection
      expect(onSelectionChange).toHaveBeenCalledWith(['device-1'], true);
    });

    it('unchecks global when clicking checked global checkbox', async () => {
      const user = userEvent.setup();
      const onSelectionChange = vi.fn();

      renderPure(
        <DeviceSelector {...defaultProps} globalSelected={true} onSelectionChange={onSelectionChange} />
      );

      const globalCheckbox = screen.getByLabelText(
        'Apply configuration globally to all devices'
      );
      await user.click(globalCheckbox);

      expect(onSelectionChange).toHaveBeenCalledWith([], false);
    });

    it('displays global as checked when globalSelected is true', () => {
      renderPure(<DeviceSelector {...defaultProps} globalSelected={true} />);

      const globalCheckbox = screen.getByLabelText(
        'Apply configuration globally to all devices'
      );
      expect(globalCheckbox).toBeChecked();
    });
  });

  describe('Device Editing', () => {
    it('renders edit button when onEditDevice is provided', () => {
      const onEditDevice = vi.fn();

      renderPure(<DeviceSelector {...defaultProps} onEditDevice={onEditDevice} />);

      const editButtons = screen.getAllByRole('button', { name: /Edit device/ });
      expect(editButtons).toHaveLength(3); // One for each device
    });

    it('does not render edit button when onEditDevice is not provided', () => {
      renderPure(<DeviceSelector {...defaultProps} />);

      expect(screen.queryByRole('button', { name: /Edit device/ })).not.toBeInTheDocument();
    });

    it('calls onEditDevice with correct device ID when edit clicked', async () => {
      const user = userEvent.setup();
      const onEditDevice = vi.fn();

      renderPure(<DeviceSelector {...defaultProps} onEditDevice={onEditDevice} />);

      const editButton1 = screen.getByRole('button', { name: 'Edit device Keyboard 1' });
      await user.click(editButton1);

      expect(onEditDevice).toHaveBeenCalledWith('device-1');
    });

    it('prevents checkbox toggle when edit button is clicked', async () => {
      const user = userEvent.setup();
      const onSelectionChange = vi.fn();
      const onEditDevice = vi.fn();

      renderPure(
        <DeviceSelector
          {...defaultProps}
          onSelectionChange={onSelectionChange}
          onEditDevice={onEditDevice}
        />
      );

      const editButton = screen.getByRole('button', { name: 'Edit device Keyboard 1' });
      await user.click(editButton);

      // Only edit should be called, not selection change
      expect(onEditDevice).toHaveBeenCalledWith('device-1');
      expect(onSelectionChange).not.toHaveBeenCalled();
    });
  });

  describe('Accessibility', () => {
    it('has proper ARIA labels for all checkboxes', () => {
      renderPure(<DeviceSelector {...defaultProps} />);

      expect(
        screen.getByLabelText('Apply configuration globally to all devices')
      ).toBeInTheDocument();
      expect(screen.getByLabelText('Select device Keyboard 1')).toBeInTheDocument();
      expect(screen.getByLabelText('Select device Keyboard 2')).toBeInTheDocument();
      expect(screen.getByLabelText('Select device Keyboard 3')).toBeInTheDocument();
    });

    it('has proper ARIA labels for connection badges', () => {
      renderPure(<DeviceSelector {...defaultProps} />);

      const connectedBadges = screen.getAllByLabelText('Device connected');
      expect(connectedBadges).toHaveLength(2);

      const disconnectedBadge = screen.getByLabelText('Device disconnected');
      expect(disconnectedBadge).toBeInTheDocument();
    });

    it('has proper ARIA labels for edit buttons', () => {
      const onEditDevice = vi.fn();
      renderPure(<DeviceSelector {...defaultProps} onEditDevice={onEditDevice} />);

      expect(screen.getByLabelText('Edit device Keyboard 1')).toBeInTheDocument();
      expect(screen.getByLabelText('Edit device Keyboard 2')).toBeInTheDocument();
      expect(screen.getByLabelText('Edit device Keyboard 3')).toBeInTheDocument();
    });

    it('warning has proper alert role and aria-live', () => {
      renderPure(<DeviceSelector {...defaultProps} />);

      const warning = screen.getByRole('alert');
      expect(warning).toHaveAttribute('aria-live', 'polite');
    });

    it('supports keyboard navigation with Tab', async () => {
      const user = userEvent.setup();
      renderPure(<DeviceSelector {...defaultProps} onEditDevice={vi.fn()} />);

      // Tab through all interactive elements
      await user.tab();
      expect(
        screen.getByLabelText('Apply configuration globally to all devices')
      ).toHaveFocus();

      await user.tab();
      expect(screen.getByLabelText('Select device Keyboard 1')).toHaveFocus();

      await user.tab();
      expect(screen.getByLabelText('Edit device Keyboard 1')).toHaveFocus();

      await user.tab();
      expect(screen.getByLabelText('Select device Keyboard 2')).toHaveFocus();

      await user.tab();
      expect(screen.getByLabelText('Edit device Keyboard 2')).toHaveFocus();
    });

    it('supports keyboard activation with Space', async () => {
      const user = userEvent.setup();
      const onSelectionChange = vi.fn();

      renderPure(
        <DeviceSelector {...defaultProps} onSelectionChange={onSelectionChange} />
      );

      const checkbox = screen.getByLabelText('Select device Keyboard 1');
      checkbox.focus();

      await user.keyboard(' '); // Space key
      expect(onSelectionChange).toHaveBeenCalledWith(['device-1'], false);
    });

    it('has visible focus indicators', () => {
      renderPure(<DeviceSelector {...defaultProps} />);

      const checkbox = screen.getByLabelText('Select device Keyboard 1');

      // Check for focus ring classes
      expect(checkbox).toHaveClass('focus:ring-2', 'focus:ring-primary-500');
    });
  });

  describe('No Scope Toggle (Regression Test)', () => {
    it('does not render any scope-related UI elements', () => {
      renderPure(<DeviceSelector {...defaultProps} />);

      // Search for common scope-related text (but not info text about mappings)
      expect(screen.queryByText(/device scope/i)).not.toBeInTheDocument();
      expect(screen.queryByText(/global scope/i)).not.toBeInTheDocument();
      expect(screen.queryByText(/scope:/i)).not.toBeInTheDocument();
      expect(screen.queryByLabelText(/scope/i)).not.toBeInTheDocument();

      // Should not have toggle switches or radio buttons for scope
      const switches = screen.queryAllByRole('switch');
      expect(switches).toHaveLength(0);

      // Only checkboxes for device/global selection should exist
      const checkboxes = screen.getAllByRole('checkbox');
      expect(checkboxes.length).toBe(4); // 1 global + 3 devices
    });

    it('does not accept or use scope property on devices', () => {
      // TypeScript will catch this at compile time, but verify runtime behavior
      const devicesWithScope = [
        {
          id: 'device-1',
          name: 'Keyboard 1',
          connected: true,
          // @ts-expect-error - Testing that scope is not used even if provided
          scope: 'device-specific',
        },
      ] as Device[];

      renderPure(<DeviceSelector {...defaultProps} devices={devicesWithScope} />);

      // Should render normally, ignoring scope
      expect(screen.getByText('Keyboard 1')).toBeInTheDocument();
      expect(screen.queryByText('device-specific')).not.toBeInTheDocument();
    });
  });

  describe('Edge Cases', () => {
    it('handles all devices disconnected', () => {
      const disconnectedDevices: Device[] = [
        { id: 'device-1', name: 'Keyboard 1', connected: false },
        { id: 'device-2', name: 'Keyboard 2', connected: false },
      ];

      renderPure(<DeviceSelector {...defaultProps} devices={disconnectedDevices} />);

      const disconnectedBadges = screen.getAllByText('Disconnected');
      expect(disconnectedBadges).toHaveLength(2);
    });

    it('handles device with minimal properties', () => {
      const minimalDevices: Device[] = [{ id: 'device-1', name: 'Minimal Device' }];

      renderPure(<DeviceSelector {...defaultProps} devices={minimalDevices} />);

      expect(screen.getByText('Minimal Device')).toBeInTheDocument();
      expect(screen.queryByText(/Serial:/)).not.toBeInTheDocument();
      expect(screen.queryByText(/Layout:/)).not.toBeInTheDocument();
      expect(screen.queryByText(/Connected/)).not.toBeInTheDocument();
    });

    it('handles long device names gracefully', () => {
      const longNameDevices: Device[] = [
        {
          id: 'device-1',
          name: 'This is a very long device name that should be truncated properly in the UI',
          connected: true,
        },
      ];

      renderPure(<DeviceSelector {...defaultProps} devices={longNameDevices} />);

      const deviceName = screen.getByText(
        'This is a very long device name that should be truncated properly in the UI'
      );
      // Check for truncate class
      expect(deviceName).toHaveClass('truncate');
    });

    it('handles rapidly toggling selections', async () => {
      const user = userEvent.setup();
      const onSelectionChange = vi.fn();

      renderPure(
        <DeviceSelector
          {...defaultProps}
          multiSelect={true}
          onSelectionChange={onSelectionChange}
        />
      );

      const checkbox1 = screen.getByLabelText('Select device Keyboard 1');

      // Rapidly click multiple times
      await user.click(checkbox1);
      await user.click(checkbox1);
      await user.click(checkbox1);

      // Should call handler each time with correct state
      expect(onSelectionChange).toHaveBeenCalledTimes(3);
    });
  });

  describe('Visual Styling', () => {
    it('applies hover styles to device items', () => {
      renderPure(<DeviceSelector {...defaultProps} />);

      const deviceLabels = screen.getAllByRole('checkbox').map((cb) => cb.closest('label'));
      deviceLabels.forEach((label) => {
        expect(label).toHaveClass('hover:bg-slate-700');
      });
    });

    it('applies proper color contrast for badges', () => {
      renderPure(<DeviceSelector {...defaultProps} />);

      // Connected badge - green with proper contrast
      const connectedBadges = screen.getAllByText('Connected');
      connectedBadges.forEach((badge) => {
        expect(badge).toHaveClass('text-green-300');
        expect(badge.className).toContain('bg-green-900');
      });

      // Disconnected badge - gray with proper contrast
      const disconnectedBadge = screen.getByText('Disconnected');
      expect(disconnectedBadge).toHaveClass('text-gray-400');
      expect(disconnectedBadge.className).toContain('bg-gray-700');
    });
  });
});
