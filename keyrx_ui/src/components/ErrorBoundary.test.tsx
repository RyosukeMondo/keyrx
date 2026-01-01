import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { ErrorBoundary } from './ErrorBoundary';

// Component that throws an error
const ThrowError = ({ shouldThrow = true }: { shouldThrow?: boolean }) => {
  if (shouldThrow) {
    throw new Error('Test error message');
  }
  return <div>No error</div>;
};

describe('ErrorBoundary', () => {
  // Suppress console.error for these tests
  const originalError = console.error;
  beforeEach(() => {
    console.error = vi.fn();
  });

  afterEach(() => {
    console.error = originalError;
  });

  it('renders children when there is no error', () => {
    render(
      <ErrorBoundary>
        <div>Child component</div>
      </ErrorBoundary>
    );

    expect(screen.getByText('Child component')).toBeInTheDocument();
  });

  it('catches errors and displays default fallback UI', () => {
    render(
      <ErrorBoundary>
        <ThrowError />
      </ErrorBoundary>
    );

    expect(screen.getByText('Something went wrong')).toBeInTheDocument();
    expect(screen.getByText('An unexpected error occurred in the application.')).toBeInTheDocument();
  });

  it('displays error message in details', () => {
    render(
      <ErrorBoundary>
        <ThrowError />
      </ErrorBoundary>
    );

    const details = screen.getByText('Show error details');
    expect(details).toBeInTheDocument();

    // Error message should be in the details
    expect(screen.getByText('Test error message')).toBeInTheDocument();
  });

  it('renders Try Again button', () => {
    render(
      <ErrorBoundary>
        <ThrowError />
      </ErrorBoundary>
    );

    expect(screen.getByLabelText('Try again')).toBeInTheDocument();
  });

  it('renders Reload Page button', () => {
    render(
      <ErrorBoundary>
        <ThrowError />
      </ErrorBoundary>
    );

    expect(screen.getByLabelText('Reload page')).toBeInTheDocument();
  });

  it('resets error state when Try Again is clicked', async () => {
    const user = userEvent.setup();
    let shouldThrow = true;

    const { rerender } = render(
      <ErrorBoundary>
        <ThrowError shouldThrow={shouldThrow} />
      </ErrorBoundary>
    );

    // Error should be caught
    expect(screen.getByText('Something went wrong')).toBeInTheDocument();

    // Fix the error condition
    shouldThrow = false;

    // Click Try Again
    await user.click(screen.getByLabelText('Try again'));

    // Rerender with fixed component
    rerender(
      <ErrorBoundary>
        <ThrowError shouldThrow={shouldThrow} />
      </ErrorBoundary>
    );

    // Should show children now
    expect(screen.getByText('No error')).toBeInTheDocument();
  });

  it('uses custom fallback when provided', () => {
    const customFallback = (error: Error, resetError: () => void) => (
      <div>
        <p>Custom error: {error.message}</p>
        <button onClick={resetError}>Custom Reset</button>
      </div>
    );

    render(
      <ErrorBoundary fallback={customFallback}>
        <ThrowError />
      </ErrorBoundary>
    );

    expect(screen.getByText('Custom error: Test error message')).toBeInTheDocument();
    expect(screen.getByText('Custom Reset')).toBeInTheDocument();
    expect(screen.queryByText('Something went wrong')).not.toBeInTheDocument();
  });

  it('calls console.error when error is caught', () => {
    const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

    render(
      <ErrorBoundary>
        <ThrowError />
      </ErrorBoundary>
    );

    expect(consoleErrorSpy).toHaveBeenCalled();
    consoleErrorSpy.mockRestore();
  });

  it('reloads page when Reload Page button is clicked', async () => {
    const user = userEvent.setup();
    const reloadMock = vi.fn();
    Object.defineProperty(window, 'location', {
      value: { reload: reloadMock },
      writable: true,
    });

    render(
      <ErrorBoundary>
        <ThrowError />
      </ErrorBoundary>
    );

    await user.click(screen.getByLabelText('Reload page'));

    expect(reloadMock).toHaveBeenCalledTimes(1);
  });
});
