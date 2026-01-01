import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { BrowserRouter } from 'react-router-dom';
import { ActiveProfileCard } from './ActiveProfileCard';

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

describe('ActiveProfileCard', () => {
  const mockProfile = {
    name: 'Gaming',
    layers: 5,
    mappings: 127,
    modifiedAt: '2 hours ago',
  };

  beforeEach(() => {
    mockNavigate.mockClear();
  });

  it('renders loading state', () => {
    renderWithRouter(<ActiveProfileCard loading={true} />);
    const loadingElements = screen.getAllByRole('generic');
    const hasAnimatePulse = loadingElements.some((el) =>
      el.classList.contains('animate-pulse')
    );
    expect(hasAnimatePulse).toBe(true);
  });

  it('renders empty state when no profile', () => {
    renderWithRouter(<ActiveProfileCard />);
    expect(screen.getByText('Active Profile')).toBeInTheDocument();
    expect(
      screen.getByText(/No profile is currently active/)
    ).toBeInTheDocument();
  });

  it('renders Manage Profiles button in empty state', async () => {
    const user = userEvent.setup();
    renderWithRouter(<ActiveProfileCard />);

    const button = screen.getByRole('button', {
      name: 'Go to profiles page',
    });
    expect(button).toBeInTheDocument();

    await user.click(button);
    expect(mockNavigate).toHaveBeenCalledWith('/profiles');
  });

  it('renders profile data correctly', () => {
    renderWithRouter(<ActiveProfileCard profile={mockProfile} />);

    expect(screen.getByText('Gaming')).toBeInTheDocument();
    expect(screen.getByText('• 5 Layers')).toBeInTheDocument();
    expect(screen.getByText('• Modified: 2 hours ago')).toBeInTheDocument();
    expect(screen.getByText('• 127 key mappings')).toBeInTheDocument();
  });

  it('renders profile icon with accessibility label', () => {
    renderWithRouter(<ActiveProfileCard profile={mockProfile} />);
    const icon = screen.getByRole('img', { name: 'Profile icon' });
    expect(icon).toBeInTheDocument();
  });

  it('renders Edit button with correct aria-label', () => {
    renderWithRouter(<ActiveProfileCard profile={mockProfile} />);
    const editButton = screen.getByRole('button', {
      name: 'Edit profile Gaming',
    });
    expect(editButton).toBeInTheDocument();
  });

  it('navigates to config page when Edit is clicked', async () => {
    const user = userEvent.setup();
    renderWithRouter(<ActiveProfileCard profile={mockProfile} />);

    const editButton = screen.getByRole('button', {
      name: 'Edit profile Gaming',
    });
    await user.click(editButton);

    expect(mockNavigate).toHaveBeenCalledWith('/config');
  });

  it('applies custom className', () => {
    const { container } = renderWithRouter(
      <ActiveProfileCard profile={mockProfile} className="custom-class" />
    );
    expect(container.querySelector('.custom-class')).toBeInTheDocument();
  });
});
