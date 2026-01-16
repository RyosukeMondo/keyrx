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
import { useKeyboardLayout } from '@/hooks/useKeyboardLayout';
import type { KeyMapping } from '@/types';

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
  /** Initial keyboard layout type */
  initialLayout?: LayoutType;
  /** Optional CSS class name */
  className?: string;
}

/**
 * KeyboardVisualizerContainer
 *
 * Manages keyboard layout selection and displays the keyboard visualizer.
 * Uses useKeyboardLayout hook to manage layout state and parsed layout keys.
 *
 * @example
 * ```tsx
 * <KeyboardVisualizerContainer
 *   profileName="Default"
 *   activeLayer="base"
 *   mappings={keyMappings}
 *   onKeyClick={handleKeyClick}
 *   selectedKeyCode="VK_A"
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
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  selectedKeyCode,
  initialLayout = 'ANSI_104',
  className = '',
}) => {
  const { layout, setLayout } = useKeyboardLayout(initialLayout);

  return (
    <div className={className}>
      {/* Layout Selector */}
      <div className="flex items-center gap-3 mb-4">
        <label
          htmlFor="layout-selector"
          className="text-sm font-medium text-slate-300 whitespace-nowrap"
        >
          Layout:
        </label>
        <select
          id="layout-selector"
          value={layout}
          onChange={(e) => setLayout(e.target.value as LayoutType)}
          className="px-3 py-2 bg-slate-700 border border-slate-600 rounded-md text-slate-100 text-sm font-medium focus:outline-none focus:ring-2 focus:ring-primary-500"
          aria-label="Select keyboard layout"
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
      <div className="overflow-x-auto p-4">
        <div className="flex justify-center min-w-fit">
          <KeyboardVisualizer
            layout={layout}
            keyMappings={mappings}
            onKeyClick={onKeyClick}
            simulatorMode={false}
          />
        </div>
      </div>

      {/* Helper text */}
      <p className="text-center text-sm text-slate-400 mt-4">
        Click any key to configure mappings
      </p>
    </div>
  );
};
