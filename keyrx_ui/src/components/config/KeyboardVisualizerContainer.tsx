/**
 * KeyboardVisualizerContainer Component
 *
 * Container component that wraps KeyboardVisualizer with layout management.
 * Provides layout selection dropdown and handles keyboard visualization display.
 *
 * @component
 */

import React from 'react';
import {
  KeyboardVisualizer,
  type LayoutType,
} from '@/components/KeyboardVisualizer';
import type { KeyMapping } from '@/types';
import { t } from '@/i18n';

export interface KeyboardVisualizerContainerProps {
  /** Currently active profile name */
  profileName: string;
  /** Active layer ID (e.g., 'base', 'md-00') */
  activeLayer: string;
  /** Key mappings for the current layer */
  mappings: Map<string, KeyMapping>;
  /** Callback when a key is clicked */
  onKeyClick: (keyCode: string) => void;
  /** Currently selected key code (optional) */
  selectedKeyCode?: string | null;
  /**
   * Selected keyboard layout. Controlled by the parent (ConfigPage owns the
   * single source of truth: it auto-detects the layout from the profile's
   * Rhai source and lets the user override it) -- this component used to
   * keep its own copy via `useKeyboardLayout(initialLayout)`, seeded only
   * once from an `initialLayout` prop, so it silently went stale the moment
   * the real detected layout changed after the initial render (e.g. once
   * the profile config finished loading). Passing `layout` as a controlled
   * prop instead removes that second, driftable source of truth.
   */
  layout: LayoutType;
  /** Called when the user picks a different layout from the dropdown */
  onLayoutChange: (layout: LayoutType) => void;
  /** Optional CSS class name */
  className?: string;
}

/**
 * KeyboardVisualizerContainer
 *
 * Renders the layout selection dropdown and the keyboard visualizer for a
 * given (parent-controlled) layout.
 *
 * @example
 * ```tsx
 * <KeyboardVisualizerContainer
 *   profileName="Default"
 *   activeLayer="base"
 *   mappings={keyMappings}
 *   onKeyClick={handleKeyClick}
 *   selectedKeyCode="VK_A"
 *   layout={layout}
 *   onLayoutChange={setLayout}
 * />
 * ```
 */
export const KeyboardVisualizerContainer: React.FC<
  KeyboardVisualizerContainerProps
> = ({
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  profileName,
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  activeLayer,
  mappings,
  onKeyClick,
  selectedKeyCode,
  layout,
  onLayoutChange,
  className = '',
}) => {
  return (
    <div className={className}>
      {/* Layout Selector */}
      <div className="flex items-center gap-3 mb-4">
        <label
          htmlFor="layout-selector"
          className="text-sm font-medium text-slate-300 whitespace-nowrap"
        >
          {t('layout.label')}
        </label>
        <select
          id="layout-selector"
          value={layout}
          onChange={(e) => onLayoutChange(e.target.value as LayoutType)}
          className="px-3 py-2 bg-slate-700 border border-slate-600 rounded-md text-slate-100 text-sm font-medium focus:outline-none focus:ring-2 focus:ring-primary-500"
          aria-label={t('layout.aria')}
        >
          <option value="ANSI_104">ANSI Full (104)</option>
          <option value="ANSI_87">ANSI TKL (87)</option>
          <option value="ISO_105">ISO Full (105)</option>
          <option value="ISO_88">ISO TKL (88)</option>
          <option value="JIS_109">JIS (109)</option>
          <option value="COMPACT_60">60% Compact</option>
          <option value="COMPACT_65">65% Compact</option>
          <option value="COMPACT_75">75% Compact</option>
          <option value="COMPACT_96">96% Compact</option>
          <option value="HHKB">HHKB</option>
          <option value="NUMPAD">Numpad</option>
        </select>
      </div>

      {/* Keyboard Visualizer */}
      <div className="overflow-x-auto p-1 2xl:p-4">
        <div className="mx-auto w-fit">
          <KeyboardVisualizer
            layout={layout}
            keyMappings={mappings}
            onKeyClick={onKeyClick}
            simulatorMode={false}
            selectedKeyCode={selectedKeyCode}
          />
        </div>
      </div>

      {/* Helper text */}
      <p className="text-center text-sm text-slate-400 mt-4">
        {t('layout.help')}
      </p>
    </div>
  );
};
