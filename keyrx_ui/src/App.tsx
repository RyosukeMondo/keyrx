import { lazy, Suspense } from 'react';
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { ErrorBoundary } from './components/ErrorBoundary';
import { Layout } from './components/Layout';
import { LoadingSpinner } from './components/LoadingSpinner';
import { LayoutPreviewProvider } from './contexts/LayoutPreviewContext';
import { ToastProvider } from './components/ToastProvider';
import { WasmProvider } from './contexts/WasmContext';

// Lazy load page components for code splitting
const ConfigPage = lazy(() => import('./pages/ConfigPage'));
const DevicesPage = lazy(() => import('./pages/DevicesPage'));
const MonitorPage = lazy(() => import('./pages/MonitorPage'));

function App() {
  return (
    <ErrorBoundary>
      <WasmProvider>
        <LayoutPreviewProvider>
          <BrowserRouter>
            <Layout>
              <Suspense
                fallback={
                  <div className="flex items-center justify-center min-h-screen">
                    <LoadingSpinner size="lg" />
                  </div>
                }
              >
                <Routes>
                  <Route path="/" element={<ConfigPage />} />
                  <Route
                    path="/profiles/:name/config"
                    element={<ConfigPage />}
                  />
                  <Route path="/devices" element={<DevicesPage />} />
                  <Route path="/monitor" element={<MonitorPage />} />

                  {/* Legacy redirects */}
                  <Route
                    path="/home"
                    element={<Navigate to="/" replace />}
                  />
                  <Route
                    path="/profiles"
                    element={<Navigate to="/" replace />}
                  />
                  <Route
                    path="/config"
                    element={<Navigate to="/" replace />}
                  />
                  <Route
                    path="/simulator"
                    element={<Navigate to="/" replace />}
                  />
                  <Route
                    path="/metrics"
                    element={<Navigate to="/monitor" replace />}
                  />
                  <Route path="*" element={<Navigate to="/" replace />} />
                </Routes>
              </Suspense>
            </Layout>
          </BrowserRouter>
          <ToastProvider />
        </LayoutPreviewProvider>
      </WasmProvider>
    </ErrorBoundary>
  );
}

export default App;
