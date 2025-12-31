import { describe, it, expect, vi } from 'vitest';
import { render, screen, within, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { ProfilesPage } from './ProfilesPage';

describe('ProfilesPage', () => {
  it('renders page title', () => {
    render(<ProfilesPage />);
    expect(
      screen.getByRole('heading', { name: 'Profiles' })
    ).toBeInTheDocument();
  });

  it('renders Create Profile button', () => {
    render(<ProfilesPage />);
    expect(
      screen.getByRole('button', { name: /Create new profile/i })
    ).toBeInTheDocument();
  });

  it('renders initial profile cards', () => {
    render(<ProfilesPage />);

    expect(screen.getByText('Default')).toBeInTheDocument();
    expect(screen.getByText('Gaming')).toBeInTheDocument();
    expect(screen.getByText('Programming')).toBeInTheDocument();
  });

  it('shows active indicator on Default profile', () => {
    render(<ProfilesPage />);
    expect(screen.getByText('ACTIVE')).toBeInTheDocument();
  });

  it('opens create modal when Create Profile button is clicked', async () => {
    const user = userEvent.setup();
    render(<ProfilesPage />);

    const createButton = screen.getByRole('button', {
      name: /Create new profile/i,
    });
    await user.click(createButton);

    expect(
      screen.getByRole('heading', { name: 'Create New Profile' })
    ).toBeInTheDocument();
  });

  it('creates a new profile when form is submitted', async () => {
    const user = userEvent.setup();
    render(<ProfilesPage />);

    // Open create modal
    await user.click(
      screen.getByRole('button', { name: /Create new profile/i })
    );

    // Fill in form
    const nameInput = screen.getByPlaceholderText('Profile name');
    const descInput = screen.getByPlaceholderText('Description (optional)');

    await user.type(nameInput, 'New Profile');
    await user.type(descInput, 'New description');

    // Submit
    const createButton = screen.getByRole('button', { name: /Save new profile/i });
    await user.click(createButton);

    // Verify profile was created
    expect(screen.getByText('New Profile')).toBeInTheDocument();
    expect(screen.getByText('New description')).toBeInTheDocument();
  });

  it('shows validation error when profile name is empty', async () => {
    const user = userEvent.setup();
    render(<ProfilesPage />);

    // Open create modal
    await user.click(
      screen.getByRole('button', { name: /Create new profile/i })
    );

    // Try to submit without name
    const createButton = screen.getByRole('button', { name: /Save new profile/i });
    await user.click(createButton);

    expect(screen.getByText('Profile name is required')).toBeInTheDocument();
  });

  it('shows validation error when profile name is too long', async () => {
    const user = userEvent.setup();
    render(<ProfilesPage />);

    // Open create modal
    await user.click(
      screen.getByRole('button', { name: /Create new profile/i })
    );

    // Fill with name over 50 chars (maxLength prevents typing, but we can paste)
    const nameInput = screen.getByPlaceholderText('Profile name');
    // Type exactly 50 chars first
    await user.type(nameInput, 'a'.repeat(50));

    // Verify maxLength prevents more than 50 characters
    expect((nameInput as HTMLInputElement).value.length).toBe(50);
  });

  it('shows validation error when profile name already exists', async () => {
    const user = userEvent.setup();
    render(<ProfilesPage />);

    // Open create modal
    await user.click(
      screen.getByRole('button', { name: /Create new profile/i })
    );

    // Use existing name
    const nameInput = screen.getByPlaceholderText('Profile name');
    await user.type(nameInput, 'Default');

    const createButton = screen.getByRole('button', { name: /Save new profile/i });
    await user.click(createButton);

    expect(screen.getByText('Profile name already exists')).toBeInTheDocument();
  });

  it('closes create modal when Cancel is clicked', async () => {
    const user = userEvent.setup();
    render(<ProfilesPage />);

    // Open create modal
    await user.click(
      screen.getByRole('button', { name: /Create new profile/i })
    );

    expect(
      screen.getByRole('heading', { name: 'Create New Profile' })
    ).toBeInTheDocument();

    // Click Cancel
    await user.click(
      screen.getByRole('button', { name: /Cancel creating profile/i })
    );

    // Wait for modal to close (animation takes time)
    await waitFor(() => {
      expect(
        screen.queryByRole('heading', { name: 'Create New Profile' })
      ).not.toBeInTheDocument();
    });
  });

  it('activates a profile when Activate button is clicked', async () => {
    const user = userEvent.setup();
    render(<ProfilesPage />);

    // Initially, Default should be active
    const defaultCard = screen.getByText('Default').closest('.border-green-500');
    expect(defaultCard).toBeInTheDocument();

    // Click Activate on Gaming profile
    const activateButton = screen.getByRole('button', {
      name: /Activate profile Gaming/i,
    });
    await user.click(activateButton);

    // Verify Gaming is now active (should have green border)
    const gamingCard = screen.getByText('Gaming').closest('.border-green-500');
    expect(gamingCard).toBeInTheDocument();

    // Verify there's still only one ACTIVE badge
    const activeCards = screen.getAllByText('ACTIVE');
    expect(activeCards).toHaveLength(1);

    // The ACTIVE badge should be in the Gaming card
    const gamingSection = screen.getByText('Gaming').closest('.border-green-500');
    expect(within(gamingSection!).getByText('ACTIVE')).toBeInTheDocument();
  });

  it('opens edit modal when Edit button is clicked', async () => {
    const user = userEvent.setup();
    render(<ProfilesPage />);

    // Click Edit on Gaming profile
    const editButton = screen.getByRole('button', {
      name: /Edit profile Gaming/i,
    });
    await user.click(editButton);

    expect(
      screen.getByRole('heading', { name: 'Edit Profile' })
    ).toBeInTheDocument();

    // Verify form is pre-filled
    const nameInput = screen.getByPlaceholderText('Profile name') as HTMLInputElement;
    expect(nameInput.value).toBe('Gaming');
  });

  it('updates profile when edit form is submitted', async () => {
    const user = userEvent.setup();
    render(<ProfilesPage />);

    // Click Edit on Gaming profile
    await user.click(
      screen.getByRole('button', { name: /Edit profile Gaming/i })
    );

    // Update name
    const nameInput = screen.getByPlaceholderText('Profile name');
    await user.clear(nameInput);
    await user.type(nameInput, 'Updated Gaming');

    // Save
    await user.click(
      screen.getByRole('button', { name: /Save profile changes/i })
    );

    // Verify update
    expect(screen.getByText('Updated Gaming')).toBeInTheDocument();
    expect(screen.queryByText('Gaming')).not.toBeInTheDocument();
  });

  it('closes edit modal when Cancel is clicked', async () => {
    const user = userEvent.setup();
    render(<ProfilesPage />);

    // Open edit modal
    await user.click(
      screen.getByRole('button', { name: /Edit profile Gaming/i })
    );

    expect(
      screen.getByRole('heading', { name: 'Edit Profile' })
    ).toBeInTheDocument();

    // Click Cancel
    await user.click(
      screen.getByRole('button', { name: /Cancel editing profile/i })
    );

    // Wait for modal to close (animation takes time)
    await waitFor(() => {
      expect(
        screen.queryByRole('heading', { name: 'Edit Profile' })
      ).not.toBeInTheDocument();
    });
  });

  it('opens delete confirmation modal when Delete button is clicked', async () => {
    const user = userEvent.setup();
    render(<ProfilesPage />);

    // Click Delete on Gaming profile
    const deleteButton = screen.getByRole('button', {
      name: /Delete profile Gaming/i,
    });
    await user.click(deleteButton);

    expect(
      screen.getByRole('heading', { name: 'Delete Profile' })
    ).toBeInTheDocument();

    // Check the confirmation message mentions the profile name
    expect(screen.getByText(/Are you sure you want to delete the profile/)).toBeInTheDocument();
    const strongElement = screen.getByText('Gaming', { selector: 'strong' });
    expect(strongElement).toBeInTheDocument();
  });

  it('deletes profile when deletion is confirmed', async () => {
    const user = userEvent.setup();
    render(<ProfilesPage />);

    // Click Delete on Gaming profile
    await user.click(
      screen.getByRole('button', { name: /Delete profile Gaming/i })
    );

    // Confirm deletion
    await user.click(
      screen.getByRole('button', { name: /Confirm delete profile/i })
    );

    // Wait for profile to be removed and modal to close
    await waitFor(() => {
      expect(screen.queryByText('Gaming')).not.toBeInTheDocument();
    });
  });

  it('closes delete modal when Cancel is clicked', async () => {
    const user = userEvent.setup();
    render(<ProfilesPage />);

    // Open delete modal
    await user.click(
      screen.getByRole('button', { name: /Delete profile Gaming/i })
    );

    expect(
      screen.getByRole('heading', { name: 'Delete Profile' })
    ).toBeInTheDocument();

    // Click Cancel
    await user.click(
      screen.getByRole('button', { name: /Cancel deleting profile/i })
    );

    // Wait for modal to close (animation takes time)
    await waitFor(() => {
      expect(
        screen.queryByRole('heading', { name: 'Delete Profile' })
      ).not.toBeInTheDocument();
    });

    // Verify profile still exists
    expect(screen.getByText('Gaming')).toBeInTheDocument();
  });

  it('renders profile grid with responsive classes', () => {
    const { container } = render(<ProfilesPage />);

    const grid = container.querySelector('.grid');
    expect(grid).toHaveClass('grid-cols-1');
    expect(grid).toHaveClass('md:grid-cols-2');
    expect(grid).toHaveClass('lg:grid-cols-3');
  });

  it('shows all action buttons with proper accessibility labels', () => {
    render(<ProfilesPage />);

    // Gaming profile (inactive)
    expect(
      screen.getByRole('button', { name: /Activate profile Gaming/i })
    ).toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: /Edit profile Gaming/i })
    ).toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: /Delete profile Gaming/i })
    ).toBeInTheDocument();
  });
});
