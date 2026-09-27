import React from 'react';
import { Radio } from 'lucide-react';
import {
  SimpleMappingForm,
  ModifierMappingForm,
  LockMappingForm,
} from './SingleKeySelectorForms';
import { TapHoldMappingForm } from './TapHoldMappingForm';
import { LayerActiveMappingForm } from './LayerActiveMappingForm';
import { useMappingConfigState } from './useMappingConfigState';
import type { MappingConfigFormProps } from './mappingConfigTypes';

export type {
  MappingType,
  MappingConfig,
  ValidationResult,
  MappingConfigFormProps,
} from './mappingConfigTypes';

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
export function MappingConfigForm({
  mappingType,
  currentConfig = {},
  onChange,
  onValidate,
  layoutKeys = [],
  enableKeyboardView = true,
}: MappingConfigFormProps) {
  const {
    tapAction,
    holdAction,
    threshold,
    modifierKey,
    lockKey,
    targetLayer,
    useKeyboard,
    setUseKeyboard,
    isListening,
    listeningFor,
    errors,
    handleTapActionChange,
    handleHoldActionChange,
    handleThresholdChange,
    handleModifierKeyChange,
    handleLockKeyChange,
    handleTargetLayerChange,
    startListening,
    stopListening,
  } = useMappingConfigState({
    mappingType,
    currentConfig,
    onChange,
    onValidate,
  });

  return (
    <div className="space-y-6">
      {mappingType === 'simple' && (
        <SimpleMappingForm
          enableKeyboardView={enableKeyboardView}
          useKeyboard={useKeyboard}
          onUseKeyboardChange={setUseKeyboard}
          tapAction={tapAction}
          onTapActionChange={handleTapActionChange}
          isListening={isListening}
          listeningFor={listeningFor}
          onStartListening={startListening}
          layoutKeys={layoutKeys}
          error={errors.tapAction}
        />
      )}
      {mappingType === 'modifier' && (
        <ModifierMappingForm
          enableKeyboardView={enableKeyboardView}
          useKeyboard={useKeyboard}
          onUseKeyboardChange={setUseKeyboard}
          modifierKey={modifierKey}
          onModifierKeyChange={handleModifierKeyChange}
          layoutKeys={layoutKeys}
          error={errors.modifierKey}
        />
      )}
      {mappingType === 'lock' && (
        <LockMappingForm
          enableKeyboardView={enableKeyboardView}
          useKeyboard={useKeyboard}
          onUseKeyboardChange={setUseKeyboard}
          lockKey={lockKey}
          onLockKeyChange={handleLockKeyChange}
          layoutKeys={layoutKeys}
          error={errors.lockKey}
        />
      )}
      {mappingType === 'tap_hold' && (
        <TapHoldMappingForm
          tapAction={tapAction}
          onTapActionChange={handleTapActionChange}
          isListening={isListening}
          listeningFor={listeningFor}
          onStartListening={startListening}
          tapError={errors.tapAction}
          holdAction={holdAction}
          onHoldActionChange={handleHoldActionChange}
          holdError={errors.holdAction}
          threshold={threshold}
          onThresholdChange={handleThresholdChange}
          thresholdError={errors.threshold}
        />
      )}
      {mappingType === 'layer_active' && (
        <LayerActiveMappingForm
          targetLayer={targetLayer}
          onTargetLayerChange={handleTargetLayerChange}
          error={errors.targetLayer}
        />
      )}

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
