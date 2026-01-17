import React from 'react';
import { Grid3x3, List, Keyboard } from 'lucide-react';
import type { ViewMode } from '@/utils/paletteHelpers.tsx';

interface PaletteHeaderProps {
  viewMode: ViewMode;
  onToggleView: () => void;
  onCaptureKey: () => void;
}

/**
 * Header for key palette with view mode toggle and capture button
 */
export const PaletteHeader: React.FC<PaletteHeaderProps> = ({
  viewMode,
  onToggleView,
  onCaptureKey,
}) => {
  return (
    <div className="flex items-center justify-between mb-4">
      <h3 className="text-lg font-semibold text-slate-100">Key Palette</h3>
      <div className="flex gap-2">
        {/* Capture Key button */}
        <button
          onClick={onCaptureKey}
          className="px-3 py-2 bg-primary-500 hover:bg-primary-600 text-white rounded-lg transition-colors flex items-center gap-2 text-sm font-medium"
          title="Press any physical key to select it"
          aria-label="Capture physical key"
        >
          <Keyboard className="w-4 h-4" />
          <span>Capture Key</span>
        </button>

        {/* View toggle buttons */}
        <div className="flex gap-1">
          <button
            onClick={onToggleView}
            className={`p-2 rounded-lg transition-colors ${
              viewMode === 'grid'
                ? 'bg-primary-500 text-white'
                : 'bg-slate-700 text-slate-400 hover:bg-slate-600 hover:text-slate-300'
            }`}
            title="Grid view"
            aria-label="Grid view"
          >
            <Grid3x3 className="w-4 h-4" />
          </button>
          <button
            onClick={onToggleView}
            className={`p-2 rounded-lg transition-colors ${
              viewMode === 'list'
                ? 'bg-primary-500 text-white'
                : 'bg-slate-700 text-slate-400 hover:bg-slate-600 hover:text-slate-300'
            }`}
            title="List view"
            aria-label="List view"
          >
            <List className="w-4 h-4" />
          </button>
        </div>
      </div>
    </div>
  );
};
