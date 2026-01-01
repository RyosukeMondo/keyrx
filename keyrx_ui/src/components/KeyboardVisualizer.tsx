import React, { useMemo, useRef } from 'react';
import { KeyButton, KeyMapping } from './KeyButton';
import { parseKLEJson } from '../utils/kle-parser';
import { cn } from '../utils/cn';
import { useArrowNavigation } from '../utils/keyboard';

// Import layout data
import ANSI_104 from '../data/layouts/ANSI_104.json';

interface KeyboardVisualizerProps {
  layout: 'ANSI_104' | 'ISO_105' | 'JIS_109' | 'HHKB' | 'NUMPAD';
  keyMappings: Map<string, KeyMapping>;
  onKeyClick: (keyCode: string) => void;
  simulatorMode?: boolean;
  pressedKeys?: Set<string>;
  className?: string;
}

const layoutData = {
  ANSI_104,
  // TODO: Add other layouts when needed
  ISO_105: ANSI_104, // Placeholder
  JIS_109: ANSI_104, // Placeholder
  HHKB: ANSI_104, // Placeholder
  NUMPAD: ANSI_104, // Placeholder
};

export const KeyboardVisualizer: React.FC<KeyboardVisualizerProps> = ({
  layout,
  keyMappings,
  onKeyClick,
  simulatorMode = false,
  pressedKeys = new Set(),
  className = '',
}) => {
  const containerRef = useRef<HTMLDivElement>(null);

  const keyButtons = useMemo(() => {
    const kleData = layoutData[layout];
    return parseKLEJson(kleData);
  }, [layout]);

  // Calculate grid dimensions
  const maxRow = useMemo(
    () => Math.max(...keyButtons.map((k) => k.gridRow)),
    [keyButtons]
  );
  const maxCol = useMemo(
    () =>
      Math.max(...keyButtons.map((k) => k.gridColumn + k.gridColumnSpan - 1)),
    [keyButtons]
  );

  // Enable arrow key navigation for keyboard keys
  useArrowNavigation(containerRef, {
    orientation: 'horizontal',
    loop: true,
  });

  return (
    <div
      ref={containerRef}
      className={cn('keyboard-grid', className)}
      role="group"
      aria-label={`${layout} keyboard layout${simulatorMode ? ' (simulator mode)' : ''}. Use arrow keys to navigate between keys, Enter to select.`}
      style={{
        display: 'grid',
        gridTemplateRows: `repeat(${maxRow}, 48px)`,
        gridTemplateColumns: `repeat(${maxCol}, 48px)`,
        gap: '4px',
        padding: '16px',
        backgroundColor: 'var(--color-bg-secondary)',
        borderRadius: '12px',
      }}
    >
      {keyButtons.map((key) => (
        <div
          key={key.keyCode}
          style={{
            gridRow: key.gridRow,
            gridColumn: `${key.gridColumn} / span ${key.gridColumnSpan}`,
          }}
        >
          <KeyButton
            keyCode={key.keyCode}
            label={key.label}
            mapping={keyMappings.get(key.keyCode)}
            onClick={() => onKeyClick(key.keyCode)}
            isPressed={pressedKeys.has(key.keyCode)}
          />
        </div>
      ))}
    </div>
  );
};
