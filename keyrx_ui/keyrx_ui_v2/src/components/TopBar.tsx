import React from 'react';
import { Settings, HelpCircle } from 'lucide-react';
import { Button } from './Button';

export interface TopBarProps {
  onSettingsClick?: () => void;
  onHelpClick?: () => void;
  className?: string;
}

export const TopBar: React.FC<TopBarProps> = ({
  onSettingsClick,
  onHelpClick,
  className = '',
}) => {
  return (
    <header
      className={`flex items-center justify-between px-6 py-4 bg-slate-800 border-b border-slate-700 ${className}`}
      role="banner"
    >
      {/* Logo and Title */}
      <div className="flex items-center gap-3">
        {/* Logo */}
        <div
          className="flex items-center justify-center w-10 h-10 bg-primary-500 rounded-lg"
          aria-label="KeyRx2 Logo"
        >
          <svg
            width="24"
            height="24"
            viewBox="0 0 24 24"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
            aria-hidden="true"
          >
            <path
              d="M7 10H5V8H7V10ZM9 10H11V8H9V10ZM13 10H15V8H13V10ZM17 10H19V8H17V10Z"
              fill="white"
            />
            <path
              d="M7 14H5V12H7V14ZM9 14H11V12H9V14ZM13 14H15V12H13V14ZM17 14H19V12H17V14Z"
              fill="white"
            />
            <path d="M9 18H15V16H9V18Z" fill="white" />
          </svg>
        </div>

        {/* Title - Hidden on mobile */}
        <h1 className="hidden md:block text-xl font-semibold text-slate-100">
          KeyRx2 Configuration
        </h1>
      </div>

      {/* Action Buttons */}
      <div className="flex items-center gap-2">
        <Button
          variant="ghost"
          size="sm"
          onClick={onHelpClick || (() => {})}
          aria-label="Help and documentation"
          className="text-slate-300 hover:text-slate-100"
        >
          <HelpCircle className="w-5 h-5" />
        </Button>

        <Button
          variant="ghost"
          size="sm"
          onClick={onSettingsClick || (() => {})}
          aria-label="Open settings"
          className="text-slate-300 hover:text-slate-100"
        >
          <Settings className="w-5 h-5" />
        </Button>
      </div>
    </header>
  );
};
