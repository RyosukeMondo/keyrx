import React, { useMemo, useState } from 'react';
import { Tooltip } from './Tooltip';
import { cn } from '../utils/cn';
import type { KeyMapping } from '@/types';

interface KeyButtonProps {
  keyCode: string;
  label: string;
  mapping?: KeyMapping;
  onClick: () => void;
  isPressed?: boolean;
  className?: string;
}

// Mapping type icons (using simple SVG shapes for performance)
const MappingTypeIcon: React.FC<{ type: string }> = ({ type }) => {
  const iconColor =
    {
      simple: 'text-green-400',
      tap_hold: 'text-red-400',
      macro: 'text-purple-400',
      layer_switch: 'text-yellow-400',
    }[type] || 'text-slate-400';

  const icon =
    {
      simple: '→',
      tap_hold: '↕',
      macro: '⚡',
      layer_switch: '⇄',
    }[type] || '';

  return (
    <span
      className={cn(
        'absolute top-0.5 right-0.5 text-[10px] font-bold',
        iconColor
      )}
      aria-hidden="true"
    >
      {icon}
    </span>
  );
};

export const KeyButton = React.memo<KeyButtonProps>(
  ({ keyCode, label, mapping, onClick, isPressed = false, className = '' }) => {
    const hasMapping = !!mapping;
    const [isClicked, setIsClicked] = useState(false);

    // Handle click with animation
    const handleClick = () => {
      setIsClicked(true);
      setTimeout(() => setIsClicked(false), 300);
      onClick();
    };

    const tooltipContent = useMemo(() => {
      if (!mapping) return `${keyCode} (Default)`;

      switch (mapping.type) {
        case 'simple':
          return `${keyCode} → ${mapping.tapAction}`;
        case 'tap_hold':
          return `${keyCode} → Tap: ${mapping.tapAction}, Hold: ${mapping.holdAction} (${mapping.threshold}ms)`;
        case 'macro':
          return `${keyCode} → Macro (${
            mapping.macroSteps?.length || 0
          } steps)`;
        case 'layer_switch':
          return `${keyCode} → Layer: ${mapping.targetLayer}`;
        default:
          return `${keyCode} (Default)`;
      }
    }, [keyCode, mapping]);

    /**
     * Format a key label for display on keycap
     * Handles: VK_ prefixes, with_* functions, long names
     */
    const formatKeyLabel = (key: string): string => {
      if (!key) return '';

      // Handle with_* helper functions: with_shift(VK_A) -> ⇧A
      const withMatch = key.match(/^with_(\w+)\(["']?(\w+)["']?\)$/);
      if (withMatch) {
        const [, modifier, innerKey] = withMatch;
        const modSymbols: Record<string, string> = {
          shift: '⇧',
          ctrl: '⌃',
          alt: '⌥',
          meta: '⌘',
          gui: '⌘',
        };
        const modSymbol =
          modSymbols[modifier.toLowerCase()] ||
          modifier.charAt(0).toUpperCase();
        const cleanKey = innerKey.replace(/^VK_/, '');
        return `${modSymbol}${cleanKey}`;
      }

      // Remove VK_ prefix for cleaner display
      const clean = key.replace(/^VK_/, '');

      // Shorten common keys
      const shortNames: Record<string, string> = {
        BACKSPACE: 'BS',
        CAPSLOCK: 'Caps',
        ESCAPE: 'Esc',
        DELETE: 'Del',
        INSERT: 'Ins',
        PAGEUP: 'PgUp',
        PAGEDOWN: 'PgDn',
        LEFTSHIFT: 'LShft',
        RIGHTSHIFT: 'RShft',
        LEFTCONTROL: 'LCtrl',
        RIGHTCONTROL: 'RCtrl',
        LEFTALT: 'LAlt',
        RIGHTALT: 'RAlt',
        LEFTMETA: 'LMeta',
        RIGHTMETA: 'RMeta',
        NUMLOCK: 'Num',
        SCROLLLOCK: 'Scrl',
        PRINTSCREEN: 'PrtSc',
      };

      const upper = clean.toUpperCase();
      if (shortNames[upper]) {
        return shortNames[upper];
      }

      // Truncate if still too long (max 5 chars for display)
      if (clean.length > 5) {
        return clean.slice(0, 4) + '…';
      }

      return clean;
    };

    // Get remap display text
    const remapText = useMemo(() => {
      if (!mapping) return '';

      switch (mapping.type) {
        case 'simple':
          return formatKeyLabel(mapping.tapAction || '');
        case 'tap_hold': {
          const tap = formatKeyLabel(mapping.tapAction || '');
          const hold = formatKeyLabel(mapping.holdAction || '');
          return `${tap}/${hold}`;
        }
        case 'macro':
          return '⚡';
        case 'layer_switch':
          return mapping.targetLayer?.replace(/^MD_/, 'L') || '';
        default:
          return '';
      }
    }, [mapping]);

    // Determine border and background color based on mapping type
    const getKeyStyle = () => {
      if (!mapping)
        return {
          border: 'border-dashed border-slate-600',
          bg: 'bg-slate-700/50',
        };

      switch (mapping.type) {
        case 'simple':
          return {
            border: 'border-solid border-green-500',
            bg: 'bg-slate-700',
          };
        case 'tap_hold':
          return {
            border: 'border-solid border-red-500',
            bg: 'bg-red-900/15',
          };
        case 'macro':
          return {
            border: 'border-solid border-purple-500',
            bg: 'bg-purple-900/15',
          };
        case 'layer_switch':
          return {
            border: 'border-solid border-yellow-500',
            bg: 'bg-yellow-900/15',
          };
        default:
          return {
            border: 'border-solid border-slate-600',
            bg: 'bg-slate-700',
          };
      }
    };

    const style = getKeyStyle();

    return (
      <Tooltip content={tooltipContent}>
        <button
          onClick={handleClick}
          aria-label={`Key ${keyCode}. Current mapping: ${tooltipContent}. Click to configure.`}
          className={cn(
            'relative flex flex-col items-center justify-center overflow-hidden',
            'rounded border-2 transition-all duration-150',
            'hover:brightness-110 hover:-translate-y-0.5 hover:shadow-lg',
            'focus:outline focus:outline-2 focus:outline-primary-500',
            'min-h-[50px] w-full',
            style.border,
            style.bg,
            isPressed && 'bg-green-500 border-green-400',
            isClicked && 'scale-95 brightness-125',
            className
          )}
          style={{
            aspectRatio: '1',
          }}
        >
          {/* Mapping type icon overlay */}
          {hasMapping && mapping && <MappingTypeIcon type={mapping.type} />}

          {/* Original key label (small, gray) */}
          <span className="text-[10px] text-slate-400 font-mono">{label}</span>

          {/* Remap label (bold, yellow) - truncated to fit keycap */}
          {hasMapping && (
            <span className="text-[11px] text-yellow-300 font-bold font-mono mt-0.5 max-w-full overflow-hidden text-ellipsis whitespace-nowrap px-0.5">
              {remapText}
            </span>
          )}

          {/* Click ripple effect */}
          {isClicked && (
            <span
              className="absolute inset-0 rounded animate-ping opacity-75 bg-primary-500/30"
              aria-hidden="true"
            />
          )}
        </button>
      </Tooltip>
    );
  }
);

KeyButton.displayName = 'KeyButton';
