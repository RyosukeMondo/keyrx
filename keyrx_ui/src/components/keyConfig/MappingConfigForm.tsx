import React, { useState, useCallback, useEffect } from 'react';
import { X, Radio, Keyboard, ListOrdered } from 'lucide-react';
import type { KeyMapping } from '@/types';
import { KeyPalette } from '../KeyPalette';
import { SVGKeyboard, type SVGKey } from '../SVGKeyboard';

/**
 * MappingConfigForm Component
 *
 * Dynamic form fields based on mapping type.
 * Extracted from KeyConfigModal and KeyConfigPanel for DRY.
 *
 * @param mappingType - The type of key mapping (simple, modifier, lock, tap_hold, layer_active)
 * @param currentConfig - Partial mapping configuration
 * @param onChange - Callback when configuration changes
 * @param onValidate - Callback for validation
 * @param layoutKeys - Optional SVG keyboard layout keys
 * @param enableKeyboardView - Whether to enable keyboard/palette toggle (default: true)
 */

export type MappingType =
  | 'simple'
  | 'modifier'
  | 'lock'
  | 'tap_hold'
  | 'layer_active';

export interface MappingConfig extends Partial<KeyMapping> {
  tapAction?: string;
  holdAction?: string;
  threshold?: number;
  modifierKey?: string;
  lockKey?: string;
  targetLayer?: string;
}

export interface ValidationResult {
  valid: boolean;
  errors: Record<string, string>;
}

export interface MappingConfigFormProps {
  mappingType: MappingType;
  currentConfig?: MappingConfig;
  onChange: (config: MappingConfig) => void;
  onValidate?: (config: MappingConfig) => ValidationResult;
  layoutKeys?: SVGKey[];
  enableKeyboardView?: boolean;
}

