import React, { useState, useCallback } from 'react';
import { Card } from '@/components/Card';
import { Dropdown } from '@/components/Dropdown';
import { KeyboardVisualizer } from '@/components/KeyboardVisualizer';
import { KeyMapping } from '@/components/KeyButton';
import { LoadingSkeleton } from '@/components/LoadingSkeleton';

interface ConfigPageProps {
  profileName?: string;
}

export const ConfigPage: React.FC<ConfigPageProps> = ({
  profileName = 'Default',
}) => {
  const [loading, setLoading] = useState(false);
  const [selectedLayout, setSelectedLayout] =
    useState<'ANSI_104' | 'ISO_105' | 'JIS_109' | 'HHKB' | 'NUMPAD'>('ANSI_104');
  const [selectedLayer, setSelectedLayer] = useState('base');
  const [previewMode, setPreviewMode] = useState(false);
  const [keyMappings] = useState<Map<string, KeyMapping>>(new Map());

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

  if (loading) {
    return (
      <div className="flex flex-col gap-4 md:gap-6 p-4 md:p-6 lg:p-8">
        <div className="flex flex-col lg:flex-row lg:items-center lg:justify-between gap-4">
          <div className="flex flex-col sm:flex-row sm:items-center gap-2 sm:gap-4">
            <LoadingSkeleton variant="text" width="250px" height="32px" />
            <LoadingSkeleton variant="text" width="150px" height="20px" />
          </div>
          <LoadingSkeleton variant="rectangular" width="180px" height="44px" />
        </div>

        <Card variant="default" padding="lg">
          <div className="flex flex-col gap-4">
            <div className="flex items-center justify-between pb-4 border-b border-slate-700">
              <LoadingSkeleton variant="text" width="150px" height="24px" />
              <LoadingSkeleton variant="rectangular" width="192px" height="40px" />
            </div>
            <LoadingSkeleton variant="rectangular" height="400px" />
          </div>
        </Card>

        <Card variant="default" padding="lg">
          <div className="flex flex-col gap-4">
            <LoadingSkeleton variant="text" width="100px" height="24px" />
            <div className="flex gap-2">
              <LoadingSkeleton variant="rectangular" width="80px" height="32px" />
              <LoadingSkeleton variant="rectangular" width="80px" height="32px" />
              <LoadingSkeleton variant="rectangular" width="80px" height="32px" />
            </div>
          </div>
        </Card>
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-4 md:gap-6 p-4 md:p-6 lg:p-8">
      {/* Header */}
      <div className="flex flex-col lg:flex-row lg:items-center lg:justify-between gap-4">
        <div className="flex flex-col sm:flex-row sm:items-center gap-2 sm:gap-4">
          <h1 className="text-xl md:text-2xl lg:text-3xl font-semibold text-slate-100">
            Configuration Editor
          </h1>
          <span className="hidden sm:inline text-slate-400">—</span>
          <span className="text-sm sm:text-base text-slate-300">
            Profile: {profileName}
          </span>
        </div>
        <button
          onClick={togglePreviewMode}
          className={`px-4 py-3 md:py-2 rounded-md font-medium transition-colors min-h-[44px] md:min-h-0 ${
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

          {/* Keyboard Visualizer - horizontal scroll on mobile */}
          <div className="py-4 overflow-x-auto md:overflow-x-visible">
            <KeyboardVisualizer
              layout={selectedLayout}
              keyMappings={keyMappings}
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
          <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-2 sm:gap-4">
            <h2 className="text-base sm:text-lg font-medium text-slate-100">
              Active Layer: MD_00 ({selectedLayer})
            </h2>
            <button
              className="text-sm text-primary-500 hover:text-primary-400 transition-colors self-start sm:self-auto min-h-[44px] sm:min-h-0 flex items-center"
              aria-label="Open layer list"
            >
              Layer List ▼
            </button>
          </div>

          {/* Layer Buttons - responsive grid on mobile */}
          <div className="grid grid-cols-2 sm:flex sm:flex-wrap gap-2" role="group" aria-label="Layer selection">
            {layerOptions.map((layer) => (
              <button
                key={layer.value}
                onClick={() => handleLayerChange(layer.value)}
                className={`px-4 py-3 sm:py-2 rounded-md font-medium transition-all min-h-[44px] sm:min-h-0 focus:outline focus:outline-2 focus:outline-primary-500 focus:outline-offset-2 ${
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
