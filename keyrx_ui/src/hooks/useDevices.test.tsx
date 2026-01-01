import { renderHook, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import {
  useDevices,
  useRenameDevice,
  useSetDeviceScope,
  useForgetDevice,
} from './useDevices';
import * as deviceApi from '../api/devices';
import type { DeviceEntry, DeviceScope } from '../types';

// Mock API module
vi.mock('../api/devices');

// Test wrapper with QueryClient
function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  });

  return ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
}

describe('useDevices', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('fetches devices successfully', async () => {
    const mockDevices: DeviceEntry[] = [
      {
        id: 'device-1',
        name: 'Keyboard 1',
        scope: 'global' as DeviceScope,
        layout: 'ANSI_104',
        connected: true,
      },
    ];

    vi.mocked(deviceApi.fetchDevices).mockResolvedValue(mockDevices);

    const { result } = renderHook(() => useDevices(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toEqual(mockDevices);
    expect(deviceApi.fetchDevices).toHaveBeenCalledTimes(1);
  });

  it('handles fetch error', async () => {
    vi.mocked(deviceApi.fetchDevices).mockRejectedValue(
      new Error('Network error')
    );

    const { result } = renderHook(() => useDevices(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isError).toBe(true));

    expect(result.current.error).toBeTruthy();
  });
});

describe('useRenameDevice', () => {
  it('renames device with optimistic update', async () => {
    const mockDevices: DeviceEntry[] = [
      {
        id: 'device-1',
        name: 'Old Name',
        scope: 'global' as DeviceScope,
        layout: 'ANSI_104',
        connected: true,
      },
    ];

    vi.mocked(deviceApi.fetchDevices).mockResolvedValue(mockDevices);
    vi.mocked(deviceApi.renameDevice).mockResolvedValue();

    const wrapper = createWrapper();

    // First fetch devices
    const { result: devicesResult } = renderHook(() => useDevices(), {
      wrapper,
    });
    await waitFor(() => expect(devicesResult.current.isSuccess).toBe(true));

    // Then rename
    const { result: mutationResult } = renderHook(() => useRenameDevice(), {
      wrapper,
    });

    mutationResult.current.mutate({ id: 'device-1', name: 'New Name' });

    await waitFor(() => expect(mutationResult.current.isSuccess).toBe(true));

    expect(deviceApi.renameDevice).toHaveBeenCalledWith('device-1', 'New Name');
  });

  it('rolls back on error', async () => {
    const mockDevices: DeviceEntry[] = [
      {
        id: 'device-1',
        name: 'Original Name',
        scope: 'global' as DeviceScope,
        layout: 'ANSI_104',
        connected: true,
      },
    ];

    vi.mocked(deviceApi.fetchDevices).mockResolvedValue(mockDevices);
    vi.mocked(deviceApi.renameDevice).mockRejectedValue(
      new Error('API error')
    );

    const wrapper = createWrapper();

    // First fetch devices
    const { result: devicesResult } = renderHook(() => useDevices(), {
      wrapper,
    });
    await waitFor(() => expect(devicesResult.current.isSuccess).toBe(true));

    // Then attempt rename
    const { result: mutationResult } = renderHook(() => useRenameDevice(), {
      wrapper,
    });

    mutationResult.current.mutate({ id: 'device-1', name: 'New Name' });

    await waitFor(() => expect(mutationResult.current.isError).toBe(true));

    // Data should be rolled back
    expect(devicesResult.current.data?.[0].name).toBe('Original Name');
  });
});

describe('useSetDeviceScope', () => {
  it('sets device scope with optimistic update', async () => {
    const mockDevices: DeviceEntry[] = [
      {
        id: 'device-1',
        name: 'Keyboard',
        scope: 'global' as DeviceScope,
        layout: 'ANSI_104',
        connected: true,
      },
    ];

    vi.mocked(deviceApi.fetchDevices).mockResolvedValue(mockDevices);
    vi.mocked(deviceApi.setDeviceScope).mockResolvedValue();

    const wrapper = createWrapper();

    const { result: devicesResult } = renderHook(() => useDevices(), {
      wrapper,
    });
    await waitFor(() => expect(devicesResult.current.isSuccess).toBe(true));

    const { result: mutationResult } = renderHook(
      () => useSetDeviceScope(),
      { wrapper }
    );

    mutationResult.current.mutate({
      id: 'device-1',
      scope: 'profile' as DeviceScope,
    });

    await waitFor(() => expect(mutationResult.current.isSuccess).toBe(true));

    expect(deviceApi.setDeviceScope).toHaveBeenCalledWith(
      'device-1',
      'profile'
    );
  });
});

describe('useForgetDevice', () => {
  it('forgets device with optimistic update', async () => {
    const mockDevices: DeviceEntry[] = [
      {
        id: 'device-1',
        name: 'Keyboard 1',
        scope: 'global' as DeviceScope,
        layout: 'ANSI_104',
        connected: true,
      },
      {
        id: 'device-2',
        name: 'Keyboard 2',
        scope: 'global' as DeviceScope,
        layout: 'ANSI_104',
        connected: true,
      },
    ];

    vi.mocked(deviceApi.fetchDevices).mockResolvedValue(mockDevices);
    vi.mocked(deviceApi.forgetDevice).mockResolvedValue();

    const wrapper = createWrapper();

    const { result: devicesResult } = renderHook(() => useDevices(), {
      wrapper,
    });
    await waitFor(() => expect(devicesResult.current.isSuccess).toBe(true));

    const { result: mutationResult } = renderHook(() => useForgetDevice(), {
      wrapper,
    });

    mutationResult.current.mutate('device-1');

    await waitFor(() => expect(mutationResult.current.isSuccess).toBe(true));

    expect(deviceApi.forgetDevice).toHaveBeenCalledWith('device-1');
  });
});
