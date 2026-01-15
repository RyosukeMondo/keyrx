import React, { useState, useMemo, useEffect, useCallback } from 'react';
import { Modal } from './Modal';
import {
  MousePointerClick,
  Timer,
  ListOrdered,
  Layers,
  Keyboard,
  ArrowRight,
  Check,
  Lock,
  Command,
  X,
  Radio
} from 'lucide-react';
import { cn } from '@/utils/cn';
import type { KeyMapping } from '@/types';
import { KeyPalette, type PaletteKey } from './KeyPalette';
import { SVGKeyboard, type SVGKey } from './SVGKeyboard';
import { CurrentMappingsSummary } from './CurrentMappingsSummary';

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

type MappingType = 'simple' | 'modifier' | 'lock' | 'tap_hold' | 'layer_active';

const MAPPING_TYPE_CONFIG = {
  simple: {
    icon: MousePointerClick,
    label: 'Simple',
    description: 'Map to a single key',
  },
  modifier: {
    icon: Command,
    label: 'Modifier',
    description: 'Act as a modifier key',
  },
  lock: {
    icon: Lock,
    label: 'Lock',
    description: 'Toggle lock state',
  },
  tap_hold: {
    icon: Timer,
    label: 'Tap/Hold',
    description: 'Different actions for tap vs hold',
  },
  layer_active: {
    icon: Layers,
    label: 'Layer Active',
    description: 'Activate a layer',
  },
} as const;

