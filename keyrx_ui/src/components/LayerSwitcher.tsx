import React, { useState, useMemo } from 'react';

/**
 * Layer Switcher - Displays all 256 layers (Base + MD_00 to MD_FF)
 * Vertical scrollable layout with search/filter capability
 */

interface LayerSwitcherProps {
  activeLayer: string;
  availableLayers: string[];
  onLayerChange: (layer: string) => void;
}

export function LayerSwitcher({
  activeLayer,
  availableLayers,
  onLayerChange,
}: LayerSwitcherProps) {
  const [searchFilter, setSearchFilter] = useState('');

  // Generate all 256 layers: Base + MD_00 through MD_FF
  const allLayers = useMemo(() => {
    const layers = ['base'];
    for (let i = 0; i <= 255; i++) {
      layers.push(`md-${i.toString(16).padStart(2, '0')}`);
    }
    return layers;
  }, []);

  // Filter layers based on search input
  const filteredLayers = useMemo(() => {
    if (!searchFilter.trim()) {
      return allLayers;
    }
    const filter = searchFilter.toLowerCase();
    return allLayers.filter(layer => layer.toLowerCase().includes(filter));
  }, [allLayers, searchFilter]);

  const formatLayerName = (layer: string) => {
    if (layer === 'base') return 'Base Layer';
    return layer.toUpperCase().replace('MD-', 'MD_');
  };

  return (
    <div className="flex flex-col bg-gradient-to-r from-red-900/20 to-red-800/20 rounded-lg border border-red-500/30">
      {/* Header with search */}
      <div className="p-4 border-b border-red-500/30">
        <div className="flex items-center justify-between mb-3">
          <span className="text-red-400 font-bold text-sm">LAYERS (256 Total)</span>
          <span className="text-slate-400 text-xs">
            {filteredLayers.length} shown
          </span>
        </div>

        <input
          type="text"
          value={searchFilter}
          onChange={(e) => setSearchFilter(e.target.value)}
          placeholder="Search layers (e.g., 'md-0a', 'base', '1f')..."
          className="w-full px-3 py-2 bg-slate-800 border border-red-500/50 rounded text-white text-sm font-mono focus:outline-none focus:ring-2 focus:ring-red-500"
          aria-label="Search layers"
        />
      </div>

      {/* Scrollable layer list */}
      <div className="overflow-y-auto max-h-96 p-2">
        <div className="space-y-1">
          {filteredLayers.map((layer) => (
            <button
              key={layer}
              onClick={() => onLayerChange(layer)}
              className={`w-full px-4 py-2 rounded-md text-sm font-medium text-left transition-all ${
                activeLayer === layer
                  ? 'bg-red-500 text-white shadow-lg shadow-red-500/50'
                  : 'bg-slate-800 text-slate-300 border border-red-500/30 hover:bg-slate-700 hover:border-red-400'
              }`}
              aria-label={`Select ${formatLayerName(layer)}`}
              aria-pressed={activeLayer === layer}
            >
              {formatLayerName(layer)}
            </button>
          ))}
        </div>
      </div>

      {/* Footer info */}
      {searchFilter && filteredLayers.length === 0 && (
        <div className="p-4 text-center text-slate-400 text-sm">
          No layers match your search
        </div>
      )}
    </div>
  );
}
