import React, { useState } from 'react';
import { BottomNav } from './BottomNav';
import { Sidebar } from './Sidebar';
import { VERSION, BUILD_TIME } from '../version';

interface LayoutProps {
  children: React.ReactNode;
}

/**
 * Layout - Responsive layout component with navigation
 *
 * Renders different navigation components based on viewport size:
 * - Mobile (< 768px): BottomNav (fixed bottom navigation bar)
 * - Desktop (>= 768px): Sidebar (fixed left sidebar)
 *
 * The layout automatically adjusts content padding to prevent overlap
 * with the navigation elements.
 *
 * @example
 * ```tsx
 * <Layout>
 *   <YourPageContent />
 * </Layout>
 * ```
 */
export const Layout: React.FC<LayoutProps> = ({ children }) => {
  const [isSidebarOpen, setIsSidebarOpen] = useState(false);
  const [isSidebarCollapsed, setIsSidebarCollapsed] = useState(false);

  const toggleSidebar = () => {
    setIsSidebarOpen(!isSidebarOpen);
  };

  const closeSidebar = () => {
    setIsSidebarOpen(false);
  };

  const toggleSidebarCollapse = () => {
    setIsSidebarCollapsed(!isSidebarCollapsed);
  };

  return (
    <div className="min-h-screen bg-slate-900 text-slate-100">
      {/* Mobile header with hamburger menu (< 768px) */}
      <header className="md:hidden fixed top-0 left-0 right-0 h-16 bg-slate-800 border-b border-slate-700 z-40 flex items-center px-4">
        <button
          onClick={toggleSidebar}
          aria-label="Toggle navigation menu"
          aria-expanded={isSidebarOpen}
          className="p-2 rounded-md hover:bg-slate-700 focus:outline focus:outline-2 focus:outline-primary-500"
        >
          <svg
            className="w-6 h-6"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            {isSidebarOpen ? (
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

      {/* Desktop Sidebar (>= 768px) - Fixed left */}
      <div className={`hidden md:block fixed top-0 left-0 bottom-0 border-r border-slate-700 z-30 transition-all duration-300 ${isSidebarCollapsed ? 'w-16' : 'w-64'}`}>
        {/* Brand header */}
        <div className="h-16 flex flex-col justify-center px-6 bg-slate-800 border-b border-slate-700 overflow-hidden">
          {!isSidebarCollapsed && (
            <>
              <span className="text-xl font-bold text-primary-500">KeyRx2</span>
              <span className="text-xs text-slate-500 whitespace-nowrap" title={`Built: ${BUILD_TIME}`}>
                v{VERSION} • {new Date(BUILD_TIME).toLocaleString()}
              </span>
            </>
          )}
        </div>
        <Sidebar
          className="h-[calc(100vh-4rem)]"
          isCollapsed={isSidebarCollapsed}
          onToggleCollapse={toggleSidebarCollapse}
        />
      </div>

      {/* Mobile Sidebar Overlay (< 768px) */}
      {isSidebarOpen && (
        <>
          {/* Backdrop */}
          <div
            className="md:hidden fixed inset-0 bg-black/50 z-40"
            onClick={closeSidebar}
            aria-hidden="true"
          />
          {/* Sidebar drawer */}
          <div className="md:hidden fixed top-16 left-0 bottom-0 w-64 z-50">
            <Sidebar
              isOpen={isSidebarOpen}
              onClose={closeSidebar}
              className="h-full"
            />
          </div>
        </>
      )}

      {/* Main Content Area */}
      <main
        className={`
          min-h-screen
          pt-16 md:pt-0
          pb-16 md:pb-0
          transition-all duration-300
          ${isSidebarCollapsed ? 'md:ml-16' : 'md:ml-64'}
        `}
      >
        {children}
      </main>

      {/* Bottom Navigation (Mobile only < 768px) */}
      <BottomNav />
    </div>
  );
};
