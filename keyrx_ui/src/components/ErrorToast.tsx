/**
 * ErrorToast - A simple toast notification component for displaying errors
 *
 * Displays error messages from the config builder store and allows users to dismiss them.
 */

import React, { useEffect } from 'react';
import { useConfigBuilderStore } from '@/store/configBuilderStore';
import './ErrorToast.css';

/**
 * ErrorToast component
 *
 * Automatically displays errors from the store and allows dismissal.
 * Errors auto-dismiss after 5 seconds.
 */
export function ErrorToast() {
  const { lastError, clearError } = useConfigBuilderStore();

  // Auto-dismiss after 5 seconds
  useEffect(() => {
    if (lastError) {
      const timer = setTimeout(() => {
        clearError();
      }, 5000);

      return () => clearTimeout(timer);
    }
  }, [lastError, clearError]);

  if (!lastError) {
    return null;
  }

  return (
    <div className="error-toast" role="alert" aria-live="assertive">
      <div className="error-toast-content">
        <span className="error-icon" aria-hidden="true">⚠️</span>
        <span className="error-message">{lastError}</span>
        <button
          className="error-dismiss"
          onClick={clearError}
          aria-label="Dismiss error"
        >
          ✕
        </button>
      </div>
    </div>
  );
}
