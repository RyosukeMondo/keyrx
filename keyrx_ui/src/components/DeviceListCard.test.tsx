import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { BrowserRouter } from 'react-router-dom';
import { DeviceListCard } from './DeviceListCard';

const mockNavigate = vi.fn();

vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom');
  return {
    ...actual,
    useNavigate: () => mockNavigate,
  };
});

const renderWithRouter = (component: React.ReactElement) => {
  return render(<BrowserRouter>{component}</BrowserRouter>);
};

describe('DeviceListCard', () => {
  const mockDevices = [
    {
      id: '1',
      name: 'Main Keyboard',
      identifier: 'USB\\VID_1234&PID_5678\\ABC',
      scope: 'global' as const,
      layout: 'ANSI 104',
      active: true,
    },
    {
      id: '2',
      name: 'Left Numpad',
      identifier: 'USB\\VID_5678&PID_1234\\XYZ',
      scope: 'device-specific' as const,
      layout: 'Numpad',
      active: false,
    },
  ];

  beforeEach(() => {
    mockNavigate.mockClear();
  });

  it('renders loading state', () => {
    renderWithRouter(<DeviceListCard loading={true} />);
    const loadingElements = screen.getAllByRole('generic');
    const hasAnimatePulse = loadingElements.some((el) =>
      el.classList.contains('animate-pulse')
    );
    expect(hasAnimatePulse).toBe(true);
  });

  it('renders empty state when no devices', () => {
    renderWithRouter(<DeviceListCard devices={[]} />);
    expect(screen.getByText('Connected Devices (0)')).toBeInTheDocument();
    expect(
      screen.getByText(/No devices connected/)
    ).toBeInTheDocument();
  });

  it('renders correct device count', () => {
    renderWithRouter(<DeviceListCard devices={mockDevices} />);
    expect(screen.getByText('Connected Devices (2)')).toBeInTheDocument();
  });

  it('renders all device cards', () => {
    renderWithRouter(<DeviceListCard devices={mockDevices} />);
    expect(screen.getByText('Main Keyboard')).toBeInTheDocument();
    expect(screen.getByText('Left Numpad')).toBeInTheDocument();
  });

  it('renders device identifiers', () => {
    renderWithRouter(<DeviceListCard devices={mockDevices} />);
    expect(
      screen.getByText('USB\\VID_1234&PID_5678\\ABC')
    ).toBeInTheDocument();
    expect(
      screen.getByText('USB\\VID_5678&PID_1234\\XYZ')
    ).toBeInTheDocument();
  });

  it('shows active indicator for active device', () => {
    renderWithRouter(<DeviceListCard devices={mockDevices} />);
    const activeIndicator = screen.getByLabelText('Active device');
    expect(activeIndicator).toHaveTextContent('✓ Active');
  });

  it('renders device scope correctly', () => {
    renderWithRouter(<DeviceListCard devices={mockDevices} />);
    expect(screen.getByText(/Scope: Global/)).toBeInTheDocument();
    expect(screen.getByText(/Scope: Device-Specific/)).toBeInTheDocument();
  });

  it('renders device layout', () => {
    renderWithRouter(<DeviceListCard devices={mockDevices} />);
    expect(screen.getByText(/Layout: ANSI 104/)).toBeInTheDocument();
    expect(screen.getByText(/Layout: Numpad/)).toBeInTheDocument();
  });

  it('renders keyboard icons with accessibility labels', () => {
    renderWithRouter(<DeviceListCard devices={mockDevices} />);
    const icons = screen.getAllByRole('img', { name: 'Keyboard' });
    expect(icons).toHaveLength(2);
  });

  it('navigates to devices page when Manage Devices is clicked', async () => {
    const user = userEvent.setup();
    renderWithRouter(<DeviceListCard devices={mockDevices} />);

    const button = screen.getByRole('button', { name: 'Manage devices' });
    await user.click(button);

    expect(mockNavigate).toHaveBeenCalledWith('/devices');
  });

  it('applies hover effect on device cards', () => {
    const { container } = renderWithRouter(
      <DeviceListCard devices={mockDevices} />
    );
    const deviceCards = container.querySelectorAll('.hover\\:border-slate-600');
    expect(deviceCards.length).toBeGreaterThan(0);
  });

  it('applies custom className', () => {
    const { container } = renderWithRouter(
      <DeviceListCard devices={mockDevices} className="custom-class" />
    );
    expect(container.querySelector('.custom-class')).toBeInTheDocument();
  });
});
