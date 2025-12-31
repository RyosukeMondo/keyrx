/**
 * macroGenerator - Convert recorded macro events to Rhai DSL code
 *
 * This module generates valid Rhai macro syntax from captured keyboard events,
 * converting MacroEvent arrays into executable macro definitions.
 */

import type { MacroEvent } from '../hooks/useMacroRecorder';

/**
 * Map Linux event codes to Rhai VK_ key names.
 * Based on evdev KEY_* constants and keyrx VK_ naming.
 */
const EVENT_CODE_TO_VK: Record<number, string> = {
  // Letters
  30: 'VK_A', 48: 'VK_B', 46: 'VK_C', 32: 'VK_D', 18: 'VK_E',
  33: 'VK_F', 34: 'VK_G', 35: 'VK_H', 23: 'VK_I', 36: 'VK_J',
  37: 'VK_K', 38: 'VK_L', 50: 'VK_M', 49: 'VK_N', 24: 'VK_O',
  25: 'VK_P', 16: 'VK_Q', 19: 'VK_R', 31: 'VK_S', 20: 'VK_T',
  22: 'VK_U', 47: 'VK_V', 17: 'VK_W', 45: 'VK_X', 21: 'VK_Y',
  44: 'VK_Z',

  // Numbers
  2: 'VK_Num1', 3: 'VK_Num2', 4: 'VK_Num3', 5: 'VK_Num4', 6: 'VK_Num5',
  7: 'VK_Num6', 8: 'VK_Num7', 9: 'VK_Num8', 10: 'VK_Num9', 11: 'VK_Num0',

  // Function keys
  59: 'VK_F1', 60: 'VK_F2', 61: 'VK_F3', 62: 'VK_F4', 63: 'VK_F5',
  64: 'VK_F6', 65: 'VK_F7', 66: 'VK_F8', 67: 'VK_F9', 68: 'VK_F10',
  87: 'VK_F11', 88: 'VK_F12',

  // Special keys
  1: 'VK_Escape', 14: 'VK_Backspace', 15: 'VK_Tab', 28: 'VK_Enter',
  57: 'VK_Space', 58: 'VK_CapsLock',

  // Modifiers
  29: 'VK_LeftCtrl', 97: 'VK_RightCtrl',
  42: 'VK_LeftShift', 54: 'VK_RightShift',
  56: 'VK_LeftAlt', 100: 'VK_RightAlt',
  125: 'VK_LeftSuper', 126: 'VK_RightSuper',

  // Navigation
  102: 'VK_Home', 107: 'VK_End',
  104: 'VK_PageUp', 109: 'VK_PageDown',
  103: 'VK_Up', 108: 'VK_Down', 105: 'VK_Left', 106: 'VK_Right',
  110: 'VK_Insert', 111: 'VK_Delete',

  // Symbols
  12: 'VK_Minus', 13: 'VK_Equal',
  26: 'VK_LeftBracket', 27: 'VK_RightBracket',
  39: 'VK_Semicolon', 40: 'VK_Quote',
  41: 'VK_Grave', 43: 'VK_Backslash',
  51: 'VK_Comma', 52: 'VK_Period', 53: 'VK_Slash',

  // Locks
  69: 'VK_NumLock', 70: 'VK_ScrollLock',

  // Media keys
  113: 'VK_Mute', 114: 'VK_VolumeDown', 115: 'VK_VolumeUp',
  163: 'VK_MediaNext', 165: 'VK_MediaPrevious', 164: 'VK_MediaPlayPause',
};

/**
 * MacroStep represents a single step in a macro sequence.
 */
export interface MacroStep {
  type: 'press' | 'release' | 'wait';
  key?: string;
  duration?: number;
}

/**
 * Options for macro generation.
 */
export interface MacroGeneratorOptions {
  /** Macro name/label */
  macroName?: string;
  /** Device ID pattern (default: "*" for all devices) */
  deviceId?: string;
  /** Minimum wait time to include in microseconds (default: 10000 = 10ms) */
  minWaitUs?: number;
  /** Include header comments */
  includeComments?: boolean;
}

/**
 * Convert event code to VK_ key name.
 * Returns a best-effort name if code is unknown.
 */
export function eventCodeToVK(code: number): string {
  return EVENT_CODE_TO_VK[code] || `VK_Unknown${code}`;
}

/**
 * Check if an event is a key press (value === 1).
 */
export function isPress(event: MacroEvent): boolean {
  return event.event.value === 1;
}

/**
 * Check if an event is a key release (value === 0).
 */
export function isRelease(event: MacroEvent): boolean {
  return event.event.value === 0;
}

/**
 * Convert MacroEvent array to MacroStep array.
 * Inserts wait() steps for timing between events.
 */
export function eventsToSteps(
  events: MacroEvent[],
  options: MacroGeneratorOptions = {}
): MacroStep[] {
  const minWaitUs = options.minWaitUs ?? 10000; // 10ms default
  const steps: MacroStep[] = [];

  for (let i = 0; i < events.length; i++) {
    const event = events[i];
    const key = eventCodeToVK(event.event.code);

    // Add press/release step
    if (isPress(event)) {
      steps.push({ type: 'press', key });
    } else if (isRelease(event)) {
      steps.push({ type: 'release', key });
    }

    // Add wait step if there's a next event
    if (i < events.length - 1) {
      const nextEvent = events[i + 1];
      const waitUs = nextEvent.relative_timestamp_us - event.relative_timestamp_us;

      if (waitUs >= minWaitUs) {
        const waitMs = Math.round(waitUs / 1000);
        steps.push({ type: 'wait', duration: waitMs });
      }
    }
  }

  return steps;
}

