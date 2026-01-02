import { lazy, Suspense } from 'react';
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { ErrorBoundary } from './components/ErrorBoundary';
import { Layout } from './components/Layout';
import { LoadingSpinner } from './components/LoadingSpinner';
import { WasmProvider } from './contexts/WasmContext';

// Lazy load all page components for code splitting
const HomePage = lazy(() => import('./pages/HomePage'));
const DevicesPage = lazy(() => import('./pages/DevicesPage'));
const ProfilesPage = lazy(() => import('./pages/ProfilesPage'));
const ConfigPage = lazy(() => import('./pages/ConfigPage'));
const MetricsPage = lazy(() => import('./pages/MetricsPage'));
const SimulatorPage = lazy(() => import('./pages/SimulatorPage'));

function App() {
  return (
    <ErrorBoundary>
      <WasmProvider>
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
                <Route path="/" element={<Navigate to="/home" replace />} />
                <Route path="/home" element={<HomePage />} />
                <Route path="/devices" element={<DevicesPage />} />
                <Route path="/profiles" element={<ProfilesPage />} />
                <Route path="/config" element={<ConfigPage />} />
                <Route path="/metrics" element={<MetricsPage />} />
                <Route path="/simulator" element={<SimulatorPage />} />
                <Route path="*" element={<Navigate to="/home" replace />} />
              </Routes>
            </Suspense>
          </Layout>
        </BrowserRouter>
      </WasmProvider>
    </ErrorBoundary>
  );
}

export default App;
