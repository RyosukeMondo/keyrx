import { useState, useMemo } from 'react';
import { parseKLEToSVG, type SVGKeyData, type KLEData } from '@/utils/kle-parser';
import type { LayoutType } from '@/components/KeyboardVisualizer';

// Import layout data
import ANSI_104 from '@/data/layouts/ANSI_104.json';
import ANSI_87 from '@/data/layouts/ANSI_87.json';
import ISO_105 from '@/data/layouts/ISO_105.json';
import ISO_88 from '@/data/layouts/ISO_88.json';
import JIS_109 from '@/data/layouts/JIS_109.json';
import COMPACT_60 from '@/data/layouts/COMPACT_60.json';
import COMPACT_65 from '@/data/layouts/COMPACT_65.json';
import COMPACT_75 from '@/data/layouts/COMPACT_75.json';
import COMPACT_96 from '@/data/layouts/COMPACT_96.json';
import HHKB from '@/data/layouts/HHKB.json';
import NUMPAD from '@/data/layouts/NUMPAD.json';

const layoutData: Record<LayoutType, KLEData> = {
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

export interface UseKeyboardLayoutReturn {
  layout: LayoutType;
  setLayout: (layout: LayoutType) => void;
  layoutKeys: SVGKeyData[];
}

/**
 * Custom hook for managing keyboard layout selection and parsed layout keys.
 *
 * Handles keyboard layout state and provides memoized layout keys that are
 * recalculated only when the layout changes.
 *
 * @param initialLayout - Initial layout type (default: 'ANSI_104')
 * @returns Object containing layout state, setter, and memoized layout keys
 *
 * @example
 * ```tsx
 * const { layout, setLayout, layoutKeys } = useKeyboardLayout('ANSI_104');
 *
 * return (
 *   <div>
 *     <select value={layout} onChange={(e) => setLayout(e.target.value as LayoutType)}>
 *       <option value="ANSI_104">ANSI 104</option>
 *       <option value="ISO_105">ISO 105</option>
 *     </select>
 *     <KeyboardVisualizer layoutKeys={layoutKeys} />
 *   </div>
 * );
 * ```
 */
export function useKeyboardLayout(
  initialLayout: LayoutType = 'ANSI_104'
): UseKeyboardLayoutReturn {
  const [layout, setLayout] = useState<LayoutType>(initialLayout);

  // Parse layout data to SVG format - only recalculate when layout changes
  const layoutKeys = useMemo(() => {
    const kleData = layoutData[layout];
    return parseKLEToSVG(kleData);
  }, [layout]);

  return {
    layout,
    setLayout,
    layoutKeys,
  };
}
