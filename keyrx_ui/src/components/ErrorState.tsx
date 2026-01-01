import React from 'react';
import { Button } from './Button';

interface ErrorStateProps {
  title?: string;
  message: string;
  onRetry?: () => void;
  retryLabel?: string;
  className?: string;
}

/**
 * ErrorState component displays user-friendly error messages for failed API calls
 * or other operations. Provides a retry button to allow users to attempt the
 * operation again.
 *
 * Used in pages when API requests fail to provide feedback and recovery options.
 */
export const ErrorState: React.FC<ErrorStateProps> = ({
  title = 'Error',
  message,
  onRetry,
  retryLabel = 'Try Again',
  className = '',
}) => {
  return (
    <div
      className={`flex flex-col items-center justify-center p-8 ${className}`}
      role="alert"
      aria-live="assertive"
    >
      <div className="flex flex-col items-center max-w-md text-center">
        {/* Error icon */}
        <div className="mb-4 p-3 bg-red-500/10 rounded-full">
          <svg
            className="h-12 w-12 text-red-500"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
            aria-hidden="true"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
            />
          </svg>
        </div>

        {/* Error title */}
        <h3 className="text-xl font-semibold text-slate-100 mb-2">{title}</h3>

        {/* Error message */}
        <p className="text-slate-400 text-sm mb-6">{message}</p>

        {/* Retry button */}
        {onRetry && (
          <Button
            variant="primary"
            size="md"
            onClick={onRetry}
            aria-label={retryLabel}
          >
            <svg
              className="h-4 w-4 mr-2"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
              aria-hidden="true"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
              />
            </svg>
            {retryLabel}
          </Button>
        )}
      </div>
    </div>
  );
};
