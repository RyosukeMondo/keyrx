import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { ProfileCard } from './ProfileCard';

describe('ProfileCard', () => {
  const mockOnActivate = vi.fn();
  const mockOnEdit = vi.fn();
  const mockOnDelete = vi.fn();

  const defaultProps = {
    name: 'Test Profile',
    isActive: false,
    onActivate: mockOnActivate,
    onEdit: mockOnEdit,
    onDelete: mockOnDelete,
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders profile name', () => {
    render(<ProfileCard {...defaultProps} />);
    expect(screen.getByText('Test Profile')).toBeInTheDocument();
  });

  it('renders description when provided', () => {
    render(
      <ProfileCard {...defaultProps} description="Test description" />
    );
    expect(screen.getByText('Test description')).toBeInTheDocument();
  });

  it('renders last modified when provided', () => {
    render(
      <ProfileCard {...defaultProps} lastModified="2025-12-29 10:30" />
    );
    expect(screen.getByText(/Modified: 2025-12-29 10:30/)).toBeInTheDocument();
  });

  it('shows ACTIVE badge when profile is active', () => {
    render(<ProfileCard {...defaultProps} isActive={true} />);
    expect(screen.getByText('ACTIVE')).toBeInTheDocument();
  });

  it('does not show ACTIVE badge when profile is inactive', () => {
    render(<ProfileCard {...defaultProps} isActive={false} />);
    expect(screen.queryByText('ACTIVE')).not.toBeInTheDocument();
  });

  it('shows Activate button when profile is inactive', () => {
    render(<ProfileCard {...defaultProps} isActive={false} />);
    expect(
      screen.getByRole('button', { name: /Activate profile Test Profile/i })
    ).toBeInTheDocument();
  });

  it('does not show Activate button when profile is active', () => {
    render(<ProfileCard {...defaultProps} isActive={true} />);
    expect(
      screen.queryByRole('button', { name: /Activate profile/i })
    ).not.toBeInTheDocument();
  });

  it('calls onActivate when Activate button is clicked', async () => {
    const user = userEvent.setup();
    render(<ProfileCard {...defaultProps} isActive={false} />);

    const activateButton = screen.getByRole('button', {
      name: /Activate profile Test Profile/i,
    });
    await user.click(activateButton);

    expect(mockOnActivate).toHaveBeenCalledTimes(1);
  });

  it('calls onEdit when Edit button is clicked', async () => {
    const user = userEvent.setup();
    render(<ProfileCard {...defaultProps} />);

    const editButton = screen.getByRole('button', {
      name: /Edit profile Test Profile/i,
    });
    await user.click(editButton);

    expect(mockOnEdit).toHaveBeenCalledTimes(1);
  });

  it('calls onDelete when Delete button is clicked', async () => {
    const user = userEvent.setup();
    render(<ProfileCard {...defaultProps} isActive={false} />);

    const deleteButton = screen.getByRole('button', {
      name: /Delete profile Test Profile/i,
    });
    await user.click(deleteButton);

    expect(mockOnDelete).toHaveBeenCalledTimes(1);
  });

  it('disables Delete button when profile is active', () => {
    render(<ProfileCard {...defaultProps} isActive={true} />);

    const deleteButton = screen.getByRole('button', {
      name: /Delete profile Test Profile/i,
    });
    expect(deleteButton).toBeDisabled();
  });

  it('has green border when profile is active', () => {
    const { container } = render(
      <ProfileCard {...defaultProps} isActive={true} />
    );

    // Find the Card element (first div child)
    const card = container.querySelector('.border-green-500');
    expect(card).toBeInTheDocument();
  });

  it('has active profile indicator icon when active', () => {
    render(<ProfileCard {...defaultProps} isActive={true} />);
    expect(
      screen.getByLabelText('Active profile indicator')
    ).toBeInTheDocument();
  });

  it('renders all buttons with proper aria-labels', () => {
    render(<ProfileCard {...defaultProps} isActive={false} />);

    expect(
      screen.getByRole('button', { name: /Activate profile Test Profile/i })
    ).toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: /Edit profile Test Profile/i })
    ).toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: /Delete profile Test Profile/i })
    ).toBeInTheDocument();
  });
});
