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
  'data-testid'?: string;
  'data-profile'?: string;
}

export const Card = React.memo<CardProps>(
  ({
    variant = 'default',
    padding = 'md',
    header,
    footer,
    children,
    className = '',
    role,
    'aria-label': ariaLabel,
    'aria-labelledby': ariaLabelledBy,
    'data-testid': dataTestId,
    'data-profile': dataProfile,
  }) => {
    const baseClasses =
      'bg-slate-800 border border-slate-700 rounded-md overflow-hidden';

    // Default to the "region" landmark role only when there's an accessible
    // name to go with it. Every Card used to get role="region" unconditionally,
    // so a page with two or more unlabelled Cards produced multiple anonymous
    // "region" landmarks -- axe's landmark-unique rule (a region landmark
    // needs a name once there's more than one) and, worse, actual screen
    // reader users landing on a wall of unnamed "region" stops when
    // navigating by landmark. A Card that's just a visual grouping
    // (no aria-label/aria-labelledby) is a plain <div>; callers that want a
    // real landmark still get one by passing `role` and a label explicitly.
    const resolvedRole = role ?? (ariaLabel || ariaLabelledBy ? 'region' : undefined);

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
        role={resolvedRole}
        aria-label={ariaLabel}
        aria-labelledby={ariaLabelledBy}
        data-testid={dataTestId}
        data-profile={dataProfile}
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
