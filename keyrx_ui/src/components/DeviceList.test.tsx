import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { renderWithProviders } from '../../tests/testUtils';
import DeviceList from './DeviceList';

// Mock device data
const mockDevices = [
  {
    id: 'device1',
    name: 'Keyboard 1',
    path: '/dev/input/event0',
    serial: 'SN12345',
    active: true,
  },
  {
    id: 'device2',
    name: 'Keyboard 2',
    path: '/dev/input/event1',
    serial: null,
    active: false,
  },
  {
    id: 'device3',
    name: 'Keyboard 3',
    path: '/dev/input/event2',
    serial: 'SN67890',
    active: true,
  },
];

// Mock WebSocket
class MockWebSocket {
  static instances: MockWebSocket[] = [];

  url: string;
  readyState: number = WebSocket.CONNECTING;
  onopen: ((event: Event) => void) | null = null;
  onclose: ((event: CloseEvent) => void) | null = null;
  onerror: ((event: Event) => void) | null = null;
  onmessage: ((event: MessageEvent) => void) | null = null;

  constructor(url: string) {
    this.url = url;
    MockWebSocket.instances.push(this);
    // Simulate connection opening after construction
    setTimeout(() => {
      this.readyState = WebSocket.OPEN;
      if (this.onopen) {
        this.onopen(new Event('open'));
      }
    }, 0);
  }

  send(data: string) {
    // Mock send
  }

  close() {
    this.readyState = WebSocket.CLOSED;
    if (this.onclose) {
      this.onclose(new CloseEvent('close'));
    }
  }

  // Helper method to simulate receiving a message
  simulateMessage(data: any) {
    if (this.onmessage) {
      this.onmessage(new MessageEvent('message', { data: JSON.stringify(data) }));
    }
  }

  // Helper method to simulate receiving non-JSON message
  simulateNonJsonMessage(data: string) {
    if (this.onmessage) {
      this.onmessage(new MessageEvent('message', { data }));
    }
  }

  // Helper method to simulate error
  simulateError() {
    if (this.onerror) {
      this.onerror(new Event('error'));
    }
  }

  static reset() {
    MockWebSocket.instances = [];
  }

  static getLastInstance(): MockWebSocket | undefined {
    return MockWebSocket.instances[MockWebSocket.instances.length - 1];
  }
}

