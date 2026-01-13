import React, { createContext, useContext, useMemo } from 'react';
import { parseKLEJson, type KeyButton, type KLEData } from '../utils/kle-parser';

// Import all layout data
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
  | 'ANSI_104' | 'ANSI_87'
  | 'ISO_105' | 'ISO_88'
  | 'JIS_109'
  | 'COMPACT_60' | 'COMPACT_65' | 'COMPACT_75' | 'COMPACT_96'
  | 'HHKB' | 'NUMPAD';

export interface LayoutDropdownOption {
  value: LayoutType;
  label: string;
  category: 'full' | 'tkl' | 'compact' | 'specialized';
}

export interface ParsedLayoutData {
  name: string;
  keys: KeyButton[];
  dimensions: {
    rows: number;
    cols: number;
  };
}

interface LayoutPreviewContextValue {
  getLayoutData: (layout: LayoutType) => ParsedLayoutData;
  getLayoutDimensions: (layout: LayoutType) => { rows: number; cols: number };
  layoutOptions: LayoutDropdownOption[];
  rawLayouts: Record<LayoutType, unknown>;
}

const rawLayoutData: Record<LayoutType, unknown> = {
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

export const LAYOUT_OPTIONS: LayoutDropdownOption[] = [
  // Full-size
  { value: 'ANSI_104', label: 'ANSI 104 (Full)', category: 'full' },
  { value: 'ISO_105', label: 'ISO 105 (Full)', category: 'full' },
  { value: 'JIS_109', label: 'JIS 109 (Full)', category: 'full' },
  // Tenkeyless
  { value: 'ANSI_87', label: 'ANSI 87 (TKL)', category: 'tkl' },
  { value: 'ISO_88', label: 'ISO 88 (TKL)', category: 'tkl' },
  // Compact
  { value: 'COMPACT_60', label: '60%', category: 'compact' },
  { value: 'COMPACT_65', label: '65%', category: 'compact' },
  { value: 'COMPACT_75', label: '75%', category: 'compact' },
  { value: 'COMPACT_96', label: '96%', category: 'compact' },
  // Specialized
  { value: 'HHKB', label: 'HHKB', category: 'specialized' },
  { value: 'NUMPAD', label: 'Numpad', category: 'specialized' },
];

const LayoutPreviewContext = createContext<LayoutPreviewContextValue | null>(null);

// Cache for parsed layout data
const layoutCache = new Map<LayoutType, ParsedLayoutData>();

function parseAndCacheLayout(layout: LayoutType): ParsedLayoutData {
  const cached = layoutCache.get(layout);
  if (cached) return cached;

  const rawData = rawLayoutData[layout];
  const keys = parseKLEJson(rawData as KLEData);

  // Calculate dimensions
  const rows = Math.max(...keys.map(k => k.gridRow));
  const cols = Math.max(...keys.map(k => k.gridColumn + k.gridColumnSpan - 1));

  const parsed: ParsedLayoutData = {
    name: (rawData as { name: string }).name,
    keys,
    dimensions: { rows, cols },
  };

  layoutCache.set(layout, parsed);
  return parsed;
}

interface LayoutPreviewProviderProps {
  children: React.ReactNode;
}

export const LayoutPreviewProvider: React.FC<LayoutPreviewProviderProps> = ({ children }) => {
  const value = useMemo<LayoutPreviewContextValue>(() => ({
    getLayoutData: parseAndCacheLayout,
    getLayoutDimensions: (layout: LayoutType) => parseAndCacheLayout(layout).dimensions,
    layoutOptions: LAYOUT_OPTIONS,
    rawLayouts: rawLayoutData,
  }), []);

  return (
    <LayoutPreviewContext.Provider value={value}>
      {children}
    </LayoutPreviewContext.Provider>
  );
};

export function useLayoutPreview(): LayoutPreviewContextValue {
  const context = useContext(LayoutPreviewContext);
  if (!context) {
    throw new Error('useLayoutPreview must be used within a LayoutPreviewProvider');
  }
  return context;
}

// Standalone hook for getting layout data without provider (fallback)
export function useLayoutData(layout: LayoutType): ParsedLayoutData {
  return useMemo(() => parseAndCacheLayout(layout), [layout]);
}
