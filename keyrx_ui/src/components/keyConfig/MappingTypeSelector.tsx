import React from 'react';
import {
  MousePointerClick,
  Timer,
  Lock,
  Command,
  Layers,
  type LucideIcon,
} from 'lucide-react';

/**
 * MappingTypeSelector Component
 *
 * Reusable component for selecting key mapping types (simple, modifier, lock, tap_hold, layer_active).
 * Used by both KeyConfigModal (all 5 types) and KeyConfigPanel (simple + tap_hold only).
 *
 * @example
 * // Modal usage - all types
 * <MappingTypeSelector
 *   selectedType="simple"
 *   onChange={setMappingType}
 *   supportedTypes={['simple', 'modifier', 'lock', 'tap_hold', 'layer_active']}
 * />
 *
 * @example
 * // Panel usage - simplified types
 * <MappingTypeSelector
 *   selectedType="simple"
 *   onChange={setMappingType}
 *   supportedTypes={['simple', 'tap_hold']}
 * />
 *
 * @example
 * // Vertical layout
 * <MappingTypeSelector
 *   selectedType="tap_hold"
 *   onChange={setMappingType}
 *   supportedTypes={['simple', 'tap_hold']}
 *   layout="vertical"
 * />
 */

export type MappingType =
  | 'simple'
  | 'modifier'
  | 'lock'
  | 'tap_hold'
  | 'layer_active';

interface MappingTypeConfig {
  icon: LucideIcon;
  label: string;
  description: string;
}

const MAPPING_TYPE_CONFIG: Record<MappingType, MappingTypeConfig> = {
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
};

export interface MappingTypeSelectorProps {
  /** Currently selected mapping type */
  selectedType: MappingType;
  /** Callback when mapping type is changed */
  onChange: (type: MappingType) => void;
  /** List of supported mapping types to display */
  supportedTypes: MappingType[];
  /** Layout direction - horizontal (default) or vertical */
  layout?: 'horizontal' | 'vertical';
}

/**
 * MappingTypeSelector - Displays mapping type options as interactive buttons
 *
 * Renders icon + label + description for each supported mapping type.
 * Highlights the selected type and calls onChange when a type is clicked.
 */
export function MappingTypeSelector({
  selectedType,
  onChange,
  supportedTypes,
  layout = 'horizontal',
}: MappingTypeSelectorProps) {
  return (
    <div
      className={`flex items-center gap-3 ${
        layout === 'vertical' ? 'flex-col items-start' : ''
      }`}
    >
      <label className="text-xs font-medium text-slate-400 uppercase tracking-wider whitespace-nowrap">
        Type
      </label>
      <div
        role="radiogroup"
        aria-label="Mapping type"
        className={`flex gap-2 ${
          layout === 'vertical' ? 'flex-col w-full' : 'flex-wrap'
        }`}
      >
        {supportedTypes.map((type) => {
          const config = MAPPING_TYPE_CONFIG[type];
          const Icon = config.icon;
          const isSelected = selectedType === type;

          return (
            <button
              key={type}
              type="button"
              role="radio"
              aria-checked={isSelected}
              onClick={() => onChange(type)}
              className={`px-3 py-1.5 rounded text-xs font-medium transition-all flex items-center gap-1.5 ${
                isSelected
                  ? 'bg-primary-500 text-white'
                  : 'bg-slate-700 text-slate-300 hover:bg-slate-600'
              } ${layout === 'vertical' ? 'w-full justify-start' : ''}`}
              title={config.description}
              aria-label={`${config.label}: ${config.description}`}
            >
              <Icon className="w-3.5 h-3.5" aria-hidden="true" />
              <span>{config.label}</span>
            </button>
          );
        })}
      </div>
    </div>
  );
}
