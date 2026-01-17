import React, { useState, useRef, useEffect, useId, ReactNode } from 'react';
import {
  useFloating,
  offset,
  flip,
  shift,
  autoUpdate,
  Placement,
} from '@floating-ui/react';

interface TooltipProps {
  content: string | ReactNode;
  children: ReactNode;
  position?: 'top' | 'bottom' | 'left' | 'right' | 'auto';
  delay?: number;
  disabled?: boolean;
  className?: string;
}

function cn(...classes: (string | boolean | undefined)[]) {
  return classes.filter(Boolean).join(' ');
}

export const Tooltip: React.FC<TooltipProps> = ({
  content,
  children,
  position = 'auto',
  delay = 500,
  disabled = false,
  className = '',
}) => {
  const tooltipId = useId();
  const [isVisible, setIsVisible] = useState(false);
  const timeoutRef = useRef<NodeJS.Timeout>();

  const placement: Placement = position === 'auto' ? 'top' : position;

  const { x, y, refs, strategy } = useFloating({
    placement,
    middleware: [offset(8), flip(), shift({ padding: 8 })],
    whileElementsMounted: autoUpdate,
  });

  const handleMouseEnter = () => {
    if (disabled) return;
    timeoutRef.current = setTimeout(() => {
      setIsVisible(true);
    }, delay);
  };

  const handleMouseLeave = () => {
    clearTimeout(timeoutRef.current);
    setIsVisible(false);
  };

  const handleFocus = () => {
    if (disabled) return;
    setIsVisible(true);
  };

  const handleBlur = () => {
    setIsVisible(false);
  };

  useEffect(() => {
    return () => clearTimeout(timeoutRef.current);
  }, []);

  return (
    <>
      <div
        ref={refs.setReference}
        onMouseEnter={handleMouseEnter}
        onMouseLeave={handleMouseLeave}
        onFocus={handleFocus}
        onBlur={handleBlur}
        aria-describedby={isVisible ? tooltipId : undefined}
      >
        {children}
      </div>

      {isVisible && !disabled && (
        <div
          ref={refs.setFloating} // eslint-disable-line react-hooks/refs
          id={tooltipId}
          role="tooltip"
          className={cn(
            'absolute bg-slate-900 text-white text-base px-4 py-3 rounded-lg shadow-2xl border-2 border-slate-700 z-tooltip',
            'transition-opacity duration-150 font-medium',
            'max-w-xs break-words',
            className
          )}
          style={{
            position: strategy,
            top: y ?? 0,
            left: x ?? 0,
          }}
        >
          {content}
        </div>
      )}
    </>
  );
};
