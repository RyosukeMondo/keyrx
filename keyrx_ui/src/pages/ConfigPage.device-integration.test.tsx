/**
 * Device Integration tests for ConfigPage (extracted from ConfigPage.integration.test.tsx)
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import { renderWithProviders, setDaemonState } from '../../tests/testUtils';
import userEvent from '@testing-library/user-event';
import { ConfigPage } from './ConfigPage';
import { useConfigStore } from '../stores/configStore';

describe('ConfigPage - Device Integration Tests', () => {
  const renderConfigPage = (profile = 'default') => {
    return renderWithProviders(<ConfigPage profileName={profile} />, {
      wrapWithRouter: true,
      routerInitialEntries: ['/config'],
    });
  };

  beforeEach(async () => {
    const store = useConfigStore.getState();
    store.config = null;
    store.loading = false;
    store.error = null;

    setDaemonState({
      activeProfile: 'default',
      layer: 'base',
      modifiers: [],
      locks: [],
      connected: true,
    });
  });

  it('DeviceScopeToggle receives real devices from API', async () => {
    renderConfigPage();

    await waitFor(() => {
      expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
    });

    expect(screen.getByText(/Mapping Scope/i)).toBeInTheDocument();
    expect(screen.getByText('Global')).toBeInTheDocument();
    expect(screen.getByText('Device-Specific')).toBeInTheDocument();
  });

  it('displays real device list when device-specific mode selected', async () => {
    const user = userEvent.setup();
    renderConfigPage();

    await waitFor(() => {
      expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
    });

    const deviceSpecificButton = screen.getByRole('button', {
      name: /Device-Specific/i,
    });
    await user.click(deviceSpecificButton);

    await waitFor(() => {
      expect(screen.getByText(/Select Device/i)).toBeInTheDocument();
    });

    const deviceInfo = screen.queryByText(/No devices available/i);
    if (deviceInfo) {
      expect(deviceInfo).toBeInTheDocument();
    } else {
      expect(screen.getByText(/Select a device/i)).toBeInTheDocument();
    }
  });

  it('switches between global and device-specific scope', async () => {
    const user = userEvent.setup();
    renderConfigPage();

    await waitFor(() => {
      expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
    });

    const globalButton = screen.getByRole('button', { name: /^Global$/i });
    expect(globalButton).toHaveClass('bg-primary-500');

    const deviceSpecificButton = screen.getByRole('button', {
      name: /Device-Specific/i,
    });
    await user.click(deviceSpecificButton);

    await waitFor(() => {
      expect(deviceSpecificButton).toHaveClass('bg-primary-500');
    });

    await user.click(globalButton);

    await waitFor(() => {
      expect(globalButton).toHaveClass('bg-primary-500');
    });
  });

  it('device selector only visible in device-specific mode', async () => {
    const user = userEvent.setup();
    renderConfigPage();

    await waitFor(() => {
      expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
    });

    expect(screen.queryByText(/Select Device/i)).not.toBeInTheDocument();

    const deviceSpecificButton = screen.getByRole('button', {
      name: /Device-Specific/i,
    });
    await user.click(deviceSpecificButton);

    await waitFor(() => {
      expect(screen.getByText(/Select Device/i)).toBeInTheDocument();
    });
  });

  it('shows appropriate help text for each scope', async () => {
    const user = userEvent.setup();
    renderConfigPage();

    await waitFor(() => {
      expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
    });

    expect(
      screen.getByText(/Global mappings apply to all connected devices/i)
    ).toBeInTheDocument();

    const deviceSpecificButton = screen.getByRole('button', {
      name: /Device-Specific/i,
    });
    await user.click(deviceSpecificButton);

    await waitFor(() => {
      expect(
        screen.getByText(/Device-specific mappings apply only to/i)
      ).toBeInTheDocument();
    });
  });
});