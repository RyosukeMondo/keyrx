import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { LoadingSpinner } from './LoadingSpinner';

describe('LoadingSpinner', () => {
  it('renders with default size', () => {
    render(<LoadingSpinner />);
    const spinner = screen.getByRole('status');
    expect(spinner).toBeInTheDocument();
    expect(spinner).toHaveAttribute('aria-label', 'Loading');
  });

  it('renders with small size', () => {
    render(<LoadingSpinner size="sm" />);
    const spinner = screen.getByRole('status');
    expect(spinner).toHaveClass('w-4', 'h-4');
  });

  it('renders with medium size', () => {
    render(<LoadingSpinner size="md" />);
    const spinner = screen.getByRole('status');
    expect(spinner).toHaveClass('w-5', 'h-5');
  });

  it('renders with large size', () => {
    render(<LoadingSpinner size="lg" />);
    const spinner = screen.getByRole('status');
    expect(spinner).toHaveClass('w-6', 'h-6');
  });

  it('applies custom className', () => {
    render(<LoadingSpinner className="text-blue-500" />);
    const spinner = screen.getByRole('status');
    expect(spinner).toHaveClass('text-blue-500');
  });

  it('has animate-spin class for animation', () => {
    render(<LoadingSpinner />);
    const spinner = screen.getByRole('status');
    expect(spinner).toHaveClass('animate-spin');
  });
});
