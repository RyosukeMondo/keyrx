import React from 'react';

export interface WasmStatusBadgeProps {
  isLoading: boolean;
  isReady: boolean;
  error: Error | null;
  className?: string;
}

/**
 * Badge component displaying WASM loading status with visual indicators
 *
 * Shows one of three states:
 * - Loading: Spinner + "Loading WASM..."
 * - Ready: Green checkmark + "WASM Ready"
 * - Error: Red X + "WASM Error: [message]" + troubleshooting hint
 */
export const WasmStatusBadge: React.FC<WasmStatusBadgeProps> = ({
  isLoading,
  isReady,
  error,
  className = '',
}) => {
  if (isLoading) {
    return (
      <div
        className={`flex items-center gap-2 px-3 py-1.5 rounded-md bg-yellow-500/10 border border-yellow-500/30 ${className}`}
        role="status"
        aria-live="polite"
      >
        <svg
          className="animate-spin h-4 w-4 text-yellow-500"
          xmlns="http://www.w3.org/2000/svg"
          fill="none"
          viewBox="0 0 24 24"
          aria-hidden="true"
        >
          <circle
            className="opacity-25"
            cx="12"
            cy="12"
            r="10"
            stroke="currentColor"
            strokeWidth="4"
          />
          <path
            className="opacity-75"
            fill="currentColor"
            d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
          />
        </svg>
        <span className="text-sm text-yellow-500 font-medium">
          Loading WASM...
        </span>
      </div>
    );
  }

  if (error) {
    return (
      <div
        className={`flex flex-col gap-1.5 px-3 py-2 rounded-md bg-red-500/10 border border-red-500/30 ${className}`}
        role="alert"
        aria-live="assertive"
      >
        <div className="flex items-center gap-2">
          <svg
            className="h-4 w-4 text-red-500 shrink-0"
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 20 20"
            fill="currentColor"
            aria-hidden="true"
          >
            <path
              fillRule="evenodd"
              d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.28 7.22a.75.75 0 00-1.06 1.06L8.94 10l-1.72 1.72a.75.75 0 101.06 1.06L10 11.06l1.72 1.72a.75.75 0 101.06-1.06L11.06 10l1.72-1.72a.75.75 0 00-1.06-1.06L10 8.94 8.28 7.22z"
              clipRule="evenodd"
            />
          </svg>
          <span className="text-sm text-red-400 font-medium">
            WASM Error: {error.message}
          </span>
        </div>
        <p className="text-xs text-red-300 ml-6">
          Run:{' '}
          <code className="bg-slate-900 px-1.5 py-0.5 rounded font-mono">
            npm run build:wasm
          </code>
        </p>
      </div>
    );
  }

  if (isReady) {
    return (
      <div
        className={`flex items-center gap-2 px-3 py-1.5 rounded-md bg-green-500/10 border border-green-500/30 ${className}`}
        role="status"
        aria-live="polite"
      >
        <svg
          className="h-4 w-4 text-green-500"
          xmlns="http://www.w3.org/2000/svg"
          viewBox="0 0 20 20"
          fill="currentColor"
          aria-hidden="true"
        >
          <path
            fillRule="evenodd"
            d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.857-9.809a.75.75 0 00-1.214-.882l-3.483 4.79-1.88-1.88a.75.75 0 10-1.06 1.061l2.5 2.5a.75.75 0 001.137-.089l4-5.5z"
            clipRule="evenodd"
          />
        </svg>
        <span className="text-sm text-green-400 font-medium">WASM Ready</span>
      </div>
    );
  }

  // Default: Not ready and not loading (shouldn't happen)
  return null;
};
