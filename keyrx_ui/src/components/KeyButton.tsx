import React, { useMemo } from 'react';
import { Tooltip } from './Tooltip';
import { cn } from '../utils/cn';

export interface KeyMapping {
  type: 'simple' | 'tap_hold' | 'macro' | 'layer_switch';
  tapAction?: string;
  holdAction?: string;
  threshold?: number;
  macroSteps?: Array<{ type: string; key?: string; delay?: number }>;
  targetLayer?: string;
}

interface KeyButtonProps {
  keyCode: string;
  label: string;
  mapping?: KeyMapping;
  onClick: () => void;
  isPressed?: boolean;
  className?: string;
}

export const KeyButton = React.memo<KeyButtonProps>(
  ({ keyCode, label, mapping, onClick, isPressed = false, className = '' }) => {
    const hasMapping = mapping && mapping.type !== 'simple';

    const tooltipContent = useMemo(() => {
      if (!mapping) return `${keyCode} (Default)`;

      switch (mapping.type) {
        case 'simple':
          return `${keyCode} → ${mapping.tapAction}`;
        case 'tap_hold':
          return `${keyCode} → Tap: ${mapping.tapAction}, Hold: ${mapping.holdAction} (${mapping.threshold}ms)`;
        case 'macro':
          return `${keyCode} → Macro (${mapping.macroSteps?.length || 0} steps)`;
        case 'layer_switch':
          return `${keyCode} → Layer: ${mapping.targetLayer}`;
        default:
          return `${keyCode} (Default)`;
      }
    }, [keyCode, mapping]);

    return (
      <Tooltip content={tooltipContent}>
        <button
          onClick={onClick}
          aria-label={`Key ${keyCode}. Current mapping: ${tooltipContent}. Click to configure.`}
          className={cn(
            'relative flex items-center justify-center',
            'rounded border border-slate-600 text-slate-100 text-xs font-mono',
            'transition-all duration-150',
            'hover:bg-slate-600 hover:scale-105',
            'focus:outline focus:outline-2 focus:outline-primary-500',
            hasMapping ? 'bg-blue-700' : 'bg-slate-700',
            isPressed && 'bg-green-500',
            className
          )}
          style={{
            aspectRatio: '1',
          }}
        >
          {label}
        </button>
      </Tooltip>
    );
  }
);

KeyButton.displayName = 'KeyButton';
