import React from 'react';

interface CardProps {
  variant?: 'default' | 'elevated';
  padding?: 'sm' | 'md' | 'lg';
  header?: React.ReactNode;
  footer?: React.ReactNode;
  children: React.ReactNode;
  className?: string;
  role?: string;
  'aria-label'?: string;
  'aria-labelledby'?: string;
}

export const Card = React.memo<CardProps>(
  ({
    variant = 'default',
    padding = 'md',
    header,
    footer,
    children,
    className = '',
    role = 'region',
    'aria-label': ariaLabel,
    'aria-labelledby': ariaLabelledBy,
  }) => {
    const baseClasses =
      'bg-slate-800 border border-slate-700 rounded-md overflow-hidden';

    const variantClasses = {
      default: 'shadow-md',
      elevated: 'shadow-xl',
    };

    const paddingClasses = {
      sm: 'p-sm',
      md: 'p-md',
      lg: 'p-lg',
    };

    return (
      <div
        className={`${baseClasses} ${variantClasses[variant]} ${className}`}
        role={role}
        aria-label={ariaLabel}
        aria-labelledby={ariaLabelledBy}
      >
        {header && (
          <div className="border-b border-slate-700 px-md py-sm bg-slate-700/50">
            {header}
          </div>
        )}
        <div className={paddingClasses[padding]}>{children}</div>
        {footer && (
          <div className="border-t border-slate-700 px-md py-sm bg-slate-700/50">
            {footer}
          </div>
        )}
      </div>
    );
  }
);

Card.displayName = 'Card';
