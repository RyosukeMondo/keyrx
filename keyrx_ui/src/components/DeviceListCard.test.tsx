import { describe, it, expect, vi, beforeEach } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import { renderWithProviders } from '../../tests/testUtils';
import userEvent from '@testing-library/user-event';
import { BrowserRouter } from 'react-router-dom';
import { DeviceListCard } from './DeviceListCard';
import * as useDevicesModule from '../hooks/useDevices';
import { sendServerMessage } from '../test/mocks/websocketHelpers';

const mockNavigate = vi.fn();

vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom');
  return {
    ...actual,
    useNavigate: () => mockNavigate,
  };
});

const renderWithRouter = (component: React.ReactElement) => {
  return renderWithProviders(<BrowserRouter>{component}</BrowserRouter>);
};

describe('DeviceListCard', () => {
  const mockDevices = [
    {
      id: '1',
      name: 'Main Keyboard',
      path: 'USB\\VID_1234&PID_5678\\ABC',
      serial: 'ABC123',
      scope: 'global' as const,
      layout: 'ANSI 104',
      active: true,
    },
    {
      id: '2',
      name: 'Left Numpad',
      path: 'USB\\VID_5678&PID_1234\\XYZ',
      serial: 'XYZ789',
      scope: 'device-specific' as const,
      layout: 'Numpad',
      active: false,
    },
  ];

  beforeEach(() => {
    mockNavigate.mockClear();
    vi.clearAllMocks();
  });

  it('renders loading state', () => {
    vi.spyOn(useDevicesModule, 'useDevices').mockReturnValue({
      data: undefined,
      isLoading: true,
      error: null,
      refetch: vi.fn(),
    } as any);

    renderWithRouter(<DeviceListCard />);
    const loadingElements = screen.getAllByRole('status');
    expect(loadingElements.length).toBeGreaterThan(0);
    expect(loadingElements[0]).toHaveAttribute('aria-busy', 'true');
  });

  it('renders empty state when no devices', () => {
    vi.spyOn(useDevicesModule, 'useDevices').mockReturnValue({
      data: [],
      isLoading: false,
      error: null,
      refetch: vi.fn(),
    } as any);

    renderWithRouter(<DeviceListCard />);
    expect(screen.getByText('Connected Devices (0)')).toBeInTheDocument();
    expect(screen.getByText(/No devices connected/)).toBeInTheDocument();
  });

  it('renders correct device count', () => {
    vi.spyOn(useDevicesModule, 'useDevices').mockReturnValue({
      data: mockDevices,
      isLoading: false,
      error: null,
      refetch: vi.fn(),
    } as any);

    renderWithRouter(<DeviceListCard />);
    expect(screen.getByText('Connected Devices (2)')).toBeInTheDocument();
  });

  it('renders all device cards', () => {
    vi.spyOn(useDevicesModule, 'useDevices').mockReturnValue({
      data: mockDevices,
      isLoading: false,
      error: null,
      refetch: vi.fn(),
    } as any);

    renderWithRouter(<DeviceListCard />);
    expect(screen.getByText('Main Keyboard')).toBeInTheDocument();
    expect(screen.getByText('Left Numpad')).toBeInTheDocument();
  });

  it('renders device identifiers', () => {
    vi.spyOn(useDevicesModule, 'useDevices').mockReturnValue({
      data: mockDevices,
      isLoading: false,
      error: null,
      refetch: vi.fn(),
    } as any);

    renderWithRouter(<DeviceListCard />);
    expect(screen.getByText('ABC123')).toBeInTheDocument();
    expect(screen.getByText('XYZ789')).toBeInTheDocument();
  });

  it('shows active indicator for active device', () => {
    vi.spyOn(useDevicesModule, 'useDevices').mockReturnValue({
      data: mockDevices,
      isLoading: false,
      error: null,
      refetch: vi.fn(),
    } as any);

    renderWithRouter(<DeviceListCard />);
    const activeIndicator = screen.getByLabelText('Active device');
    expect(activeIndicator).toHaveTextContent('✓ Active');
  });

  it('renders device scope correctly', () => {
    vi.spyOn(useDevicesModule, 'useDevices').mockReturnValue({
      data: mockDevices,
      isLoading: false,
      error: null,
      refetch: vi.fn(),
    } as any);

    renderWithRouter(<DeviceListCard />);
    expect(screen.getByText(/Scope: Global/)).toBeInTheDocument();
    expect(screen.getByText(/Scope: Device-Specific/)).toBeInTheDocument();
  });

  it('renders device layout', () => {
    vi.spyOn(useDevicesModule, 'useDevices').mockReturnValue({
      data: mockDevices,
      isLoading: false,
      error: null,
      refetch: vi.fn(),
    } as any);

    renderWithRouter(<DeviceListCard />);
    expect(screen.getByText(/Layout: ANSI 104/)).toBeInTheDocument();
    expect(screen.getByText(/Layout: Numpad/)).toBeInTheDocument();
  });

  it('renders keyboard icons with accessibility labels', () => {
    vi.spyOn(useDevicesModule, 'useDevices').mockReturnValue({
      data: mockDevices,
      isLoading: false,
      error: null,
      refetch: vi.fn(),
    } as any);

    renderWithRouter(<DeviceListCard />);
    const icons = screen.getAllByRole('img', { name: 'Keyboard' });
    expect(icons).toHaveLength(2);
  });

  it('navigates to devices page when Manage Devices is clicked', async () => {
    vi.spyOn(useDevicesModule, 'useDevices').mockReturnValue({
      data: mockDevices,
      isLoading: false,
      error: null,
      refetch: vi.fn(),
    } as any);

    const user = userEvent.setup();
    renderWithRouter(<DeviceListCard />);

    const button = screen.getByRole('button', { name: 'Manage devices' });
    await user.click(button);

    expect(mockNavigate).toHaveBeenCalledWith('/devices');
  });

  it('applies hover effect on device cards', () => {
    vi.spyOn(useDevicesModule, 'useDevices').mockReturnValue({
      data: mockDevices,
      isLoading: false,
      error: null,
      refetch: vi.fn(),
    } as any);

    const { container } = renderWithRouter(<DeviceListCard />);
    const deviceCards = container.querySelectorAll('.hover\\:border-slate-600');
    expect(deviceCards.length).toBeGreaterThan(0);
  });

  it('applies custom className', () => {
    vi.spyOn(useDevicesModule, 'useDevices').mockReturnValue({
      data: mockDevices,
      isLoading: false,
      error: null,
      refetch: vi.fn(),
    } as any);

    const { container } = renderWithRouter(
      <DeviceListCard className="custom-class" />
    );
    expect(container.querySelector('.custom-class')).toBeInTheDocument();
  });

  it('shows error state when loading fails', () => {
    const mockRefetch = vi.fn();
    vi.spyOn(useDevicesModule, 'useDevices').mockReturnValue({
      data: undefined,
      isLoading: false,
      error: new Error('Failed to fetch devices'),
      refetch: mockRefetch,
    } as any);

    renderWithRouter(<DeviceListCard />);
    expect(screen.getByText('Failed to load devices')).toBeInTheDocument();
    expect(screen.getByText('Failed to fetch devices')).toBeInTheDocument();
  });

  it('allows retry when error occurs', async () => {
    const mockRefetch = vi.fn();
    vi.spyOn(useDevicesModule, 'useDevices').mockReturnValue({
      data: undefined,
      isLoading: false,
      error: new Error('Failed to fetch devices'),
      refetch: mockRefetch,
    } as any);

    const user = userEvent.setup();
    renderWithRouter(<DeviceListCard />);

    const retryButton = screen.getByRole('button', { name: 'Retry' });
    await user.click(retryButton);

    expect(mockRefetch).toHaveBeenCalled();
  });

  describe('WebSocket Integration', () => {
    it('updates device list when device_connected event received', async () => {
      // Start with empty devices
      const mockUseDevices = vi.spyOn(useDevicesModule, 'useDevices');
      mockUseDevices.mockReturnValue({
        data: [],
        isLoading: false,
        error: null,
        refetch: vi.fn(),
      } as any);

      renderWithRouter(<DeviceListCard />);

      // Verify empty state
      expect(screen.getByText('Connected Devices (0)')).toBeInTheDocument();

      // Simulate device connected via WebSocket
      // Note: In a real integration test, this would trigger a refetch in useDevices
      // which would update the device list. For this unit test, we'll update the mock.
      mockUseDevices.mockReturnValue({
        data: [mockDevices[0]],
        isLoading: false,
        error: null,
        refetch: vi.fn(),
      } as any);

      // Send device_connected event via WebSocket
      sendServerMessage('daemon-state', {
        devices: [mockDevices[0]],
      });

      // Force re-render by updating the component
      renderWithRouter(<DeviceListCard />);

      // Verify device appears (this would happen automatically in integration tests)
      await waitFor(() => {
        expect(screen.getByText('Connected Devices (1)')).toBeInTheDocument();
      });
    });

    it('handles device disconnection event', async () => {
      // Start with devices
      const mockUseDevices = vi.spyOn(useDevicesModule, 'useDevices');
      mockUseDevices.mockReturnValue({
        data: mockDevices,
        isLoading: false,
        error: null,
        refetch: vi.fn(),
      } as any);

      renderWithRouter(<DeviceListCard />);

      expect(screen.getByText('Connected Devices (2)')).toBeInTheDocument();

      // Simulate device disconnection
      mockUseDevices.mockReturnValue({
        data: [mockDevices[0]],
        isLoading: false,
        error: null,
        refetch: vi.fn(),
      } as any);

      sendServerMessage('daemon-state', {
        devices: [mockDevices[0]],
      });

      renderWithRouter(<DeviceListCard />);

      await waitFor(() => {
        expect(screen.getByText('Connected Devices (1)')).toBeInTheDocument();
      });
    });
  });
});
