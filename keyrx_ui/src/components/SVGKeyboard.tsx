/**
 * SVG-based Keyboard Visualizer
 * Renders keyboard layouts using SVG for precise key shapes including ISO Enter
 *
 * MIT License compatible - no GPL dependencies
 */

import React, { useMemo, useState, useCallback } from 'react';
import type { KeyMapping } from '@/types';

// Constants for SVG rendering
const UNIT_SIZE = 54; // pixels per key unit (1u)
const KEY_GAP = 2; // gap between keys
const _KEY_PADDING = 2; // padding inside key
const KEY_RADIUS = 6; // border radius
const KEY_INSET = 3; // 3D effect inset

export interface SVGKey {
  code: string;
  label: string;
  x: number;
  y: number;
  w: number;
  h: number;
  /** Special shape: 'iso-enter' | 'standard' */
  shape?: 'iso-enter' | 'standard';
}

interface SVGKeyboardProps {
  keys: SVGKey[];
  keyMappings: Map<string, KeyMapping>;
  onKeyClick: (keyCode: string) => void;
  simulatorMode?: boolean;
  pressedKeys?: Set<string>;
  className?: string;
  layoutName?: string;
}

interface KeySVGProps {
  keyData: SVGKey;
  mapping?: KeyMapping;
  isPressed: boolean;
  onClick: () => void;
  simulatorMode?: boolean;
}

/**
 * Generate SVG path for ISO Enter key (L-shaped)
 * The ISO Enter spans 2 rows with different widths
 */
function generateISOEnterPath(
  x: number,
  y: number,
  w: number,
  h: number
): string {
  const px = x * UNIT_SIZE;
  const py = y * UNIT_SIZE;
  const topWidth = 1.5 * UNIT_SIZE - KEY_GAP; // Top part is 1.5u
  const bottomWidth = w * UNIT_SIZE - KEY_GAP; // Bottom uses actual width
  const halfHeight = (h * UNIT_SIZE) / 2 - KEY_GAP / 2;
  const r = KEY_RADIUS;

  // L-shape path (clockwise from top-left)
  // Note: The top part extends further left than the bottom
  const leftOffset = topWidth - bottomWidth;

  return `
    M ${px + leftOffset + r} ${py}
    L ${px + topWidth - r} ${py}
    Q ${px + topWidth} ${py} ${px + topWidth} ${py + r}
    L ${px + topWidth} ${py + h * UNIT_SIZE - KEY_GAP - r}
    Q ${px + topWidth} ${py + h * UNIT_SIZE - KEY_GAP} ${px + topWidth - r} ${
      py + h * UNIT_SIZE - KEY_GAP
    }
    L ${px + leftOffset + r} ${py + h * UNIT_SIZE - KEY_GAP}
    Q ${px + leftOffset} ${py + h * UNIT_SIZE - KEY_GAP} ${px + leftOffset} ${
      py + h * UNIT_SIZE - KEY_GAP - r
    }
    L ${px + leftOffset} ${py + halfHeight + r}
    Q ${px + leftOffset} ${py + halfHeight} ${px + leftOffset - r} ${
      py + halfHeight
    }
    L ${px + r} ${py + halfHeight}
    Q ${px} ${py + halfHeight} ${px} ${py + halfHeight - r}
    L ${px} ${py + r}
    Q ${px} ${py} ${px + r} ${py}
    Z
  `.trim();
}

/**
 * Generate SVG path for standard rectangular key
 */
function generateRectPath(x: number, y: number, w: number, h: number): string {
  const px = x * UNIT_SIZE;
  const py = y * UNIT_SIZE;
  const width = w * UNIT_SIZE - KEY_GAP;
  const height = h * UNIT_SIZE - KEY_GAP;
  const r = KEY_RADIUS;

  return `
    M ${px + r} ${py}
    L ${px + width - r} ${py}
    Q ${px + width} ${py} ${px + width} ${py + r}
    L ${px + width} ${py + height - r}
    Q ${px + width} ${py + height} ${px + width - r} ${py + height}
    L ${px + r} ${py + height}
    Q ${px} ${py + height} ${px} ${py + height - r}
    L ${px} ${py + r}
    Q ${px} ${py} ${px + r} ${py}
    Z
  `.trim();
}

/**
 * Get styling based on mapping type
 */
function getMappingStyle(mapping?: KeyMapping) {
  if (!mapping) {
    return {
      fill: '#334155', // slate-700
      stroke: '#475569', // slate-600
      strokeDasharray: '4 2',
    };
  }

  switch (mapping.type) {
    case 'simple':
      return { fill: '#334155', stroke: '#22c55e', strokeDasharray: 'none' }; // green-500
    case 'tap_hold':
      return {
        fill: 'rgba(127, 29, 29, 0.15)',
        stroke: '#ef4444',
        strokeDasharray: 'none',
      }; // red-500
    case 'macro':
      return {
        fill: 'rgba(88, 28, 135, 0.15)',
        stroke: '#a855f7',
        strokeDasharray: 'none',
      }; // purple-500
    case 'layer_switch':
      return {
        fill: 'rgba(113, 63, 18, 0.15)',
        stroke: '#eab308',
        strokeDasharray: 'none',
      }; // yellow-500
    default:
      return { fill: '#334155', stroke: '#475569', strokeDasharray: 'none' };
  }
}

