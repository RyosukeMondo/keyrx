import React, { useMemo, useRef, useState } from 'react';
import { t } from '@/i18n';

/**
 * Layer Switcher.
 *
 * By default it lists only the layers a profile actually uses (Base plus the
 * layers that have mappings or were opened). The full MD_00..MD_FF range
 * (256 layers) lives behind an "All layers" advanced toggle, because a
 * 257-item list is noise for nearly everyone.
 *
 * The list is a single-select listbox with a roving tabindex: one Tab stop,
 * Up/Down/Home/End move, Enter/Space selects.
 */

interface LayerSwitcherProps {
  activeLayer: string;
  /** Layers in use by the profile (the base layer is always shown). */
  availableLayers: string[];
  onLayerChange: (layer: string) => void;
}

const ALL_LAYERS: string[] = [
  'base',
  ...Array.from(
    { length: 256 },
    (_, i) => `md-${i.toString(16).padStart(2, '0')}`
  ),
];

export const formatLayerName = (layer: string) =>
  layer === 'base' ? t('layers.base') : layer.toUpperCase().replace(/-/g, '_');

/** Base first, then the used layers in order, always including the active one. */
function visibleLayers(used: string[], active: string): string[] {
  const set = new Set<string>(['base', ...used, active]);
  const rest = [...set].filter((l) => l !== 'base').sort();
  return ['base', ...rest];
}

export function LayerSwitcher({
  activeLayer,
  availableLayers,
  onLayerChange,
}: LayerSwitcherProps) {
  const [showAll, setShowAll] = useState(false);
  const [searchFilter, setSearchFilter] = useState('');
  const [focusedLayer, setFocusedLayer] = useState<string | null>(null);
  const listRef = useRef<HTMLDivElement>(null);

  const layers = useMemo(() => {
    if (!showAll) return visibleLayers(availableLayers, activeLayer);
    const filter = searchFilter.trim().toLowerCase();
    if (!filter) return ALL_LAYERS;
    return ALL_LAYERS.filter((layer) => layer.includes(filter));
  }, [showAll, availableLayers, activeLayer, searchFilter]);

  // Roving tabindex target: last focused option, else the active one, else first.
  const tabStop =
    focusedLayer && layers.includes(focusedLayer)
      ? focusedLayer
      : layers.includes(activeLayer)
        ? activeLayer
        : layers[0];

  const moveFocus = (layer: string | undefined) => {
    if (!layer) return;
    setFocusedLayer(layer);
    listRef.current
      ?.querySelector<HTMLElement>(`[data-layer="${layer}"]`)
      ?.focus();
  };

  const handleKeyDown = (e: React.KeyboardEvent, layer: string) => {
    const i = layers.indexOf(layer);
    switch (e.key) {
      case 'ArrowDown':
      case 'ArrowRight':
        e.preventDefault();
        moveFocus(layers[Math.min(i + 1, layers.length - 1)]);
        break;
      case 'ArrowUp':
      case 'ArrowLeft':
        e.preventDefault();
        moveFocus(layers[Math.max(i - 1, 0)]);
        break;
      case 'Home':
        e.preventDefault();
        moveFocus(layers[0]);
        break;
      case 'End':
        e.preventDefault();
        moveFocus(layers[layers.length - 1]);
        break;
      case 'Enter':
      case ' ':
        e.preventDefault();
        onLayerChange(layer);
        break;
    }
  };

  return (
    <div className="flex w-full flex-shrink-0 flex-row flex-wrap items-center gap-1 rounded-lg border border-slate-700/50 bg-slate-800/50 2xl:w-24 2xl:flex-col 2xl:items-stretch 2xl:gap-0">
      <div className="p-2 2xl:border-b border-slate-700/50">
        <div className="2xl:mb-2">
          <span
            id="layer-switcher-title"
            className="text-slate-300 font-semibold text-xs block text-center"
          >
            {t('layers.title')}
          </span>
          <span className="text-slate-400 text-xs block text-center">
            {layers.length}
          </span>
        </div>

        {showAll && (
          <input
            type="text"
            value={searchFilter}
            onChange={(e) => setSearchFilter(e.target.value)}
            placeholder="..."
            title={t('layers.searchTitle')}
            className="w-full px-1 py-1 bg-slate-900/50 border border-slate-600 rounded text-white text-xs font-mono focus:outline-none focus:ring-1 focus:ring-primary-500/50"
            aria-label={t('layers.search')}
          />
        )}
      </div>

      <div
        ref={listRef}
        role="listbox"
        aria-labelledby="layer-switcher-title"
        className="flex max-h-96 flex-row flex-wrap gap-1 overflow-y-auto p-1 scrollbar-thin scrollbar-thumb-slate-600 scrollbar-track-slate-800/50 2xl:block 2xl:space-y-1"
      >
        {layers.map((layer) => (
          <div
            key={layer}
            role="option"
            aria-selected={activeLayer === layer}
            data-layer={layer}
            tabIndex={layer === tabStop ? 0 : -1}
            onClick={() => onLayerChange(layer)}
            onFocus={() => setFocusedLayer(layer)}
            onKeyDown={(e) => handleKeyDown(e, layer)}
            className={`min-w-14 px-2 py-1 2xl:w-full 2xl:px-1 rounded text-xs font-medium text-center cursor-pointer transition-all break-words ${
              activeLayer === layer
                ? 'bg-primary-500 text-white shadow-md'
                : 'bg-slate-700/50 text-slate-300 border border-slate-600/50 hover:bg-slate-700 hover:border-slate-500'
            }`}
          >
            {formatLayerName(layer)}
          </div>
        ))}
      </div>

      {showAll && searchFilter && layers.length === 0 && (
        <div className="p-4 text-center text-slate-400 text-sm">
          {t('layers.noMatch')}
        </div>
      )}

      <button
        type="button"
        onClick={() => setShowAll((v) => !v)}
        aria-expanded={showAll}
        className="m-1 rounded border border-slate-600/60 px-2 py-1 text-[11px] text-slate-300 hover:bg-slate-700 2xl:px-1"
      >
        {showAll ? t('layers.fewer') : t('layers.all')}
      </button>
    </div>
  );
}
