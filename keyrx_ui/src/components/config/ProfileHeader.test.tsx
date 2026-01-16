import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { ProfileHeader } from './ProfileHeader';

describe('ProfileHeader', () => {
  it('renders profile name with "Editing:" prefix', () => {
    render(<ProfileHeader profileName="my-profile" />);
    expect(screen.getByText(/Editing: my-profile/i)).toBeInTheDocument();
  });

  it('displays active badge when isActive is true', () => {
    render(<ProfileHeader profileName="my-profile" isActive={true} />);
    expect(
      screen.getByRole('status', { name: /active profile/i })
    ).toBeInTheDocument();
    expect(screen.getByText('Active')).toBeInTheDocument();
  });

  it('does not display active badge when isActive is false', () => {
    render(<ProfileHeader profileName="my-profile" isActive={false} />);
    expect(
      screen.queryByRole('status', { name: /active profile/i })
    ).not.toBeInTheDocument();
  });

  it('displays last modified date when provided', () => {
    const lastModified = new Date('2025-01-03T12:30:00');
    render(
      <ProfileHeader profileName="my-profile" lastModified={lastModified} />
    );
    expect(screen.getByText(/Last modified:/i)).toBeInTheDocument();
    expect(screen.getByText(/2025-01-03/i)).toBeInTheDocument();
  });

  it('does not display last modified when not provided', () => {
    render(<ProfileHeader profileName="my-profile" />);
    expect(screen.queryByText(/Last modified:/i)).not.toBeInTheDocument();
  });

  it('renders profile selector dropdown when profiles are provided', () => {
    const availableProfiles = ['default', 'my-profile', 'gaming'];
    render(
      <ProfileHeader
        profileName="my-profile"
        availableProfiles={availableProfiles}
        onProfileChange={vi.fn()}
      />
    );
    expect(
      screen.getByRole('button', { name: /select profile to edit/i })
    ).toBeInTheDocument();
  });

  it('does not render dropdown when no profiles provided', () => {
    render(<ProfileHeader profileName="my-profile" />);
    expect(
      screen.queryByRole('button', { name: /select profile to edit/i })
    ).not.toBeInTheDocument();
  });

  it('does not render dropdown when no onProfileChange callback provided', () => {
    render(
      <ProfileHeader
        profileName="my-profile"
        availableProfiles={['default', 'my-profile']}
      />
    );
    expect(
      screen.queryByRole('button', { name: /select profile to edit/i })
    ).not.toBeInTheDocument();
  });

  it('calls onProfileChange when user selects different profile', async () => {
    const user = userEvent.setup();
    const handleChange = vi.fn();
    const availableProfiles = ['default', 'my-profile', 'gaming'];

    render(
      <ProfileHeader
        profileName="my-profile"
        availableProfiles={availableProfiles}
        onProfileChange={handleChange}
      />
    );

    // Open dropdown
    const dropdown = screen.getByRole('button', {
      name: /select profile to edit/i,
    });
    await user.click(dropdown);

    // Select different profile
    const gamingOption = screen.getByRole('option', { name: 'gaming' });
    await user.click(gamingOption);

    expect(handleChange).toHaveBeenCalledWith('gaming');
  });

  it('displays all required elements when fully configured', () => {
    const lastModified = new Date('2025-01-03T12:30:00');
    const availableProfiles = ['default', 'my-profile', 'gaming'];

    render(
      <ProfileHeader
        profileName="my-profile"
        isActive={true}
        lastModified={lastModified}
        availableProfiles={availableProfiles}
        onProfileChange={vi.fn()}
      />
    );

    // Check all elements are present
    expect(screen.getByText(/Editing: my-profile/i)).toBeInTheDocument();
    expect(
      screen.getByRole('status', { name: /active profile/i })
    ).toBeInTheDocument();
    expect(screen.getByText(/Last modified:/i)).toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: /select profile to edit/i })
    ).toBeInTheDocument();
  });

  it('has proper ARIA labels for accessibility', () => {
    render(
      <ProfileHeader
        profileName="my-profile"
        isActive={true}
        availableProfiles={['default', 'my-profile']}
        onProfileChange={vi.fn()}
      />
    );

    expect(
      screen.getByRole('status', { name: /active profile/i })
    ).toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: /select profile to edit/i })
    ).toBeInTheDocument();
  });
});