// Quick Assign removed per UAT feedback - unnecessary clutter

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
    const validTypes: MappingType[] = ['simple', 'modifier', 'lock', 'tap_hold', 'layer_active'];
    return validTypes.includes(currentMapping.type as MappingType)
      ? (currentMapping.type as MappingType)
      : 'simple';
  }, [currentMapping]);

  const [mappingType, setMappingType] = useState<MappingType>(initialMappingType);
  const [tapAction, setTapAction] = useState(currentMapping?.tapAction || '');
  const [holdAction, setHoldAction] = useState(currentMapping?.holdAction || '');
  const [threshold, setThreshold] = useState(currentMapping?.threshold || 200);
  const [modifierKey, setModifierKey] = useState(currentMapping?.modifierKey || '');
  const [lockKey, setLockKey] = useState(currentMapping?.lockKey || '');
  const [targetLayer, setTargetLayer] = useState(currentMapping?.targetLayer || '');
  const [useKeyboard, setUseKeyboard] = useState(false); // Toggle between keyboard and palette

  // Key listening state
  const [isListening, setIsListening] = useState(false);
  const [listeningFor, setListeningFor] = useState<'tap' | 'hold' | null>(null);

  // Key listening effect
  const handleKeyCapture = useCallback((event: KeyboardEvent) => {
    if (!isListening) return;

    event.preventDefault();
    event.stopPropagation();

    // Allow Escape to cancel listening
    if (event.key === 'Escape') {
      stopListening();
      return;
    }

    // Convert event.code to VK format
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
  }, [isListening, listeningFor]);

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

  const handleSave = () => {
    let mapping: KeyMapping;

    switch (mappingType) {
      case 'simple':
        mapping = {
          type: 'simple',
          tapAction: tapAction,
        };
        break;
      case 'modifier':
        mapping = {
          type: 'modifier',
          modifierKey: modifierKey,
        };
        break;
      case 'lock':
        mapping = {
          type: 'lock',
          lockKey: lockKey,
        };
        break;
      case 'tap_hold':
        mapping = {
          type: 'tap_hold',
          tapAction: tapAction,
          holdAction: holdAction,
          threshold: threshold,
        };
        break;
      case 'layer_active':
        mapping = {
          type: 'layer_active',
          targetLayer: targetLayer,
        };
        break;
      default:
        mapping = {
          type: 'simple',
          tapAction: tapAction,
        };
    }

    onSave(mapping);
    onClose();
  };

  // handleQuickAssign removed - Quick Actions section removed per UAT feedback

  const getPreviewText = (): string => {
    switch (mappingType) {
      case 'simple':
        return tapAction
          ? `Press ${physicalKey} → Output ${tapAction}`
          : 'Select a target key to map to';
      case 'modifier':
        return modifierKey
          ? `${physicalKey} acts as ${modifierKey} modifier`
          : 'Select a modifier key';
      case 'lock':
        return lockKey
          ? `${physicalKey} toggles ${lockKey} lock state`
          : 'Select a lock key';
      case 'tap_hold':
        if (!tapAction && !holdAction) {
          return 'Configure tap and hold actions';
        }
        return `Quick tap: ${physicalKey} → ${tapAction || '?'}\nHold ${threshold}ms: ${physicalKey} → ${holdAction || '?'}`;
      case 'layer_active':
        return targetLayer
          ? `${physicalKey} activates ${targetLayer} layer`
          : 'Select a target layer';
      default:
        return '';
    }
  };

  return (
    <Modal open={isOpen} onClose={onClose} title="Configure Key Mapping" size="xl">
      <div className="space-y-6">
        {/* Mapping Type Selector - Compact horizontal layout */}
        <div className="flex items-center gap-3">
          <label className="text-xs font-medium text-slate-400 uppercase tracking-wider whitespace-nowrap">
            Type
          </label>
          <div className="flex gap-2 flex-wrap">
            {(Object.keys(MAPPING_TYPE_CONFIG) as MappingType[]).map((type) => {
              const config = MAPPING_TYPE_CONFIG[type];
              const Icon = config.icon;
              return (
                <button
                  key={type}
                  onClick={() => setMappingType(type)}
                  className={`px-3 py-1.5 rounded text-xs font-medium transition-all flex items-center gap-1.5 ${
                    mappingType === type
                      ? 'bg-primary-500 text-white'
                      : 'bg-slate-700 text-slate-300 hover:bg-slate-600'
                  }`}
                  title={config.description}
                >
                  <Icon className="w-3.5 h-3.5" />
                  <span>{config.label}</span>
                </button>
              );
            })}
          </div>
        </div>

        {/* Key Info Header - Compact row layout */}
        <div className="bg-primary-500/10 border border-primary-500/30 rounded-lg px-3 py-2">
          <div className="flex items-center gap-3">
            <div className="flex items-center gap-2">
              <Keyboard className="w-4 h-4 text-primary-400" />
              <span className="text-xs text-slate-400 uppercase tracking-wide">Key</span>
              <span className="text-base font-bold text-slate-100">{physicalKey}</span>
            </div>

            <ArrowRight className="w-4 h-4 text-slate-500" />

            <div className="flex items-center gap-2">
              <span className="text-xs text-slate-400 uppercase tracking-wide">Target</span>
              <span className="text-base font-bold text-green-400">{tapAction || '—'}</span>
            </div>

            <div className="ml-auto flex items-center gap-2">
              <span className="text-xs text-slate-400 uppercase tracking-wide">Layer</span>
              <span className="text-sm font-bold text-yellow-400">
                {activeLayer === 'base' ? 'Base' : activeLayer.toUpperCase().replace('-', '_')}
              </span>
            </div>
          </div>
        </div>


        {/* View Toggle - Keyboard vs Palette */}
        {(mappingType === 'simple' || mappingType === 'modifier' || mappingType === 'lock') && (
          <div className="flex items-center justify-between">
            <label className="text-sm font-medium text-slate-300">
              Select Key
            </label>
            <div className="flex gap-2">
              <button
                onClick={() => setUseKeyboard(true)}
                className={`px-3 py-1.5 rounded-md text-xs font-medium transition-colors ${
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
                className={`px-3 py-1.5 rounded-md text-xs font-medium transition-colors ${
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

        {/* Simple Mapping */}
        {mappingType === 'simple' && (
          <div>
            <div className="flex items-center justify-between mb-2">
              <div className="flex items-center gap-2">
                {tapAction && (
                  <>
                    <div className="px-4 py-2 bg-primary-500/20 border border-primary-500 rounded-lg">
                      <span className="text-xl font-bold text-primary-300 font-mono">{tapAction}</span>
                    </div>
                    <button
                      onClick={() => setTapAction('')}
                      className="p-1 text-slate-400 hover:text-red-400 transition-colors"
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
                className={`px-3 py-1.5 rounded text-xs font-medium transition-colors flex items-center gap-1.5 ${
                  isListening && listeningFor === 'tap'
                    ? 'bg-green-500 text-white animate-pulse'
                    : 'bg-slate-600 text-slate-300 hover:bg-slate-500'
                }`}
                title="Press any key on your keyboard to capture it"
              >
                <Radio className="w-4 h-4" />
                {isListening && listeningFor === 'tap' ? 'Listening...' : 'Listen for Key'}
              </button>
            </div>
            <p className="text-xs text-slate-400 mb-2">Click a key below or use Listen button</p>
            {useKeyboard && layoutKeys.length > 0 ? (
              <div className="border border-slate-600 rounded-lg overflow-auto max-h-96 bg-slate-900">
                <SVGKeyboard
                  keys={layoutKeys}
                  keyMappings={new Map()}
                  onKeyClick={(keyCode) => setTapAction(keyCode)}
                  className="w-full"
                />
              </div>
            ) : (
              <div className="border border-slate-600 rounded-lg overflow-y-auto max-h-72">
                <KeyPalette
                  compact
                  onKeySelect={(key) => setTapAction(key.id)}
                  selectedKey={tapAction ? { id: tapAction, label: tapAction, category: 'basic' } : null}
                />
              </div>
            )}
          </div>
        )}

        {/* Modifier Mapping */}
        {mappingType === 'modifier' && (
          <div>
            <div className="flex items-center justify-between mb-2">
              <div className="flex items-center gap-2">
                {modifierKey && (
                  <>
                    <div className="px-4 py-2 bg-cyan-500/20 border border-cyan-500 rounded-lg">
                      <span className="text-xl font-bold text-cyan-300 font-mono">{modifierKey}</span>
                    </div>
                    <button
                      onClick={() => setModifierKey('')}
                      className="p-1 text-slate-400 hover:text-red-400 transition-colors"
                      title="Clear selection"
                    >
                      <X className="w-5 h-5" />
                    </button>
                  </>
                )}
              </div>
            </div>
            <p className="text-xs text-slate-400 mb-2">Select a modifier key (Ctrl, Shift, Alt, etc.)</p>
            {useKeyboard && layoutKeys.length > 0 ? (
              <div className="border border-slate-600 rounded-lg overflow-auto max-h-96 bg-slate-900">
                <SVGKeyboard
                  keys={layoutKeys}
                  keyMappings={new Map()}
                  onKeyClick={(keyCode) => setModifierKey(keyCode)}
                  className="w-full"
                />
              </div>
            ) : (
              <div className="border border-slate-600 rounded-lg overflow-y-auto max-h-72">
                <KeyPalette
                  compact
                  onKeySelect={(key) => setModifierKey(key.id)}
                  selectedKey={modifierKey ? { id: modifierKey, label: modifierKey, category: 'modifiers' } : null}
                />
              </div>
            )}
          </div>
        )}

        {/* Lock Mapping */}
        {mappingType === 'lock' && (
          <div>
            <div className="flex items-center justify-between mb-2">
              <div className="flex items-center gap-2">
                {lockKey && (
                  <>
                    <div className="px-4 py-2 bg-purple-500/20 border border-purple-500 rounded-lg">
                      <span className="text-xl font-bold text-purple-300 font-mono">{lockKey}</span>
                    </div>
                    <button
                      onClick={() => setLockKey('')}
                      className="p-1 text-slate-400 hover:text-red-400 transition-colors"
                      title="Clear selection"
                    >
                      <X className="w-5 h-5" />
                    </button>
                  </>
                )}
              </div>
            </div>
            <p className="text-xs text-slate-400 mb-2">Select a lock key (CapsLock, NumLock, etc.)</p>
            {useKeyboard && layoutKeys.length > 0 ? (
              <div className="border border-slate-600 rounded-lg overflow-auto max-h-96 bg-slate-900">
                <SVGKeyboard
                  keys={layoutKeys}
                  keyMappings={new Map()}
                  onKeyClick={(keyCode) => setLockKey(keyCode)}
                  className="w-full"
                />
              </div>
            ) : (
              <div className="border border-slate-600 rounded-lg overflow-y-auto max-h-72">
                <KeyPalette
                  compact
                  onKeySelect={(key) => setLockKey(key.id)}
                  selectedKey={lockKey ? { id: lockKey, label: lockKey, category: 'special' } : null}
                />
              </div>
            )}
          </div>
        )}

        {/* Layer Active Mapping */}
        {mappingType === 'layer_active' && (
          <div>
            <div className="flex items-center justify-between mb-2">
              {targetLayer && (
                <div className="px-4 py-2 bg-yellow-500/20 border border-yellow-500 rounded-lg">
                  <span className="text-xl font-bold text-yellow-300 font-mono">{targetLayer}</span>
                </div>
              )}
            </div>
            <p className="text-xs text-slate-400 mb-2">Select a layer to activate (MO, TO, TG, OSL)</p>
            <div className="border border-slate-600 rounded-lg overflow-y-auto max-h-72">
              <KeyPalette
                compact
                onKeySelect={(key) => setTargetLayer(key.id)}
                selectedKey={targetLayer ? { id: targetLayer, label: targetLayer, category: 'layers' } : null}
              />
            </div>
          </div>
        )}

        {/* Tap/Hold Mapping - Simplified */}
        {mappingType === 'tap_hold' && (
          <>
            {/* Tap Action - Simplified */}
            <div>
              <div className="flex items-center justify-between mb-3">
                <label className="text-sm font-medium text-slate-300">
                  Tap Action
                </label>
                <div className="flex items-center gap-2">
                  {tapAction && (
                    <>
                      <div className="px-3 py-1 bg-green-500/20 border border-green-500 rounded">
                        <span className="text-sm font-bold text-green-300 font-mono">{tapAction}</span>
                      </div>
                      <button
                        onClick={() => setTapAction('')}
                        className="p-1 text-slate-400 hover:text-red-400 transition-colors"
                        title="Clear selection"
                      >
                        <X className="w-4 h-4" />
                      </button>
                    </>
                  )}
                  <button
                    onClick={() => startListening('tap')}
                    disabled={isListening}
                    className={`px-3 py-1 rounded text-xs font-medium transition-colors flex items-center gap-1.5 ${
                      isListening && listeningFor === 'tap'
                        ? 'bg-green-500 text-white animate-pulse'
                        : 'bg-slate-600 text-slate-300 hover:bg-slate-500'
                    }`}
                    title="Press any key on your keyboard to capture it"
                  >
                    <Radio className="w-3.5 h-3.5" />
                    {isListening && listeningFor === 'tap' ? 'Listening...' : 'Listen'}
                  </button>
                </div>
              </div>
              <p className="text-xs text-slate-400 mb-3">Click a key to select it</p>
              <div className="border border-slate-600 rounded-lg overflow-y-auto max-h-72">
                <KeyPalette
                  compact
                  onKeySelect={(key) => setTapAction(key.id)}
                  selectedKey={tapAction ? { id: tapAction, label: tapAction, category: 'basic' } : null}
                />
              </div>
            </div>

            {/* Hold Action - Numerical Selector */}
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
                      <span className="text-sm font-bold text-red-300 font-mono">{holdAction}</span>
                    </div>
                    <button
                      onClick={() => setHoldAction('')}
                      className="p-1 text-slate-400 hover:text-red-400 transition-colors"
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
                    const val = Math.max(0, Math.min(255, parseInt(e.target.value) || 0));
                    const hex = val.toString(16).toUpperCase().padStart(2, '0');
                    setHoldAction(`MD_${hex}`);
                  }}
                  className="w-full px-4 py-2 bg-slate-800 border border-slate-600 rounded-md text-slate-100 focus:outline-none focus:ring-2 focus:ring-primary-500"
                  placeholder="Enter value 0-255"
                />
                <div className="mt-3">
                  <input
                    type="range"
                    min="0"
                    max="255"
                    value={holdAction ? parseInt(holdAction.replace('MD_', ''), 16) : 0}
                    onChange={(e) => {
                      const val = parseInt(e.target.value);
                      const hex = val.toString(16).toUpperCase().padStart(2, '0');
                      setHoldAction(`MD_${hex}`);
                    }}
                    className="w-full"
                  />
                  <div className="flex justify-between text-xs text-slate-500 mt-1">
                    <span>0 (MD_00)</span>
                    <span>255 (MD_FF)</span>
                  </div>
                </div>
              </div>
            </div>

            <div>
              <label className="block text-sm font-medium text-slate-300 mb-2">
                6. Hold Threshold (ms): {threshold}
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
              onEditMapping={(keyCode) => {
                // This will be handled by parent component
                console.log('Edit mapping for:', keyCode);
              }}
              onClearMapping={(keyCode) => {
                // This will be handled by parent component
                console.log('Clear mapping for:', keyCode);
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
            disabled={
              (mappingType === 'simple' && !tapAction) ||
              (mappingType === 'modifier' && !modifierKey) ||
              (mappingType === 'lock' && !lockKey) ||
              (mappingType === 'tap_hold' && (!tapAction || !holdAction)) ||
              (mappingType === 'layer_active' && !targetLayer)
            }
            className="px-6 py-2 bg-primary-500 text-white rounded-md hover:bg-primary-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors font-medium"
          >
            Save Mapping
          </button>
        </div>

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
                Press <kbd className="px-2 py-1 bg-slate-700 rounded text-slate-200">Escape</kbd> to cancel
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
    </Modal>
  );
}
