/**
 * Toast provider component using sonner
 * Wraps the app to enable toast notifications
 */

import { Toaster } from 'sonner';

export function ToastProvider() {
  return (
    <Toaster
      position="top-right"
      expand={false}
      richColors
      closeButton
      duration={3000}
      toastOptions={{
        style: {
          background: 'rgb(30, 41, 59)', // slate-800
          border: '1px solid rgb(51, 65, 85)', // slate-700
          color: 'rgb(226, 232, 240)', // slate-200
        },
        className: 'toast',
      }}
    />
  );
}
