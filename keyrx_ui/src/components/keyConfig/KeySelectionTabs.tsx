import React from 'react';
import { Keyboard, Command, Lock, Layers, type LucideIcon } from 'lucide-react';
import { SVGKeyboard, type SVGKey } from '../SVGKeyboard';
import { IdGrid } from './IdGrid';
import { t, type MessageKey } from '@/i18n';

/**
 * KeySelectionTabs Component
 *
 * Tabbed interface for key selection with different categories (keyboard, modifier, lock, layer).
 * Used by KeyConfigPanel for inline key selection.
 *
 * @example
 * // Panel usage - keyboard, modifier, lock tabs
 * <KeySelectionTabs
 *   activeTab="keyboard"
 *   onTabChange={setActiveTab}
 *   availableTabs={['keyboard', 'modifier', 'lock']}
 *   onKeySelect={(keyCode) => setTapAction(keyCode)}
 *   layoutKeys={layoutKeys}
 * />
 *
 * @example
 * // With layer tab
 * <KeySelectionTabs
 *   activeTab="layer"
 *   onTabChange={setActiveTab}
 *   availableTabs={['keyboard', 'layer']}
 *   onKeySelect={(layerId) => setTargetLayer(layerId)}
 *   layoutKeys={layoutKeys}
 * />
 */

export type KeySelectionTab = 'keyboard' | 'modifier' | 'lock' | 'layer';

interface TabConfig {
  icon: LucideIcon;
  label: MessageKey;
  ariaLabel: MessageKey;
}

const TAB_CONFIG: Record<KeySelectionTab, TabConfig> = {
  keyboard: {
    icon: Keyboard,
    label: 'tabs.keyboard',
    ariaLabel: 'tabs.keyboard.aria',
  },
  modifier: {
    icon: Command,
    label: 'tabs.modifier',
    ariaLabel: 'tabs.modifier.aria',
  },
  lock: {
    icon: Lock,
    label: 'tabs.lock',
    ariaLabel: 'tabs.lock.aria',
  },
  layer: {
    icon: Layers,
    label: 'tabs.layer',
    ariaLabel: 'tabs.layer.aria',
  },
};

export interface KeySelectionTabsProps {
  /** Currently active tab */
  activeTab: KeySelectionTab;
  /** Callback when tab is changed */
  onTabChange: (tab: KeySelectionTab) => void;
  /** List of available tabs to display */
  availableTabs: KeySelectionTab[];
  /** Callback when a key is selected */
  onKeySelect: (keyCode: string) => void;
  /** Optional layout keys for keyboard tab (SVGKeyboard) */
  layoutKeys?: SVGKey[];
  /** Optional max height for tab content (default: 96 = 384px) */
  maxHeight?: string;
}

const hex2 = (i: number) => i.toString(16).toUpperCase().padStart(2, '0');

const LOCK_LABELS: Record<string, string> = {
  LK_00: 'CapsLock',
  LK_01: 'NumLock',
  LK_02: 'ScrollLock',
};

const MODIFIER_ITEMS = Array.from({ length: 256 }, (_, i) => ({
  id: `MD_${hex2(i)}`,
  label: hex2(i),
  ariaLabel: (): string => t('tabs.modifier.item', { id: hex2(i) }),
}));

const LOCK_ITEMS = Array.from({ length: 256 }, (_, i) => {
  const id = `LK_${hex2(i)}`;
  const label = LOCK_LABELS[id] || hex2(i);
  return {
    id,
    label,
    ariaLabel: (): string => t('tabs.lock.item', { id: label }),
  };
});

/** Custom modifier picker (MD_00 to MD_FF) */
function renderModifierGrid(onSelect: (id: string) => void) {
  return (
    <div className="border border-slate-600 rounded-lg p-4 bg-slate-900 max-h-96 overflow-y-auto">
      <p className="text-xs text-slate-400 mb-3">{t('tabs.modifier.help')}</p>
      <IdGrid
        items={MODIFIER_ITEMS}
        columns={8}
        onSelect={onSelect}
        label={t('tabs.modifier.group')}
      />
    </div>
  );
}

