import { describe, it, expect, vi } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithProviders } from '../../tests/testUtils';
import userEvent from '@testing-library/user-event';
import { ProfileCard } from './ProfileCard';

describe('ProfileCard', () => {
  const mockOnActivate = vi.fn();
  const mockOnDelete = vi.fn();
  const mockOnPathClick = vi.fn();
  const mockOnUpdateName = vi.fn();
  const mockOnUpdateDescription = vi.fn();

  const defaultProps = {
    name: 'Test Profile',
    isActive: false,
    onActivate: mockOnActivate,
    onDelete: mockOnDelete,
    onUpdateName: mockOnUpdateName,
    onUpdateDescription: mockOnUpdateDescription,
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders profile name', () => {
    renderWithProviders(<ProfileCard {...defaultProps} />);
    expect(screen.getByText('Test Profile')).toBeInTheDocument();
  });

  it('renders description when provided', () => {
    renderWithProviders(
      <ProfileCard {...defaultProps} description="Test description" />
    );
    expect(screen.getByText('Test description')).toBeInTheDocument();
  });

  it('renders last modified when provided', () => {
    renderWithProviders(
      <ProfileCard {...defaultProps} lastModified="2025-12-29 10:30" />
    );
    expect(screen.getByText(/Modified: 2025-12-29 10:30/)).toBeInTheDocument();
  });

  it('shows ACTIVE badge when profile is active', () => {
    renderWithProviders(<ProfileCard {...defaultProps} isActive={true} />);
    expect(screen.getByText('ACTIVE')).toBeInTheDocument();
  });

  it('does not show ACTIVE badge when profile is inactive', () => {
    renderWithProviders(<ProfileCard {...defaultProps} isActive={false} />);
    expect(screen.queryByText('ACTIVE')).not.toBeInTheDocument();
  });

  it('shows Activate button when profile is inactive', () => {
    renderWithProviders(<ProfileCard {...defaultProps} isActive={false} />);
    expect(
      screen.getByRole('button', { name: /Activate profile Test Profile/i })
    ).toBeInTheDocument();
  });

  it('does not show Activate button when profile is active', () => {
    renderWithProviders(<ProfileCard {...defaultProps} isActive={true} />);
    expect(
      screen.queryByRole('button', { name: /Activate profile/i })
    ).not.toBeInTheDocument();
  });

  it('calls onActivate when Activate button is clicked', async () => {
    const user = userEvent.setup();
    renderWithProviders(<ProfileCard {...defaultProps} isActive={false} />);

    const activateButton = screen.getByRole('button', {
      name: /Activate profile Test Profile/i,
    });
    await user.click(activateButton);

    expect(mockOnActivate).toHaveBeenCalledTimes(1);
  });

  it('calls onUpdateName when name is edited inline', async () => {
    const user = userEvent.setup();
    renderWithProviders(<ProfileCard {...defaultProps} />);

    // Find the inline edit button for the name
    const nameButton = screen.getByRole('button', {
      name: /Edit profile name: Test Profile/i,
    });
    await user.click(nameButton);

    // Input should now be visible
    const nameInput = screen.getByDisplayValue('Test Profile');
    await user.clear(nameInput);
    await user.type(nameInput, 'Updated Profile');

    // Blur to trigger save
    await user.tab();

    // Wait for debounced save (500ms)
    await new Promise((resolve) => setTimeout(resolve, 600));

    expect(mockOnUpdateName).toHaveBeenCalledWith('Updated Profile');
  });

  it('calls onDelete when Delete button is clicked', async () => {
    const user = userEvent.setup();
    renderWithProviders(<ProfileCard {...defaultProps} isActive={false} />);

    const deleteButton = screen.getByRole('button', {
      name: /Delete profile Test Profile/i,
    });
    await user.click(deleteButton);

    expect(mockOnDelete).toHaveBeenCalledTimes(1);
  });

  it('disables Delete button when profile is active', () => {
    renderWithProviders(<ProfileCard {...defaultProps} isActive={true} />);

    const deleteButton = screen.getByRole('button', {
      name: /Delete profile Test Profile/i,
    });
    expect(deleteButton).toBeDisabled();
  });

  it('has blue left border and gradient background when profile is active', () => {
    const { container } = renderWithProviders(
      <ProfileCard {...defaultProps} isActive={true} />
    );

    // Find the Card element with blue left border and gradient
    const card = container.querySelector('.border-l-blue-500');
    expect(card).toBeInTheDocument();
    expect(card).toHaveClass('bg-gradient-to-r');
  });

  it('has active profile indicator icon when active', () => {
    renderWithProviders(<ProfileCard {...defaultProps} isActive={true} />);
    expect(
      screen.getByLabelText('Active profile indicator')
    ).toBeInTheDocument();
  });

  it('renders all buttons with proper aria-labels', () => {
    renderWithProviders(<ProfileCard {...defaultProps} isActive={false} />);

    expect(
      screen.getByRole('button', { name: /Activate profile Test Profile/i })
    ).toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: /Edit profile name: Test Profile/i })
    ).toBeInTheDocument();
    expect(
      screen.getByRole('button', {
        name: /Edit profile description: Test Profile/i,
      })
    ).toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: /Delete profile Test Profile/i })
    ).toBeInTheDocument();
  });

  describe('Rhai path display', () => {
    it('displays Rhai file path when provided', () => {
      renderWithProviders(
        <ProfileCard
          {...defaultProps}
          rhaiPath="/home/user/.config/keyrx/profiles/gaming.rhai"
        />
      );

      // Should display truncated path (with ~ replacement)
      expect(screen.getByText(/gaming\.rhai/)).toBeInTheDocument();
    });

    it('does not display Rhai path when not provided', () => {
      const { container } = renderWithProviders(
        <ProfileCard {...defaultProps} />
      );

      // Should not find any file icon or path text
      expect(
        container.querySelector('[data-lucide="file-code"]')
      ).not.toBeInTheDocument();
    });

    it('shows tooltip with full path on hover', async () => {
      const fullPath = '/home/user/.config/keyrx/profiles/gaming.rhai';
      renderWithProviders(
        <ProfileCard {...defaultProps} rhaiPath={fullPath} />
      );

      // Tooltip component should exist with the full path
      const pathButton = screen.getByRole('button', {
        name: new RegExp(`Open configuration file: ${fullPath}`),
      });
      expect(pathButton).toBeInTheDocument();
    });

    it('calls onPathClick when Rhai path is clicked and onPathClick is provided', async () => {
      const user = userEvent.setup();
      renderWithProviders(
        <ProfileCard
          {...defaultProps}
          rhaiPath="/home/user/.config/keyrx/profiles/gaming.rhai"
          onPathClick={mockOnPathClick}
        />
      );

      const pathButton = screen.getByRole('button', {
        name: /Open configuration file/,
      });
      await user.click(pathButton);

      expect(mockOnPathClick).toHaveBeenCalledTimes(1);
    });

    it('does not crash when Rhai path is clicked and onPathClick is not provided', async () => {
      const user = userEvent.setup();
      renderWithProviders(
        <ProfileCard
          {...defaultProps}
          rhaiPath="/home/user/.config/keyrx/profiles/gaming.rhai"
        />
      );

      const pathButton = screen.getByRole('button', {
        name: /Open configuration file/,
      });

      // Should not throw error when clicked without onPathClick handler
      await expect(user.click(pathButton)).resolves.not.toThrow();
    });

    it('shows warning icon when file does not exist', () => {
      const { container } = renderWithProviders(
        <ProfileCard
          {...defaultProps}
          rhaiPath="/home/user/.config/keyrx/profiles/missing.rhai"
          fileExists={false}
        />
      );

      // Should have AlertTriangle icon with "File not found" label
      const warningIcon = container.querySelector(
        '[aria-label="File not found"]'
      );
      expect(warningIcon).toBeInTheDocument();
    });

    it('does not show warning icon when file exists', () => {
      const { container } = renderWithProviders(
        <ProfileCard
          {...defaultProps}
          rhaiPath="/home/user/.config/keyrx/profiles/gaming.rhai"
          fileExists={true}
        />
      );

      // Should not have AlertTriangle icon with "File not found" label
      const warningIcon = container.querySelector(
        '[aria-label="File not found"]'
      );
      expect(warningIcon).not.toBeInTheDocument();
    });

    it('truncates long paths for display', () => {
      const longPath =
        '/home/user/.config/keyrx/profiles/very-long-profile-name-that-should-be-truncated.rhai';
      renderWithProviders(
        <ProfileCard {...defaultProps} rhaiPath={longPath} />
      );

      // Should display truncated version with ellipsis
      const displayText = screen.getByText(/\.rhai/);
      expect(displayText.textContent).toContain('...');
    });
  });
});