/**
 * Format key label for display
 */
function formatKeyLabel(key: string): string {
  if (!key) return '';

  // Handle with_* helper functions
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
      modSymbols[modifier.toLowerCase()] || modifier.charAt(0).toUpperCase();
    return `${modSymbol}${innerKey.replace(/^VK_/, '')}`;
  }

  const clean = key.replace(/^VK_/, '');
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
    NUMLOCK: 'Num',
    SCROLLLOCK: 'Scrl',
    PRINTSCREEN: 'PrtSc',
  };

  const upper = clean.toUpperCase();
  if (shortNames[upper]) return shortNames[upper];
  if (clean.length > 5) return clean.slice(0, 4) + '…';
  return clean;
}

/**
 * Get mapping display text
 */
function getRemapText(mapping?: KeyMapping): string {
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
}

/**
 * Mapping type indicator icon
 */
function getMappingIcon(type?: string): string {
  switch (type) {
    case 'simple':
      return '→';
    case 'tap_hold':
      return '↕';
    case 'macro':
      return '⚡';
    case 'layer_switch':
      return '⇄';
    default:
      return '';
  }
}

function getMappingIconColor(type?: string): string {
  switch (type) {
    case 'simple':
      return '#4ade80'; // green-400
    case 'tap_hold':
      return '#f87171'; // red-400
    case 'macro':
      return '#c084fc'; // purple-400
    case 'layer_switch':
      return '#facc15'; // yellow-400
    default:
      return '#94a3b8'; // slate-400
  }
}

/**
 * Individual key SVG component
 */
const KeySVG: React.FC<KeySVGProps> = React.memo(
  ({ keyData, mapping, isPressed, onClick, simulatorMode = false }) => {
    const [isHovered, setIsHovered] = useState(false);
    const [isClicked, setIsClicked] = useState(false);

    const { code, label, x, y, w, h, shape } = keyData;
    const style = getMappingStyle(mapping);
    const remapText = getRemapText(mapping);
    const icon = getMappingIcon(mapping?.type);
    const iconColor = getMappingIconColor(mapping?.type);

    // Generate path based on shape
    const path =
      shape === 'iso-enter'
        ? generateISOEnterPath(x, y, w, h)
        : generateRectPath(x, y, w, h);

    // Calculate center position for text
    const centerX = x * UNIT_SIZE + (w * UNIT_SIZE - KEY_GAP) / 2;
    const centerY = y * UNIT_SIZE + (h * UNIT_SIZE - KEY_GAP) / 2;

    // Icon position (top-right)
    const iconX = x * UNIT_SIZE + w * UNIT_SIZE - KEY_GAP - 12;
    const iconY = y * UNIT_SIZE + 12;

    const handleClick = useCallback(() => {
      setIsClicked(true);
      setTimeout(() => setIsClicked(false), 150);
      onClick();
    }, [onClick]);

    const tooltipContent = useMemo(() => {
      if (!mapping) return `${code} (Default)`;
      switch (mapping.type) {
        case 'simple':
          return `${code} → ${mapping.tapAction}`;
        case 'tap_hold':
          return `${code} → Tap: ${mapping.tapAction}, Hold: ${mapping.holdAction} (${mapping.threshold}ms)`;
        case 'macro':
          return `${code} → Macro (${mapping.macroSteps?.length || 0} steps)`;
        case 'layer_switch':
          return `${code} → Layer: ${mapping.targetLayer}`;
        default:
          return `${code} (Default)`;
      }
    }, [code, mapping]);

    // Colors
    const fillColor = isPressed
      ? '#22c55e'
      : isClicked
        ? '#3b82f6'
        : style.fill;
    const strokeColor = isPressed ? '#4ade80' : style.stroke;
    const brightness = isHovered ? 1.15 : 1;

    // Build className for simulator mode
    const className = simulatorMode
      ? 'key-group opacity-50 cursor-not-allowed'
      : 'key-group';

    return (
      <g
        className={className}
        style={{ cursor: simulatorMode ? 'not-allowed' : 'pointer' }}
        onClick={handleClick}
        onMouseEnter={() => !simulatorMode && setIsHovered(true)}
        onMouseLeave={() => !simulatorMode && setIsHovered(false)}
        role="button"
        tabIndex={0}
        aria-label={`Key ${code}. ${tooltipContent}. ${
          simulatorMode ? 'Simulator mode active.' : 'Click to configure.'
        }`}
        onKeyDown={(e) => e.key === 'Enter' && handleClick()}
      >
        {/* Native SVG tooltip */}
        <title>{tooltipContent}</title>

        {/* Key shadow/depth effect */}
        <path
          d={path}
          fill="#1e293b"
          transform={`translate(0, ${KEY_INSET})`}
        />

        {/* Main key surface */}
        <path
          d={path}
          fill={fillColor}
          stroke={strokeColor}
          strokeWidth={2}
          strokeDasharray={style.strokeDasharray}
          style={{
            filter: `brightness(${brightness})`,
            transition: 'all 0.15s ease',
            transform: isHovered ? 'translateY(-1px)' : 'translateY(0)',
          }}
        />

        {/* Key label (original key) */}
        <text
          x={centerX}
          y={centerY - (mapping ? 6 : 0)}
          textAnchor="middle"
          dominantBaseline="middle"
          fill="#94a3b8"
          fontSize={10}
          fontFamily="monospace"
        >
          {label}
        </text>

        {/* Mapping text */}
        {mapping && (
          <text
            x={centerX}
            y={centerY + 8}
            textAnchor="middle"
            dominantBaseline="middle"
            fill="#fde047"
            fontSize={11}
            fontWeight="bold"
            fontFamily="monospace"
          >
            {remapText}
          </text>
        )}

        {/* Mapping type icon */}
        {mapping && icon && (
          <text
            x={iconX}
            y={iconY}
            textAnchor="end"
            dominantBaseline="middle"
            fill={iconColor}
            fontSize={10}
            fontWeight="bold"
          >
            {icon}
          </text>
        )}
      </g>
    );
  }
);

