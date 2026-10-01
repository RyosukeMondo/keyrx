/**
 * SVG-based Keyboard Visualizer
 * Renders keyboard layouts using SVG for precise key shapes including ISO Enter
 *
 * MIT License compatible - no GPL dependencies
 */

import React, { useMemo, useState, useCallback, useRef } from 'react';
import type { KeyMapping } from '@/types';
import { normalizeKeyCode } from '@/utils/keyNames';
import { NAV_KEYS, nextKeyIndex, type NavKey } from '@/utils/spatialNav';
import {
  UNIT_SIZE,
  KEY_GAP,
  KEY_INSET,
  MIN_SCALE,
  LABEL_MAX,
  LABEL_MIN,
  MAPPING_MAX,
  MAPPING_MIN,
  fitText,
  describeKey,
  generateISOEnterPath,
  generateRectPath,
  getMappingStyle,
  getRemapText,
  getMappingIcon,
  getMappingIconColor,
  type SVGKey,
} from '@/utils/keycap';

export { describeKey };
export type { SVGKey };

interface SVGKeyboardProps {
  keys: SVGKey[];
  keyMappings: Map<string, KeyMapping>;
  onKeyClick: (keyCode: string) => void;
  simulatorMode?: boolean;
  pressedKeys?: Set<string>;
  className?: string;
  layoutName?: string;
  /** Dynamic key labels from layout detection API */
  labelOverrides?: Record<string, string>;
  /** Normalized (VK_*) code of the key being edited; exposed as aria-pressed. */
  selectedKeyCode?: string | null;
}

interface KeySVGProps {
  keyData: SVGKey;
  /** Normalized (VK_*) code, used for data attributes and the accessible name. */
  normalizedCode: string;
  index: number;
  mapping?: KeyMapping;
  isPressed: boolean;
  isSelected: boolean;
  /** Roving tabindex: exactly one key of the keyboard is a Tab stop. */
  isTabStop: boolean;
  onClick: () => void;
  onFocusKey: (index: number) => void;
  onNavigate: (index: number, key: NavKey) => void;
  simulatorMode?: boolean;
}

/**
 * Individual key SVG component
 */
