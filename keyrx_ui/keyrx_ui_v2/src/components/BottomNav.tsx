import React from 'react';
import { NavLink } from 'react-router-dom';
import { Home, Smartphone, User, Settings, BarChart3 } from 'lucide-react';

interface BottomNavProps {
  className?: string;
}

interface NavItem {
  to: string;
  icon: React.ComponentType<{ className?: string }>;
  label: string;
  ariaLabel: string;
}

const navItems: NavItem[] = [
  {
    to: '/',
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
];

export const BottomNav: React.FC<BottomNavProps> = ({ className = '' }) => {
  return (
    <nav
      className={`
        fixed bottom-0 left-0 right-0
        bg-slate-800 border-t border-slate-700
        md:hidden
        ${className}
      `}
      aria-label="Mobile bottom navigation"
      style={{ zIndex: 'var(--z-fixed)' }}
    >
      <ul className="flex justify-around items-center h-16">
        {navItems.map((item) => {
          const Icon = item.icon;
          return (
            <li key={item.to} className="flex-1">
              <NavLink
                to={item.to}
                aria-label={item.ariaLabel}
                className={({ isActive }) =>
                  `
                  flex flex-col items-center justify-center
                  h-16 px-2
                  text-xs font-medium
                  transition-colors duration-150
                  focus:outline focus:outline-2 focus:outline-primary-500
                  ${
                    isActive
                      ? 'text-primary-500'
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
                      {item.label}
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