/**
 * Format a single macro step as Rhai code.
 */
function formatStep(step: MacroStep): string {
  switch (step.type) {
    case 'press':
      return `press("${step.key}")`;
    case 'release':
      return `release("${step.key}")`;
    case 'wait':
      return `wait(${step.duration})`;
    default:
      return '';
  }
}

/**
 * Generate Rhai macro syntax from macro steps.
 * Format: macro("VK_TriggerKey", [press("VK_A"), wait(50), release("VK_A")]);
 */
export function generateMacroLine(
  triggerKey: string,
  steps: MacroStep[],
  indent: string = '  '
): string {
  const formattedSteps = steps.map(formatStep).join(', ');
  return `${indent}macro("${triggerKey}", [${formattedSteps}]);`;
}

/**
 * Generate complete Rhai configuration from macro events.
 * Includes device block and optional comments.
 */
export function generateRhaiMacro(
  events: MacroEvent[],
  triggerKey: string,
  options: MacroGeneratorOptions = {}
): string {
  const {
    macroName = 'Recorded Macro',
    deviceId = '*',
    includeComments = true,
  } = options;

  const steps = eventsToSteps(events, options);
  const lines: string[] = [];

  // Header comment
  if (includeComments) {
    lines.push('// ============================================================================');
    lines.push(`// ${macroName}`);
    lines.push('// ============================================================================');
    lines.push('//');
    lines.push('// Auto-generated from macro recorder');
    lines.push(`// Events: ${events.length}`);
    lines.push(`// Steps: ${steps.length}`);
    lines.push('//');
    lines.push('// USAGE:');
    lines.push(`//   Press ${triggerKey} to trigger this macro`);
    lines.push('//');
    lines.push('// ============================================================================');
    lines.push('');
  }

  // Device block
  lines.push(`device_start("${deviceId}");`);
  lines.push('');

  // Macro definition
  lines.push(generateMacroLine(triggerKey, steps));
  lines.push('');

  // Close device block
  lines.push('device_end();');

  return lines.join('\n');
}

/**
 * Generate JSON export of macro events.
 * Useful for saving and loading recorded macros.
 */
export function generateMacroJSON(
  events: MacroEvent[],
  metadata?: Record<string, unknown>
): string {
  const exportData = {
    version: '1.0',
    timestamp: new Date().toISOString(),
    metadata: metadata || {},
    events: events.map((e) => ({
      code: e.event.code,
      value: e.event.value,
      key: eventCodeToVK(e.event.code),
      timestamp_us: e.relative_timestamp_us,
    })),
  };

  return JSON.stringify(exportData, null, 2);
}

/**
 * Parse JSON export back to MacroEvent array.
 */
export function parseMacroJSON(json: string): MacroEvent[] {
  try {
    const data = JSON.parse(json);

    if (!data.events || !Array.isArray(data.events)) {
      throw new Error('Invalid macro JSON: missing events array');
    }

    return data.events.map((e: { code: number; value: number; timestamp_us: number }) => ({
      event: {
        code: e.code,
        value: e.value,
      },
      relative_timestamp_us: e.timestamp_us,
    }));
  } catch (err) {
    throw new Error(`Failed to parse macro JSON: ${err instanceof Error ? err.message : 'unknown error'}`);
  }
}

/**
 * Optimize macro by removing redundant events.
 * - Removes duplicate press/release pairs
 * - Merges consecutive wait() calls
 */
export function optimizeMacroSteps(steps: MacroStep[]): MacroStep[] {
  const optimized: MacroStep[] = [];
  let lastWaitIndex = -1;

  for (const step of steps) {
    if (step.type === 'wait') {
      if (lastWaitIndex >= 0) {
        // Merge with previous wait
        const prevWait = optimized[lastWaitIndex];
        if (prevWait.duration !== undefined && step.duration !== undefined) {
          prevWait.duration += step.duration;
        }
      } else {
        optimized.push(step);
        lastWaitIndex = optimized.length - 1;
      }
    } else {
      optimized.push(step);
      lastWaitIndex = -1;
    }
  }

  return optimized;
}

/**
 * Get summary statistics for a macro.
 */
export function getMacroStats(events: MacroEvent[]): {
  totalEvents: number;
  pressEvents: number;
  releaseEvents: number;
  durationMs: number;
  uniqueKeys: number;
} {
  const pressEvents = events.filter(isPress).length;
  const releaseEvents = events.filter(isRelease).length;
  const uniqueKeys = new Set(events.map((e) => e.event.code)).size;
  const durationUs = events.length > 0
    ? events[events.length - 1].relative_timestamp_us
    : 0;
  const durationMs = Math.round(durationUs / 1000);

  return {
    totalEvents: events.length,
    pressEvents,
    releaseEvents,
    durationMs,
    uniqueKeys,
  };
}
