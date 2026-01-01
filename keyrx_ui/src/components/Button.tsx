import React from 'react';
import { motion } from 'framer-motion';
import { cn } from '@/utils/cn';
import { hoverScale, tapScale, prefersReducedMotion } from '@/utils/animations';
import { LoadingSpinner } from './LoadingSpinner';

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
      'relative overflow-hidden rounded-md font-medium transition-colors duration-150 focus:outline focus:outline-2 focus:outline-primary-500 focus:outline-offset-2 flex items-center justify-center';

    const variantClasses = {
      primary:
        'bg-primary-500 text-white hover:bg-primary-600 active:bg-primary-700',
      secondary:
        'bg-transparent border-2 border-primary-500 text-primary-500 hover:bg-primary-500 hover:text-white',
      danger:
        'bg-red-500 text-white hover:bg-red-600 active:bg-red-700',
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

    // Disable animations if user prefers reduced motion
    const shouldAnimate = !prefersReducedMotion() && !disabled && !loading;

    return (
      <motion.button
        type={type}
        disabled={disabled || loading}
        onClick={handleClick}
        aria-label={ariaLabel}
        aria-disabled={disabled}
        aria-busy={loading}
        whileHover={shouldAnimate ? hoverScale : undefined}
        whileTap={shouldAnimate ? tapScale : undefined}
        className={cn(
          baseClasses,
          variantClasses[variant],
          sizeClasses[size],
          disabledClasses,
          className
        )}
      >
        {loading && <LoadingSpinner size="sm" className="mr-2" />}
        {children}
      </motion.button>
    );
  }
);

Button.displayName = 'Button';