KeySVG.displayName = 'KeySVG';

/**
 * Normalize key code to VK_ format for mapping lookup
 * Maps QMK-style KC_ codes to system VK_ codes based on DSL manual
 *
 * Handles:
 * - KC_A -> VK_A (letters)
 * - KC_0-9 -> VK_Num0-9 (top row numbers)
 * - KC_P0-9 -> VK_Numpad0-9 (numpad digit keys)
 * - KC_NLCK -> VK_NumLock, etc. (numpad special keys)
 * - VK_A -> VK_A (already normalized)
 */
function normalizeKeyCode(code: string): string {
  if (!code) return code;

  // Already in VK_ format
  if (code.startsWith('VK_')) return code;

  // Handle top row number keys: KC_0-KC_9 -> VK_Num0-VK_Num9
  if (code.match(/^KC_[0-9]$/)) {
    const digit = code.charAt(code.length - 1);
    return `VK_Num${digit}`;
  }

  // Handle numpad digit keys: KC_P0-KC_P9 -> VK_Numpad0-VK_Numpad9
  if (code.match(/^KC_P[0-9]$/)) {
    const digit = code.charAt(code.length - 1);
    return `VK_Numpad${digit}`;
  }

  // Handle special numpad keys
  const numpadMap: Record<string, string> = {
    KC_NLCK: 'VK_NumLock',
    KC_PSLS: 'VK_NumpadDivide',
    KC_PAST: 'VK_NumpadMultiply',
    KC_PMNS: 'VK_NumpadSubtract',
    KC_PPLS: 'VK_NumpadAdd',
    KC_PENT: 'VK_NumpadEnter',
    KC_PDOT: 'VK_NumpadDecimal',
  };

  if (numpadMap[code]) {
    return numpadMap[code];
  }

  // Convert KC_ to VK_
  if (code.startsWith('KC_')) return code.replace(/^KC_/, 'VK_');

  // No prefix - add VK_
  return `VK_${code}`;
}

/**
 * Main SVG Keyboard component
 */
export const SVGKeyboard: React.FC<SVGKeyboardProps> = ({
  keys,
  keyMappings,
  onKeyClick,
  simulatorMode = false,
  pressedKeys = new Set(),
  className = '',
  layoutName = 'Keyboard',
}) => {
  // Calculate SVG dimensions
  const dimensions = useMemo(() => {
    if (keys.length === 0) {
      return { width: 800, height: 300 }; // Default fallback
    }
    const maxX = Math.max(...keys.map((k) => k.x + k.w));
    const maxY = Math.max(...keys.map((k) => k.y + k.h));
    return {
      width: maxX * UNIT_SIZE + 16, // padding
      height: maxY * UNIT_SIZE + 16,
    };
  }, [keys]);

  return (
    <svg
      width={dimensions.width}
      height={dimensions.height}
      viewBox={`0 0 ${dimensions.width} ${dimensions.height}`}
      className={className}
      style={{
        backgroundColor: 'var(--color-bg-secondary, #1e293b)',
        borderRadius: '12px',
        maxWidth: '100%',
        height: 'auto',
        display: 'block',
      }}
      role="group"
      aria-label={`${layoutName} keyboard layout${
        simulatorMode ? ' (simulator mode)' : ''
      }. Click keys to configure.`}
    >
      <g transform="translate(8, 8)">
        {keys.map((key) => {
          const normalizedCode = normalizeKeyCode(key.code);
          return (
            <KeySVG
              key={key.code}
              keyData={key}
              mapping={keyMappings.get(normalizedCode)}
              isPressed={
                pressedKeys.has(key.code) || pressedKeys.has(normalizedCode)
              }
              onClick={() => !simulatorMode && onKeyClick(normalizedCode)}
              simulatorMode={simulatorMode}
            />
          );
        })}
      </g>
    </svg>
  );
};

export default SVGKeyboard;
