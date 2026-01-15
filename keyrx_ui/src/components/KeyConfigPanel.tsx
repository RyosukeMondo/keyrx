import React, { useState, useMemo, useEffect, useCallback } from 'react';
import {
  MousePointerClick,
  Timer,
  Keyboard,
  ArrowRight,
  Lock,
  Command,
  X,
  Radio
} from 'lucide-react';
import type { KeyMapping } from '@/types';
import { SVGKeyboard, type SVGKey } from './SVGKeyboard';
import { CurrentMappingsSummary } from './CurrentMappingsSummary';

/**
 * Inline Key Configuration Panel
 * No modal - always visible, updates when key is selected
 *
 * Simplified UI:
 * - 2 mapping types: Simple, Tap/Hold
 * - 3 key selection tabs: Keyboard, Modifier, Lock
 */

interface KeyConfigPanelProps {
  physicalKey: string | null;
  currentMapping?: KeyMapping;
  onSave: (mapping: KeyMapping) => void;
  onClearMapping: (keyCode: string) => void;
  onEditMapping: (keyCode: string) => void;
  activeLayer?: string;
  keyMappings: Map<string, KeyMapping>;
  layoutKeys?: SVGKey[];
}

type MappingType = 'simple' | 'tap_hold';
type KeySelectionTab = 'keyboard' | 'modifier' | 'lock';

const MAPPING_TYPE_CONFIG = {
  simple: {
    icon: MousePointerClick,
    label: 'Simple',
    description: 'Single key remap',
  },
  tap_hold: {
    icon: Timer,
    label: 'Tap/Hold',
    description: 'Tap or hold for different actions',
  },
} as const;

