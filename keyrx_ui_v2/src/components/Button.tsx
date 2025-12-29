import React from 'react';
import { cn } from '@/utils/cn';

export interface ButtonProps {
  variant?: 'primary' | 'secondary' | 'danger' | 'ghost';
  size?: 'sm' | 'md' | 'lg';
  disabled?: boolean;
  loading?: boolean;
  onClick: (event: React.MouseEvent<HTMLButtonElement>) => void;
  'aria-label': string;
  children: React.ReactNode;
  type?: 'button' | 'submit' | 'reset';
  className?: string;
}

const LoadingSpinner: React.FC = () => (
  <svg
    className="animate-spin h-4 w-4 mr-2"
    xmlns="http://www.w3.org/2000/svg"
    fill="none"
    viewBox="0 0 24 24"
  >
    <circle
      className="opacity-25"
      cx="12"
      cy="12"
      r="10"
      stroke="currentColor"
      strokeWidth="4"
    />
    <path
      className="opacity-75"
      fill="currentColor"
      d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
    />
  </svg>
);

export const Button = React.memo<ButtonProps>(
  ({
    variant = 'primary',
    size = 'md',
    disabled = false,
    loading = false,
    onClick,
    'aria-label': ariaLabel,
    children,
    type = 'button',
    className = '',
  }) => {
    const handleClick = (e: React.MouseEvent<HTMLButtonElement>) => {
      if (disabled || loading) return;

      const button = e.currentTarget;
      const rect = button.getBoundingClientRect();
      const ripple = document.createElement('span');
      const diameter = Math.max(rect.width, rect.height);
      const radius = diameter / 2;

      ripple.style.width = ripple.style.height = `${diameter}px`;
      ripple.style.left = `${e.clientX - rect.left - radius}px`;
      ripple.style.top = `${e.clientY - rect.top - radius}px`;
      ripple.className = 'ripple';

      button.appendChild(ripple);

      setTimeout(() => ripple.remove(), 600);

      onClick(e);
    };

    const baseClasses =
      'relative overflow-hidden rounded-md font-medium transition-all duration-150 focus:outline focus:outline-2 focus:outline-primary-500 focus:outline-offset-2 flex items-center justify-center';

    const variantClasses = {
      primary:
        'bg-primary-500 text-white hover:bg-primary-600 active:bg-primary-700 hover:scale-[1.02] active:scale-[0.98]',
      secondary:
        'bg-transparent border-2 border-primary-500 text-primary-500 hover:bg-primary-500 hover:text-white',
      danger:
        'bg-red-500 text-white hover:bg-red-600 active:bg-red-700 hover:scale-[1.02] active:scale-[0.98]',
      ghost:
        'bg-transparent text-primary-500 hover:bg-primary-500/10',
    };

    const sizeClasses = {
      sm: 'py-2 px-3 text-sm',
      md: 'py-3 px-4 text-base',
      lg: 'py-4 px-6 text-lg',
    };

    const disabledClasses = disabled || loading
      ? 'opacity-50 cursor-not-allowed'
      : '';

    return (
      <button
        type={type}
        disabled={disabled || loading}
        onClick={handleClick}
        aria-label={ariaLabel}
        aria-disabled={disabled}
        aria-busy={loading}
        className={cn(
          baseClasses,
          variantClasses[variant],
          sizeClasses[size],
          disabledClasses,
          className
        )}
      >
        {loading && <LoadingSpinner />}
        {children}
      </button>
    );
  }
);

Button.displayName = 'Button';
