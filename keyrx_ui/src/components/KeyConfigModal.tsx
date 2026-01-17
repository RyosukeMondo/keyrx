import React, { useState, useMemo } from 'react';
import { Modal } from './Modal';
import { Keyboard, ArrowRight } from 'lucide-react';
import type { KeyMapping } from '@/types';
import type { SVGKey } from './SVGKeyboard';
import { CurrentMappingsSummary } from './CurrentMappingsSummary';
import {
  MappingTypeSelector,
  type MappingType,
} from './keyConfig/MappingTypeSelector';
import {
  MappingConfigForm,
  type MappingConfig,
} from './keyConfig/MappingConfigForm';

/**
 * Advanced Key Configuration Modal
 * Supports: simple, modifier, lock, tap_hold, layer_active
 */

interface KeyConfigModalProps {
  isOpen: boolean;
  onClose: () => void;
  physicalKey: string;
  currentMapping?: KeyMapping;
  onSave: (mapping: KeyMapping) => void;
  activeLayer?: string;
  keyMappings?: Map<string, KeyMapping>;
  layoutKeys?: SVGKey[];
}

export function KeyConfigModal({
  isOpen,
  onClose,
  physicalKey,
  currentMapping,
  onSave,
  activeLayer = 'base',
  keyMappings = new Map(),
  layoutKeys = [],
}: KeyConfigModalProps) {
  // Determine initial mapping type
  const initialMappingType = useMemo(() => {
    if (!currentMapping) return 'simple';
    const validTypes: MappingType[] = [
      'simple',
      'modifier',
      'lock',
      'tap_hold',
      'layer_active',
    ];
    return validTypes.includes(currentMapping.type as MappingType)
      ? (currentMapping.type as MappingType)
      : 'simple';
  }, [currentMapping]);

  const [mappingType, setMappingType] =
    useState<MappingType>(initialMappingType);
  const [config, setConfig] = useState<MappingConfig>(currentMapping || {});

  const handleSave = () => {
    onSave(config as KeyMapping);
    onClose();
  };

  const getPreviewText = (): string => {
    switch (mappingType) {
      case 'simple':
        return config.tapAction
          ? `Press ${physicalKey} → Output ${config.tapAction}`
          : 'Select a target key to map to';
      case 'modifier':
        return config.modifierKey
          ? `${physicalKey} acts as ${config.modifierKey} modifier`
          : 'Select a modifier key';
      case 'lock':
        return config.lockKey
          ? `${physicalKey} toggles ${config.lockKey} lock state`
          : 'Select a lock key';
      case 'tap_hold':
        if (!config.tapAction && !config.holdAction) {
          return 'Configure tap and hold actions';
        }
        return `Quick tap: ${physicalKey} → ${
          config.tapAction || '?'
        }\nHold ${config.threshold || 200}ms: ${physicalKey} → ${
          config.holdAction || '?'
        }`;
      case 'layer_active':
        return config.targetLayer
          ? `${physicalKey} activates ${config.targetLayer} layer`
          : 'Select a target layer';
      default:
        return '';
    }
  };

  const isConfigValid = (): boolean => {
    switch (mappingType) {
      case 'simple':
        return !!config.tapAction;
      case 'modifier':
        return !!config.modifierKey;
      case 'lock':
        return !!config.lockKey;
      case 'tap_hold':
        return !!config.tapAction && !!config.holdAction;
      case 'layer_active':
        return !!config.targetLayer;
      default:
        return false;
    }
  };

  return (
    <Modal
      open={isOpen}
      onClose={onClose}
      title="Configure Key Mapping"
      size="xl"
    >
      <div className="space-y-6">
        {/* Mapping Type Selector */}
        <MappingTypeSelector
          selectedType={mappingType}
          onChange={setMappingType}
          supportedTypes={['simple', 'modifier', 'lock', 'tap_hold', 'layer_active']}
        />

        {/* Key Info Header - Compact row layout */}
        <div className="bg-primary-500/10 border border-primary-500/30 rounded-lg px-3 py-2">
          <div className="flex items-center gap-3">
            <div className="flex items-center gap-2">
              <Keyboard className="w-4 h-4 text-primary-400" />
              <span className="text-xs text-slate-400 uppercase tracking-wide">
                Key
              </span>
              <span className="text-base font-bold text-slate-100">
                {physicalKey}
              </span>
            </div>

            <ArrowRight className="w-4 h-4 text-slate-500" />

            <div className="flex items-center gap-2">
              <span className="text-xs text-slate-400 uppercase tracking-wide">
                Target
              </span>
              <span className="text-base font-bold text-green-400">
                {config.tapAction || '—'}
              </span>
            </div>

            <div className="ml-auto flex items-center gap-2">
              <span className="text-xs text-slate-400 uppercase tracking-wide">
                Layer
              </span>
              <span className="text-sm font-bold text-yellow-400">
                {activeLayer === 'base'
                  ? 'Base'
                  : activeLayer.toUpperCase().replace('-', '_')}
              </span>
            </div>
          </div>
        </div>

        {/* Configuration Form */}
        <MappingConfigForm
          mappingType={mappingType}
          currentConfig={config}
          onChange={setConfig}
          layoutKeys={layoutKeys}
          enableKeyboardView={true}
        />

        {/* Preview Panel */}
        <div className="bg-slate-800 border border-slate-700 rounded-lg p-4">
          <div className="flex items-center gap-2 mb-2">
            <ArrowRight className="w-4 h-4 text-primary-400" />
            <label className="text-sm font-medium text-slate-300">
              Preview
            </label>
          </div>
          <div className="bg-slate-900 rounded-md p-3 font-mono text-sm text-slate-300 whitespace-pre-wrap min-h-[60px] flex items-center">
            {getPreviewText()}
          </div>
        </div>

        {/* Current Mappings Summary */}
        {keyMappings.size > 0 && (
          <div className="border-t border-slate-700 pt-4">
            <CurrentMappingsSummary
              keyMappings={keyMappings}
              onEditMapping={(_keyCode) => {
                // This will be handled by parent component
                // TODO: Implement mapping editing
              }}
              onClearMapping={(_keyCode) => {
                // This will be handled by parent component
                // TODO: Implement mapping clearing
              }}
            />
          </div>
        )}

        {/* Actions */}
        <div className="flex justify-end gap-3 pt-4 border-t border-slate-700">
          <button
            onClick={onClose}
            className="px-4 py-2 text-slate-300 hover:text-slate-100 transition-colors"
          >
            Cancel
          </button>
          <button
            onClick={handleSave}
            disabled={!isConfigValid()}
            className="px-6 py-2 bg-primary-500 text-white rounded-md hover:bg-primary-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors font-medium"
          >
            Save Mapping
          </button>
        </div>
      </div>
    </Modal>
  );
}
