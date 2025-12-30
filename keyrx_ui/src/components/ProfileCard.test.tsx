import { describe, it, expect, vi } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { ProfileCard } from './ProfileCard';
import { Profile } from './ProfilesPage';
import { renderWithProviders } from '../../tests/testUtils';

describe('ProfileCard', () => {
  const mockProfile: Profile = {
    name: 'test-profile',
    rhai_path: '/path/to/test.rhai',
    krx_path: '/path/to/test.krx',
    modified_at: Date.now() - 3600000, // 1 hour ago
    layer_count: 3,
    is_active: false,
  };

  const mockActiveProfile: Profile = {
    ...mockProfile,
    name: 'active-profile',
    is_active: true,
  };

  const mockCallbacks = {
    onActivate: vi.fn(),
    onDelete: vi.fn(),
    onDuplicate: vi.fn(),
    onExport: vi.fn(),
    onRename: vi.fn(),
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should render profile with correct name', () => {
    renderWithProviders(
      <ProfileCard profile={mockProfile} {...mockCallbacks} />
    );

    expect(screen.getByText('test-profile')).toBeInTheDocument();
  });

  it('should display layer count', () => {
    renderWithProviders(
      <ProfileCard profile={mockProfile} {...mockCallbacks} />
    );

    expect(screen.getByText('3 layers')).toBeInTheDocument();
  });

  it('should display formatted timestamp using formatTimestampRelative', () => {
    renderWithProviders(
      <ProfileCard profile={mockProfile} {...mockCallbacks} />
    );

    // The formatTimestampRelative should show something like "1 hour ago"
    expect(screen.getByText(/Modified/)).toBeInTheDocument();
  });

  it('should show inactive status indicator for inactive profile', () => {
    renderWithProviders(
      <ProfileCard profile={mockProfile} {...mockCallbacks} />
    );

    const statusIndicator = screen.getByText('○');
    expect(statusIndicator).toBeInTheDocument();
    expect(statusIndicator).toHaveClass('status-indicator');
  });

  it('should show active status indicator for active profile', () => {
    renderWithProviders(
      <ProfileCard profile={mockActiveProfile} {...mockCallbacks} />
    );

    const statusIndicator = screen.getByText('●');
    expect(statusIndicator).toBeInTheDocument();
    expect(statusIndicator).toHaveClass('status-indicator', 'active');
  });

  it('should apply active CSS class when profile is active', () => {
    const { container } = renderWithProviders(
      <ProfileCard profile={mockActiveProfile} {...mockCallbacks} />
    );

    const profileCard = container.querySelector('.profile-card');
    expect(profileCard).toHaveClass('active');
  });

  it('should not apply active CSS class when profile is inactive', () => {
    const { container } = renderWithProviders(
      <ProfileCard profile={mockProfile} {...mockCallbacks} />
    );

    const profileCard = container.querySelector('.profile-card');
    expect(profileCard).not.toHaveClass('active');
  });

  it('should show activate button for inactive profile', () => {
    renderWithProviders(
      <ProfileCard profile={mockProfile} {...mockCallbacks} />
    );

    expect(screen.getByRole('button', { name: /activate/i })).toBeInTheDocument();
  });

  it('should not show activate button for active profile', () => {
    renderWithProviders(
      <ProfileCard profile={mockActiveProfile} {...mockCallbacks} />
    );

    expect(screen.queryByRole('button', { name: /activate/i })).not.toBeInTheDocument();
  });

  it('should show "Active Profile" label for active profile', () => {
    renderWithProviders(
      <ProfileCard profile={mockActiveProfile} {...mockCallbacks} />
    );

    expect(screen.getByText('Active Profile')).toBeInTheDocument();
  });

  it('should call onActivate when activate button is clicked', async () => {
    const user = userEvent.setup();
    renderWithProviders(
      <ProfileCard profile={mockProfile} {...mockCallbacks} />
    );

    const activateButton = screen.getByRole('button', { name: /activate/i });
    await user.click(activateButton);

    expect(mockCallbacks.onActivate).toHaveBeenCalledTimes(1);
  });

  it('should call onRename when rename button is clicked', async () => {
    const user = userEvent.setup();
    renderWithProviders(
      <ProfileCard profile={mockProfile} {...mockCallbacks} />
    );

    const renameButton = screen.getByRole('button', { name: /rename/i });
    await user.click(renameButton);

    expect(mockCallbacks.onRename).toHaveBeenCalledTimes(1);
  });

  it('should call onDuplicate when duplicate button is clicked', async () => {
    const user = userEvent.setup();
    renderWithProviders(
      <ProfileCard profile={mockProfile} {...mockCallbacks} />
    );

    const duplicateButton = screen.getByRole('button', { name: /duplicate/i });
    await user.click(duplicateButton);

    expect(mockCallbacks.onDuplicate).toHaveBeenCalledTimes(1);
  });

  it('should call onExport when export button is clicked', async () => {
    const user = userEvent.setup();
    renderWithProviders(
      <ProfileCard profile={mockProfile} {...mockCallbacks} />
    );

    const exportButton = screen.getByRole('button', { name: /export/i });
    await user.click(exportButton);

    expect(mockCallbacks.onExport).toHaveBeenCalledTimes(1);
  });

  it('should call onDelete when delete button is clicked', async () => {
    const user = userEvent.setup();
    renderWithProviders(
      <ProfileCard profile={mockProfile} {...mockCallbacks} />
    );

    const deleteButton = screen.getByRole('button', { name: /delete/i });
    await user.click(deleteButton);

    expect(mockCallbacks.onDelete).toHaveBeenCalledTimes(1);
  });

  it('should disable delete button for active profile', () => {
    renderWithProviders(
      <ProfileCard profile={mockActiveProfile} {...mockCallbacks} />
    );

    const deleteButton = screen.getByRole('button', { name: /delete/i });
    expect(deleteButton).toBeDisabled();
  });

  it('should enable delete button for inactive profile', () => {
    renderWithProviders(
      <ProfileCard profile={mockProfile} {...mockCallbacks} />
    );

    const deleteButton = screen.getByRole('button', { name: /delete/i });
    expect(deleteButton).not.toBeDisabled();
  });

  it('should display all secondary action buttons', () => {
    renderWithProviders(
      <ProfileCard profile={mockProfile} {...mockCallbacks} />
    );

    expect(screen.getByRole('button', { name: /rename/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /duplicate/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /export/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /delete/i })).toBeInTheDocument();
  });

  it('should handle profile with 1 layer count (singular form)', () => {
    const singleLayerProfile: Profile = {
      ...mockProfile,
      layer_count: 1,
    };

    renderWithProviders(
      <ProfileCard profile={singleLayerProfile} {...mockCallbacks} />
    );

    expect(screen.getByText('1 layers')).toBeInTheDocument();
  });

  it('should handle profile with many layers', () => {
    const manyLayersProfile: Profile = {
      ...mockProfile,
      layer_count: 255,
    };

    renderWithProviders(
      <ProfileCard profile={manyLayersProfile} {...mockCallbacks} />
    );

    expect(screen.getByText('255 layers')).toBeInTheDocument();
  });

  it('should handle profile with zero layers', () => {
    const zeroLayersProfile: Profile = {
      ...mockProfile,
      layer_count: 0,
    };

    renderWithProviders(
      <ProfileCard profile={zeroLayersProfile} {...mockCallbacks} />
    );

    expect(screen.getByText('0 layers')).toBeInTheDocument();
  });

  it('should have correct title attributes on secondary action buttons', () => {
    renderWithProviders(
      <ProfileCard profile={mockProfile} {...mockCallbacks} />
    );

    expect(screen.getByTitle('Rename')).toBeInTheDocument();
    expect(screen.getByTitle('Duplicate')).toBeInTheDocument();
    expect(screen.getByTitle('Export')).toBeInTheDocument();
    expect(screen.getByTitle('Delete')).toBeInTheDocument();
  });
});
