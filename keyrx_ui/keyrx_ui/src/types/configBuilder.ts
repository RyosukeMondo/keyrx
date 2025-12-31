/**
 * Type definitions for the Visual Configuration Builder
 *
 * These types represent the state of the visual config builder,
 * which can be serialized to Rhai code and imported from existing Rhai configs.
 */

/**
 * Type of mapping between source and target keys
 */
export type MappingType = 'simple' | 'modifier_trigger' | 'layer_switch';

/**
 * Represents a single key mapping
 */
export interface Mapping {
  /** Unique identifier for this mapping */
  id: string;
  /** Source key code (e.g., "KEY_Q") */
  sourceKey: string;
  /** Target key code (e.g., "KEY_A") */
  targetKey: string;
  /** Type of mapping */
  type: MappingType;
  /** For layer_switch type: target layer ID */
  targetLayerId?: string;
  /** For modifier_trigger type: modifier ID */
  modifierId?: string;
}

/**
 * Represents a keyboard layer
 */
export interface Layer {
  /** Unique identifier for this layer */
  id: string;
  /** Human-readable name (e.g., "base", "symbols") */
  name: string;
  /** All key mappings for this layer */
  mappings: Mapping[];
  /** Whether this is the default/base layer */
  isBase?: boolean;
}

/**
 * Represents a modifier (e.g., custom Shift, Ctrl)
 */
export interface Modifier {
  /** Unique identifier for this modifier */
  id: string;
  /** Human-readable name (e.g., "my_shift") */
  name: string;
  /** Key code that activates this modifier */
  triggerKey: string;
  /** Whether this modifier is currently active (UI state) */
  active?: boolean;
}

/**
 * Represents a lock (toggle state)
 */
export interface Lock {
  /** Unique identifier for this lock */
  id: string;
  /** Human-readable name (e.g., "caps_lock") */
  name: string;
  /** Key code that toggles this lock */
  triggerKey: string;
  /** Whether this lock is currently active (UI state) */
  active?: boolean;
}

/**
 * Complete configuration state for the visual builder
 */
export interface ConfigState {
  /** All layers in the configuration */
  layers: Layer[];
  /** All modifiers in the configuration */
  modifiers: Modifier[];
  /** All locks in the configuration */
  locks: Lock[];
  /** Currently selected layer ID (UI state) */
  currentLayerId?: string;
  /** Whether the config has unsaved changes (UI state) */
  isDirty?: boolean;
}

/**
 * Result of parsing a Rhai file
 */
export interface RhaiParseResult {
  /** Parsed configuration state */
  config: ConfigState;
  /** List of unsupported features found in the Rhai code */
  warnings: string[];
  /** Critical errors that prevented full parsing */
  errors: string[];
}

/**
 * Options for Rhai code generation
 */
export interface RhaiGenerateOptions {
  /** Whether to include comments in the generated code */
  includeComments?: boolean;
  /** Indentation string (default: 2 spaces) */
  indent?: string;
  /** Whether to format the code with extra whitespace */
  pretty?: boolean;
}

/**
 * Drag and drop item types for @dnd-kit
 */
export type DragItemType = 'key' | 'layer' | 'modifier' | 'lock';

/**
 * Data carried during drag operations
 */
export interface DragData {
  /** Type of item being dragged */
  type: DragItemType;
  /** Key code for key drags */
  keyCode?: string;
  /** Layer ID for layer drags */
  layerId?: string;
  /** Modifier ID for modifier drags */
  modifierId?: string;
  /** Lock ID for lock drags */
  lockId?: string;
}

/**
 * Validation result for a configuration
 */
export interface ValidationResult {
  /** Whether the configuration is valid */
  isValid: boolean;
  /** List of validation errors */
  errors: string[];
  /** List of validation warnings */
  warnings: string[];
}

/**
 * Keyboard layout information
 */
export interface KeyboardLayout {
  /** Name of the layout (e.g., "QWERTY", "DVORAK") */
  name: string;
  /** Rows of keys with their positions */
  rows: KeyRow[];
}

/**
 * A row of keys in the keyboard layout
 */
export interface KeyRow {
  /** Keys in this row */
  keys: KeyInfo[];
  /** Vertical offset from top (in key units) */
  offsetY?: number;
  /** Left margin (in key units) */
  marginLeft?: number;
}

/**
 * Information about a single key
 */
export interface KeyInfo {
  /** Key code (e.g., "KEY_Q") */
  code: string;
  /** Display label (e.g., "Q") */
  label: string;
  /** Width in key units (default: 1.0) */
  width?: number;
  /** Whether this is a modifier key */
  isModifier?: boolean;
  /** Whether this is a special key (Esc, Enter, etc.) */
  isSpecial?: boolean;
}
