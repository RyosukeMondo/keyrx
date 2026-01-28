/**
 * Toast notification hook using sonner
 * Provides consistent error/success messaging across the app
 */

import { toast as sonnerToast } from 'sonner';
import { getErrorMessage } from '../utils/typeGuards';

export interface ToastOptions {
  duration?: number;
  description?: string;
  action?: {
    label: string;
    onClick: () => void;
  };
}

/**
 * Custom toast hook with standardized error handling
 */
export function useToast() {
  const success = (message: string, options?: ToastOptions) => {
    sonnerToast.success(message, {
      duration: options?.duration ?? 3000,
      description: options?.description,
      action: options?.action,
    });
  };

  const error = (messageOrError: string | unknown, options?: ToastOptions) => {
    const message = typeof messageOrError === 'string'
      ? messageOrError
      : getErrorMessage(messageOrError);

    sonnerToast.error(message, {
      duration: options?.duration ?? 5000,
      description: options?.description,
      action: options?.action,
    });
  };

  const info = (message: string, options?: ToastOptions) => {
    sonnerToast.info(message, {
      duration: options?.duration ?? 3000,
      description: options?.description,
      action: options?.action,
    });
  };

  const warning = (message: string, options?: ToastOptions) => {
    sonnerToast.warning(message, {
      duration: options?.duration ?? 4000,
      description: options?.description,
      action: options?.action,
    });
  };

  const promise = <T,>(
    promise: Promise<T>,
    messages: {
      loading: string;
      success: string | ((data: T) => string);
      error: string | ((error: unknown) => string);
    }
  ) => {
    return sonnerToast.promise(promise, messages);
  };

  return {
    success,
    error,
    info,
    warning,
    promise,
    dismiss: sonnerToast.dismiss,
  };
}
