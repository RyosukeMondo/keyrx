import React, { ReactNode } from 'react';
import type { PaletteKey } from '../KeyPalette';
import type { ViewMode } from '@/utils/paletteHelpers.tsx';

interface QuickAccessSectionProps {
  title: string;
  icon: ReactNode;
  keys: PaletteKey[];
  viewMode: ViewMode;
  renderKeyItem: (
    key: PaletteKey,
    onClick: () => void,
    compact: boolean
  ) => ReactNode;
  onKeySelect: (key: PaletteKey) => void;
}

/**
 * Section for displaying favorites or recent keys
 */
export const QuickAccessSection: React.FC<QuickAccessSectionProps> = ({
  title,
  icon,
  keys,
  viewMode,
  renderKeyItem,
  onKeySelect,
}) => {
  if (keys.length === 0) return null;

  return (
    <div className="mb-4">
      <div className="flex items-center gap-2 mb-2">
        {icon}
        <h4 className="text-sm font-semibold text-slate-300">{title}</h4>
      </div>
      <div
        className={`p-3 bg-slate-800/50 rounded-lg ${
          viewMode === 'grid'
            ? 'grid grid-cols-8 gap-2'
            : 'flex flex-col gap-2'
        }`}
      >
        {keys.map((key) => renderKeyItem(key, () => onKeySelect(key), true))}
      </div>
    </div>
  );
};
