import React from 'react';
import { cn } from '@/lib/utils';

interface LoadingSkeletonProps {
  variant?: 'text' | 'circular' | 'rectangular' | 'card';
  width?: string | number;
  height?: string | number;
  className?: string;
  count?: number;
}

/**
 * LoadingSkeleton component
 *
 * Displays animated skeleton placeholders during data loading.
 * Matches layout structure to prevent layout shifts.
 */
export const LoadingSkeleton: React.FC<LoadingSkeletonProps> = ({
  variant = 'rectangular',
  width,
  height,
  className = '',
  count = 1,
}) => {
  const baseClasses = 'animate-pulse bg-slate-700';

  const variantClasses = {
    text: 'rounded h-4',
    circular: 'rounded-full',
    rectangular: 'rounded-md',
    card: 'rounded-lg',
  };

  const variantDefaults = {
    text: { width: '100%', height: '1rem' },
    circular: { width: '2.5rem', height: '2.5rem' },
    rectangular: { width: '100%', height: '1.5rem' },
    card: { width: '100%', height: '12rem' },
  };

  const defaults = variantDefaults[variant];
  const computedWidth = width ?? defaults.width;
  const computedHeight = height ?? defaults.height;

  const skeletonElement = (
    <div
      className={cn(baseClasses, variantClasses[variant], className)}
      style={{
        width: typeof computedWidth === 'number' ? `${computedWidth}px` : computedWidth,
        height: typeof computedHeight === 'number' ? `${computedHeight}px` : computedHeight,
      }}
      aria-busy="true"
      aria-live="polite"
      aria-label="Loading content"
    />
  );

  if (count === 1) {
    return skeletonElement;
  }

  return (
    <>
      {Array.from({ length: count }).map((_, index) => (
        <React.Fragment key={index}>
          {skeletonElement}
        </React.Fragment>
      ))}
    </>
  );
};

// Preset skeleton components for common patterns
export const SkeletonCard: React.FC<{ className?: string }> = ({ className }) => (
  <div className={cn('p-6 bg-slate-800 rounded-lg', className)}>
    <LoadingSkeleton variant="text" width="60%" className="mb-4" />
    <LoadingSkeleton variant="text" width="100%" className="mb-2" />
    <LoadingSkeleton variant="text" width="100%" className="mb-2" />
    <LoadingSkeleton variant="text" width="80%" />
  </div>
);

export const SkeletonTable: React.FC<{ rows?: number; className?: string }> = ({
  rows = 5,
  className
}) => (
  <div className={cn('space-y-3', className)}>
    <LoadingSkeleton variant="rectangular" height="48px" />
    {Array.from({ length: rows }).map((_, index) => (
      <LoadingSkeleton key={index} variant="rectangular" height="40px" />
    ))}
  </div>
);

export const SkeletonProfile: React.FC<{ className?: string }> = ({ className }) => (
  <div className={cn('flex items-center gap-4', className)}>
    <LoadingSkeleton variant="circular" width="48px" height="48px" />
    <div className="flex-1">
      <LoadingSkeleton variant="text" width="40%" className="mb-2" />
      <LoadingSkeleton variant="text" width="60%" height="12px" />
    </div>
  </div>
);

export const SkeletonButton: React.FC<{ className?: string }> = ({ className }) => (
  <LoadingSkeleton
    variant="rectangular"
    width="120px"
    height="40px"
    className={className}
  />
);
