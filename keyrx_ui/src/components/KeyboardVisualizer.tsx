/**
 * Keyboard Visualizer Component
 * Renders keyboard layouts using SVG for accurate key shapes
 *
 * Supports: ANSI, ISO (with L-shaped Enter), JIS, compact layouts
 */

import React, { useMemo } from 'react';
import { SVGKeyboard } from './SVGKeyboard';
import type { KeyMapping } from '@/types';
import { parseKLEToSVG } from '../utils/kle-parser';
import { cn } from '../utils/cn';

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

export type LayoutType =
  | 'ANSI_104'
  | 'ANSI_87'
  | 'ISO_105'
  | 'ISO_88'
  | 'JIS_109'
  | 'COMPACT_60'
  | 'COMPACT_65'
  | 'COMPACT_75'
  | 'COMPACT_96'
  | 'HHKB'
  | 'NUMPAD';

interface KeyboardVisualizerProps {
  layout: LayoutType;
  keyMappings: Map<string, KeyMapping>;
  onKeyClick: (keyCode: string) => void;
  simulatorMode?: boolean;
  pressedKeys?: Set<string>;
  className?: string;
}

interface KeyLayout {
  code: string;
  label: string;
  x: number;
  y: number;
  w: number;
  h?: number;
}

const layoutData: Record<LayoutType, { name: string; keys: KeyLayout[] }> = {
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

export const KeyboardVisualizer: React.FC<KeyboardVisualizerProps> = ({
  layout,
  keyMappings,
  onKeyClick,
  simulatorMode = false,
  pressedKeys = new Set(),
  className = '',
}) => {
  // Parse layout data to SVG format
  const svgKeys = useMemo(() => {
    const kleData = layoutData[layout];
    return parseKLEToSVG(kleData);
  }, [layout]);

  // Debug: log keys count
  if (svgKeys.length === 0) {
    console.warn('KeyboardVisualizer: No keys parsed from layout', layout);
  }

  const layoutName = layoutData[layout].name;
  const ariaLabel = simulatorMode
    ? `${layout} - ${layoutName} keyboard layout in simulator mode. Keys show active state but cannot be configured.`
    : `${layout} - ${layoutName} keyboard layout. Use arrow keys to navigate between keys, press Enter to select a key, or click on any key to configure it.`;

  return (
    <div
      className={cn('keyboard-visualizer', className)}
      data-testid="keyboard-visualizer"
      role="group"
      aria-label={ariaLabel}
      style={{ display: 'grid', minHeight: '200px', minWidth: '400px' }}
    >
      <SVGKeyboard
        keys={svgKeys}
        keyMappings={keyMappings}
        onKeyClick={onKeyClick}
        simulatorMode={simulatorMode}
        pressedKeys={pressedKeys}
        layoutName={layoutName}
      />
    </div>
  );
};
