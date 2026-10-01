import React from 'react';
import { NavLink } from 'react-router-dom';
import { Settings, Smartphone, BarChart3 } from 'lucide-react';
import { t, type MessageKey } from '@/i18n';

interface BottomNavProps {
  className?: string;
}

interface NavItem {
  to: string;
  icon: React.ComponentType<{ className?: string }>;
  label: MessageKey;
  ariaLabel: MessageKey;
}

const navItems: NavItem[] = [
  {
    to: '/',
    icon: Settings,
    label: 'nav.config',
    ariaLabel: 'nav.config.aria',
  },
  {
    to: '/devices',
    icon: Smartphone,
    label: 'nav.devices',
    ariaLabel: 'nav.devices.aria',
  },
  {
    to: '/monitor',
    icon: BarChart3,
    label: 'nav.monitor',
    ariaLabel: 'nav.monitor.aria',
  },
];

export const BottomNav: React.FC<BottomNavProps> = ({ className = '' }) => {
  return (
    <nav
      className={`
        chrome-bottom fixed bottom-0 left-0 right-0
        bg-slate-800 border-t border-slate-700
        md:hidden
        ${className}
      `}
      aria-label={t('nav.mobile')}
      style={{ zIndex: 'var(--z-fixed)' }}
    >
      <ul className="flex justify-around items-center h-16">
        {navItems.map((item) => {
          const Icon = item.icon;
          return (
            <li key={item.to} className="flex-1">
              <NavLink
                to={item.to}
                aria-label={t(item.ariaLabel)}
                className={({ isActive }) =>
                  `
                  flex flex-col items-center justify-center
                  h-16 px-2
                  text-xs font-medium
                  transition-colors duration-150
                  focus:outline focus:outline-2 focus:outline-primary-500
                  ${
                    isActive
                      ? 'text-primary-400'
                      : 'text-slate-400 hover:text-slate-300'
                  }
                `
                }
              >
                {({ isActive }) => (
                  <>
                    <Icon
                      className={`w-6 h-6 mb-1 ${
                        isActive ? 'fill-current' : ''
                      }`}
                      aria-hidden="true"
                    />
                    <span className={isActive ? 'font-semibold' : ''}>
                      {t(item.label)}
                    </span>
                  </>
                )}
              </NavLink>
            </li>
          );
        })}
      </ul>
    </nav>
  );
};
