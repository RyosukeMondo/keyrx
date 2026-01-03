/**
 * Type definitions for KeyRx configuration and drag-and-drop functionality.
 *
 * These types represent the data models used in the visual configuration editor,
 * including key mappings, assignable keys, and keyboard layers.
 */

/**
 * Represents a key mapping configuration for a physical key.
 *
 * A key mapping defines how a physical key press should be interpreted and
 * what action(s) should be performed. Supports simple mappings, tap-hold
 * dual-function keys, macros, and layer switches.
 *
 * @example
 * // Simple mapping: CapsLock → A
 * const simpleMapping: KeyMapping = {
 *   keyCode: 'CapsLock',
 *   type: 'simple',
 *   simple: 'VK_A'
 * };
 *
 * @example
 * // Tap-hold mapping: CapsLock → Tap:Escape, Hold:Control
 * const tapHoldMapping: KeyMapping = {
 *   keyCode: 'CapsLock',
 *   type: 'tap_hold',
 *   tapHold: {
 *     tap: 'VK_ESCAPE',
 *     hold: 'MD_CTRL',
 *     timeoutMs: 200
 *   }
 * };
 */
export interface KeyMapping {
  /** Physical key code (e.g., "CapsLock", "A", "Enter") */
  keyCode: string;

  /** Type of key mapping */
  type: 'simple' | 'tap_hold' | 'macro' | 'layer_switch';

  /**
   * Simple key mapping - maps to a single virtual key.
   *
   * Examples: "VK_A", "VK_ENTER", "MD_SHIFT"
   * Used when type === 'simple'
   */
  simple?: string;

  /**
   * Tap-hold configuration - different actions for tap vs hold.
   *
   * Tap action fires if key is released before timeout.
   * Hold action fires if key is held longer than timeout.
   * Used when type === 'tap_hold'
   */
  tapHold?: {
    /** Action to perform on tap (quick press) */
    tap: string;
    /** Action to perform on hold (sustained press) */
    hold: string;
    /** Threshold in milliseconds (100-500ms) */
    timeoutMs: number;
  };

  /**
   * Macro sequence - array of key codes to emit in sequence.
   *
   * Example: ["VK_H", "VK_E", "VK_L", "VK_L", "VK_O"] types "HELLO"
   * Used when type === 'macro'
   */
  macro?: string[];

  /**
   * Layer name for layer switch.
   *
   * Example: "nav" switches to navigation layer
   * Used when type === 'layer_switch'
   */
  layer?: string;
}

/**
 * Represents a draggable virtual key in the key palette.
 *
 * AssignableKeys are displayed in the DragKeyPalette component and can be
 * dragged onto keyboard keys to create mappings.
 *
 * @example
 * const vkA: AssignableKey = {
 *   id: 'VK_A',
 *   category: 'vk',
 *   label: 'A',
 *   description: 'Virtual Key A'
 * };
 */
export interface AssignableKey {
  /**
   * Unique identifier for the key.
   *
   * Examples:
   * - Virtual keys: "VK_A", "VK_ENTER", "VK_ESCAPE"
   * - Modifiers: "MD_SHIFT", "MD_CTRL", "MD_ALT"
   * - Locks: "LK_CAPSLOCK", "LK_NUMLOCK"
   * - Layers: "layer_nav", "layer_num"
   */
  id: string;

  /**
   * Category for filtering and organization.
   *
   * - vk: Virtual keys (letters, numbers, symbols)
   * - modifier: Modifier keys (Shift, Ctrl, Alt, Super)
   * - lock: Lock keys (CapsLock, NumLock, ScrollLock)
   * - layer: Layer switches
   * - macro: Saved macros
   */
  category: 'vk' | 'modifier' | 'lock' | 'layer' | 'macro';

  /** Display label shown in the palette */
  label: string;

  /** Tooltip description for the key */
  description: string;

  /** Optional icon to display alongside label */
  icon?: React.ReactNode;
}

/**
 * Represents a keyboard layer with its key mappings.
 *
 * Layers allow multiple key mapping configurations that can be switched
 * between dynamically. Similar to Vim modes or QMK layers.
 *
 * @example
 * const navLayer: Layer = {
 *   id: 'nav',
 *   name: 'Navigation',
 *   mappings: new Map([
 *     ['H', { keyCode: 'H', type: 'simple', simple: 'VK_LEFT' }],
 *     ['J', { keyCode: 'J', type: 'simple', simple: 'VK_DOWN' }],
 *     ['K', { keyCode: 'K', type: 'simple', simple: 'VK_UP' }],
 *     ['L', { keyCode: 'L', type: 'simple', simple: 'VK_RIGHT' }]
 *   ])
 * };
 */
export interface Layer {
  /**
   * Layer identifier (lowercase, no spaces).
   *
   * Examples: "base", "nav", "num", "fn"
   */
  id: string;

  /** Display name for the layer */
  name: string;

  /**
   * Key mappings specific to this layer.
   *
   * Map key: Physical key code (e.g., "CapsLock", "A")
   * Map value: KeyMapping configuration
   */
  mappings: Map<string, KeyMapping>;
}

/**
 * Mapping scope - determines whether a mapping is global or device-specific.
 */
export type MappingScope = 'global' | 'device-specific';

/**
 * Device option for device selector dropdown.
 */
export interface DeviceOption {
  /** Device serial number or unique identifier */
  serial: string;
  /** Human-readable device name */
  name: string;
}
