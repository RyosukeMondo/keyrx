import React from 'react';
import { NavLink, useLocation } from 'react-router-dom';
import {
  Home,
  Smartphone,
  User,
  Settings,
  BarChart3,
  Gamepad2,
  ChevronLeft,
  ChevronRight,
} from 'lucide-react';
import { VERSION } from '../version';

interface SidebarProps {
  isOpen?: boolean;
  onClose?: () => void;
  className?: string;
  isCollapsed?: boolean;
  onToggleCollapse?: () => void;
}

interface NavItem {
  to: string;
  icon: React.ComponentType<{ className?: string }>;
  label: string;
  ariaLabel: string;
}

const navItems: NavItem[] = [
  {
    to: '/home',
    icon: Home,
    label: 'Home',
    ariaLabel: 'Navigate to Home page',
  },
  {
    to: '/devices',
    icon: Smartphone,
    label: 'Devices',
    ariaLabel: 'Navigate to Devices page',
  },
  {
    to: '/profiles',
    icon: User,
    label: 'Profiles',
    ariaLabel: 'Navigate to Profiles page',
  },
  {
    to: '/config',
    icon: Settings,
    label: 'Config',
    ariaLabel: 'Navigate to Configuration page',
  },
  {
    to: '/metrics',
    icon: BarChart3,
    label: 'Metrics',
    ariaLabel: 'Navigate to Metrics page',
  },
  {
    to: '/simulator',
    icon: Gamepad2,
    label: 'Simulator',
    ariaLabel: 'Navigate to Simulator page',
  },
];

export const Sidebar: React.FC<SidebarProps> = ({
  isOpen = true,
  onClose,
  className = '',
  isCollapsed = false,
  onToggleCollapse,
}) => {
  const location = useLocation();

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Escape' && onClose) {
      onClose();
    }
  };

  // Custom active state check for Config and Profiles routes
  // Config should be active for both /config and /profiles/:name/config
  // Profiles should NOT be active when on /config page (fix dual-highlight bug)
  const isRouteActive = (path: string): boolean => {
    if (path === '/config') {
      return (
        location.pathname === '/config' ||
        (location.pathname.startsWith('/profiles/') &&
          location.pathname.endsWith('/config'))
      );
    }
    if (path === '/profiles') {
      // Profiles should NOT be active when on /config page
      return (
        location.pathname === '/profiles' ||
        (location.pathname.startsWith('/profiles/') &&
          !location.pathname.includes('/config'))
      );
    }
    return false;
  };

  return (
    <aside
      className={`
        bg-slate-800
        flex flex-col
        ${isOpen ? 'translate-x-0' : '-translate-x-full md:translate-x-0'}
        transition-transform duration-300 ease-in-out
        ${className}
      `}
      aria-label="Main navigation sidebar"
      onKeyDown={handleKeyDown}
    >
      <nav className="flex-1 px-3 py-4" aria-label="Primary navigation">
        <ul className="space-y-1">
          {navItems.map((item) => {
            const Icon = item.icon;
            // For Config and Profiles routes, use custom active state check
            const customIsActive =
              item.to === '/config' || item.to === '/profiles'
                ? isRouteActive(item.to)
                : undefined;

            return (
              <li key={item.to}>
                <NavLink
                  to={item.to}
                  onClick={onClose}
                  aria-label={item.ariaLabel}
                  className={({ isActive }) => {
                    // Override isActive for Config route
                    const actuallyActive =
                      customIsActive !== undefined ? customIsActive : isActive;
                    return `
                    flex items-center gap-3 rounded-md
                    text-sm font-medium
                    transition-all duration-150
                    focus:outline focus:outline-2 focus:outline-primary-500 focus:outline-offset-2
                    ${isCollapsed ? 'justify-center px-2 py-3' : 'px-4 py-3'}
                    ${
                      actuallyActive
                        ? 'bg-primary-600 text-white shadow-md'
                        : 'text-slate-300 hover:bg-slate-700 hover:text-white'
                    }
                  `;
                  }}
                  title={isCollapsed ? item.label : undefined}
                >
                  {({ isActive }) => {
                    // Override isActive for Config route
                    const actuallyActive =
                      customIsActive !== undefined ? customIsActive : isActive;
                    return (
                      <>
                        <Icon
                          className={`w-5 h-5 ${
                            actuallyActive ? 'text-white' : 'text-slate-400'
                          } ${isCollapsed ? 'mx-auto' : ''}`}
                          aria-hidden="true"
                        />
                        {!isCollapsed && (
                          <>
                            <span>{item.label}</span>
                            {actuallyActive && (
                              <span
                                className="ml-auto w-1 h-6 bg-white rounded-full"
                                aria-hidden="true"
                              />
                            )}
                          </>
                        )}
                      </>
                    );
                  }}
                </NavLink>
              </li>
            );
          })}
        </ul>
      </nav>

      {/* Footer with version and collapse button */}
      <div className="px-3 py-3 border-t border-slate-700">
        <div className="flex items-center justify-between gap-2">
          {/* Version text - always show, even when collapsed */}
          {!isCollapsed && (
            <p className="text-xs text-slate-500 flex-1">KeyRx v{VERSION}</p>
          )}

          {/* Toggle button */}
          {onToggleCollapse && (
            <button
              onClick={onToggleCollapse}
              className={`flex items-center justify-center px-2 py-1 rounded-md text-slate-400 hover:text-white hover:bg-slate-700 transition-colors ${
                isCollapsed ? 'w-full' : ''
              }`}
              aria-label={isCollapsed ? 'Expand sidebar' : 'Collapse sidebar'}
              title={isCollapsed ? 'Expand sidebar' : 'Collapse sidebar'}
            >
              {isCollapsed ? (
                <ChevronRight className="w-5 h-5" />
              ) : (
                <ChevronLeft className="w-5 h-5" />
              )}
            </button>
          )}
        </div>
      </div>
    </aside>
  );
};
