import React, { useMemo, useRef } from 'react';
import { useDroppable } from '@dnd-kit/core';
import { KeyButton } from './KeyButton';
import type { KeyMapping } from '@/types';
import { parseKLEJson } from '../utils/kle-parser';
import { cn } from '../utils/cn';
import { useArrowNavigation } from '../utils/keyboard';
import type { AssignableKey } from './KeyAssignmentPanel';

// Import layout data
import ANSI_104 from '../data/layouts/ANSI_104.json';
import ANSI_87 from '../data/layouts/ANSI_87.json';
import ISO_105 from '../data/layouts/ISO_105.json';
import ISO_88 from '../data/layouts/ISO_88.json';
import JIS_109 from '../data/layouts/JIS_109.json';
import COMPACT_60 from '../data/layouts/COMPACT_60.json';
import COMPACT_65 from '../data/layouts/COMPACT_65.json';
import COMPACT_75 from '../data/layouts/COMPACT_75.json';
import COMPACT_96 from '../data/layouts/COMPACT_96.json';
import HHKB from '../data/layouts/HHKB.json';
import NUMPAD from '../data/layouts/NUMPAD.json';

interface KeyboardVisualizerProps {
  layout: 'ANSI_104' | 'ANSI_87' | 'ISO_105' | 'ISO_88' | 'JIS_109' | 'COMPACT_60' | 'COMPACT_65' | 'COMPACT_75' | 'COMPACT_96' | 'HHKB' | 'NUMPAD';
  keyMappings: Map<string, KeyMapping>;
  onKeyClick: (keyCode: string) => void;
  onKeyDrop?: (keyCode: string, droppedKey: AssignableKey) => void;
  simulatorMode?: boolean;
  pressedKeys?: Set<string>;
  className?: string;
}

const layoutData = {
  ANSI_104,
  ANSI_87,
  ISO_105,
  ISO_88,
  JIS_109,
  COMPACT_60,
  COMPACT_65,
  COMPACT_75,
  COMPACT_96,
  HHKB,
  NUMPAD,
};

interface DroppableKeyWrapperProps {
  keyCode: string;
  label: string;
  mapping?: KeyMapping;
  isPressed: boolean;
  onClick: () => void;
  onDrop?: (droppedKey: AssignableKey) => void;
  disabled?: boolean;
}

/**
 * Wrapper component for individual keys that makes them droppable zones
 */
const DroppableKeyWrapper: React.FC<DroppableKeyWrapperProps> = ({
  keyCode,
  label,
  mapping,
  isPressed,
  onClick,
  onDrop,
  disabled = false,
}) => {
  const { setNodeRef, isOver } = useDroppable({
    id: `drop-${keyCode}`,
    data: { keyCode },
    disabled,
  });

  const handleClick = () => {
    if (!disabled) {
      onClick();
    }
  };

  // Build comprehensive aria-label for drop zone
  const mappingDescription = mapping
    ? `Currently mapped to ${mapping.tapAction || 'custom action'}`
    : 'No mapping assigned';

  const ariaLabel = disabled
    ? `${label} key. ${mappingDescription}. Not configurable.`
    : `${label} key. ${mappingDescription}. Drop zone for key assignment. ${isOver ? 'Drop here to assign' : ''}`;

  return (
    <div
      ref={setNodeRef}
      className={cn(
        'relative',
        isOver && !disabled && 'ring-2 ring-primary-500 ring-offset-2 ring-offset-slate-800'
      )}
      aria-label={ariaLabel}
      aria-dropeffect={disabled ? 'none' : isOver ? 'move' : 'none'}
    >
      <KeyButton
        keyCode={keyCode}
        label={label}
        mapping={mapping}
        onClick={handleClick}
        isPressed={isPressed}
        className={cn(
          disabled && 'opacity-50 cursor-not-allowed'
        )}
      />
    </div>
  );
};

DroppableKeyWrapper.displayName = 'DroppableKeyWrapper';

export const KeyboardVisualizer: React.FC<KeyboardVisualizerProps> = ({
  layout,
  keyMappings,
  onKeyClick,
  onKeyDrop,
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

  const handleKeyDrop = (keyCode: string) => (droppedKey: AssignableKey) => {
    if (onKeyDrop) {
      onKeyDrop(keyCode, droppedKey);
    }
  };

  return (
    <div
      ref={containerRef}
      className={cn('keyboard-grid', className)}
      role="group"
      data-testid="keyboard-visualizer"
      aria-label={`${layout} keyboard layout${simulatorMode ? ' (simulator mode)' : ''}. Use arrow keys to navigate between keys, Enter to select.`}
      style={{
        display: 'grid',
        gridTemplateRows: `repeat(${maxRow}, 52px)`,
        gridTemplateColumns: `repeat(${maxCol}, 52px)`,
        gap: '2px',
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
          <DroppableKeyWrapper
            keyCode={key.keyCode}
            label={key.label}
            mapping={keyMappings.get(key.keyCode)}
            onClick={() => onKeyClick(key.keyCode)}
            onDrop={handleKeyDrop(key.keyCode)}
            isPressed={pressedKeys.has(key.keyCode)}
            disabled={simulatorMode}
          />
        </div>
      ))}
    </div>
  );
};
