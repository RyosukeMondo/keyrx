import React from 'react';
import { Card } from '@/components/Card';
import { LayerSwitcher } from '@/components/LayerSwitcher';
import { KeyboardVisualizerContainer } from './KeyboardVisualizerContainer';
import type { KeyMapping } from '@/types';

interface GlobalKeyboardPanelProps {
  profileName: string;
  activeLayer: string;
  availableLayers: string[];
  onLayerChange: (layer: string) => void;
  globalSelected: boolean;
  onToggleGlobal: (selected: boolean) => void;
  keyMappings: Map<string, KeyMapping>;
  onKeyClick: (keyCode: string) => void;
  selectedKeyCode: string | null;
  initialLayout: string;
  isVisible: boolean;
}

/**
 * Panel for configuring global keyboard mappings (applies to all devices)
 */
export const GlobalKeyboardPanel: React.FC<GlobalKeyboardPanelProps> = ({
  profileName,
  activeLayer,
  availableLayers,
  onLayerChange,
  globalSelected,
  onToggleGlobal,
  keyMappings,
  onKeyClick,
  selectedKeyCode,
  initialLayout,
  isVisible,
}) => {
  if (!globalSelected) return null;

  return (
    <div
      role="tabpanel"
      id="panel-global"
      aria-labelledby="tab-global"
      className={`flex flex-col gap-3 ${isVisible ? 'flex' : 'hidden'}`}
    >
      {/* Global Pane Header */}
      <div className="flex items-center justify-between px-4 py-2 bg-slate-800/50 border border-slate-700 rounded-md">
        <h2 className="text-lg font-semibold text-slate-200">Global Keys</h2>
        <div className="flex items-center gap-2">
          <input
            type="checkbox"
            id="global-checkbox"
            checked={globalSelected}
            onChange={(e) => onToggleGlobal(e.target.checked)}
            className="w-4 h-4 text-primary-600 bg-slate-700 border-slate-600 rounded focus:ring-primary-500 focus:ring-2"
          />
          <label htmlFor="global-checkbox" className="text-sm text-slate-300">
            Enable
          </label>
        </div>
      </div>

      {/* Global Keyboard Content */}
      <div className="flex gap-2 flex-1 bg-slate-900/30 rounded-lg p-3">
        <LayerSwitcher
          activeLayer={activeLayer}
          availableLayers={availableLayers}
          onLayerChange={onLayerChange}
        />
        <Card
          className="bg-gradient-to-br from-slate-800 to-slate-900 flex-1"
          aria-label="Global Keyboard Configuration"
        >
          <h3 className="text-xl font-bold text-primary-400 mb-4">
            Global Keyboard (All Devices)
          </h3>
          <KeyboardVisualizerContainer
            profileName={profileName}
            activeLayer={activeLayer}
            mappings={keyMappings}
            onKeyClick={onKeyClick}
            selectedKeyCode={selectedKeyCode}
            initialLayout={initialLayout}
          />
        </Card>
      </div>
    </div>
  );
};