export function MappingConfigForm({
  mappingType,
  currentConfig = {},
  onChange,
  onValidate,
  layoutKeys = [],
  enableKeyboardView = true,
}: MappingConfigFormProps) {
  // Form state
  const [tapAction, setTapAction] = useState(currentConfig.tapAction || '');
  const [holdAction, setHoldAction] = useState(currentConfig.holdAction || '');
  const [threshold, setThreshold] = useState(currentConfig.threshold || 200);
  const [modifierKey, setModifierKey] = useState(
    currentConfig.modifierKey || ''
  );
  const [lockKey, setLockKey] = useState(currentConfig.lockKey || '');
  const [targetLayer, setTargetLayer] = useState(
    currentConfig.targetLayer || ''
  );

  // UI state
  const [useKeyboard, setUseKeyboard] = useState(false);
  const [isListening, setIsListening] = useState(false);
  const [listeningFor, setListeningFor] = useState<'tap' | 'hold' | null>(null);
  const [errors, setErrors] = useState<Record<string, string>>({});

  // Update form when currentConfig changes
  // This is needed to sync form state when parent changes the config (e.g., editing existing mapping)
  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect
    setTapAction(currentConfig.tapAction || '');
    // eslint-disable-next-line react-hooks/set-state-in-effect
    setHoldAction(currentConfig.holdAction || '');
    // eslint-disable-next-line react-hooks/set-state-in-effect
    setThreshold(currentConfig.threshold || 200);
    // eslint-disable-next-line react-hooks/set-state-in-effect
    setModifierKey(currentConfig.modifierKey || '');
    // eslint-disable-next-line react-hooks/set-state-in-effect
    setLockKey(currentConfig.lockKey || '');
    // eslint-disable-next-line react-hooks/set-state-in-effect
    setTargetLayer(currentConfig.targetLayer || '');
  }, [currentConfig]);

  // Build current config object
  const buildConfig = useCallback((): MappingConfig => {
    switch (mappingType) {
      case 'simple':
        return { type: 'simple', tapAction };
      case 'modifier':
        return { type: 'modifier', modifierKey };
      case 'lock':
        return { type: 'lock', lockKey };
      case 'tap_hold':
        return { type: 'tap_hold', tapAction, holdAction, threshold };
      case 'layer_active':
        return { type: 'layer_active', targetLayer };
      default:
        return { type: 'simple', tapAction };
    }
  }, [
    mappingType,
    tapAction,
    holdAction,
    threshold,
    modifierKey,
    lockKey,
    targetLayer,
  ]);

  // Validate and notify onChange
  const notifyChange = useCallback(() => {
    const config = buildConfig();
    if (onValidate) {
      const result = onValidate(config);
      setErrors(result.errors);
    }
    onChange(config);
  }, [buildConfig, onChange, onValidate]);

  // Key listening
  const handleKeyCapture = useCallback(
    (event: KeyboardEvent) => {
      if (!isListening) return;

      event.preventDefault();
      event.stopPropagation();

      if (event.key === 'Escape') {
        setIsListening(false);
        setListeningFor(null);
        return;
      }

      let vkCode = event.code;
      if (vkCode.startsWith('Key')) {
        vkCode = 'VK_' + vkCode.substring(3);
      } else if (vkCode.startsWith('Digit')) {
        vkCode = 'VK_' + vkCode.substring(5);
      } else {
        vkCode = 'VK_' + vkCode.toUpperCase();
      }

      if (listeningFor === 'tap') {
        setTapAction(vkCode);
      } else if (listeningFor === 'hold') {
        setHoldAction(vkCode);
      }

      setIsListening(false);
      setListeningFor(null);
    },
    [isListening, listeningFor]
  );

  useEffect(() => {
    if (isListening) {
      document.addEventListener('keydown', handleKeyCapture);
      return () => document.removeEventListener('keydown', handleKeyCapture);
    }
  }, [isListening, handleKeyCapture]);

  const startListening = (target: 'tap' | 'hold') => {
    setIsListening(true);
    setListeningFor(target);
  };

  const stopListening = () => {
    setIsListening(false);
    setListeningFor(null);
  };

  // Update handlers
  const handleTapActionChange = (value: string) => {
    setTapAction(value);
    setTimeout(notifyChange, 0);
  };

  const handleHoldActionChange = (value: string) => {
    setHoldAction(value);
    setTimeout(notifyChange, 0);
  };

  const handleThresholdChange = (value: number) => {
    setThreshold(value);
    setTimeout(notifyChange, 0);
  };

  const handleModifierKeyChange = (value: string) => {
    setModifierKey(value);
    setTimeout(notifyChange, 0);
  };

  const handleLockKeyChange = (value: string) => {
    setLockKey(value);
    setTimeout(notifyChange, 0);
  };

  const handleTargetLayerChange = (value: string) => {
    setTargetLayer(value);
    setTimeout(notifyChange, 0);
  };

  // Render simple mapping form
  const renderSimpleForm = () => (
    <div>
      {/* View toggle */}
      {enableKeyboardView && (
        <div className="flex items-center justify-between mb-3">
          <label className="text-sm font-medium text-slate-300">
            Select Key
          </label>
          <div className="flex gap-2">
            <button
              onClick={() => setUseKeyboard(true)}
              className={`px-3 py-1.5 rounded-md text-xs font-medium ${
                useKeyboard
                  ? 'bg-primary-500 text-white'
                  : 'bg-slate-700 text-slate-400 hover:bg-slate-600'
              }`}
            >
              <Keyboard className="w-4 h-4 inline-block mr-1" />
              Keyboard
            </button>
            <button
              onClick={() => setUseKeyboard(false)}
              className={`px-3 py-1.5 rounded-md text-xs font-medium ${
                !useKeyboard
                  ? 'bg-primary-500 text-white'
                  : 'bg-slate-700 text-slate-400 hover:bg-slate-600'
              }`}
            >
              <ListOrdered className="w-4 h-4 inline-block mr-1" />
              List
            </button>
          </div>
        </div>
      )}

      {/* Selected key display */}
      <div className="flex items-center justify-between mb-2">
        <div className="flex items-center gap-2">
          {tapAction && (
            <>
              <div className="px-4 py-2 bg-primary-500/20 border border-primary-500 rounded-lg">
                <span className="text-xl font-bold text-primary-300 font-mono">
                  {tapAction}
                </span>
              </div>
              <button
                onClick={() => handleTapActionChange('')}
                className="p-1 text-slate-400 hover:text-red-400"
                title="Clear selection"
              >
                <X className="w-5 h-5" />
              </button>
            </>
          )}
        </div>
        <button
          onClick={() => startListening('tap')}
          disabled={isListening}
          className={`px-3 py-1.5 rounded text-xs font-medium flex items-center gap-1.5 ${
            isListening && listeningFor === 'tap'
              ? 'bg-green-500 text-white animate-pulse'
              : 'bg-slate-600 text-slate-300 hover:bg-slate-500'
          }`}
          title="Press any key to capture"
        >
          <Radio className="w-4 h-4" />
          {isListening && listeningFor === 'tap'
            ? 'Listening...'
            : 'Listen for Key'}
        </button>
      </div>
      <p className="text-xs text-slate-400 mb-2">
        Click a key below or use Listen button
      </p>

      {/* Key selection */}
      {useKeyboard && layoutKeys.length > 0 ? (
        <div className="border border-slate-600 rounded-lg overflow-auto max-h-96 bg-slate-900">
          <SVGKeyboard
            keys={layoutKeys}
            keyMappings={new Map()}
            onKeyClick={handleTapActionChange}
            className="w-full"
          />
        </div>
      ) : (
        <div className="border border-slate-600 rounded-lg overflow-y-auto max-h-72">
          <KeyPalette
            compact
            onKeySelect={(key) => handleTapActionChange(key.id)}
            selectedKey={
              tapAction
                ? { id: tapAction, label: tapAction, category: 'basic' }
                : null
            }
          />
        </div>
      )}

      {errors.tapAction && (
        <p className="text-xs text-red-400 mt-2">{errors.tapAction}</p>
      )}
    </div>
  );

  // Render modifier mapping form
  const renderModifierForm = () => (
    <div>
      {enableKeyboardView && (
        <div className="flex items-center justify-between mb-3">
          <label className="text-sm font-medium text-slate-300">
            Select Key
          </label>
          <div className="flex gap-2">
            <button
              onClick={() => setUseKeyboard(true)}
              className={`px-3 py-1.5 rounded-md text-xs font-medium ${
                useKeyboard
                  ? 'bg-primary-500 text-white'
                  : 'bg-slate-700 text-slate-400 hover:bg-slate-600'
              }`}
            >
              <Keyboard className="w-4 h-4 inline-block mr-1" />
              Keyboard
            </button>
            <button
              onClick={() => setUseKeyboard(false)}
              className={`px-3 py-1.5 rounded-md text-xs font-medium ${
                !useKeyboard
                  ? 'bg-primary-500 text-white'
                  : 'bg-slate-700 text-slate-400 hover:bg-slate-600'
              }`}
            >
              <ListOrdered className="w-4 h-4 inline-block mr-1" />
              List
            </button>
          </div>
        </div>
      )}

      <div className="flex items-center justify-between mb-2">
        <div className="flex items-center gap-2">
          {modifierKey && (
            <>
              <div className="px-4 py-2 bg-cyan-500/20 border border-cyan-500 rounded-lg">
                <span className="text-xl font-bold text-cyan-300 font-mono">
                  {modifierKey}
                </span>
              </div>
              <button
                onClick={() => handleModifierKeyChange('')}
                className="p-1 text-slate-400 hover:text-red-400"
                title="Clear selection"
              >
                <X className="w-5 h-5" />
              </button>
            </>
          )}
        </div>
      </div>
      <p className="text-xs text-slate-400 mb-2">
        Select a modifier key (Ctrl, Shift, Alt, etc.)
      </p>

      {useKeyboard && layoutKeys.length > 0 ? (
        <div className="border border-slate-600 rounded-lg overflow-auto max-h-96 bg-slate-900">
          <SVGKeyboard
            keys={layoutKeys}
            keyMappings={new Map()}
            onKeyClick={handleModifierKeyChange}
            className="w-full"
          />
        </div>
      ) : (
        <div className="border border-slate-600 rounded-lg overflow-y-auto max-h-72">
          <KeyPalette
            compact
            onKeySelect={(key) => handleModifierKeyChange(key.id)}
            selectedKey={
              modifierKey
                ? {
                    id: modifierKey,
                    label: modifierKey,
                    category: 'modifiers',
                  }
                : null
            }
          />
        </div>
      )}

      {errors.modifierKey && (
        <p className="text-xs text-red-400 mt-2">{errors.modifierKey}</p>
      )}
    </div>
  );

  // Render lock mapping form
  const renderLockForm = () => (
    <div>
      {enableKeyboardView && (
        <div className="flex items-center justify-between mb-3">
          <label className="text-sm font-medium text-slate-300">
            Select Key
          </label>
          <div className="flex gap-2">
            <button
              onClick={() => setUseKeyboard(true)}
              className={`px-3 py-1.5 rounded-md text-xs font-medium ${
                useKeyboard
                  ? 'bg-primary-500 text-white'
                  : 'bg-slate-700 text-slate-400 hover:bg-slate-600'
              }`}
            >
              <Keyboard className="w-4 h-4 inline-block mr-1" />
              Keyboard
            </button>
            <button
              onClick={() => setUseKeyboard(false)}
              className={`px-3 py-1.5 rounded-md text-xs font-medium ${
                !useKeyboard
                  ? 'bg-primary-500 text-white'
                  : 'bg-slate-700 text-slate-400 hover:bg-slate-600'
              }`}
            >
              <ListOrdered className="w-4 h-4 inline-block mr-1" />
              List
            </button>
          </div>
        </div>
      )}

      <div className="flex items-center justify-between mb-2">
        <div className="flex items-center gap-2">
          {lockKey && (
            <>
              <div className="px-4 py-2 bg-purple-500/20 border border-purple-500 rounded-lg">
                <span className="text-xl font-bold text-purple-300 font-mono">
                  {lockKey}
                </span>
              </div>
              <button
                onClick={() => handleLockKeyChange('')}
                className="p-1 text-slate-400 hover:text-red-400"
                title="Clear selection"
              >
                <X className="w-5 h-5" />
              </button>
            </>
          )}
        </div>
      </div>
      <p className="text-xs text-slate-400 mb-2">
        Select a lock key (CapsLock, NumLock, etc.)
      </p>

      {useKeyboard && layoutKeys.length > 0 ? (
        <div className="border border-slate-600 rounded-lg overflow-auto max-h-96 bg-slate-900">
          <SVGKeyboard
            keys={layoutKeys}
            keyMappings={new Map()}
            onKeyClick={handleLockKeyChange}
            className="w-full"
          />
        </div>
      ) : (
        <div className="border border-slate-600 rounded-lg overflow-y-auto max-h-72">
          <KeyPalette
            compact
            onKeySelect={(key) => handleLockKeyChange(key.id)}
            selectedKey={
              lockKey
                ? { id: lockKey, label: lockKey, category: 'special' }
                : null
            }
          />
        </div>
      )}

      {errors.lockKey && (
        <p className="text-xs text-red-400 mt-2">{errors.lockKey}</p>
      )}
    </div>
  );

  // Render tap/hold mapping form
  const renderTapHoldForm = () => (
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
                  onClick={() => handleTapActionChange('')}
                  className="p-1 text-slate-400 hover:text-red-400"
                  title="Clear selection"
                >
                  <X className="w-4 h-4" />
                </button>
              </>
            )}
            <button
              onClick={() => startListening('tap')}
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
            onKeySelect={(key) => handleTapActionChange(key.id)}
            selectedKey={
              tapAction
                ? { id: tapAction, label: tapAction, category: 'basic' }
                : null
            }
          />
        </div>
        {errors.tapAction && (
          <p className="text-xs text-red-400 mt-2">{errors.tapAction}</p>
        )}
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
                onClick={() => handleHoldActionChange('')}
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
              handleHoldActionChange(`MD_${hex}`);
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
                handleHoldActionChange(`MD_${hex}`);
              }}
              className="w-full"
            />
            <div className="flex justify-between text-xs text-slate-500 mt-1">
              <span>0 (MD_00)</span>
              <span>255 (MD_FF)</span>
            </div>
          </div>
        </div>
        {errors.holdAction && (
          <p className="text-xs text-red-400 mt-2">{errors.holdAction}</p>
        )}
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
          onChange={(e) => handleThresholdChange(parseInt(e.target.value))}
          className="w-full"
        />
        <div className="flex justify-between text-xs text-slate-500 mt-1">
          <span>50ms (fast)</span>
          <span>500ms (slow)</span>
        </div>
        {errors.threshold && (
          <p className="text-xs text-red-400 mt-2">{errors.threshold}</p>
        )}
      </div>
    </>
  );

  // Render layer active mapping form
  const renderLayerActiveForm = () => (
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
          onKeySelect={(key) => handleTargetLayerChange(key.id)}
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
      {errors.targetLayer && (
        <p className="text-xs text-red-400 mt-2">{errors.targetLayer}</p>
      )}
    </div>
  );

  return (
    <div className="space-y-6">
      {mappingType === 'simple' && renderSimpleForm()}
      {mappingType === 'modifier' && renderModifierForm()}
      {mappingType === 'lock' && renderLockForm()}
      {mappingType === 'tap_hold' && renderTapHoldForm()}
      {mappingType === 'layer_active' && renderLayerActiveForm()}

      {/* Listening Overlay */}
      {isListening && (
        <div className="fixed inset-0 bg-black/80 backdrop-blur-sm z-[100] flex items-center justify-center">
          <div className="bg-slate-800 border border-primary-500 rounded-lg p-8 max-w-md text-center space-y-4 shadow-2xl">
            <Radio className="w-16 h-16 text-primary-400 mx-auto animate-pulse" />
            <h3 className="text-2xl font-bold text-slate-100">
              Listening for key press...
            </h3>
            <p className="text-slate-300">
              Press any key on your keyboard to capture it
            </p>
            <p className="text-sm text-slate-400">
              Press{' '}
              <kbd className="px-2 py-1 bg-slate-700 rounded text-slate-200">
                Escape
              </kbd>{' '}
              to cancel
            </p>
            <button
              onClick={stopListening}
              className="px-6 py-2 bg-slate-700 text-slate-200 rounded-md hover:bg-slate-600 transition-colors"
            >
              Cancel
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
