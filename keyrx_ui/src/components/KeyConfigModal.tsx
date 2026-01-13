import React, { useState } from 'react';
import { Modal } from './Modal';
import {
  MousePointerClick,
  Timer,
  ListOrdered,
  Layers,
  Keyboard,
  ArrowRight
} from 'lucide-react';
import type { KeyMapping } from '@/types';
import type { PaletteKey } from './KeyPalette';

/**
 * Advanced Key Configuration Modal
 * Supports: simple, tap_hold, modified, macro, layer_switch
 */

interface KeyConfigModalProps {
  isOpen: boolean;
  onClose: () => void;
  physicalKey: string;
  currentMapping?: KeyMapping;
  onSave: (mapping: KeyMapping) => void;
  availableKeys: PaletteKey[];
}

type MappingType = 'simple' | 'tap_hold' | 'macro' | 'layer_switch';

const MAPPING_TYPE_CONFIG = {
  simple: {
    icon: MousePointerClick,
    label: 'Simple',
    description: 'Map to a single key',
  },
  tap_hold: {
    icon: Timer,
    label: 'Tap/Hold',
    description: 'Different actions for tap vs hold',
  },
  macro: {
    icon: ListOrdered,
    label: 'Macro',
    description: 'Execute a sequence of keys',
  },
  layer_switch: {
    icon: Layers,
    label: 'Layer',
    description: 'Switch to another layer',
  },
} as const;

const QUICK_ASSIGN_KEYS = [
  { id: 'Escape', label: 'Esc', icon: '⎋' },
  { id: 'Enter', label: 'Enter', icon: '↵' },
  { id: 'Backspace', label: 'Backspace', icon: '⌫' },
  { id: 'Delete', label: 'Delete', icon: '⌦' },
  { id: 'Space', label: 'Space', icon: '␣' },
  { id: 'Tab', label: 'Tab', icon: '⇥' },
] as const;

