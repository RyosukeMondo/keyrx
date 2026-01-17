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
  availableLayers: _availableLayers,
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
    return allLayers.filter((layer) => layer.toLowerCase().includes(filter));
  }, [allLayers, searchFilter]);

  const formatLayerName = (layer: string) => {
    if (layer === 'base') return 'Base';
    return layer.toUpperCase().replace('MD-', 'MD_');
  };

  return (
    <div className="w-24 flex flex-col bg-slate-800/50 rounded-lg border border-slate-700/50 flex-shrink-0">
      {/* Header with search - compact for narrow width */}
      <div className="p-2 border-b border-slate-700/50">
        <div className="mb-2">
          <span className="text-slate-300 font-semibold text-xs block text-center">
            LAYERS
          </span>
          <span className="text-slate-500 text-xs block text-center">
            {filteredLayers.length}
          </span>
        </div>

        <input
          type="text"
          value={searchFilter}
          onChange={(e) => setSearchFilter(e.target.value)}
          placeholder="..."
          title="Search layers (e.g., 'md-0a', 'base', '1f')"
          className="w-full px-1 py-1 bg-slate-900/50 border border-slate-600 rounded text-white text-xs font-mono focus:outline-none focus:ring-1 focus:ring-primary-500/50"
          aria-label="Search layers"
        />
      </div>

      {/* Scrollable layer list with custom scrollbar */}
      <div className="overflow-y-auto max-h-96 p-1 scrollbar-thin scrollbar-thumb-slate-600 scrollbar-track-slate-800/50">
        <div className="space-y-1">
          {filteredLayers.map((layer) => (
            <button
              key={layer}
              onClick={() => onLayerChange(layer)}
              className={`w-full px-1 py-1 rounded text-xs font-medium text-center transition-all break-words ${
                activeLayer === layer
                  ? 'bg-primary-500/80 text-white shadow-md'
                  : 'bg-slate-700/50 text-slate-300 border border-slate-600/50 hover:bg-slate-700 hover:border-slate-500'
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
