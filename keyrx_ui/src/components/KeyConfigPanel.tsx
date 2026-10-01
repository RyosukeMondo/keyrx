import React, { useState, useMemo, useEffect } from 'react';
import { Keyboard, ArrowRight, X } from 'lucide-react';
import type { KeyMapping } from '@/types';
import type { SVGKey } from './SVGKeyboard';
import { CurrentMappingsSummary } from './CurrentMappingsSummary';
import { formatLayerName } from './LayerSwitcher';
import { friendlyKeyName } from '@/utils/keyNames';
import { KeyName } from './KeyName';
import { t } from '@/i18n';
import {
  MappingTypeSelector,
  type MappingType,
} from './keyConfig/MappingTypeSelector';
import {
  KeySelectionTabs,
  type KeySelectionTab,
} from './keyConfig/KeySelectionTabs';

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
    if (!currentMapping) return 'simple' as MappingType;
    return (
      currentMapping.type === 'tap_hold' ? 'tap_hold' : 'simple'
    ) as MappingType;
  }, [currentMapping]);

  const [mappingType, setMappingType] =
    useState<MappingType>(initialMappingType);
  const [tapAction, setTapAction] = useState(currentMapping?.tapAction || '');
  const [holdAction, setHoldAction] = useState(
    currentMapping?.holdAction || ''
  );
  const [threshold, setThreshold] = useState(currentMapping?.threshold || 200);
  const [activeTab, setActiveTab] = useState<KeySelectionTab>('keyboard');

  // Reset form when physical key changes
  useEffect(() => {
    if (currentMapping) {
      setMappingType(initialMappingType);
      setTapAction(currentMapping.tapAction || '');
      setHoldAction(currentMapping.holdAction || '');
      setThreshold(currentMapping.threshold || 200);
    } else {
      setMappingType('simple');
      setTapAction('');
      setHoldAction('');
      setThreshold(200);
    }
  }, [physicalKey, currentMapping, initialMappingType]);

  const handleSave = () => {
    if (!physicalKey) return;

    const mapping: KeyMapping =
      mappingType === 'tap_hold'
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

  const name = (code: string) => (code ? friendlyKeyName(code) : '?');

  const getPreviewText = (): string => {
    if (!physicalKey) return t('kc.selectPrompt');

    if (mappingType === 'tap_hold') {
      if (!tapAction && !holdAction) return t('kc.configureTapHold');
      const from = name(physicalKey);
      return `${t('kc.quickTap', { from, to: name(tapAction) })}\n${t('kc.holdLine', { ms: threshold, from, to: name(holdAction) })}`;
    }

    return tapAction
      ? t('kc.press', { from: name(physicalKey), to: name(tapAction) })
      : t('kc.selectTarget');
  };

  // Screen readers hear what a pick did (the picker itself is a silent grid).
  const announcement = tapAction
    ? mappingType === 'tap_hold' && holdAction
      ? t('kc.pickedHold', { key: name(holdAction) })
      : t('kc.pickedTarget', { key: name(tapAction) })
    : t('kc.selectionCleared');

  const isSaveDisabled =
    !physicalKey ||
    (mappingType === 'simple' && !tapAction) ||
    (mappingType === 'tap_hold' && (!tapAction || !holdAction));

  return (
    <div className="bg-slate-800 rounded-lg border border-slate-700 p-6 space-y-6">
      <div role="status" aria-live="polite" className="sr-only">
        {physicalKey ? announcement : ''}
      </div>

      {/* Mapping Type Selector */}
      <MappingTypeSelector
        selectedType={mappingType}
        onChange={setMappingType}
        supportedTypes={['simple', 'tap_hold']}
        layout="horizontal"
      />

      {!physicalKey ? (
        <div className="text-center py-12 text-slate-400">
          <Keyboard className="w-16 h-16 mx-auto mb-4 opacity-50" />
          <p className="text-lg">{t('kc.clickKey')}</p>
        </div>
      ) : (
        <>
          {/* Key Info Header - Compact row layout */}
          <div className="bg-primary-500/10 border border-primary-500/30 rounded-lg px-3 py-2">
            <div className="flex items-center gap-3">
              <div className="flex items-center gap-2">
                <Keyboard className="w-4 h-4 text-primary-400" />
                <span className="text-xs text-slate-400 uppercase tracking-wide">
                  {t('kc.key')}
                </span>
                <KeyName
                  code={physicalKey}
                  className="text-base font-bold text-slate-100"
                />
              </div>

              <ArrowRight className="w-4 h-4 text-slate-400" />

              <div className="flex items-center gap-2">
                <span className="text-xs text-slate-400 uppercase tracking-wide">
                  {t('kc.target')}
                </span>
                {tapAction ? (
                  <KeyName
                    code={tapAction}
                    className="text-base font-bold text-green-400"
                  />
                ) : (
                  <span className="text-base font-bold text-green-400">—</span>
                )}
              </div>

              <div className="ml-auto flex items-center gap-2">
                <span className="text-xs text-slate-400 uppercase tracking-wide">
                  {t('kc.layer')}
                </span>
                <span className="text-sm font-bold text-yellow-400">
                  {formatLayerName(activeLayer)}
                </span>
              </div>
            </div>
          </div>

          {/* Key Selection for Simple Mapping */}
          {mappingType === 'simple' && (
            <div>
              <div className="flex items-center justify-between mb-3">
                <label className="text-sm font-medium text-slate-300">
                  {t('kc.selectKey')}
                </label>
                {tapAction && (
                  <button
                    onClick={() => setTapAction('')}
                    className="px-3 py-1 text-xs text-red-300 hover:text-red-100 hover:bg-red-500/20 rounded transition-colors flex items-center gap-1"
                    title={t('kc.clearSel')}
                  >
                    <X className="w-3.5 h-3.5" aria-hidden="true" />
                    {t('kc.clear')}
                  </button>
                )}
              </div>
              <KeySelectionTabs
                activeTab={activeTab}
                onTabChange={setActiveTab}
                availableTabs={['keyboard', 'modifier', 'lock']}
                onKeySelect={setTapAction}
                layoutKeys={layoutKeys}
                maxHeight="max-h-96"
              />
            </div>
          )}

          {/* Tap/Hold Mapping */}
          {mappingType === 'tap_hold' && (
            <>
              {/* Tap Action */}
              <div>
                <div className="flex items-center justify-between mb-3">
                  <label className="text-sm font-medium text-slate-300">
                    {t('kc.tapAction')}
                  </label>
                  {tapAction && (
                    <div className="flex items-center gap-2">
                      <div className="px-3 py-1 bg-green-500/20 border border-green-500 rounded">
                        <span className="text-sm font-bold text-green-300 font-mono">
                          {friendlyKeyName(tapAction)}
                        </span>
                      </div>
                      <button
                        onClick={() => setTapAction('')}
                        className="p-1 text-slate-400 hover:text-red-400 transition-colors"
                        title={t('kc.clearSel')}
                        aria-label={t('kc.clearSel')}
                      >
                        <X className="w-4 h-4" />
                      </button>
                    </div>
                  )}
                </div>
                <KeySelectionTabs
                  activeTab={activeTab}
                  onTabChange={setActiveTab}
                  availableTabs={['keyboard', 'modifier', 'lock']}
                  onKeySelect={setTapAction}
                  layoutKeys={layoutKeys}
                  maxHeight="max-h-64"
                />
              </div>

              {/* Hold Action - Numerical Selector */}
              <div>
                <div className="flex items-center justify-between mb-3">
                  <div>
                    <label className="text-sm font-medium text-slate-300">
                      {t('kc.holdAction')}
                    </label>
                    <p className="text-xs text-slate-400 mt-1">
                      {t('kc.holdHelp')}
                    </p>
                  </div>
                  {holdAction && (
                    <div className="flex items-center gap-2">
                      <div className="px-3 py-1 bg-red-500/20 border border-red-500 rounded">
                        <span className="text-sm font-bold text-red-300 font-mono">
                          {holdAction}
                        </span>
                      </div>
                      <button
                        onClick={() => setHoldAction('')}
                        className="p-1 text-slate-400 hover:text-red-400 transition-colors"
                        title={t('kc.clearSel')}
                        aria-label={t('kc.clearSel')}
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
                    value={
                      holdAction
                        ? parseInt(holdAction.replace('MD_', ''), 16)
                        : 0
                    }
                    onChange={(e) => {
                      const val = Math.max(
                        0,
                        Math.min(255, parseInt(e.target.value) || 0)
                      );
                      const hex = val
                        .toString(16)
                        .toUpperCase()
                        .padStart(2, '0');
                      setHoldAction(`MD_${hex}`);
                    }}
                    className="w-full px-4 py-2 bg-slate-800 border border-slate-600 rounded-md text-slate-100 focus:outline-none focus:ring-2 focus:ring-primary-500"
                    placeholder={t('kc.holdPlaceholder')}
                    aria-label={t('kc.holdAction')}
                  />
                  <div className="mt-3">
                    <input
                      type="range"
                      min="0"
                      max="255"
                      value={
                        holdAction
                          ? parseInt(holdAction.replace('MD_', ''), 16)
                          : 0
                      }
                      onChange={(e) => {
                        const val = parseInt(e.target.value);
                        const hex = val
                          .toString(16)
                          .toUpperCase()
                          .padStart(2, '0');
                        setHoldAction(`MD_${hex}`);
                      }}
                      className="w-full"
                    />
                    <div className="flex justify-between text-xs text-slate-400 mt-1">
                      <span>0 (MD_00)</span>
                      <span>255 (MD_FF)</span>
                    </div>
                  </div>
                </div>
              </div>

              {/* Threshold */}
              <div>
                <label className="block text-sm font-medium text-slate-300 mb-2">
                  {t('kc.threshold', { ms: threshold })}
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
                <div className="flex justify-between text-xs text-slate-400 mt-1">
                  <span>{t('kc.fast')}</span>
                  <span>{t('kc.slow')}</span>
                </div>
              </div>
            </>
          )}

          {/* Preview Panel */}
          <div className="bg-slate-800 border border-slate-700 rounded-lg p-4">
            <div className="flex items-center gap-2 mb-2">
              <ArrowRight className="w-4 h-4 text-primary-400" />
              <label className="text-sm font-medium text-slate-300">
                {t('kc.preview')}
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
                {t('kc.clearMapping')}
              </button>
            )}
            <button
              onClick={handleSave}
              disabled={isSaveDisabled}
              className="px-6 py-2 bg-primary-500 text-white rounded-md hover:bg-primary-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors font-medium"
            >
              {t('kc.saveMapping')}
            </button>
          </div>
        </>
      )}

      {/* Current Mappings Summary (owns the one "Current Mappings" heading) */}
      <div className="border-t border-slate-700 pt-6">
        <CurrentMappingsSummary
          keyMappings={keyMappings}
          onEditMapping={onEditMapping}
          onClearMapping={onClearMapping}
        />
      </div>
    </div>
  );
}
