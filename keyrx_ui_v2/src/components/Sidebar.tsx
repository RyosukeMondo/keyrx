import React from 'react';
import { NavLink } from 'react-router-dom';
import {
  Home,
  Smartphone,
  User,
  Settings,
  BarChart3,
  Gamepad2,
} from 'lucide-react';

interface SidebarProps {
  isOpen?: boolean;
  onClose?: () => void;
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
}) => {
  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Escape' && onClose) {
      onClose();
    }
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
            return (
              <li key={item.to}>
                <NavLink
                  to={item.to}
                  onClick={onClose}
                  aria-label={item.ariaLabel}
                  className={({ isActive }) =>
                    `
                    flex items-center gap-3 px-4 py-3 rounded-md
                    text-sm font-medium
                    transition-all duration-150
                    focus:outline focus:outline-2 focus:outline-primary-500 focus:outline-offset-2
                    ${
                      isActive
                        ? 'bg-primary-600 text-white shadow-md'
                        : 'text-slate-300 hover:bg-slate-700 hover:text-white'
                    }
                  `
                  }
                >
                  {({ isActive }) => (
                    <>
                      <Icon
                        className={`w-5 h-5 ${isActive ? 'text-white' : 'text-slate-400'}`}
                        aria-hidden="true"
                      />
                      <span>{item.label}</span>
                      {isActive && (
                        <span
                          className="ml-auto w-1 h-6 bg-white rounded-full"
                          aria-hidden="true"
                        />
                      )}
                    </>
                  )}
                </NavLink>
              </li>
            );
          })}
        </ul>
      </nav>

      {/* Footer section for version or other info */}
      <div className="px-4 py-3 border-t border-slate-700">
        <p className="text-xs text-slate-500 text-center">KeyRx v2.0.0</p>
      </div>
    </aside>
  );
};
