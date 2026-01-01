import React, { useState } from 'react';
import { Outlet } from 'react-router-dom';
import { SkipToContent } from './SkipToContent';
import { useKeyboardShortcuts, CommonShortcuts } from '../hooks/useKeyboardShortcuts';

interface AppShellProps {
  children?: React.ReactNode;
}

export const AppShell: React.FC<AppShellProps> = ({ children }) => {
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false);

  const toggleSidebar = () => {
    setSidebarCollapsed(!sidebarCollapsed);
  };

  // Register global keyboard shortcuts
  useKeyboardShortcuts([
    CommonShortcuts.toggleSidebar(toggleSidebar),
  ]);

  return (
    <div className="app-shell min-h-screen bg-slate-900 text-slate-100">
      <SkipToContent />
      {/* Mobile/Tablet Header - hidden on desktop */}
      <header className="lg:hidden fixed top-0 left-0 right-0 h-16 bg-slate-800 border-b border-slate-700 z-fixed flex items-center px-4">
        <button
          onClick={toggleSidebar}
          aria-label="Toggle navigation menu"
          className="md:inline-flex lg:hidden p-2 rounded-md hover:bg-slate-700 focus:outline focus:outline-2 focus:outline-primary-500"
        >
          <svg
            className="w-6 h-6"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            {sidebarCollapsed ? (
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M6 18L18 6M6 6l12 12"
              />
            ) : (
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M4 6h16M4 12h16M4 18h16"
              />
            )}
          </svg>
        </button>
        <div className="flex-1 flex items-center justify-center">
          <span className="text-lg font-semibold">KeyRx2</span>
        </div>
      </header>

      {/* Desktop Sidebar */}
      <aside
        className={`
          hidden lg:block
          fixed top-0 left-0 bottom-0
          w-64 bg-slate-800 border-r border-slate-700
          transition-transform duration-300
          ${sidebarCollapsed ? '-translate-x-full' : 'translate-x-0'}
        `}
        aria-label="Main navigation"
      >
        <div className="h-full flex flex-col">
          {/* Logo/Brand */}
          <div className="h-16 flex items-center px-6 border-b border-slate-700">
            <span className="text-xl font-bold text-primary-500">KeyRx2</span>
          </div>

          {/* Navigation placeholder - will be replaced by Sidebar component */}
          <nav className="flex-1 overflow-y-auto py-4">
            <div className="px-3 space-y-1">
              <div className="px-3 py-2 rounded-md text-sm text-slate-400">
                Navigation items will go here
              </div>
            </div>
          </nav>
        </div>
      </aside>

      {/* Tablet Sidebar Overlay */}
      <aside
        className={`
          fixed top-16 left-0 bottom-0
          w-64 bg-slate-800 border-r border-slate-700
          transform transition-transform duration-300 z-fixed
          md:block lg:hidden
          ${sidebarCollapsed ? 'translate-x-0' : '-translate-x-full'}
        `}
        aria-label="Main navigation"
      >
        <nav className="h-full overflow-y-auto py-4">
          <div className="px-3 space-y-1">
            <div className="px-3 py-2 rounded-md text-sm text-slate-400">
              Navigation items will go here
            </div>
          </div>
        </nav>
      </aside>

      {/* Backdrop for tablet sidebar */}
      {sidebarCollapsed && (
        <div
          className="fixed inset-0 bg-black/50 z-sticky md:block lg:hidden"
          onClick={toggleSidebar}
          aria-hidden="true"
        />
      )}

      {/* Main Content Area */}
      <main
        id="main-content"
        tabIndex={-1}
        className={`
          min-h-screen
          pt-16 lg:pt-0
          pb-16 md:pb-0
          transition-all duration-300
          ${sidebarCollapsed ? 'lg:pl-0' : 'lg:pl-64'}
          focus:outline-none
        `}
        aria-label="Main content"
      >
        <div className="h-full">
          {children || <Outlet />}
        </div>
      </main>

      {/* Mobile Bottom Navigation - hidden on tablet/desktop */}
      <nav
        className="md:hidden fixed bottom-0 left-0 right-0 h-16 bg-slate-800 border-t border-slate-700 z-fixed"
        aria-label="Mobile navigation"
      >
        <div className="h-full flex items-center justify-around px-2">
          <button
            className="flex flex-col items-center justify-center h-full px-3 py-1 text-slate-400 hover:text-primary-500 focus:outline focus:outline-2 focus:outline-primary-500"
            aria-label="Home"
          >
            <svg className="w-6 h-6" fill="currentColor" viewBox="0 0 24 24">
              <path d="M10 20v-6h4v6h5v-8h3L12 3 2 12h3v8z" />
            </svg>
            <span className="text-xs mt-1">Home</span>
          </button>

          <button
            className="flex flex-col items-center justify-center h-full px-3 py-1 text-slate-400 hover:text-primary-500 focus:outline focus:outline-2 focus:outline-primary-500"
            aria-label="Devices"
          >
            <svg className="w-6 h-6" fill="currentColor" viewBox="0 0 24 24">
              <path d="M20 18c1.1 0 1.99-.9 1.99-2L22 6c0-1.1-.9-2-2-2H4c-1.1 0-2 .9-2 2v10c0 1.1.9 2 2 2H0v2h24v-2h-4zM4 6h16v10H4V6z" />
            </svg>
            <span className="text-xs mt-1">Devices</span>
          </button>

          <button
            className="flex flex-col items-center justify-center h-full px-3 py-1 text-slate-400 hover:text-primary-500 focus:outline focus:outline-2 focus:outline-primary-500"
            aria-label="Profiles"
          >
            <svg className="w-6 h-6" fill="currentColor" viewBox="0 0 24 24">
              <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 3c1.66 0 3 1.34 3 3s-1.34 3-3 3-3-1.34-3-3 1.34-3 3-3zm0 14.2c-2.5 0-4.71-1.28-6-3.22.03-1.99 4-3.08 6-3.08 1.99 0 5.97 1.09 6 3.08-1.29 1.94-3.5 3.22-6 3.22z" />
            </svg>
            <span className="text-xs mt-1">Profiles</span>
          </button>

          <button
            className="flex flex-col items-center justify-center h-full px-3 py-1 text-slate-400 hover:text-primary-500 focus:outline focus:outline-2 focus:outline-primary-500"
            aria-label="Config"
          >
            <svg className="w-6 h-6" fill="currentColor" viewBox="0 0 24 24">
              <path d="M19.14 12.94c.04-.3.06-.61.06-.94 0-.32-.02-.64-.07-.94l2.03-1.58c.18-.14.23-.41.12-.61l-1.92-3.32c-.12-.22-.37-.29-.59-.22l-2.39.96c-.5-.38-1.03-.7-1.62-.94L14.4 2.81c-.04-.24-.24-.41-.48-.41h-3.84c-.24 0-.43.17-.47.41l-.36 2.54c-.59.24-1.13.57-1.62.94l-2.39-.96c-.22-.08-.47 0-.59.22L2.74 8.87c-.12.21-.08.47.12.61l2.03 1.58c-.05.3-.09.63-.09.94s.02.64.07.94l-2.03 1.58c-.18.14-.23.41-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96c.5.38 1.03.7 1.62.94l.36 2.54c.05.24.24.41.48.41h3.84c.24 0 .44-.17.47-.41l.36-2.54c.59-.24 1.13-.56 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32c.12-.22.07-.47-.12-.61l-2.01-1.58zM12 15.6c-1.98 0-3.6-1.62-3.6-3.6s1.62-3.6 3.6-3.6 3.6 1.62 3.6 3.6-1.62 3.6-3.6 3.6z" />
            </svg>
            <span className="text-xs mt-1">Config</span>
          </button>

          <button
            className="flex flex-col items-center justify-center h-full px-3 py-1 text-slate-400 hover:text-primary-500 focus:outline focus:outline-2 focus:outline-primary-500"
            aria-label="Metrics"
          >
            <svg className="w-6 h-6" fill="currentColor" viewBox="0 0 24 24">
              <path d="M16 6l2.29 2.29-4.88 4.88-4-4L2 16.59 3.41 18l6-6 4 4 6.3-6.29L22 12V6z" />
            </svg>
            <span className="text-xs mt-1">Metrics</span>
          </button>
        </div>
      </nav>
    </div>
  );
};
