import React, { useState, useCallback } from 'react';
import { Card } from '@/components/Card';
import { Dropdown } from '@/components/Dropdown';

// Placeholder for KeyboardVisualizer (Task 16)
// TODO: Replace with actual KeyboardVisualizer component once implemented
const KeyboardVisualizerPlaceholder: React.FC<{
  layout: string;
  onKeyClick: (keyCode: string) => void;
}> = ({ layout }) => {
  return (
    <div className="bg-slate-800 rounded-lg p-8 text-center">
      <p className="text-slate-400 mb-2">KeyboardVisualizer Component</p>
      <p className="text-sm text-slate-500">
        Layout: {layout} - This will be replaced with Task 16 implementation
      </p>
      <div className="mt-4 text-xs text-slate-600">
        Interactive keyboard with {layout === 'ANSI_104' ? '104' : '105'} keys
      </div>
    </div>
  );
};

interface ConfigPageProps {
  profileName?: string;
}

export const ConfigPage: React.FC<ConfigPageProps> = ({
  profileName = 'Default',
}) => {
  const [selectedLayout, setSelectedLayout] = useState('ANSI_104');
  const [selectedLayer, setSelectedLayer] = useState('base');
  const [previewMode, setPreviewMode] = useState(false);

  // Layout options
  const layoutOptions = [
    { value: 'ANSI_104', label: 'ANSI 104' },
    { value: 'ISO_105', label: 'ISO 105' },
    { value: 'JIS_109', label: 'JIS 109' },
    { value: 'HHKB', label: 'HHKB' },
    { value: 'NUMPAD', label: 'Numpad' },
  ];

  // Layer options (mock data - would come from API)
  const layerOptions = [
    { value: 'base', label: 'Base' },
    { value: 'nav', label: 'Nav' },
    { value: 'num', label: 'Num' },
    { value: 'fn', label: 'Fn' },
    { value: 'gaming', label: 'Gaming' },
  ];

  const handleKeyClick = useCallback((keyCode: string) => {
    console.log('Key clicked:', keyCode);
    // TODO: Open KeyConfigDialog modal (Task 17)
  }, []);

  const handleLayoutChange = useCallback((value: string) => {
    setSelectedLayout(value);
  }, []);

  const handleLayerChange = useCallback((value: string) => {
    setSelectedLayer(value);
  }, []);

  const togglePreviewMode = useCallback(() => {
    setPreviewMode((prev) => !prev);
  }, []);

  // Mock modified keys count (would come from configuration store)
  const modifiedKeysCount = 37;

  return (
    <div className="flex flex-col gap-6 p-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-4">
          <h1 className="text-2xl font-semibold text-slate-100">
            Configuration Editor
          </h1>
          <span className="text-slate-400">—</span>
          <span className="text-slate-300">Profile: {profileName}</span>
        </div>
        <button
          onClick={togglePreviewMode}
          className={`px-4 py-2 rounded-md font-medium transition-colors ${
            previewMode
              ? 'bg-green-600 text-white'
              : 'bg-slate-700 text-slate-300 hover:bg-slate-600'
          }`}
          aria-label={`Preview mode is ${previewMode ? 'on' : 'off'}`}
        >
          🧪 Preview Mode: {previewMode ? 'ON' : 'OFF'}
        </button>
      </div>

      {/* Keyboard Visualizer Card */}
      <Card variant="default" padding="lg">
        <div className="flex flex-col gap-4">
          {/* Card Header with Layout Selector */}
          <div className="flex items-center justify-between pb-4 border-b border-slate-700">
            <h2 className="text-lg font-medium text-slate-100">
              Keyboard Layout
            </h2>
            <div className="w-48">
              <Dropdown
                options={layoutOptions}
                value={selectedLayout}
                onChange={handleLayoutChange}
                aria-label="Select keyboard layout"
                searchable={false}
              />
            </div>
          </div>

          {/* Keyboard Visualizer */}
          <div className="py-4">
            <KeyboardVisualizerPlaceholder
              layout={selectedLayout}
              onKeyClick={handleKeyClick}
            />
          </div>

          {/* Example Mapping Display */}
          <div className="text-sm text-slate-400 italic pt-4 border-t border-slate-700">
            Example: <span className="font-mono text-slate-300">*Caps*</span> ={' '}
            Tap: Escape, Hold (200ms): Ctrl
          </div>
        </div>
      </Card>

      {/* Layer Selector Card */}
      <Card variant="default" padding="lg">
        <div className="flex flex-col gap-4">
          {/* Card Header */}
          <div className="flex items-center justify-between">
            <h2 className="text-lg font-medium text-slate-100">
              Active Layer: MD_00 ({selectedLayer})
            </h2>
            <button
              className="text-sm text-primary-500 hover:text-primary-400 transition-colors"
              aria-label="Open layer list"
            >
              Layer List ▼
            </button>
          </div>

          {/* Layer Buttons */}
          <div className="flex gap-2 flex-wrap">
            {layerOptions.map((layer) => (
              <button
                key={layer.value}
                onClick={() => handleLayerChange(layer.value)}
                className={`px-4 py-2 rounded-md font-medium transition-all ${
                  selectedLayer === layer.value
                    ? 'bg-primary-500 text-white'
                    : 'bg-slate-700 text-slate-300 hover:bg-slate-600'
                }`}
                aria-label={`Switch to ${layer.label} layer`}
                aria-pressed={selectedLayer === layer.value}
              >
                {layer.label}
              </button>
            ))}
          </div>

          {/* Modified Keys Count */}
          <div className="text-sm text-slate-400 pt-2 border-t border-slate-700">
            Modified keys in this layer:{' '}
            <span className="font-semibold text-slate-300">
              {modifiedKeysCount}
            </span>
          </div>
        </div>
      </Card>
    </div>
  );
};

export default ConfigPage;