const KeySVG: React.FC<KeySVGProps> = React.memo(
  ({
    keyData,
    normalizedCode,
    index,
    mapping,
    isPressed,
    isSelected,
    isTabStop,
    onClick,
    onFocusKey,
    onNavigate,
    simulatorMode = false,
  }) => {
    const [isHovered, setIsHovered] = useState(false);
    const [isClicked, setIsClicked] = useState(false);

    const { x, y, w, h, shape, label } = keyData;
    const style = getMappingStyle(mapping);
    const icon = getMappingIcon(mapping?.type);
    const iconColor = getMappingIconColor(mapping?.type);

    // Generate path based on shape
    const path =
      shape === 'iso-enter'
        ? generateISOEnterPath(x, y, w, h)
        : generateRectPath(x, y, w, h);

    // Calculate center position for text
    const keyWidth = w * UNIT_SIZE - KEY_GAP;
    const centerX = x * UNIT_SIZE + keyWidth / 2;
    const centerY = y * UNIT_SIZE + (h * UNIT_SIZE - KEY_GAP) / 2;
    const innerWidth = keyWidth - 8;
    const legend = fitText(label, innerWidth, LABEL_MAX, LABEL_MIN);
    const remap = fitText(
      getRemapText(mapping),
      innerWidth,
      MAPPING_MAX,
      MAPPING_MIN
    );

    // Icon position (top-right)
    const iconX = x * UNIT_SIZE + w * UNIT_SIZE - KEY_GAP - 6;
    const iconY = y * UNIT_SIZE + 10;

    const handleClick = useCallback(() => {
      // In simulator mode, don't trigger onClick (only visual feedback)
      if (simulatorMode) {
        return;
      }
      setIsClicked(true);
      setTimeout(() => setIsClicked(false), 150);
      onClick();
    }, [onClick, simulatorMode]);

    const handleKeyDown = (e: React.KeyboardEvent) => {
      if (e.key === 'Enter' || e.key === ' ') {
        e.preventDefault();
        handleClick();
      } else if (NAV_KEYS.has(e.key)) {
        e.preventDefault();
        onNavigate(index, e.key as NavKey);
      }
    };

    const description = describeKey(normalizedCode, mapping);

    // Colors
    const fillColor = isPressed
      ? '#22c55e'
      : isClicked
        ? '#3b82f6'
        : style.fill;
    const strokeColor = isPressed
      ? '#4ade80'
      : isSelected
        ? '#38bdf8'
        : style.stroke;
    const brightness = isHovered ? 1.15 : 1;

    // In simulator mode the keyboard is a read-only display: no tab stops.
    const interactive = !simulatorMode;
    const className = interactive
      ? 'key-group'
      : 'key-group opacity-50 cursor-not-allowed';

    return (
      <g
        className={className}
        style={{ cursor: interactive ? 'pointer' : 'not-allowed' }}
        onClick={handleClick}
        onMouseEnter={() => setIsHovered(true)}
        onMouseLeave={() => setIsHovered(false)}
        role={interactive ? 'button' : 'img'}
        tabIndex={interactive ? (isTabStop ? 0 : -1) : undefined}
        aria-pressed={interactive ? isSelected : undefined}
        aria-label={description}
        data-key-code={normalizedCode}
        data-key-index={index}
        onFocus={interactive ? () => onFocusKey(index) : undefined}
        onKeyDown={interactive ? handleKeyDown : undefined}
      >
        {/* Native SVG tooltip */}
        <title>{description}</title>

        {/* Key shadow/depth effect */}
        <path
          d={path}
          fill="#1e293b"
          transform={`translate(0, ${KEY_INSET})`}
        />

        {/* Main key surface */}
        <path
          className="key-face"
          d={path}
          fill={fillColor}
          stroke={strokeColor}
          strokeWidth={isSelected ? 3 : 2}
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
          y={centerY - (mapping ? 8 : 0)}
          textAnchor="middle"
          dominantBaseline="middle"
          fill="#cbd5e1"
          fontSize={legend.size}
          fontFamily="monospace"
          aria-hidden="true"
        >
          {legend.text}
        </text>

        {/* Mapping text */}
        {mapping && (
          <text
            x={centerX}
            y={centerY + 9}
            textAnchor="middle"
            dominantBaseline="middle"
            fill="#fde047"
            fontSize={remap.size}
            fontWeight="bold"
            fontFamily="monospace"
            aria-hidden="true"
          >
            {remap.text}
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
            fontSize={11}
            fontWeight="bold"
            aria-hidden="true"
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
 * Main SVG Keyboard component
 *
 * The keyboard is ONE Tab stop (roving tabindex): Tab enters it, arrow keys
 * move between keycaps spatially, Home/End jump to the row edges, and
 * Enter/Space selects the focused key.
 */
export const SVGKeyboard: React.FC<SVGKeyboardProps> = ({
  keys,
  keyMappings,
  onKeyClick,
  simulatorMode = false,
  pressedKeys = new Set(),
  className = '',
  layoutName = 'Keyboard',
  labelOverrides,
  selectedKeyCode = null,
}) => {
  const svgRef = useRef<SVGSVGElement>(null);

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

  const normalizedCodes = useMemo(
    () => keys.map((k) => normalizeKeyCode(k.code)),
    [keys]
  );

  // Roving tabindex: remember the last focused key; before any focus, the
  // selected key (or the first key) is the Tab stop.
  const [focusedIndex, setFocusedIndex] = useState<number | null>(null);
  const selectedIndex = selectedKeyCode
    ? normalizedCodes.indexOf(selectedKeyCode)
    : -1;
  const tabStopIndex = Math.min(
    focusedIndex ?? (selectedIndex >= 0 ? selectedIndex : 0),
    Math.max(keys.length - 1, 0)
  );

  const handleNavigate = useCallback(
    (from: number, key: NavKey) => {
      const to = nextKeyIndex(keys, from, key);
      if (to === from) return;
      setFocusedIndex(to);
      svgRef.current
        ?.querySelector<SVGGElement>(`[data-key-index="${to}"]`)
        ?.focus();
    },
    [keys]
  );

  return (
    <svg
      ref={svgRef}
      width={dimensions.width}
      height={dimensions.height}
      viewBox={`0 0 ${dimensions.width} ${dimensions.height}`}
      className={className}
      style={{
        backgroundColor: 'var(--color-bg-secondary, #1e293b)',
        borderRadius: '12px',
        maxWidth: '100%',
        // Never shrink so far that keycap text becomes unreadable; the
        // surrounding container scrolls horizontally instead.
        minWidth: dimensions.width * MIN_SCALE,
        height: 'auto',
        display: 'block',
      }}
      role="group"
      aria-label={
        simulatorMode
          ? `${layoutName} keyboard layout (simulator mode, read-only)`
          : `${layoutName} keyboard. Use the arrow keys to move between keys and Enter to edit the focused key.`
      }
    >
      <g transform="translate(8, 8)">
        {keys.map((key, index) => {
          const normalizedCode = normalizedCodes[index];
          const keyName = normalizedCode.replace(/^VK_/, '');
          const displayLabel = labelOverrides?.[keyName] || key.label;
          return (
            <KeySVG
              key={key.code}
              keyData={{ ...key, label: displayLabel }}
              normalizedCode={normalizedCode}
              index={index}
              mapping={keyMappings.get(normalizedCode)}
              isPressed={
                pressedKeys.has(key.code) || pressedKeys.has(normalizedCode)
              }
              isSelected={selectedKeyCode === normalizedCode}
              isTabStop={index === tabStopIndex}
              onClick={() => onKeyClick(normalizedCode)}
              onFocusKey={setFocusedIndex}
              onNavigate={handleNavigate}
              simulatorMode={simulatorMode}
            />
          );
        })}
      </g>
    </svg>
  );
};

export default SVGKeyboard;
