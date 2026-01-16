import { describe, it, expect, vi } from 'vitest';
import { screen, fireEvent, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { renderWithProviders } from '../../../tests/testUtils';
import { ProfileSelector } from './ProfileSelector';
import type { ProfileMetadata } from '@/types';

describe('ProfileSelector', () => {
  const mockProfiles: ProfileMetadata[] = [
    { name: 'Default', isActive: true },
    { name: 'Gaming', isActive: false },
    { name: 'Work', isActive: false },
  ];

  const defaultProps = {
    value: 'Default',
    onChange: vi.fn(),
    profiles: mockProfiles,
    isLoading: false,
  };

  it('renders profile selector with label', () => {
    renderWithProviders(<ProfileSelector {...defaultProps} />);
    expect(screen.getByText('Profile:')).toBeInTheDocument();
    expect(
      screen.getByRole('combobox', { name: /select profile/i })
    ).toBeInTheDocument();
  });

  it('displays all available profiles', () => {
    renderWithProviders(<ProfileSelector {...defaultProps} />);
    const select = screen.getByRole('combobox');
    const options = Array.from(select.querySelectorAll('option'));

    expect(options).toHaveLength(3);
    expect(options[0]).toHaveTextContent('Default');
    expect(options[1]).toHaveTextContent('Gaming');
    expect(options[2]).toHaveTextContent('Work');
  });

  it('displays selected profile value', () => {
    renderWithProviders(<ProfileSelector {...defaultProps} value="Gaming" />);
    const select = screen.getByRole('combobox') as HTMLSelectElement;
    expect(select.value).toBe('Gaming');
  });

  it('calls onChange when profile selection changes', () => {
    const handleChange = vi.fn();
    renderWithProviders(
      <ProfileSelector {...defaultProps} onChange={handleChange} />
    );

    const select = screen.getByRole('combobox');
    fireEvent.change(select, { target: { value: 'Gaming' } });

    expect(handleChange).toHaveBeenCalledWith('Gaming');
    expect(handleChange).toHaveBeenCalledTimes(1);
  });

  it('disables select when loading', () => {
    renderWithProviders(<ProfileSelector {...defaultProps} isLoading />);
    const select = screen.getByRole('combobox');
    expect(select).toBeDisabled();
  });

  it('disables select when disabled prop is true', () => {
    renderWithProviders(<ProfileSelector {...defaultProps} disabled />);
    const select = screen.getByRole('combobox');
    expect(select).toBeDisabled();
  });

  it('renders create button when onCreateProfile is provided', () => {
    const handleCreate = vi.fn();
    renderWithProviders(
      <ProfileSelector {...defaultProps} onCreateProfile={handleCreate} />
    );

    expect(
      screen.getByRole('button', { name: /create new profile/i })
    ).toBeInTheDocument();
  });

  it('does not render create button when onCreateProfile is not provided', () => {
    renderWithProviders(<ProfileSelector {...defaultProps} />);
    expect(
      screen.queryByRole('button', { name: /create new profile/i })
    ).not.toBeInTheDocument();
  });

  it('disables create button when disabled', () => {
    const handleCreate = vi.fn();
    renderWithProviders(
      <ProfileSelector
        {...defaultProps}
        onCreateProfile={handleCreate}
        disabled
      />
    );

    const createButton = screen.getByRole('button', {
      name: /create new profile/i,
    });
    expect(createButton).toBeDisabled();
  });

  describe('Profile Creation', () => {
    it('shows input field when create button is clicked', async () => {
      const user = userEvent.setup();
      const handleCreate = vi.fn();
      renderWithProviders(
        <ProfileSelector {...defaultProps} onCreateProfile={handleCreate} />
      );

      const createButton = screen.getByRole('button', {
        name: /create new profile/i,
      });
      await user.click(createButton);

      expect(
        screen.getByRole('textbox', { name: /new profile name/i })
      ).toBeInTheDocument();
      expect(
        screen.getByRole('button', { name: /confirm create profile/i })
      ).toBeInTheDocument();
      expect(
        screen.getByRole('button', { name: /cancel create profile/i })
      ).toBeInTheDocument();
    });

    it('hides select dropdown when in creation mode', async () => {
      const user = userEvent.setup();
      const handleCreate = vi.fn();
      renderWithProviders(
        <ProfileSelector {...defaultProps} onCreateProfile={handleCreate} />
      );

      const createButton = screen.getByRole('button', {
        name: /create new profile/i,
      });
      await user.click(createButton);

      expect(screen.queryByRole('combobox')).not.toBeInTheDocument();
    });

    it('focuses input field when entering creation mode', async () => {
      const user = userEvent.setup();
      const handleCreate = vi.fn();
      renderWithProviders(
        <ProfileSelector {...defaultProps} onCreateProfile={handleCreate} />
      );

      const createButton = screen.getByRole('button', {
        name: /create new profile/i,
      });
      await user.click(createButton);

      const input = screen.getByRole('textbox', { name: /new profile name/i });
      expect(input).toHaveFocus();
    });

    it('calls onCreateProfile when confirm button is clicked with valid name', async () => {
      const user = userEvent.setup();
      const handleCreate = vi.fn();
      renderWithProviders(
        <ProfileSelector {...defaultProps} onCreateProfile={handleCreate} />
      );

      const createButton = screen.getByRole('button', {
        name: /create new profile/i,
      });
      await user.click(createButton);

      const input = screen.getByRole('textbox', { name: /new profile name/i });
      await user.type(input, 'NewProfile');

      const confirmButton = screen.getByRole('button', {
        name: /confirm create profile/i,
      });
      await user.click(confirmButton);

      expect(handleCreate).toHaveBeenCalledTimes(1);
    });

    it('exits creation mode after successful confirmation', async () => {
      const user = userEvent.setup();
      const handleCreate = vi.fn();
      renderWithProviders(
        <ProfileSelector {...defaultProps} onCreateProfile={handleCreate} />
      );

      const createButton = screen.getByRole('button', {
        name: /create new profile/i,
      });
      await user.click(createButton);

      const input = screen.getByRole('textbox', { name: /new profile name/i });
      await user.type(input, 'NewProfile');

      const confirmButton = screen.getByRole('button', {
        name: /confirm create profile/i,
      });
      await user.click(confirmButton);

      await waitFor(() => {
        expect(
          screen.queryByRole('textbox', { name: /new profile name/i })
        ).not.toBeInTheDocument();
      });

      expect(screen.getByRole('combobox')).toBeInTheDocument();
    });

    it('shows error when trying to create profile with empty name', async () => {
      const user = userEvent.setup();
      const handleCreate = vi.fn();
      renderWithProviders(
        <ProfileSelector {...defaultProps} onCreateProfile={handleCreate} />
      );

      const createButton = screen.getByRole('button', {
        name: /create new profile/i,
      });
      await user.click(createButton);

      const confirmButton = screen.getByRole('button', {
        name: /confirm create profile/i,
      });
      await user.click(confirmButton);

      expect(screen.getByRole('alert')).toHaveTextContent(
        'Profile name cannot be empty'
      );
      expect(handleCreate).not.toHaveBeenCalled();
    });

    it('shows error when trying to create profile with whitespace-only name', async () => {
      const user = userEvent.setup();
      const handleCreate = vi.fn();
      renderWithProviders(
        <ProfileSelector {...defaultProps} onCreateProfile={handleCreate} />
      );

      const createButton = screen.getByRole('button', {
        name: /create new profile/i,
      });
      await user.click(createButton);

      const input = screen.getByRole('textbox', { name: /new profile name/i });
      await user.type(input, '   ');

      const confirmButton = screen.getByRole('button', {
        name: /confirm create profile/i,
      });
      await user.click(confirmButton);

      expect(screen.getByRole('alert')).toHaveTextContent(
        'Profile name cannot be empty'
      );
      expect(handleCreate).not.toHaveBeenCalled();
    });

    it('shows error when trying to create profile with duplicate name', async () => {
      const user = userEvent.setup();
      const handleCreate = vi.fn();
      renderWithProviders(
        <ProfileSelector {...defaultProps} onCreateProfile={handleCreate} />
      );

      const createButton = screen.getByRole('button', {
        name: /create new profile/i,
      });
      await user.click(createButton);

      const input = screen.getByRole('textbox', { name: /new profile name/i });
      await user.type(input, 'Default');

      const confirmButton = screen.getByRole('button', {
        name: /confirm create profile/i,
      });
      await user.click(confirmButton);

      expect(screen.getByRole('alert')).toHaveTextContent(
        'Profile name already exists'
      );
      expect(handleCreate).not.toHaveBeenCalled();
    });

    it('clears error message when typing in input', async () => {
      const user = userEvent.setup();
      const handleCreate = vi.fn();
      renderWithProviders(
        <ProfileSelector {...defaultProps} onCreateProfile={handleCreate} />
      );

      const createButton = screen.getByRole('button', {
        name: /create new profile/i,
      });
      await user.click(createButton);

      const confirmButton = screen.getByRole('button', {
        name: /confirm create profile/i,
      });
      await user.click(confirmButton);

      expect(screen.getByRole('alert')).toBeInTheDocument();

      const input = screen.getByRole('textbox', { name: /new profile name/i });
      await user.type(input, 'N');

      expect(screen.queryByRole('alert')).not.toBeInTheDocument();
    });

    it('cancels creation mode when cancel button is clicked', async () => {
      const user = userEvent.setup();
      const handleCreate = vi.fn();
      renderWithProviders(
        <ProfileSelector {...defaultProps} onCreateProfile={handleCreate} />
      );

      const createButton = screen.getByRole('button', {
        name: /create new profile/i,
      });
      await user.click(createButton);

      const input = screen.getByRole('textbox', { name: /new profile name/i });
      await user.type(input, 'NewProfile');

      const cancelButton = screen.getByRole('button', {
        name: /cancel create profile/i,
      });
      await user.click(cancelButton);

      expect(
        screen.queryByRole('textbox', { name: /new profile name/i })
      ).not.toBeInTheDocument();
      expect(screen.getByRole('combobox')).toBeInTheDocument();
      expect(handleCreate).not.toHaveBeenCalled();
    });

    it('confirms creation when Enter key is pressed', async () => {
      const user = userEvent.setup();
      const handleCreate = vi.fn();
      renderWithProviders(
        <ProfileSelector {...defaultProps} onCreateProfile={handleCreate} />
      );

      const createButton = screen.getByRole('button', {
        name: /create new profile/i,
      });
      await user.click(createButton);

      const input = screen.getByRole('textbox', { name: /new profile name/i });
      await user.type(input, 'NewProfile{Enter}');

      expect(handleCreate).toHaveBeenCalledTimes(1);
    });

    it('cancels creation when Escape key is pressed', async () => {
      const user = userEvent.setup();
      const handleCreate = vi.fn();
      renderWithProviders(
        <ProfileSelector {...defaultProps} onCreateProfile={handleCreate} />
      );

      const createButton = screen.getByRole('button', {
        name: /create new profile/i,
      });
      await user.click(createButton);

      const input = screen.getByRole('textbox', { name: /new profile name/i });
      await user.type(input, 'NewProfile{Escape}');

      expect(
        screen.queryByRole('textbox', { name: /new profile name/i })
      ).not.toBeInTheDocument();
      expect(handleCreate).not.toHaveBeenCalled();
    });

    it('shows error instead of disabling button when input is empty', async () => {
      const user = userEvent.setup();
      const handleCreate = vi.fn();
      renderWithProviders(
        <ProfileSelector {...defaultProps} onCreateProfile={handleCreate} />
      );

      const createButton = screen.getByRole('button', {
        name: /create new profile/i,
      });
      await user.click(createButton);

      const confirmButton = screen.getByRole('button', {
        name: /confirm create profile/i,
      });
      expect(confirmButton).not.toBeDisabled();

      // Clicking confirm with empty input shows error
      await user.click(confirmButton);
      expect(screen.getByRole('alert')).toHaveTextContent(
        'Profile name cannot be empty'
      );
    });

    it('sets aria-invalid on input when error exists', async () => {
      const user = userEvent.setup();
      const handleCreate = vi.fn();
      renderWithProviders(
        <ProfileSelector {...defaultProps} onCreateProfile={handleCreate} />
      );

      const createButton = screen.getByRole('button', {
        name: /create new profile/i,
      });
      await user.click(createButton);

      const confirmButton = screen.getByRole('button', {
        name: /confirm create profile/i,
      });
      await user.click(confirmButton);

      const input = screen.getByRole('textbox', { name: /new profile name/i });
      expect(input).toHaveAttribute('aria-invalid', 'true');
      expect(input).toHaveAttribute('aria-describedby', 'create-error');
    });
  });

  describe('Accessibility', () => {
    it('has proper ARIA label for select', () => {
      renderWithProviders(<ProfileSelector {...defaultProps} />);
      const select = screen.getByRole('combobox', { name: /select profile/i });
      expect(select).toHaveAttribute('aria-label', 'Select profile');
    });

    it('has proper ARIA label for create button', () => {
      const handleCreate = vi.fn();
      renderWithProviders(
        <ProfileSelector {...defaultProps} onCreateProfile={handleCreate} />
      );
      const createButton = screen.getByRole('button', {
        name: /create new profile/i,
      });
      expect(createButton).toHaveAttribute('aria-label', 'Create new profile');
    });

    it('has proper ARIA labels for confirm and cancel buttons', async () => {
      const user = userEvent.setup();
      const handleCreate = vi.fn();
      renderWithProviders(
        <ProfileSelector {...defaultProps} onCreateProfile={handleCreate} />
      );

      const createButton = screen.getByRole('button', {
        name: /create new profile/i,
      });
      await user.click(createButton);

      expect(
        screen.getByRole('button', { name: /confirm create profile/i })
      ).toHaveAttribute('aria-label', 'Confirm create profile');
      expect(
        screen.getByRole('button', { name: /cancel create profile/i })
      ).toHaveAttribute('aria-label', 'Cancel create profile');
    });

    it('links error message to input with aria-describedby', async () => {
      const user = userEvent.setup();
      const handleCreate = vi.fn();
      renderWithProviders(
        <ProfileSelector {...defaultProps} onCreateProfile={handleCreate} />
      );

      const createButton = screen.getByRole('button', {
        name: /create new profile/i,
      });
      await user.click(createButton);

      const confirmButton = screen.getByRole('button', {
        name: /confirm create profile/i,
      });
      await user.click(confirmButton);

      const input = screen.getByRole('textbox', { name: /new profile name/i });
      const errorId = input.getAttribute('aria-describedby');
      expect(errorId).toBe('create-error');

      const error = screen.getByRole('alert');
      expect(error).toHaveAttribute('id', 'create-error');
    });
  });

  describe('Edge Cases', () => {
    it('handles empty profiles array', () => {
      renderWithProviders(<ProfileSelector {...defaultProps} profiles={[]} />);
      const select = screen.getByRole('combobox');
      expect(select.querySelectorAll('option')).toHaveLength(0);
    });

    it('handles undefined profiles', () => {
      renderWithProviders(
        <ProfileSelector {...defaultProps} profiles={undefined} />
      );
      expect(screen.getByRole('combobox')).toBeInTheDocument();
    });

    it('trims whitespace from profile name before validation', async () => {
      const user = userEvent.setup();
      const handleCreate = vi.fn();
      renderWithProviders(
        <ProfileSelector {...defaultProps} onCreateProfile={handleCreate} />
      );

      const createButton = screen.getByRole('button', {
        name: /create new profile/i,
      });
      await user.click(createButton);

      const input = screen.getByRole('textbox', { name: /new profile name/i });
      await user.type(input, '  Default  ');

      const confirmButton = screen.getByRole('button', {
        name: /confirm create profile/i,
      });
      await user.click(confirmButton);

      expect(screen.getByRole('alert')).toHaveTextContent(
        'Profile name already exists'
      );
    });
  });
});
