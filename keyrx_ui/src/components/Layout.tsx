import React, { useState } from 'react';
import { BottomNav } from './BottomNav';
import { GlobalDebugPanel } from './GlobalDebugPanel';
import { Sidebar } from './Sidebar';
import { Keyboard } from 'lucide-react';
import { SkipToContent } from './SkipToContent';

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
 * with the navigation elements, and reserves bottom padding on desktop
 * so page content doesn't render under the fixed-position GlobalDebugPanel
 * toggle (bottom-right corner on every page).
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
      <SkipToContent />
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
        <div className="flex flex-1 items-center justify-center gap-2">
          <span className="flex h-8 w-8 items-center justify-center rounded-lg bg-primary-500/15 text-primary-300 ring-1 ring-primary-400/20">
            <Keyboard className="h-4 w-4" aria-hidden="true" />
          </span>
          <span className="text-lg font-semibold tracking-tight">KeyRx2</span>
        </div>
      </header>

      {/* Desktop Sidebar (>= 768px) - Fixed left */}
      <div
        className={`hidden md:block fixed top-0 left-0 bottom-0 border-r border-slate-700 z-30 transition-all duration-300 ${
          isSidebarCollapsed ? 'w-16' : 'w-64'
        }`}
      >
        {/* Brand header */}
        <div className="h-20 flex flex-col justify-center px-4 bg-slate-800 border-b border-slate-700 overflow-hidden">
          {!isSidebarCollapsed && (
            <div className="flex items-center gap-3">
              <span className="flex h-10 w-10 shrink-0 items-center justify-center rounded-xl bg-primary-500/15 text-primary-300 ring-1 ring-primary-400/20">
                <Keyboard className="h-5 w-5" aria-hidden="true" />
              </span>
              <div className="min-w-0">
                <span className="block text-lg font-bold leading-tight tracking-tight text-slate-50">
                  KeyRx2
                </span>
                <span className="block text-[11px] text-slate-400">
                  Keyboard studio
                </span>
              </div>
            </div>
          )}
          {isSidebarCollapsed && (
            <Keyboard
              className="mx-auto h-5 w-5 text-primary-300"
              aria-label="KeyRx2"
            />
          )}
        </div>
        <Sidebar
          className="h-[calc(100vh-5rem)]"
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
        id="main-content"
        tabIndex={-1}
        className={`
          min-h-screen
          pt-16 md:pt-0
          pb-16 md:pb-12
          transition-all duration-300
          ${isSidebarCollapsed ? 'md:ml-16' : 'md:ml-64'}
          focus:outline-none
        `}
      >
        {children}
      </main>

      {/* Bottom Navigation (Mobile only < 768px) */}
      <BottomNav />

      {/* Global debug console (all pages) */}
      <GlobalDebugPanel />
    </div>
  );
};
