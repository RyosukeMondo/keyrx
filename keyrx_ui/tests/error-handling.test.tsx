/**
 * Error handling tests
 * Tests for proper error display, recovery, and toast notifications
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { ToastProvider } from '../src/components/ToastProvider';
import { ErrorBoundary } from '../src/components/ErrorBoundary';

describe('Error Handling Tests', () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    queryClient = new QueryClient({
      defaultOptions: {
        queries: { retry: false },
        mutations: { retry: false },
      },
    });
  });

  it('should catch and display errors in ErrorBoundary', async () => {
    const ThrowError = () => {
      throw new Error('Test error');
    };

    render(
      <ErrorBoundary>
        <ThrowError />
      </ErrorBoundary>
    );

    await waitFor(() => {
      expect(screen.getByText('Something went wrong')).toBeInTheDocument();
      expect(screen.getByText(/Test error/)).toBeInTheDocument();
    });
  });

  it('should show toast on promise rejection', async () => {
    const errorFn = vi.fn().mockRejectedValue(new Error('API Error'));

    const TestComponent = () => {
      const { error } = useToast();

      const handleClick = async () => {
        try {
          await errorFn();
        } catch (err) {
          error(err);
        }
      };

      return <button onClick={handleClick}>Trigger Error</button>;
    };

    render(
      <>
        <ToastProvider />
        <TestComponent />
      </>
    );

    await userEvent.click(screen.getByText('Trigger Error'));

    await waitFor(() => {
      expect(screen.getByText(/API Error/)).toBeInTheDocument();
    });
  });

  it('should display validation errors inline', async () => {
    const TestComponent = () => {
      const [name, setName] = React.useState('');
      const [error, setError] = React.useState('');

      const handleSubmit = () => {
        if (!name.trim()) {
          setError('Name is required');
          return;
        }
        setError('');
      };

      return (
        <div>
          <input
            value={name}
            onChange={e => setName(e.target.value)}
            data-testid="input"
          />
          {error && <div data-testid="error">{error}</div>}
          <button onClick={handleSubmit}>Submit</button>
        </div>
      );
    };

    render(<TestComponent />);

    await userEvent.click(screen.getByText('Submit'));

    await waitFor(() => {
      expect(screen.getByTestId('error')).toHaveTextContent('Name is required');
    });
  });

  it('should handle network errors gracefully', async () => {
    const fetchMock = vi.fn().mockRejectedValue(new Error('Failed to fetch'));

    const TestComponent = () => {
      const [error, setError] = React.useState<string | null>(null);

      const handleFetch = async () => {
        try {
          setError(null);
          await fetchMock();
        } catch (err) {
          setError(err instanceof Error ? err.message : 'Unknown error');
        }
      };

      return (
        <div>
          {error && <div data-testid="error" role="alert">{error}</div>}
          <button onClick={handleFetch}>Fetch</button>
        </div>
      );
    };

    render(<TestComponent />);

    await userEvent.click(screen.getByText('Fetch'));

    await waitFor(() => {
      expect(screen.getByTestId('error')).toHaveTextContent('Failed to fetch');
      expect(screen.getByRole('alert')).toBeInTheDocument();
    });
  });

  it('should show retry button on error', async () => {
    let shouldFail = true;
    const fetchMock = vi.fn().mockImplementation(() => {
      if (shouldFail) {
        return Promise.reject(new Error('API Error'));
      }
      return Promise.resolve({ data: 'success' });
    });

    const TestComponent = () => {
      const [data, setData] = React.useState<string | null>(null);
      const [error, setError] = React.useState<string | null>(null);

      const handleFetch = async () => {
        try {
          setError(null);
          const result = await fetchMock();
          setData(result.data);
        } catch (err) {
          setError(err instanceof Error ? err.message : 'Unknown error');
        }
      };

      return (
        <div>
          {error && (
            <div>
              <div data-testid="error">{error}</div>
              <button onClick={handleFetch}>Retry</button>
            </div>
          )}
          {data && <div data-testid="data">{data}</div>}
          {!error && !data && <button onClick={handleFetch}>Fetch</button>}
        </div>
      );
    };

    render(<TestComponent />);

    // First fetch fails
    await userEvent.click(screen.getByText('Fetch'));

    await waitFor(() => {
      expect(screen.getByTestId('error')).toHaveTextContent('API Error');
    });

    // Retry succeeds
    shouldFail = false;
    await userEvent.click(screen.getByText('Retry'));

    await waitFor(() => {
      expect(screen.getByTestId('data')).toHaveTextContent('success');
    });
  });

  it('should clear errors when user corrects input', async () => {
    const TestComponent = () => {
      const [email, setEmail] = React.useState('');
      const [error, setError] = React.useState('');

      const handleChange = (value: string) => {
        setEmail(value);
        if (error) setError(''); // Clear error on change
      };

      const handleSubmit = () => {
        if (!email.includes('@')) {
          setError('Invalid email');
        }
      };

      return (
        <div>
          <input
            value={email}
            onChange={e => handleChange(e.target.value)}
            data-testid="email"
          />
          {error && <div data-testid="error">{error}</div>}
          <button onClick={handleSubmit}>Submit</button>
        </div>
      );
    };

    render(<TestComponent />);

    const input = screen.getByTestId('email');

    // Invalid input
    await userEvent.type(input, 'invalid');
    await userEvent.click(screen.getByText('Submit'));

    await waitFor(() => {
      expect(screen.getByTestId('error')).toHaveTextContent('Invalid email');
    });

    // Correct input
    await userEvent.clear(input);
    await userEvent.type(input, 'valid@email.com');

    await waitFor(() => {
      expect(screen.queryByTestId('error')).not.toBeInTheDocument();
    });
  });
});
