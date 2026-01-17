import React from 'react';
import { Dropdown } from './Dropdown';
import { cn } from '@/utils/cn';

/**
 * Represents a keyboard layer
 */
export interface Layer {
  /** Unique layer identifier (e.g., "base", "nav", "num") */
  id: string;
  /** Display name for the layer (e.g., "Base Layer", "Navigation") */
  name: string;
}

export interface LayerSelectorProps {
  /** Available layers to select from */
  layers: Layer[];
  /** Currently selected layer ID */
  selectedLayer: string;
  /** Callback when layer selection changes */
  onLayerChange: (layerId: string) => void;
  /** Whether the component is in a loading state */
  loading?: boolean;
  /** Whether the component is disabled */
  disabled?: boolean;
  /** CSS class name for styling */
  className?: string;
}

/**
 * LayerSelector component for switching between keyboard layers in the visual editor.
 *
 * Features:
 * - Dropdown to select from available layers
 * - Loading state while layers are being fetched
 * - Empty state when no layers are available
 * - Accessible with proper ARIA labels
 * - Controlled component pattern (value/onChange props)
 *
 * @example
 * ```tsx
 * const layers = [
 *   { id: 'base', name: 'Base Layer' },
 *   { id: 'nav', name: 'Navigation Layer' },
 * ];
 *
 * <LayerSelector
 *   layers={layers}
 *   selectedLayer="base"
 *   onLayerChange={setSelectedLayer}
 * />
 * ```
 */
export const LayerSelector: React.FC<LayerSelectorProps> = ({
  layers,
  selectedLayer,
  onLayerChange,
  loading = false,
  disabled = false,
  className = '',
}) => {
  // Convert layers to dropdown options
  const layerOptions = layers.map((layer) => ({
    value: layer.id,
    label: layer.name,
  }));

  return (
    <div className={cn('space-y-2', className)}>
      <label
        htmlFor="layer-selector"
        className="block text-sm font-medium text-slate-300"
      >
        Layer
      </label>

      {loading ? (
        // Loading state
        <div
          className="rounded-md border border-slate-600 bg-slate-700 px-4 py-3 text-sm text-slate-400"
          role="status"
          aria-live="polite"
        >
          <div className="flex items-center gap-2">
            <svg
              className="animate-spin h-4 w-4 text-primary-500"
              xmlns="http://www.w3.org/2000/svg"
              fill="none"
              viewBox="0 0 24 24"
            >
              <circle
                className="opacity-25"
                cx="12"
                cy="12"
                r="10"
                stroke="currentColor"
                strokeWidth="4"
              />
              <path
                className="opacity-75"
                fill="currentColor"
                d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
              />
            </svg>
            Loading layers...
          </div>
        </div>
      ) : layers.length === 0 ? (
        // Empty state
        <div
          className="rounded-md border border-slate-600 bg-slate-800/50 px-4 py-3 text-sm text-slate-400"
          role="alert"
        >
          No layers available. Create layers in your profile configuration.
        </div>
      ) : (
        // Layer dropdown
        <Dropdown
          options={layerOptions}
          value={selectedLayer}
          onChange={onLayerChange}
          aria-label="Select keyboard layer"
          placeholder="Select a layer..."
          disabled={disabled || loading}
        />
      )}

      {/* Help text */}
      {!loading && layers.length > 0 && (
        <p className="text-xs text-slate-400">
          Switch between layers to configure different key mappings.
        </p>
      )}
    </div>
  );
};

LayerSelector.displayName = 'LayerSelector';
