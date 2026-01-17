import React from 'react';
import { Check, AlertCircle, HelpCircle } from 'lucide-react';
import type { ValidationResult } from '@/utils/paletteHelpers.tsx';

interface CustomKeycodeInputProps {
  customKeycode: string;
  customValidation: ValidationResult;
  onKeycodeChange: (value: string) => void;
  onApplyKeycode: () => void;
}

/**
 * Input panel for custom QMK-style keycodes
 */
export const CustomKeycodeInput: React.FC<CustomKeycodeInputProps> = ({
  customKeycode,
  customValidation,
  onKeycodeChange,
  onApplyKeycode,
}) => {
  return (
    <div className="p-6 space-y-6">
      <div>
        <h4 className="text-lg font-semibold text-slate-200 mb-2">
          Custom Keycode
        </h4>
        <p className="text-sm text-slate-400 mb-4">
          Enter any valid QMK-style keycode for advanced customization.
        </p>
      </div>

      {/* Input field with validation */}
      <div className="space-y-3">
        <div className="relative">
          <input
            type="text"
            value={customKeycode}
            onChange={(e) => onKeycodeChange(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter' && customValidation.valid) {
                onApplyKeycode();
              }
            }}
            placeholder="e.g., KC_A, LCTL(KC_C), MO(1), LT(2,KC_SPC)"
            className={`
              w-full px-4 py-3 pr-10
              bg-slate-800 border-2 rounded-lg
              text-slate-100 font-mono text-base
              placeholder-slate-500
              focus:outline-none focus:ring-2
              transition-colors
              ${
                customValidation.valid
                  ? 'border-green-500 focus:border-green-400 focus:ring-green-500/50'
                  : customKeycode && !customValidation.valid
                    ? 'border-red-500 focus:border-red-400 focus:ring-red-500/50'
                    : 'border-slate-700 focus:border-primary-500 focus:ring-primary-500/50'
              }
            `}
          />
          {/* Validation icon */}
          <div className="absolute right-3 top-1/2 -translate-y-1/2">
            {customValidation.valid ? (
              <Check className="w-5 h-5 text-green-400" />
            ) : customKeycode && !customValidation.valid ? (
              <AlertCircle className="w-5 h-5 text-red-400" />
            ) : null}
          </div>
        </div>

        {/* Validation message */}
        {customKeycode && (
          <div
            className={`text-sm ${
              customValidation.valid ? 'text-green-400' : 'text-red-400'
            }`}
          >
            {customValidation.valid ? (
              <div className="flex items-start gap-2">
                <Check className="w-4 h-4 mt-0.5 flex-shrink-0" />
                <div>
                  <p className="font-medium">Valid keycode</p>
                  <p className="text-slate-400 text-xs mt-0.5">
                    Will be mapped as:{' '}
                    <span className="font-mono">{customValidation.label}</span>
                  </p>
                </div>
              </div>
            ) : (
              <div className="flex items-start gap-2">
                <AlertCircle className="w-4 h-4 mt-0.5 flex-shrink-0" />
                <p>{customValidation.error}</p>
              </div>
            )}
          </div>
        )}

        {/* Apply button */}
        <button
          onClick={onApplyKeycode}
          disabled={!customValidation.valid}
          className={`
            w-full px-4 py-3 rounded-lg font-medium
            transition-all
            ${
              customValidation.valid
                ? 'bg-primary-500 hover:bg-primary-600 text-white shadow-lg hover:shadow-xl'
                : 'bg-slate-700 text-slate-500 cursor-not-allowed'
            }
          `}
        >
          {customValidation.valid
            ? 'Apply Keycode'
            : 'Enter a valid keycode'}
        </button>
      </div>

      {/* Help section */}
      <div className="pt-6 border-t border-slate-700">
        <div className="flex items-start gap-2 mb-3">
          <HelpCircle className="w-4 h-4 text-slate-400 mt-0.5 flex-shrink-0" />
          <h5 className="text-sm font-semibold text-slate-300">
            Supported Syntax
          </h5>
        </div>
        <div className="space-y-3 text-sm text-slate-400">
          <div>
            <p className="font-mono text-xs text-primary-400 mb-1">
              Simple Keys
            </p>
            <p className="text-xs">
              <span className="font-mono text-slate-300">A</span>,{' '}
              <span className="font-mono text-slate-300">KC_A</span>,{' '}
              <span className="font-mono text-slate-300">KC_ENTER</span>
            </p>
          </div>
          <div>
            <p className="font-mono text-xs text-primary-400 mb-1">
              Modifier Combinations
            </p>
            <p className="text-xs">
              <span className="font-mono text-slate-300">LCTL(KC_C)</span>,{' '}
              <span className="font-mono text-slate-300">LSFT(A)</span>
            </p>
          </div>
          <div>
            <p className="font-mono text-xs text-primary-400 mb-1">
              Layer Functions
            </p>
            <p className="text-xs">
              <span className="font-mono text-slate-300">MO(1)</span>,{' '}
              <span className="font-mono text-slate-300">LT(2,KC_SPC)</span>
            </p>
          </div>
          <div>
            <p className="font-mono text-xs text-primary-400 mb-1">Reference</p>
            <p className="text-xs">
              See{' '}
              <a
                href="https://docs.qmk.fm/#/keycodes"
                target="_blank"
                rel="noopener noreferrer"
                className="text-primary-400 hover:text-primary-300 underline"
              >
                QMK Keycode Reference
              </a>{' '}
              for full syntax.
            </p>
          </div>
        </div>
      </div>
    </div>
  );
};
