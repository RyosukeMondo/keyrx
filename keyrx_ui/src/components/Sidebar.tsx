import React from 'react';
import { NavLink, useLocation } from 'react-router-dom';
import {
  Settings,
  Smartphone,
  BarChart3,
  ChevronLeft,
  ChevronRight,
} from 'lucide-react';
import { VERSION } from '../version';
import { t, type MessageKey } from '@/i18n';

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
  label: MessageKey;
  ariaLabel: MessageKey;
  description: MessageKey;
}

const navItems: NavItem[] = [
  {
    to: '/',
    icon: Settings,
    label: 'nav.config',
    ariaLabel: 'nav.config.aria',
    description: 'nav.config.desc',
  },
  {
    to: '/devices',
    icon: Smartphone,
    label: 'nav.devices',
    ariaLabel: 'nav.devices.aria',
    description: 'nav.devices.desc',
  },
  {
    to: '/monitor',
    icon: BarChart3,
    label: 'nav.monitor',
    ariaLabel: 'nav.monitor.aria',
    description: 'nav.monitor.desc',
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

  // Config is active for / and /profiles/:name/config
  const isRouteActive = (path: string): boolean => {
    if (path === '/') {
      return (
        location.pathname === '/' ||
        (location.pathname.startsWith('/profiles/') &&
          location.pathname.endsWith('/config'))
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
      aria-label={t('nav.sidebar')}
      onKeyDown={handleKeyDown}
    >
      <nav className="flex-1 px-3 py-4" aria-label={t('nav.primary')}>
        <ul className="space-y-1">
          {navItems.map((item) => {
            const Icon = item.icon;
            // For Config route, use custom active state check
            const customIsActive =
              item.to === '/' ? isRouteActive(item.to) : undefined;

            return (
              <li key={item.to}>
                <NavLink
                  to={item.to}
                  onClick={onClose}
                  aria-label={t(item.ariaLabel)}
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
                  title={isCollapsed ? t(item.label) : undefined}
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
                            <span className="min-w-0">
                              <span className="block">{t(item.label)}</span>
                              <span
                                className={`block text-[11px] font-normal ${actuallyActive ? 'text-primary-50' : 'text-slate-400'}`}
                              >
                                {t(item.description)}
                              </span>
                            </span>
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
      <div className="px-3 py-3 border-t border-slate-700/80">
        <div className="flex items-center justify-between gap-2">
          {/* Version text - always show, even when collapsed */}
          {!isCollapsed && (
            <p className="text-[11px] text-slate-400 flex-1">
              KeyRx v{VERSION} · {t('nav.local')}
            </p>
          )}

          {/* Toggle button */}
          {onToggleCollapse && (
            <button
              onClick={onToggleCollapse}
              className={`flex items-center justify-center px-2 py-1 rounded-md text-slate-400 hover:text-white hover:bg-slate-700 transition-colors ${
                isCollapsed ? 'w-full' : ''
              }`}
              aria-label={isCollapsed ? t('nav.expand') : t('nav.collapse')}
              title={isCollapsed ? t('nav.expand') : t('nav.collapse')}
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