export function KeyConfigPanel({
  physicalKey,
  currentMapping,
  onSave,
  onClearMapping,
  onEditMapping,
  activeLayer = 'base',
  keyMappings = new Map(),
  layoutKeys = [],
}: KeyConfigPanelProps) {
  // Determine initial mapping type
  const initialMappingType = useMemo(() => {
    if (!currentMapping) return 'simple';
    return currentMapping.type === 'tap_hold' ? 'tap_hold' : 'simple';
  }, [currentMapping]);

  const [mappingType, setMappingType] = useState<MappingType>(initialMappingType);
  const [tapAction, setTapAction] = useState(currentMapping?.tapAction || '');
  const [holdAction, setHoldAction] = useState(currentMapping?.holdAction || '');
  const [threshold, setThreshold] = useState(currentMapping?.threshold || 200);
  const [activeTab, setActiveTab] = useState<KeySelectionTab>('keyboard');

  // Key listening state
  const [isListening, setIsListening] = useState(false);
  const [listeningFor, setListeningFor] = useState<'tap' | 'hold' | null>(null);

  // Reset form when physical key changes
  React.useEffect(() => {
    if (currentMapping) {
      setMappingType(initialMappingType);
      setTapAction(currentMapping.tapAction || '');
      setHoldAction(currentMapping.holdAction || '');
      setThreshold(currentMapping.threshold || 200);
    } else {
      // Reset to defaults when no mapping
      setMappingType('simple');
      setTapAction('');
      setHoldAction('');
      setThreshold(200);
    }
  }, [physicalKey, currentMapping, initialMappingType]);

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

    // Convert event.code to VK format (e.g., "KeyA" -> "VK_A")
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
    if (!physicalKey) return;

    const mapping: KeyMapping = mappingType === 'tap_hold'
      ? {
          type: 'tap_hold',
          tapAction: tapAction,
          holdAction: holdAction,
          threshold: threshold,
        }
      : {
          type: 'simple',
          tapAction: tapAction,
        };

    onSave(mapping);
  };

  const getPreviewText = (): string => {
    if (!physicalKey) return 'Select a key from the keyboard above';

    if (mappingType === 'tap_hold') {
      if (!tapAction && !holdAction) {
        return 'Configure tap and hold actions';
      }
      return `Quick tap: ${physicalKey} → ${tapAction || '?'}\nHold ${threshold}ms: ${physicalKey} → ${holdAction || '?'}`;
    }

    return tapAction
      ? `Press ${physicalKey} → Output ${tapAction}`
      : 'Select a target key';
  };

  const isSaveDisabled = !physicalKey || (
    (mappingType === 'simple' && !tapAction) ||
    (mappingType === 'tap_hold' && (!tapAction || !holdAction))
  );

  return (
    <div className="bg-slate-800 rounded-lg border border-slate-700 p-6 space-y-6">
      {/* Mapping Type Selector - Compact horizontal layout */}
      <div className="flex items-center gap-3">
        <label className="text-xs font-medium text-slate-400 uppercase tracking-wider whitespace-nowrap">
          Type
        </label>
        <div className="flex gap-2 flex-1">
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

      {!physicalKey ? (
        <div className="text-center py-12 text-slate-400">
          <Keyboard className="w-16 h-16 mx-auto mb-4 opacity-50" />
          <p className="text-lg">Click a key on the keyboard above to configure it</p>
        </div>
      ) : (
        <>
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

          {/* Key Selection Tabs - Keyboard / Modifier / Lock */}
          {mappingType === 'simple' && (
            <div>
              <div className="flex items-center justify-between mb-3">
                <label className="text-sm font-medium text-slate-300">
                  Select Key
                </label>
                <div className="flex items-center gap-2">
                  {tapAction && (
                    <button
                      onClick={() => setTapAction('')}
                      className="px-3 py-1 text-xs text-red-300 hover:text-red-100 hover:bg-red-500/20 rounded transition-colors flex items-center gap-1"
                      title="Clear selection"
                    >
                      <X className="w-3.5 h-3.5" />
                      Clear
                    </button>
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
              <div className="flex gap-2 mb-4 border-b border-slate-700">
                <button
                  onClick={() => setActiveTab('keyboard')}
                  className={`px-4 py-2 text-sm font-medium transition-colors border-b-2 ${
                    activeTab === 'keyboard'
                      ? 'text-primary-400 border-primary-400'
                      : 'text-slate-400 border-transparent hover:text-slate-300'
                  }`}
                >
                  <Keyboard className="w-4 h-4 inline-block mr-2" />
                  Keyboard
                </button>
                <button
                  onClick={() => setActiveTab('modifier')}
                  className={`px-4 py-2 text-sm font-medium transition-colors border-b-2 ${
                    activeTab === 'modifier'
                      ? 'text-primary-400 border-primary-400'
                      : 'text-slate-400 border-transparent hover:text-slate-300'
                  }`}
                >
                  <Command className="w-4 h-4 inline-block mr-2" />
                  Modifier
                </button>
                <button
                  onClick={() => setActiveTab('lock')}
                  className={`px-4 py-2 text-sm font-medium transition-colors border-b-2 ${
                    activeTab === 'lock'
                      ? 'text-primary-400 border-primary-400'
                      : 'text-slate-400 border-transparent hover:text-slate-300'
                  }`}
                >
                  <Lock className="w-4 h-4 inline-block mr-2" />
                  Lock
                </button>
              </div>
            </div>
          )}

          {/* Simple Mapping - Tab Content */}
          {mappingType === 'simple' && (
            <div>
              {/* Keyboard Tab */}
              {activeTab === 'keyboard' && layoutKeys.length > 0 && (
                <div className="border border-slate-600 rounded-lg overflow-auto max-h-96 bg-slate-900">
                  <SVGKeyboard
                    keys={layoutKeys}
                    keyMappings={new Map()}
                    onKeyClick={(keyCode) => setTapAction(keyCode)}
                    className="w-full"
                  />
                </div>
              )}

              {/* Modifier Tab - MD_00 to MD_FF */}
              {activeTab === 'modifier' && (
                <div className="border border-slate-600 rounded-lg p-4 bg-slate-900 max-h-96 overflow-y-auto">
                  <p className="text-xs text-slate-400 mb-3">Select a custom modifier (MD_00 to MD_FF)</p>
                  <div className="grid grid-cols-8 gap-2">
                    {Array.from({ length: 256 }, (_, i) => {
                      const hex = i.toString(16).toUpperCase().padStart(2, '0');
                      const id = `MD_${hex}`;
                      return (
                        <button
                          key={id}
                          onClick={() => setTapAction(id)}
                          className={`px-2 py-2 rounded text-xs font-mono transition-colors ${
                            tapAction === id
                              ? 'bg-primary-500 text-white'
                              : 'bg-slate-700 text-slate-300 hover:bg-slate-600'
                          }`}
                          title={`Modifier ${hex} (${i})`}
                        >
                          {hex}
                        </button>
                      );
                    })}
                  </div>
                </div>
              )}

              {/* Lock Tab - LK_00 to LK_FF */}
              {activeTab === 'lock' && (
                <div className="border border-slate-600 rounded-lg p-4 bg-slate-900 max-h-96 overflow-y-auto">
                  <p className="text-xs text-slate-400 mb-3">Select a lock state (LK_00 to LK_FF)</p>
                  <div className="grid grid-cols-8 gap-2">
                    {Array.from({ length: 256 }, (_, i) => {
                      const hex = i.toString(16).toUpperCase().padStart(2, '0');
                      const id = `LK_${hex}`;
                      const labels: Record<string, string> = {
                        'LK_00': 'CapsLock',
                        'LK_01': 'NumLock',
                        'LK_02': 'ScrollLock',
                      };
                      return (
                        <button
                          key={id}
                          onClick={() => setTapAction(id)}
                          className={`px-2 py-2 rounded text-xs font-mono transition-colors ${
                            tapAction === id
                              ? 'bg-primary-500 text-white'
                              : 'bg-slate-700 text-slate-300 hover:bg-slate-600'
                          }`}
                          title={labels[id] || `Lock ${hex} (${i})`}
                        >
                          {hex}
                        </button>
                      );
                    })}
                  </div>
                </div>
              )}
            </div>
          )}

          {/* Tap/Hold Mapping */}
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

                {/* Tabs */}
                <div className="flex gap-2 mb-4 border-b border-slate-700">
                  <button
                    onClick={() => setActiveTab('keyboard')}
                    className={`px-4 py-2 text-sm font-medium transition-colors border-b-2 ${
                      activeTab === 'keyboard'
                        ? 'text-primary-400 border-primary-400'
                        : 'text-slate-400 border-transparent hover:text-slate-300'
                    }`}
                  >
                    <Keyboard className="w-4 h-4 inline-block mr-2" />
                    Keyboard
                  </button>
                  <button
                    onClick={() => setActiveTab('modifier')}
                    className={`px-4 py-2 text-sm font-medium transition-colors border-b-2 ${
                      activeTab === 'modifier'
                        ? 'text-primary-400 border-primary-400'
                        : 'text-slate-400 border-transparent hover:text-slate-300'
                    }`}
                  >
                    <Command className="w-4 h-4 inline-block mr-2" />
                    Modifier
                  </button>
                  <button
                    onClick={() => setActiveTab('lock')}
                    className={`px-4 py-2 text-sm font-medium transition-colors border-b-2 ${
                      activeTab === 'lock'
                        ? 'text-primary-400 border-primary-400'
                        : 'text-slate-400 border-transparent hover:text-slate-300'
                    }`}
                  >
                    <Lock className="w-4 h-4 inline-block mr-2" />
                    Lock
                  </button>
                </div>

                {/* Tab Content */}
                <div>
                  {/* Keyboard Tab */}
                  {activeTab === 'keyboard' && layoutKeys.length > 0 && (
                    <div className="border border-slate-600 rounded-lg overflow-auto max-h-64 bg-slate-900">
                      <SVGKeyboard
                        keys={layoutKeys}
                        keyMappings={new Map()}
                        onKeyClick={(keyCode) => setTapAction(keyCode)}
                        className="w-full"
                      />
                    </div>
                  )}

                  {/* Modifier Tab */}
                  {activeTab === 'modifier' && (
                    <div className="border border-slate-600 rounded-lg p-3 bg-slate-900 max-h-64 overflow-y-auto">
                      <div className="grid grid-cols-8 gap-2">
                        {Array.from({ length: 256 }, (_, i) => {
                          const hex = i.toString(16).toUpperCase().padStart(2, '0');
                          const id = `MD_${hex}`;
                          return (
                            <button
                              key={id}
                              onClick={() => setTapAction(id)}
                              className={`px-2 py-2 rounded text-xs font-mono transition-colors ${
                                tapAction === id
                                  ? 'bg-green-500 text-white'
                                  : 'bg-slate-700 text-slate-300 hover:bg-slate-600'
                              }`}
                            >
                              {hex}
                            </button>
                          );
                        })}
                      </div>
                    </div>
                  )}

                  {/* Lock Tab */}
                  {activeTab === 'lock' && (
                    <div className="border border-slate-600 rounded-lg p-3 bg-slate-900 max-h-64 overflow-y-auto">
                      <div className="grid grid-cols-8 gap-2">
                        {Array.from({ length: 256 }, (_, i) => {
                          const hex = i.toString(16).toUpperCase().padStart(2, '0');
                          const id = `LK_${hex}`;
                          return (
                            <button
                              key={id}
                              onClick={() => setTapAction(id)}
                              className={`px-2 py-2 rounded text-xs font-mono transition-colors ${
                                tapAction === id
                                  ? 'bg-green-500 text-white'
                                  : 'bg-slate-700 text-slate-300 hover:bg-slate-600'
                              }`}
                            >
                              {hex}
                            </button>
                          );
                        })}
                      </div>
                    </div>
                  )}
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
            {currentMapping && (
              <button
                onClick={() => onClearMapping(physicalKey)}
                className="px-4 py-2 text-red-300 hover:text-red-100 hover:bg-red-500/20 rounded-md transition-colors"
              >
                Clear Mapping
              </button>
            )}
            <button
              onClick={handleSave}
              disabled={isSaveDisabled}
              className="px-6 py-2 bg-primary-500 text-white rounded-md hover:bg-primary-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors font-medium"
            >
              Save Mapping
            </button>
          </div>
        </>
      )}

      {/* Current Mappings Summary */}
      <div className="border-t border-slate-700 pt-6">
        <h3 className="text-lg font-semibold text-slate-200 mb-4">
          Current Mappings ({keyMappings.size} mappings)
        </h3>
        <CurrentMappingsSummary
          keyMappings={keyMappings}
          onEditMapping={onEditMapping}
          onClearMapping={onClearMapping}
        />
      </div>

      {/* Listening Overlay */}
      {isListening && (
        <div className="fixed inset-0 bg-black/80 backdrop-blur-sm z-50 flex items-center justify-center">
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
  );
}