describe('DeviceList', () => {
  let originalWebSocket: typeof WebSocket;
  let fetchMock: ReturnType<typeof vi.fn>;

  beforeEach(() => {
    // Store original WebSocket and replace with mock
    originalWebSocket = global.WebSocket;
    global.WebSocket = MockWebSocket as any;
    MockWebSocket.reset();

    // Mock fetch
    fetchMock = vi.fn();
    global.fetch = fetchMock;
  });

  afterEach(() => {
    // Restore original WebSocket
    global.WebSocket = originalWebSocket;
    vi.restoreAllMocks();
  });

  describe('Device Rendering', () => {
    it('should render loading state initially', () => {
      fetchMock.mockImplementation(() => new Promise(() => {})); // Never resolves

      renderWithProviders(<DeviceList />);

      expect(screen.getByText('Connected Devices')).toBeInTheDocument();
      expect(screen.getByText('Loading devices...')).toBeInTheDocument();
    });

    it('should render list of devices after successful fetch', async () => {
      fetchMock.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ devices: mockDevices }),
      });

      renderWithProviders(<DeviceList />);

      await waitFor(() => {
        expect(screen.getByText('Keyboard 1')).toBeInTheDocument();
      });

      expect(screen.getByText('Keyboard 2')).toBeInTheDocument();
      expect(screen.getByText('Keyboard 3')).toBeInTheDocument();
    });

    it('should display device names correctly', async () => {
      fetchMock.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ devices: mockDevices }),
      });

      renderWithProviders(<DeviceList />);

      await waitFor(() => {
        expect(screen.getByText('Keyboard 1')).toBeInTheDocument();
      });

      mockDevices.forEach(device => {
        expect(screen.getByText(device.name)).toBeInTheDocument();
      });
    });

    it('should display device serial numbers', async () => {
      fetchMock.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ devices: mockDevices }),
      });

      renderWithProviders(<DeviceList />);

      await waitFor(() => {
        expect(screen.getByText('SN12345')).toBeInTheDocument();
      });

      expect(screen.getByText('SN67890')).toBeInTheDocument();
    });

    it('should display N/A for devices without serial numbers', async () => {
      fetchMock.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ devices: mockDevices }),
      });

      renderWithProviders(<DeviceList />);

      await waitFor(() => {
        expect(screen.getByText('N/A')).toBeInTheDocument();
      });
    });

    it('should display device paths', async () => {
      fetchMock.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ devices: mockDevices }),
      });

      renderWithProviders(<DeviceList />);

      await waitFor(() => {
        expect(screen.getByText('/dev/input/event0')).toBeInTheDocument();
      });

      expect(screen.getByText('/dev/input/event1')).toBeInTheDocument();
      expect(screen.getByText('/dev/input/event2')).toBeInTheDocument();
    });

    it('should show device connection status', async () => {
      fetchMock.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ devices: mockDevices }),
      });

      renderWithProviders(<DeviceList />);

      await waitFor(() => {
        const connectedElements = screen.getAllByText('Connected');
        expect(connectedElements.length).toBe(2); // devices 1 and 3
      });

      expect(screen.getByText('Disconnected')).toBeInTheDocument(); // device 2
    });

    it('should display empty state when no devices are found', async () => {
      fetchMock.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ devices: [] }),
      });

      renderWithProviders(<DeviceList />);

      await waitFor(() => {
        expect(screen.getByText('No keyboard devices found.')).toBeInTheDocument();
      });

      expect(screen.getByText('Make sure the daemon has permission to access input devices.')).toBeInTheDocument();
    });
  });

  describe('Error Handling', () => {
    it('should display error state when fetch fails', async () => {
      fetchMock.mockRejectedValueOnce(new Error('Network error'));

      renderWithProviders(<DeviceList />);

      await waitFor(() => {
        expect(screen.getByText(/Failed to fetch devices: Network error/)).toBeInTheDocument();
      });

      expect(screen.getByRole('button', { name: 'Retry' })).toBeInTheDocument();
    });

    it('should display error for HTTP error responses', async () => {
      fetchMock.mockResolvedValueOnce({
        ok: false,
        status: 500,
        statusText: 'Internal Server Error',
      });

      renderWithProviders(<DeviceList />);

      await waitFor(() => {
        expect(screen.getByText(/Failed to fetch devices: HTTP 500: Internal Server Error/)).toBeInTheDocument();
      });
    });

    it('should retry fetching devices when retry button is clicked', async () => {
      fetchMock
        .mockRejectedValueOnce(new Error('Network error'))
        .mockResolvedValueOnce({
          ok: true,
          json: async () => ({ devices: mockDevices }),
        });

      const user = userEvent.setup();
      renderWithProviders(<DeviceList />);

      await waitFor(() => {
        expect(screen.getByText(/Failed to fetch devices/)).toBeInTheDocument();
      });

      const retryButton = screen.getByRole('button', { name: 'Retry' });
      await user.click(retryButton);

      await waitFor(() => {
        expect(screen.getByText('Keyboard 1')).toBeInTheDocument();
      });
    });
  });

  describe('WebSocket Connection', () => {
    it('should establish WebSocket connection on mount', async () => {
      fetchMock.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ devices: mockDevices }),
      });

      renderWithProviders(<DeviceList wsUrl="ws://localhost:9867/ws" />);

      await waitFor(() => {
        const ws = MockWebSocket.getLastInstance();
        expect(ws).toBeDefined();
        expect(ws?.url).toBe('ws://localhost:9867/ws');
      });
    });

    it('should use custom WebSocket URL from props', async () => {
      fetchMock.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ devices: mockDevices }),
      });

      renderWithProviders(<DeviceList wsUrl="ws://custom:8080/ws" />);

      await waitFor(() => {
        const ws = MockWebSocket.getLastInstance();
        expect(ws?.url).toBe('ws://custom:8080/ws');
      });
    });

    it('should receive and display real-time device updates', async () => {
      fetchMock.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ devices: mockDevices }),
      });

      renderWithProviders(<DeviceList />);

      await waitFor(() => {
        expect(screen.getByText('Keyboard 1')).toBeInTheDocument();
      });

      // Get WebSocket instance
      const ws = MockWebSocket.getLastInstance();
      expect(ws).toBeDefined();

      // Simulate device activity message
      ws!.simulateMessage({ device_id: 'device1' });

      // Wait for the active class to be applied
      await waitFor(() => {
        const rows = screen.getAllByRole('row');
        const device1Row = rows.find(row => row.textContent?.includes('Keyboard 1'));
        expect(device1Row).toHaveClass('active');
      });

      // Wait for timeout to clear active state (500ms)
      await waitFor(() => {
        const rows = screen.getAllByRole('row');
        const device1Row = rows.find(row => row.textContent?.includes('Keyboard 1'));
        expect(device1Row).not.toHaveClass('active');
      }, { timeout: 1000 });
    });

    it('should handle malformed WebSocket messages gracefully', async () => {
      fetchMock.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ devices: mockDevices }),
      });

      const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
      const consoleDebugSpy = vi.spyOn(console, 'debug').mockImplementation(() => {});

      renderWithProviders(<DeviceList />);

      await waitFor(() => {
        expect(screen.getByText('Keyboard 1')).toBeInTheDocument();
      });

      const ws = MockWebSocket.getLastInstance();
      expect(ws).toBeDefined();

      // Simulate non-JSON message (should be logged but not throw)
      ws!.simulateNonJsonMessage('invalid json {');

      // Component should still be functional
      expect(screen.getByText('Keyboard 1')).toBeInTheDocument();

      // Should not have thrown an error
      expect(consoleErrorSpy).not.toHaveBeenCalled();

      // Should have logged the non-JSON message in dev mode
      if (import.meta.env.DEV) {
        expect(consoleDebugSpy).toHaveBeenCalledWith(
          'Received non-JSON WebSocket message:',
          expect.objectContaining({
            message: 'invalid json {',
            error: expect.any(String)
          })
        );
      }

      consoleErrorSpy.mockRestore();
      consoleDebugSpy.mockRestore();
    });

    it('should handle WebSocket messages without device_id', async () => {
      fetchMock.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ devices: mockDevices }),
      });

      renderWithProviders(<DeviceList />);

      await waitFor(() => {
        expect(screen.getByText('Keyboard 1')).toBeInTheDocument();
      });

      const ws = MockWebSocket.getLastInstance();

      // Simulate message without device_id
      ws!.simulateMessage({ some_other_field: 'value' });

      // Component should still be functional, no active device
      await waitFor(() => {
        const rows = screen.getAllByRole('row');
        const activeRows = rows.filter(row => row.className.includes('active'));
        expect(activeRows.length).toBe(0);
      });
    });

    it('should handle WebSocket reconnection on close', async () => {
      fetchMock.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ devices: mockDevices }),
      });

      renderWithProviders(<DeviceList />);

      await waitFor(() => {
        expect(MockWebSocket.instances.length).toBe(1);
      });

      const firstWs = MockWebSocket.getLastInstance();

      // Simulate close
      firstWs!.close();

      // Should create a new WebSocket instance after 5 seconds
      await waitFor(() => {
        expect(MockWebSocket.instances.length).toBe(2);
      }, { timeout: 6000 });
    });

    it('should handle WebSocket reconnection on error', async () => {
      fetchMock.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ devices: mockDevices }),
      });

      const consoleDebugSpy = vi.spyOn(console, 'debug').mockImplementation(() => {});

      renderWithProviders(<DeviceList />);

      await waitFor(() => {
        expect(MockWebSocket.instances.length).toBe(1);
      });

      const firstWs = MockWebSocket.getLastInstance();

      // Simulate error
      firstWs!.simulateError();

      // Should attempt reconnection
      await waitFor(() => {
        expect(MockWebSocket.instances.length).toBeGreaterThan(1);
      }, { timeout: 6000 });

      consoleDebugSpy.mockRestore();
    });

    it('should log WebSocket connection failures in dev mode', async () => {
      fetchMock.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ devices: mockDevices }),
      });

      const consoleDebugSpy = vi.spyOn(console, 'debug').mockImplementation(() => {});

      // Mock WebSocket to throw an error on construction
      const OriginalWebSocket = global.WebSocket;
      global.WebSocket = class ThrowingWebSocket {
        constructor() {
          throw new Error('WebSocket connection failed');
        }
      } as any;

      renderWithProviders(<DeviceList wsUrl="ws://localhost:9867/ws" />);

      await waitFor(() => {
        expect(screen.getByText('Keyboard 1')).toBeInTheDocument();
      });

      // Should have logged the connection error in dev mode
      if (import.meta.env.DEV) {
        await waitFor(() => {
          expect(consoleDebugSpy).toHaveBeenCalledWith(
            'WebSocket connection failed, scheduling reconnection:',
            expect.objectContaining({
              wsUrl: 'ws://localhost:9867/ws',
              reconnectDelay: 5000,
              error: expect.any(String)
            })
          );
        });
      }

      // Restore original WebSocket
      global.WebSocket = OriginalWebSocket;
      consoleDebugSpy.mockRestore();
    });

    it('should clear active device timeout on multiple updates', async () => {
      fetchMock.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ devices: mockDevices }),
      });

      renderWithProviders(<DeviceList />);

      await waitFor(() => {
        expect(screen.getByText('Keyboard 1')).toBeInTheDocument();
      });

      const ws = MockWebSocket.getLastInstance();

      // Send first device activity
      ws!.simulateMessage({ device_id: 'device1' });

      await waitFor(() => {
        const rows = screen.getAllByRole('row');
        const device1Row = rows.find(row => row.textContent?.includes('Keyboard 1'));
        expect(device1Row).toHaveClass('active');
      });

      // Send second device activity before timeout expires
      ws!.simulateMessage({ device_id: 'device2' });

      await waitFor(() => {
        const rows = screen.getAllByRole('row');
        const device2Row = rows.find(row => row.textContent?.includes('Keyboard 2'));
        expect(device2Row).toHaveClass('active');
      });

      // device1 should no longer be active
      const rows = screen.getAllByRole('row');
      const device1Row = rows.find(row => row.textContent?.includes('Keyboard 1'));
      expect(device1Row).not.toHaveClass('active');

      // After timeout, device2 should become inactive
      await waitFor(() => {
        const rows = screen.getAllByRole('row');
        const device2Row = rows.find(row => row.textContent?.includes('Keyboard 2'));
        expect(device2Row).not.toHaveClass('active');
      }, { timeout: 1000 });
    });
  });

  describe('API Integration', () => {
    it('should use custom API base URL from props', async () => {
      fetchMock.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ devices: mockDevices }),
      });

      renderWithProviders(<DeviceList apiBaseUrl="http://custom:8080/api" />);

      await waitFor(() => {
        expect(fetchMock).toHaveBeenCalledWith('http://custom:8080/api/devices');
      });
    });

    it('should use default API base URL', async () => {
      fetchMock.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ devices: mockDevices }),
      });

      renderWithProviders(<DeviceList />);

      await waitFor(() => {
        expect(fetchMock).toHaveBeenCalledWith('http://localhost:9867/api/devices');
      });
    });
  });

  describe('Cleanup', () => {
    it('should cleanup WebSocket and timers on unmount', async () => {
      fetchMock.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ devices: mockDevices }),
      });

      const { unmount } = renderWithProviders(<DeviceList />);

      await waitFor(() => {
        expect(MockWebSocket.instances.length).toBe(1);
      });

      const ws = MockWebSocket.getLastInstance();
      const closeSpy = vi.spyOn(ws!, 'close');

      // Simulate active device
      ws!.simulateMessage({ device_id: 'device1' });

      // Unmount component
      unmount();

      // WebSocket should be closed
      expect(closeSpy).toHaveBeenCalled();
    });
  });
});
