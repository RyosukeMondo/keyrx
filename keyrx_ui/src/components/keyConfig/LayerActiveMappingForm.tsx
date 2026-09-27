import React from 'react';
import { KeyPalette } from '../KeyPalette';

export interface LayerActiveMappingFormProps {
  targetLayer: string;
  onTargetLayerChange: (value: string) => void;
  error?: string;
}

/** Render layer active mapping form */
export function LayerActiveMappingForm({
  targetLayer,
  onTargetLayerChange,
  error,
}: LayerActiveMappingFormProps) {
  return (
    <div>
      <div className="flex items-center justify-between mb-2">
        {targetLayer && (
          <div className="px-4 py-2 bg-yellow-500/20 border border-yellow-500 rounded-lg">
            <span className="text-xl font-bold text-yellow-300 font-mono">
              {targetLayer}
            </span>
          </div>
        )}
      </div>
      <p className="text-xs text-slate-400 mb-2">
        Select a layer to activate (MO, TO, TG, OSL)
      </p>
      <div className="border border-slate-600 rounded-lg overflow-y-auto max-h-72">
        <KeyPalette
          compact
          onKeySelect={(key) => onTargetLayerChange(key.id)}
          selectedKey={
            targetLayer
              ? {
                  id: targetLayer,
                  label: targetLayer,
                  category: 'layers',
                }
              : null
          }
        />
      </div>
      {error && <p className="text-xs text-red-400 mt-2">{error}</p>}
    </div>
  );
}
