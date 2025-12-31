/**
 * Built-in test scenario generators for keyboard event simulation.
 *
 * This module provides ready-to-use test scenarios for common keyboard patterns
 * like tap-hold, layer switching, and modifier combinations.
 *
 * @module utils/scenarios
 */

import type { EventSequence, SimKeyEvent } from '../wasm/core';

// ============================================================================
// Scenario Generator Options
// ============================================================================

/**
 * Options for tap-hold scenarios.
 */
export interface TapHoldOptions {
  /** The key to test (default: "VK_Space") */
  key?: string;
  /** Tap-hold threshold in microseconds (default: 200000 = 200ms) */
  threshold?: number;
  /** Duration to hold the key in microseconds */
  holdDuration?: number;
}

/**
 * Options for layer switch scenarios.
 */
export interface LayerSwitchOptions {
  /** The layer modifier key (default: "MD_00") */
  layerKey?: string;
  /** The key to press while layer is active (default: "VK_A") */
  targetKey?: string;
  /** Delay between layer activation and key press (default: 50000 = 50ms) */
  delay?: number;
}

/**
 * Options for modifier combo scenarios.
 */
export interface ModifierComboOptions {
  /** The modifiers to activate (default: ["VK_LeftShift", "VK_LeftControl"]) */
  modifiers?: string[];
  /** The target key to press with modifiers (default: "VK_A") */
  targetKey?: string;
  /** Delay between modifier presses (default: 10000 = 10ms) */
  modifierDelay?: number;
  /** Delay before target key press (default: 20000 = 20ms) */
  targetDelay?: number;
}

// ============================================================================
// Scenario Generators
// ============================================================================

/**
 * Generate a tap-hold scenario where the key is released BEFORE the threshold.
 *
 * This tests the "tap" behavior - the key should be interpreted as a tap
 * rather than a hold because it's released within the threshold time.
 *
 * Expected behavior: Key generates tap output (not hold output).
 *
 * @param options - Configuration options
 * @returns Event sequence for tap-hold-under scenario
 */
export function generateTapHoldUnder(
  options: TapHoldOptions = {}
): EventSequence {
  const {
    key = 'VK_Space',
    threshold = 200000, // 200ms in microseconds
    holdDuration = threshold - 50000 // 50ms before threshold
  } = options;

  if (holdDuration >= threshold) {
    throw new Error('holdDuration must be less than threshold for tap-hold-under scenario');
  }

  if (holdDuration < 0) {
    throw new Error('holdDuration must be non-negative');
  }

  const events: SimKeyEvent[] = [
    {
      keycode: key,
      event_type: 'press',
      timestamp_us: 0
    },
    {
      keycode: key,
      event_type: 'release',
      timestamp_us: holdDuration
    }
  ];

  return { events };
}

/**
 * Generate a tap-hold scenario where the key is released AFTER the threshold.
 *
 * This tests the "hold" behavior - the key should be interpreted as a hold
 * because it's held longer than the threshold time.
 *
 * Expected behavior: Key generates hold output (not tap output).
 *
 * @param options - Configuration options
 * @returns Event sequence for tap-hold-over scenario
 */
export function generateTapHoldOver(
  options: TapHoldOptions = {}
): EventSequence {
  const {
    key = 'VK_Space',
    threshold = 200000, // 200ms in microseconds
    holdDuration = threshold + 50000 // 50ms after threshold
  } = options;

  if (holdDuration <= threshold) {
    throw new Error('holdDuration must be greater than threshold for tap-hold-over scenario');
  }

  const events: SimKeyEvent[] = [
    {
      keycode: key,
      event_type: 'press',
      timestamp_us: 0
    },
    {
      keycode: key,
      event_type: 'release',
      timestamp_us: holdDuration
    }
  ];

  return { events };
}

/**
 * Generate a layer switch scenario.
 *
 * This tests layer activation - press a layer modifier key, then press
 * a target key while the layer is active, then release both keys.
 *
 * Expected behavior: The target key should produce output according to
 * the layer mapping (not the base layer mapping).
 *
 * @param options - Configuration options
 * @returns Event sequence for layer-switch scenario
 */
