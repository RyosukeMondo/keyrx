import { describe, it, expect, vi } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithProviders } from '../../tests/testUtils';
import userEvent from '@testing-library/user-event';
import { ErrorState } from './ErrorState';

describe('ErrorState', () => {
  it('renders with default title', () => {
    renderWithProviders(<ErrorState message="Something went wrong" />);
    expect(screen.getByText('Error')).toBeInTheDocument();
  });

  it('renders with custom title', () => {
    renderWithProviders(
      <ErrorState title="Custom Error" message="Something went wrong" />
    );
    expect(screen.getByText('Custom Error')).toBeInTheDocument();
  });

  it('renders error message', () => {
    renderWithProviders(<ErrorState message="Failed to load data" />);
    expect(screen.getByText('Failed to load data')).toBeInTheDocument();
  });

  it('does not render retry button when onRetry is not provided', () => {
    renderWithProviders(<ErrorState message="Something went wrong" />);
    expect(screen.queryByRole('button')).not.toBeInTheDocument();
  });

  it('renders retry button when onRetry is provided', () => {
    const onRetry = vi.fn();
    renderWithProviders(
      <ErrorState message="Something went wrong" onRetry={onRetry} />
    );
    expect(screen.getByLabelText('Try Again')).toBeInTheDocument();
  });

  it('renders custom retry label', () => {
    const onRetry = vi.fn();
    renderWithProviders(
      <ErrorState
        message="Something went wrong"
        onRetry={onRetry}
        retryLabel="Reload"
      />
    );
    expect(screen.getByLabelText('Reload')).toBeInTheDocument();
    expect(screen.getByText('Reload')).toBeInTheDocument();
  });

  it('calls onRetry when retry button is clicked', async () => {
    const user = userEvent.setup();
    const onRetry = vi.fn();
    renderWithProviders(
      <ErrorState message="Something went wrong" onRetry={onRetry} />
    );

    await user.click(screen.getByLabelText('Try Again'));
    expect(onRetry).toHaveBeenCalledTimes(1);
  });

  it('has proper accessibility attributes', () => {
    renderWithProviders(<ErrorState message="Something went wrong" />);
    const container = screen.getByRole('alert');
    expect(container).toHaveAttribute('aria-live', 'assertive');
  });

  it('applies custom className', () => {
    renderWithProviders(
      <ErrorState message="Something went wrong" className="my-custom-class" />
    );
    const container = screen.getByRole('alert');
    expect(container).toHaveClass('my-custom-class');
  });

  it('renders error icon', () => {
    const { container } = renderWithProviders(
      <ErrorState message="Something went wrong" />
    );
    const icon = container.querySelector('svg');
    expect(icon).toBeInTheDocument();
    expect(icon).toHaveClass('text-red-500');
  });

  it('renders retry icon when retry button is shown', () => {
    const onRetry = vi.fn();
    const { container } = renderWithProviders(
      <ErrorState message="Something went wrong" onRetry={onRetry} />
    );
    const icons = container.querySelectorAll('svg');
    // Should have 2 SVGs: error icon and retry icon
    expect(icons.length).toBeGreaterThanOrEqual(2);
  });

  it('centers content', () => {
    renderWithProviders(<ErrorState message="Something went wrong" />);
    const container = screen.getByRole('alert');
    expect(container).toHaveClass(
      'flex',
      'flex-col',
      'items-center',
      'justify-center'
    );
  });
});
