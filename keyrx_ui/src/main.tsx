import React, { StrictMode } from 'react';
import ReactDOM from 'react-dom';
import { createRoot } from 'react-dom/client';
import { QueryClientProvider } from '@tanstack/react-query';
import { queryClient } from './lib/queryClient';
import { ErrorBoundary } from './components/ErrorBoundary';
import { logger } from './utils/logger';
import './styles/tokens.css';
import './index.css';
import App from './App.tsx';
import { applyDocumentLanguage } from './i18n';
import { installStaleBuildRecovery } from './utils/lazyWithReload';

applyDocumentLanguage();
installStaleBuildRecovery();

// Enable axe-core accessibility testing in development
if (import.meta.env.DEV) {
  import('@axe-core/react')
    .then((axe) => {
      axe.default(React, ReactDOM, 1000);
    })
    .catch((error) => {
      logger.error('axe_core_load_failed', error);
    });
}

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <ErrorBoundary>
      <QueryClientProvider client={queryClient}>
        <App />
      </QueryClientProvider>
    </ErrorBoundary>
  </StrictMode>
);