export function generateLayerSwitch(
  options: LayerSwitchOptions = {}
): EventSequence {
  const {
    layerKey = 'MD_00',
    targetKey = 'VK_A',
    delay = 50000 // 50ms
  } = options;

  if (delay < 0) {
    throw new Error('delay must be non-negative');
  }

  const events: SimKeyEvent[] = [
    // Press layer modifier
    {
      keycode: layerKey,
      event_type: 'press',
      timestamp_us: 0
    },
    // Press target key (while layer is active)
    {
      keycode: targetKey,
      event_type: 'press',
      timestamp_us: delay
    },
    // Release target key
    {
      keycode: targetKey,
      event_type: 'release',
      timestamp_us: delay + 100000 // Hold for 100ms
    },
    // Release layer modifier
    {
      keycode: layerKey,
      event_type: 'release',
      timestamp_us: delay + 150000
    }
  ];

  return { events };
}

/**
 * Generate a modifier combination scenario.
 *
 * This tests modifier key combinations (e.g., Shift+Ctrl+A) - press
 * multiple modifiers in sequence, then press a target key, then release
 * all keys in reverse order.
 *
 * Expected behavior: The target key should produce output with all
 * modifiers applied (e.g., Shift+Ctrl+A rather than just A).
 *
 * @param options - Configuration options
 * @returns Event sequence for modifier-combo scenario
 */
export function generateModifierCombo(
  options: ModifierComboOptions = {}
): EventSequence {
  const {
    modifiers = ['VK_LeftShift', 'VK_LeftControl'],
    targetKey = 'VK_A',
    modifierDelay = 10000, // 10ms between modifier presses
    targetDelay = 20000 // 20ms before target key
  } = options;

  if (modifiers.length === 0) {
    throw new Error('modifiers array cannot be empty');
  }

  if (modifierDelay < 0) {
    throw new Error('modifierDelay must be non-negative');
  }

  if (targetDelay < 0) {
    throw new Error('targetDelay must be non-negative');
  }

  const events: SimKeyEvent[] = [];
  let timestamp = 0;

  // Press all modifiers in sequence
  for (const modifier of modifiers) {
    events.push({
      keycode: modifier,
      event_type: 'press',
      timestamp_us: timestamp
    });
    timestamp += modifierDelay;
  }

  // Wait before pressing target key
  timestamp += targetDelay;

  // Press target key
  events.push({
    keycode: targetKey,
    event_type: 'press',
    timestamp_us: timestamp
  });

  // Release target key after 50ms
  timestamp += 50000;
  events.push({
    keycode: targetKey,
    event_type: 'release',
    timestamp_us: timestamp
  });

  // Release all modifiers in reverse order
  timestamp += 10000;
  for (let i = modifiers.length - 1; i >= 0; i--) {
    events.push({
      keycode: modifiers[i],
      event_type: 'release',
      timestamp_us: timestamp
    });
    timestamp += modifierDelay;
  }

  return { events };
}

// ============================================================================
// Scenario Metadata
// ============================================================================

/**
 * Metadata for a built-in scenario.
 */
export interface ScenarioMetadata {
  /** Unique identifier */
  id: string;
  /** Display name */
  name: string;
  /** Description of what the scenario tests */
  description: string;
  /** Generator function */
  generator: () => EventSequence;
}

/**
 * List of all built-in scenarios with metadata.
 */
export const BUILT_IN_SCENARIOS: ScenarioMetadata[] = [
  {
    id: 'tap-hold-under',
    name: 'Tap-Hold (Under Threshold)',
    description: 'Tests tap behavior by pressing and releasing a key within 200ms threshold. Expected: Key generates tap output.',
    generator: () => generateTapHoldUnder()
  },
  {
    id: 'tap-hold-over',
    name: 'Tap-Hold (Over Threshold)',
    description: 'Tests hold behavior by pressing a key for longer than 200ms threshold. Expected: Key generates hold output.',
    generator: () => generateTapHoldOver()
  },
  {
    id: 'layer-switch',
    name: 'Layer Switch',
    description: 'Tests layer activation by pressing a layer modifier then a target key. Expected: Target key produces layer-mapped output.',
    generator: () => generateLayerSwitch()
  },
  {
    id: 'modifier-combo',
    name: 'Modifier Combo (Shift+Ctrl+A)',
    description: 'Tests modifier combinations by pressing Shift and Ctrl together, then A. Expected: Output is Shift+Ctrl+A.',
    generator: () => generateModifierCombo()
  }
];

/**
 * Get a scenario by ID.
 *
 * @param id - Scenario ID
 * @returns Scenario metadata, or undefined if not found
 */
export function getScenarioById(id: string): ScenarioMetadata | undefined {
  return BUILT_IN_SCENARIOS.find(s => s.id === id);
}
