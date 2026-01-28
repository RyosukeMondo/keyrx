/**
 * Race condition tests
 * Tests for proper handling of concurrent async operations
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { ProfilesPage } from '../src/pages/ProfilesPage';

describe('Race Condition Tests', () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    queryClient = new QueryClient({
      defaultOptions: {
        queries: { retry: false },
        mutations: { retry: false },
      },
    });
  });

  it('should handle rapid profile activation clicks - prevents double activation', async () => {
    const activateMock = vi.fn().mockImplementation(() =>
      new Promise(resolve => setTimeout(() => resolve({ success: true }), 100))
    );

    vi.mock('../src/hooks/useProfiles', () => ({
      useProfiles: () => ({
        data: [
          { name: 'profile1', isActive: false, modifiedAt: Date.now() },
          { name: 'profile2', isActive: true, modifiedAt: Date.now() },
        ],
        isLoading: false,
        error: null,
      }),
      useActivateProfile: () => ({
        mutateAsync: activateMock,
        isPending: false,
      }),
      useCreateProfile: () => ({ mutateAsync: vi.fn() }),
      useDeleteProfile: () => ({ mutateAsync: vi.fn() }),
      useUpdateProfile: () => ({ mutateAsync: vi.fn() }),
    }));

    render(
      <QueryClientProvider client={queryClient}>
        <ProfilesPage />
      </QueryClientProvider>
    );

    const activateButton = screen.getByText('Activate');

    // Rapid clicks
    await userEvent.click(activateButton);
    await userEvent.click(activateButton);
    await userEvent.click(activateButton);

    await waitFor(() => {
      // Should only call once due to isPending check
      expect(activateMock).toHaveBeenCalledTimes(1);
    });
  });

  it('should use functional setState updates to avoid stale closures', async () => {
    const TestComponent = () => {
      const [count, setCount] = React.useState(0);

      const handleClick = () => {
        // Bad: stale closure
        // setCount(count + 1);

        // Good: functional update
        setCount(prev => prev + 1);
      };

      return (
        <div>
          <div data-testid="count">{count}</div>
          <button onClick={handleClick}>Increment</button>
        </div>
      );
    };

    render(<TestComponent />);

    const button = screen.getByText('Increment');

    // Rapid clicks
    await userEvent.click(button);
    await userEvent.click(button);
    await userEvent.click(button);

    await waitFor(() => {
      expect(screen.getByTestId('count')).toHaveTextContent('3');
    });
  });

  it('should cancel previous requests when making new ones', async () => {
    let requestCount = 0;
    const abortedRequests: number[] = [];

    const fetchMock = vi.fn().mockImplementation((signal: AbortSignal) => {
      const requestId = ++requestCount;

      return new Promise((resolve, reject) => {
        signal.addEventListener('abort', () => {
          abortedRequests.push(requestId);
          reject(new DOMException('Aborted', 'AbortError'));
        });

        setTimeout(() => resolve({ id: requestId }), 100);
      });
    });

    const TestComponent = () => {
      const [query, setQuery] = React.useState('');
      const [results, setResults] = React.useState<{ id: number } | null>(null);
      const abortControllerRef = React.useRef<AbortController | null>(null);

      React.useEffect(() => {
        if (!query) return;

        // Cancel previous request
        if (abortControllerRef.current) {
          abortControllerRef.current.abort();
        }

        const controller = new AbortController();
        abortControllerRef.current = controller;

        fetchMock(controller.signal)
          .then(setResults)
          .catch((err: Error) => {
            if (err.name !== 'AbortError') throw err;
          });

        return () => controller.abort();
      }, [query]);

      return (
        <div>
          <input
            value={query}
            onChange={e => setQuery(e.target.value)}
            data-testid="search"
          />
          <div data-testid="results">
            {results ? `Result: ${results.id}` : 'No results'}
          </div>
        </div>
      );
    };

    render(<TestComponent />);

    const input = screen.getByTestId('search');

    // Rapid typing
    await userEvent.type(input, 'abc');

    await waitFor(() => {
      // Should abort previous requests
      expect(abortedRequests.length).toBeGreaterThan(0);
      // Only last request should complete
      expect(screen.getByTestId('results')).toHaveTextContent(`Result: ${requestCount}`);
    });
  });

  it('should handle concurrent mutations with optimistic updates', async () => {
    const mutations: string[] = [];

    const mutateMock = vi.fn().mockImplementation((value: string) => {
      mutations.push(value);
      return new Promise(resolve => setTimeout(() => resolve({ value }), 50));
    });

    const TestComponent = () => {
      const [value, setValue] = React.useState('initial');
      const [pending, setPending] = React.useState(false);

      const handleMutate = async (newValue: string) => {
        // Optimistic update
        const previousValue = value;
        setValue(newValue);
        setPending(true);

        try {
          await mutateMock(newValue);
        } catch (error) {
          // Rollback on error
          setValue(previousValue);
        } finally {
          setPending(false);
        }
      };

      return (
        <div>
          <div data-testid="value">{value}</div>
          <div data-testid="pending">{pending ? 'pending' : 'idle'}</div>
          <button onClick={() => handleMutate('value1')}>Mutate 1</button>
          <button onClick={() => handleMutate('value2')}>Mutate 2</button>
        </div>
      );
    };

    render(<TestComponent />);

    const button1 = screen.getByText('Mutate 1');
    const button2 = screen.getByText('Mutate 2');

    // Click both rapidly
    await userEvent.click(button1);
    await userEvent.click(button2);

    // Value should update optimistically immediately
    expect(screen.getByTestId('value')).toHaveTextContent('value2');

    await waitFor(() => {
      expect(mutations).toContain('value1');
      expect(mutations).toContain('value2');
    });
  });
});
