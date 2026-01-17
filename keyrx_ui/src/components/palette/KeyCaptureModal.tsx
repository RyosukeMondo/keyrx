import React from 'react';
import { Check, Keyboard } from 'lucide-react';
import type { PaletteKey } from '../KeyPalette';

interface KeyCaptureModalProps {
  isCapturingKey: boolean;
  capturedKey: PaletteKey | null;
  onCancel: () => void;
  onConfirm: () => void;
}

/**
 * Modal for capturing physical key presses
 */
export const KeyCaptureModal: React.FC<KeyCaptureModalProps> = ({
  isCapturingKey,
  capturedKey,
  onCancel,
  onConfirm,
}) => {
  if (!isCapturingKey) return null;

  return (
    <div
      className="fixed inset-0 bg-black/70 backdrop-blur-sm flex items-center justify-center z-50"
      onClick={onCancel}
    >
      <div
        className="bg-slate-800 border-2 border-primary-500 rounded-xl p-8 shadow-2xl max-w-md w-full mx-4"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Waiting for key press */}
        {!capturedKey ? (
          <div className="text-center">
            <div className="mb-6">
              <Keyboard className="w-16 h-16 text-primary-400 mx-auto animate-pulse" />
            </div>
            <h3 className="text-2xl font-bold text-white mb-3">
              Press any key...
            </h3>
            <p className="text-slate-400 mb-6">
              Press the physical key you want to select.
            </p>
            <div className="text-xs text-slate-500">
              Press{' '}
              <kbd className="px-2 py-1 bg-slate-700 rounded border border-slate-600 font-mono">
                Esc
              </kbd>{' '}
              to cancel
            </div>
          </div>
        ) : (
          /* Key captured - show confirmation */
          <div className="text-center">
            <div className="mb-6">
              <div className="inline-block p-4 bg-green-500/20 rounded-full">
                <Check className="w-12 h-12 text-green-400" />
              </div>
            </div>
            <h3 className="text-2xl font-bold text-white mb-3">
              Key Captured!
            </h3>
            <div className="mb-6 p-4 bg-slate-900/50 rounded-lg border border-slate-700">
              <div className="text-3xl font-bold text-white font-mono mb-2">
                {capturedKey.label}
              </div>
              <div className="text-sm text-slate-400 font-mono mb-1">
                {capturedKey.id}
              </div>
              <div className="text-xs text-slate-500">
                {capturedKey.description}
              </div>
            </div>
            <p className="text-slate-400 mb-6">Use this key for mapping?</p>
            <div className="flex gap-3">
              <button
                onClick={onCancel}
                className="flex-1 px-4 py-3 bg-slate-700 hover:bg-slate-600 text-slate-300 rounded-lg transition-colors font-medium"
              >
                Cancel
              </button>
              <button
                onClick={onConfirm}
                className="flex-1 px-4 py-3 bg-primary-500 hover:bg-primary-600 text-white rounded-lg transition-colors font-medium"
              >
                Use This Key
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
