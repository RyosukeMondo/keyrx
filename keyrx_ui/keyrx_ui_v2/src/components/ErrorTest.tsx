import React, { useState } from 'react';
import { Button } from './Button';
import { ErrorState } from './ErrorState';

/**
 * Test component for demonstrating ErrorBoundary and ErrorState functionality.
 * This component is for development/testing purposes only.
 */
export const ErrorTest: React.FC = () => {
  const [showApiError, setShowApiError] = useState(false);
  const [throwError, setThrowError] = useState(false);

  // Simulate API error state
  const handleApiError = () => {
    setShowApiError(true);
  };

  // Retry handler for API error
  const handleRetry = () => {
    setShowApiError(false);
  };

  // This will trigger the ErrorBoundary
  if (throwError) {
    throw new Error('Test error thrown intentionally to demonstrate ErrorBoundary');
  }

  return (
    <div className="min-h-screen bg-slate-900 p-8">
      <div className="max-w-4xl mx-auto">
        <h1 className="text-3xl font-bold text-slate-100 mb-8">
          Error Handling Test
        </h1>

        <div className="space-y-6">
          {/* ErrorState Demo */}
          <div className="bg-slate-800 rounded-lg p-6">
            <h2 className="text-xl font-semibold text-slate-100 mb-4">
              ErrorState Component (API Errors)
            </h2>
            <p className="text-slate-400 text-sm mb-4">
              This demonstrates how failed API calls are displayed with a retry option.
            </p>

            {showApiError ? (
              <ErrorState
                title="Failed to Load Data"
                message="Unable to fetch data from the server. Please check your connection and try again."
                onRetry={handleRetry}
                retryLabel="Retry Request"
              />
            ) : (
              <div className="space-y-4">
                <p className="text-slate-300">
                  API request successful. Click below to simulate a failure.
                </p>
                <Button
                  variant="danger"
                  size="md"
                  onClick={handleApiError}
                  aria-label="Simulate API error"
                >
                  Simulate API Error
                </Button>
              </div>
            )}
          </div>

          {/* ErrorBoundary Demo */}
          <div className="bg-slate-800 rounded-lg p-6">
            <h2 className="text-xl font-semibold text-slate-100 mb-4">
              ErrorBoundary Component (React Errors)
            </h2>
            <p className="text-slate-400 text-sm mb-4">
              This demonstrates how JavaScript errors in components are caught to
              prevent the entire app from crashing.
            </p>
            <div className="bg-red-500/10 border border-red-500/20 rounded p-4 mb-4">
              <p className="text-red-400 text-sm">
                <strong>Warning:</strong> Clicking this button will throw an error
                and trigger the ErrorBoundary. The entire app will show the error
                fallback UI.
              </p>
            </div>
            <Button
              variant="danger"
              size="md"
              onClick={() => setThrowError(true)}
              aria-label="Throw error to test ErrorBoundary"
            >
              Throw Error (Test ErrorBoundary)
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
};
