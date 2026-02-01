import React, { Component, ReactNode } from 'react';
import { Button } from './Button';

interface ErrorBoundaryProps {
  children: ReactNode;
  fallback?: (error: Error, resetError: () => void) => ReactNode;
}

interface ErrorBoundaryState {
  hasError: boolean;
  error: Error | null;
  errorInfo: React.ErrorInfo | null;
}

/**
 * ErrorBoundary catches JavaScript errors anywhere in the child component tree,
 * logs those errors, and displays a fallback UI instead of crashing the whole app.
 *
 * Used to wrap the entire application or specific feature areas to provide
 * graceful error handling and prevent white screens.
 */
export class ErrorBoundary extends Component<
  ErrorBoundaryProps,
  ErrorBoundaryState
> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = {
      hasError: false,
      error: null,
      errorInfo: null,
    };
  }

  static getDerivedStateFromError(error: Error): Partial<ErrorBoundaryState> {
    // Update state so the next render will show the fallback UI
    return {
      hasError: true,
      error,
    };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo): void {
    // Update state with error details
    this.setState({
      error,
      errorInfo,
    });

    // In production, you could send this to an error reporting service
    // Example: Sentry.captureException(error, { extra: errorInfo });
  }

  resetError = (): void => {
    this.setState({
      hasError: false,
      error: null,
      errorInfo: null,
    });
  };

  render(): ReactNode {
    const { hasError, error } = this.state;
    const { children, fallback } = this.props;

    if (hasError && error) {
      // Use custom fallback if provided
      if (fallback) {
        return fallback(error, this.resetError);
      }

      // Default fallback UI
      return (
        <div className="min-h-screen flex items-center justify-center bg-slate-900 p-4">
          <div className="max-w-md w-full bg-slate-800 rounded-lg p-6 shadow-xl">
            <div className="flex items-start mb-4">
              <div className="flex-shrink-0">
                <svg
                  className="h-6 w-6 text-red-500"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                  aria-hidden="true"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
                  />
                </svg>
              </div>
              <div className="ml-3 flex-1">
                <h3 className="text-lg font-semibold text-slate-100">
                  Something went wrong
                </h3>
                <div className="mt-2 text-sm text-slate-400">
                  <p>An unexpected error occurred in the application.</p>
                </div>
              </div>
            </div>

            {/* Error details (collapsed by default in production) */}
            <details className="mt-4">
              <summary className="text-sm text-slate-400 cursor-pointer hover:text-slate-300 transition-colors">
                Show error details
              </summary>
              <div className="mt-2 p-3 bg-slate-900 rounded border border-slate-700">
                <p className="text-xs text-red-400 font-mono break-all">
                  {error.message}
                </p>
                {this.state.errorInfo && (
                  <pre className="mt-2 text-xs text-slate-400 overflow-auto max-h-48">
                    {this.state.errorInfo.componentStack}
                  </pre>
                )}
              </div>
            </details>

            {/* Action buttons */}
            <div className="mt-6 flex gap-3">
              <Button
                variant="primary"
                size="md"
                onClick={this.resetError}
                aria-label="Try again"
                className="flex-1"
              >
                Try Again
              </Button>
              <Button
                variant="secondary"
                size="md"
                onClick={() => window.location.reload()}
                aria-label="Reload page"
                className="flex-1"
              >
                Reload Page
              </Button>
            </div>
          </div>
        </div>
      );
    }

    return children;
  }
}
