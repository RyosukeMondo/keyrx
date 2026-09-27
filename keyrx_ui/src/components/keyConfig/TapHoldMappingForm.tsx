import React from 'react';
import { X, Radio } from 'lucide-react';
import { KeyPalette } from '../KeyPalette';

export interface TapHoldMappingFormProps {
  tapAction: string;
  onTapActionChange: (value: string) => void;
  isListening: boolean;
  listeningFor: 'tap' | 'hold' | null;
  onStartListening: (target: 'tap' | 'hold') => void;
  tapError?: string;
  holdAction: string;
  onHoldActionChange: (value: string) => void;
  holdError?: string;
  threshold: number;
  onThresholdChange: (value: number) => void;
  thresholdError?: string;
}

/** Render tap/hold mapping form */
export function TapHoldMappingForm({
  tapAction,
  onTapActionChange,
  isListening,
  listeningFor,
  onStartListening,
  tapError,
  holdAction,
  onHoldActionChange,
  holdError,
  threshold,
  onThresholdChange,
  thresholdError,
}: TapHoldMappingFormProps) {
  return (
    <>
      {/* Tap Action */}
      <div>
        <div className="flex items-center justify-between mb-3">
          <label className="text-sm font-medium text-slate-300">
            Tap Action
          </label>
          <div className="flex items-center gap-2">
            {tapAction && (
              <>
                <div className="px-3 py-1 bg-green-500/20 border border-green-500 rounded">
                  <span className="text-sm font-bold text-green-300 font-mono">
                    {tapAction}
                  </span>
                </div>
                <button
                  onClick={() => onTapActionChange('')}
                  className="p-1 text-slate-400 hover:text-red-400"
                  title="Clear selection"
                >
                  <X className="w-4 h-4" />
                </button>
              </>
            )}
            <button
              onClick={() => onStartListening('tap')}
              disabled={isListening}
              className={`px-3 py-1 rounded text-xs font-medium flex items-center gap-1.5 ${
                isListening && listeningFor === 'tap'
                  ? 'bg-green-500 text-white animate-pulse'
                  : 'bg-slate-600 text-slate-300 hover:bg-slate-500'
              }`}
            >
              <Radio className="w-3.5 h-3.5" />
              {isListening && listeningFor === 'tap'
                ? 'Listening...'
                : 'Listen'}
            </button>
          </div>
        </div>
        <p className="text-xs text-slate-400 mb-3">Click a key to select it</p>
        <div className="border border-slate-600 rounded-lg overflow-y-auto max-h-72">
          <KeyPalette
            compact
            onKeySelect={(key) => onTapActionChange(key.id)}
            selectedKey={
              tapAction
                ? { id: tapAction, label: tapAction, category: 'basic' }
                : null
            }
          />
        </div>
        {tapError && <p className="text-xs text-red-400 mt-2">{tapError}</p>}
      </div>

      {/* Hold Action */}
      <div>
        <div className="flex items-center justify-between mb-3">
          <div>
            <label className="text-sm font-medium text-slate-300">
              Hold Action (modifier)
            </label>
            <p className="text-xs text-slate-400 mt-1">Select modifier 0-255</p>
          </div>
          {holdAction && (
            <div className="flex items-center gap-2">
              <div className="px-3 py-1 bg-red-500/20 border border-red-500 rounded">
                <span className="text-sm font-bold text-red-300 font-mono">
                  {holdAction}
                </span>
              </div>
              <button
                onClick={() => onHoldActionChange('')}
                className="p-1 text-slate-400 hover:text-red-400"
                title="Clear selection"
              >
                <X className="w-4 h-4" />
              </button>
            </div>
          )}
        </div>
        <div className="border border-slate-600 rounded-lg p-4 bg-slate-900">
          <input
            type="number"
            min="0"
            max="255"
            value={holdAction ? parseInt(holdAction.replace('MD_', ''), 16) : 0}
            onChange={(e) => {
              const val = Math.max(
                0,
                Math.min(255, parseInt(e.target.value) || 0)
              );
              const hex = val.toString(16).toUpperCase().padStart(2, '0');
              onHoldActionChange(`MD_${hex}`);
            }}
            className="w-full px-4 py-2 bg-slate-800 border border-slate-600 rounded-md text-slate-100 focus:outline-none focus:ring-2 focus:ring-primary-500"
            placeholder="Enter value 0-255"
          />
          <div className="mt-3">
            <input
              type="range"
              min="0"
              max="255"
              value={
                holdAction ? parseInt(holdAction.replace('MD_', ''), 16) : 0
              }
              onChange={(e) => {
                const val = parseInt(e.target.value);
                const hex = val.toString(16).toUpperCase().padStart(2, '0');
                onHoldActionChange(`MD_${hex}`);
              }}
              className="w-full"
            />
            <div className="flex justify-between text-xs text-slate-500 mt-1">
              <span>0 (MD_00)</span>
              <span>255 (MD_FF)</span>
            </div>
          </div>
        </div>
        {holdError && <p className="text-xs text-red-400 mt-2">{holdError}</p>}
      </div>

      {/* Threshold */}
      <div>
        <label className="block text-sm font-medium text-slate-300 mb-2">
          Hold Threshold (ms): {threshold}
        </label>
        <input
          type="range"
          min="50"
          max="500"
          step="10"
          value={threshold}
          onChange={(e) => onThresholdChange(parseInt(e.target.value))}
          className="w-full"
        />
        <div className="flex justify-between text-xs text-slate-500 mt-1">
          <span>50ms (fast)</span>
          <span>500ms (slow)</span>
        </div>
        {thresholdError && (
          <p className="text-xs text-red-400 mt-2">{thresholdError}</p>
        )}
      </div>
    </>
  );
}