/** Lock state picker (LK_00 to LK_FF) */
function renderLockGrid(onSelect: (id: string) => void) {
  return (
    <div className="border border-slate-600 rounded-lg p-4 bg-slate-900 max-h-96 overflow-y-auto">
      <p className="text-xs text-slate-400 mb-3">{t('tabs.lock.help')}</p>
      <IdGrid
        items={LOCK_ITEMS}
        columns={8}
        onSelect={onSelect}
        label={t('tabs.lock.group')}
      />
    </div>
  );
}

/**
 * KeySelectionTabs - Displays tabbed interface for selecting keys from different categories
 *
 * Renders tab buttons and corresponding content for each category.
 * Supports keyboard layout visualization via SVGKeyboard, and grid-based selection for modifiers/locks.
 */
export function KeySelectionTabs({
  activeTab,
  onTabChange,
  availableTabs,
  onKeySelect,
  layoutKeys = [],
  maxHeight = 'max-h-96',
}: KeySelectionTabsProps) {
  return (
    <div>
      {/* Tab Buttons */}
      <div
        role="tablist"
        aria-label={t('tabs.aria')}
        className="flex gap-2 mb-4 border-b border-slate-700"
      >
        {availableTabs.map((tab) => {
          const config = TAB_CONFIG[tab];
          const Icon = config.icon;
          const isActive = activeTab === tab;

          return (
            <button
              key={tab}
              role="tab"
              aria-selected={isActive}
              aria-controls={`${tab}-panel`}
              id={`${tab}-tab`}
              onClick={() => onTabChange(tab)}
              className={`px-4 py-2 text-sm font-medium transition-colors border-b-2 ${
                isActive
                  ? 'text-primary-400 border-primary-400'
                  : 'text-slate-400 border-transparent hover:text-slate-300'
              }`}
              aria-label={t(config.ariaLabel)}
            >
              <Icon className="w-4 h-4 inline-block mr-2" aria-hidden="true" />
              {t(config.label)}
            </button>
          );
        })}
      </div>

      {/* Tab Content */}
      <div>
        {/* Keyboard Tab */}
        {activeTab === 'keyboard' && layoutKeys.length > 0 && (
          <div
            role="tabpanel"
            id="keyboard-panel"
            aria-labelledby="keyboard-tab"
            className={`border border-slate-600 rounded-lg overflow-auto ${maxHeight} bg-slate-900`}
          >
            <SVGKeyboard
              keys={layoutKeys}
              keyMappings={new Map()}
              onKeyClick={onKeySelect}
              className="w-full"
            />
          </div>
        )}

        {/* Keyboard Tab - No Layout */}
        {activeTab === 'keyboard' && layoutKeys.length === 0 && (
          <div
            role="tabpanel"
            id="keyboard-panel"
            aria-labelledby="keyboard-tab"
            className="border border-slate-600 rounded-lg p-4 bg-slate-900 text-center"
          >
            <p className="text-slate-400 text-sm">{t('tabs.noLayout')}</p>
          </div>
        )}

        {/* Modifier Tab */}
        {activeTab === 'modifier' && (
          <div
            role="tabpanel"
            id="modifier-panel"
            aria-labelledby="modifier-tab"
          >
            {renderModifierGrid(onKeySelect)}
          </div>
        )}

        {/* Lock Tab */}
        {activeTab === 'lock' && (
          <div role="tabpanel" id="lock-panel" aria-labelledby="lock-tab">
            {renderLockGrid(onKeySelect)}
          </div>
        )}

        {/* Layer Tab */}
        {activeTab === 'layer' && (
          <div role="tabpanel" id="layer-panel" aria-labelledby="layer-tab">
            <div className="border border-slate-600 rounded-lg p-4 bg-slate-900">
              <p className="text-xs text-slate-400 mb-3">
                {t('tabs.layer.help')}
              </p>
              <div className="grid grid-cols-8 gap-2">
                {Array.from({ length: 16 }, (_, i) => {
                  const layerId = `layer_${i}`;
                  return (
                    <button
                      key={layerId}
                      onClick={() => onKeySelect(layerId)}
                      className="px-3 py-2 bg-slate-700 hover:bg-primary-500 text-slate-300 hover:text-white rounded text-sm font-mono transition-colors"
                      aria-label={t('tabs.layer.item', { n: i })}
                    >
                      {i}
                    </button>
                  );
                })}
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