export function KeyConfigModal({
  isOpen,
  onClose,
  physicalKey,
  currentMapping,
  onSave,
  availableKeys,
}: KeyConfigModalProps) {
  const [mappingType, setMappingType] = useState<MappingType>(
    currentMapping?.type || 'simple'
  );
  const [tapAction, setTapAction] = useState(currentMapping?.tapAction || '');
  const [holdAction, setHoldAction] = useState(currentMapping?.holdAction || '');
  const [threshold, setThreshold] = useState(currentMapping?.threshold || 200);
  const [targetLayer, setTargetLayer] = useState(currentMapping?.targetLayer || '');

  const handleSave = () => {
    const mapping: KeyMapping = {
      type: mappingType,
      tapAction: mappingType !== 'layer_switch' ? tapAction : undefined,
      holdAction: mappingType === 'tap_hold' ? holdAction : undefined,
      threshold: mappingType === 'tap_hold' ? threshold : undefined,
      targetLayer: mappingType === 'layer_switch' ? targetLayer : undefined,
    };
    onSave(mapping);
    onClose();
  };

  const handleQuickAssign = (keyId: string) => {
    setMappingType('simple');
    setTapAction(keyId);
  };

  const getPreviewText = (): string => {
    switch (mappingType) {
      case 'simple':
        return tapAction
          ? `Press ${physicalKey} → Output ${tapAction}`
          : 'Select a target key to map to';
      case 'tap_hold':
        if (!tapAction && !holdAction) {
          return 'Configure tap and hold actions';
        }
        return `Quick tap: ${physicalKey} → ${tapAction || '?'}\nHold ${threshold}ms: ${physicalKey} → ${holdAction || '?'}`;
      case 'macro':
        return 'Macro configuration coming soon. Use Code Editor for now.';
      case 'layer_switch':
        return targetLayer
          ? `Press ${physicalKey} → Switch to ${targetLayer}`
          : 'Select a target layer';
      default:
        return '';
    }
  };

  return (
    <Modal open={isOpen} onClose={onClose} title="Configure Key Mapping">
      <div className="space-y-6">
        {/* Key Info Header */}
        <div className="bg-primary-500/10 border border-primary-500/30 rounded-lg p-4">
          <div className="flex items-center gap-3">
            <div className="bg-primary-500/20 p-2 rounded">
              <Keyboard className="w-6 h-6 text-primary-400" />
            </div>
            <div>
              <div className="text-xs text-slate-400 uppercase tracking-wide">
                Physical Key
              </div>
              <div className="text-2xl font-bold text-slate-100">
                {physicalKey}
              </div>
            </div>
          </div>
        </div>

        {/* Quick Assign Section */}
        <div>
          <label className="block text-sm font-medium text-slate-300 mb-2">
            Quick Assign (Common Keys)
          </label>
          <div className="grid grid-cols-3 gap-2">
            {QUICK_ASSIGN_KEYS.map((key) => (
              <button
                key={key.id}
                onClick={() => handleQuickAssign(key.id)}
                className="px-3 py-2 bg-slate-700 hover:bg-slate-600 text-slate-200 rounded-md text-sm font-medium transition-colors flex items-center justify-center gap-2"
                title={`Quick assign to ${key.label}`}
              >
                <span className="text-lg">{key.icon}</span>
                <span>{key.label}</span>
              </button>
            ))}
          </div>
        </div>

        {/* Mapping Type Selector with Icons */}
        <div>
          <label className="block text-sm font-medium text-slate-300 mb-3">
            Mapping Type
          </label>
          <div className="grid grid-cols-2 gap-3">
            {(Object.keys(MAPPING_TYPE_CONFIG) as MappingType[]).map((type) => {
              const config = MAPPING_TYPE_CONFIG[type];
              const Icon = config.icon;
              return (
                <button
                  key={type}
                  onClick={() => setMappingType(type)}
                  className={`px-4 py-3 rounded-lg text-sm font-medium transition-all flex flex-col items-start gap-2 ${
                    mappingType === type
                      ? 'bg-primary-500 text-white shadow-lg shadow-primary-500/50'
                      : 'bg-slate-700 text-slate-300 hover:bg-slate-600'
                  }`}
                  title={config.description}
                >
                  <Icon className="w-5 h-5" />
                  <div className="text-left">
                    <div className="font-semibold">{config.label}</div>
                    <div className={`text-xs ${mappingType === type ? 'text-primary-100' : 'text-slate-400'}`}>
                      {config.description}
                    </div>
                  </div>
                </button>
              );
            })}
          </div>
        </div>

        {/* Simple Mapping */}
        {mappingType === 'simple' && (
          <div>
            <label className="block text-sm font-medium text-slate-300 mb-2">
              Target Key
            </label>
            <select
              value={tapAction}
              onChange={(e) => setTapAction(e.target.value)}
              className="w-full px-3 py-2 bg-slate-700 border border-slate-600 rounded-md text-slate-100 focus:outline-none focus:ring-2 focus:ring-primary-500"
            >
              <option value="">Select a key...</option>
              {availableKeys.map((key) => (
                <option key={key.id} value={key.id}>
                  {key.label} ({key.id})
                </option>
              ))}
            </select>
          </div>
        )}

        {/* Tap/Hold Mapping */}
        {mappingType === 'tap_hold' && (
          <>
            <div>
              <label className="block text-sm font-medium text-slate-300 mb-2">
                Tap Action (quick press)
              </label>
              <select
                value={tapAction}
                onChange={(e) => setTapAction(e.target.value)}
                className="w-full px-3 py-2 bg-slate-700 border border-slate-600 rounded-md text-slate-100 focus:outline-none focus:ring-2 focus:ring-primary-500"
              >
                <option value="">Select a key...</option>
                {availableKeys.map((key) => (
                  <option key={key.id} value={key.id}>
                    {key.label} ({key.id})
                  </option>
                ))}
              </select>
            </div>

            <div>
              <label className="block text-sm font-medium text-slate-300 mb-2">
                Hold Action (when held)
              </label>
              <select
                value={holdAction}
                onChange={(e) => setHoldAction(e.target.value)}
                className="w-full px-3 py-2 bg-slate-700 border border-slate-600 rounded-md text-slate-100 focus:outline-none focus:ring-2 focus:ring-primary-500"
              >
                <option value="">Select a key...</option>
                {availableKeys.filter(k => k.category === 'modifier' || k.category === 'layer').map((key) => (
                  <option key={key.id} value={key.id}>
                    {key.label} ({key.id})
                  </option>
                ))}
              </select>
            </div>

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
                onChange={(e) => setThreshold(parseInt(e.target.value))}
                className="w-full"
              />
              <div className="flex justify-between text-xs text-slate-500 mt-1">
                <span>50ms (fast)</span>
                <span>500ms (slow)</span>
              </div>
            </div>
          </>
        )}

        {/* Layer Switch */}
        {mappingType === 'layer_switch' && (
          <div>
            <label className="block text-sm font-medium text-slate-300 mb-2">
              Target Layer
            </label>
            <select
              value={targetLayer}
              onChange={(e) => setTargetLayer(e.target.value)}
              className="w-full px-3 py-2 bg-slate-700 border border-slate-600 rounded-md text-slate-100 focus:outline-none focus:ring-2 focus:ring-primary-500"
            >
              <option value="">Select a layer...</option>
              {Array.from({ length: 10 }, (_, i) => (
                <option key={i} value={`MD_${i.toString().padStart(2, '0')}`}>
                  Layer MD_{i.toString().padStart(2, '0')}
                </option>
              ))}
            </select>
          </div>
        )}

        {/* Macro (placeholder) */}
        {mappingType === 'macro' && (
          <div className="p-4 bg-slate-700/50 rounded-lg border border-slate-600">
            <p className="text-sm text-slate-400">
              Macro configuration coming soon. Use Code Editor tab for now.
            </p>
          </div>
        )}

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
            disabled={
              (mappingType === 'simple' && !tapAction) ||
              (mappingType === 'tap_hold' && (!tapAction || !holdAction)) ||
              (mappingType === 'layer_switch' && !targetLayer)
            }
            className="px-6 py-2 bg-primary-500 text-white rounded-md hover:bg-primary-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors font-medium"
          >
            Save Mapping
          </button>
        </div>
      </div>
    </Modal>
  );
}
