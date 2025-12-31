/**
 * textSnippetTemplate - Convert text strings to keyboard macro sequences
 *
 * This module converts plain text into keyboard event sequences that can be
 * used as macro templates for typing automation.
 */

import type { MacroEvent } from '../hooks/useMacroRecorder';
import type { MacroStep } from './macroGenerator';

/**
 * Character to VK key mapping for US keyboard layout.
 * Maps characters to their corresponding keyboard keys and shift state.
 */
interface KeyMapping {
  key: string; // VK_ key name
  shift: boolean; // Whether Shift is required
}

/**
 * Character mappings for US keyboard layout.
 */
const CHAR_TO_KEY: Record<string, KeyMapping> = {
  // Lowercase letters (no shift)
  'a': { key: 'VK_A', shift: false }, 'b': { key: 'VK_B', shift: false },
  'c': { key: 'VK_C', shift: false }, 'd': { key: 'VK_D', shift: false },
  'e': { key: 'VK_E', shift: false }, 'f': { key: 'VK_F', shift: false },
  'g': { key: 'VK_G', shift: false }, 'h': { key: 'VK_H', shift: false },
  'i': { key: 'VK_I', shift: false }, 'j': { key: 'VK_J', shift: false },
  'k': { key: 'VK_K', shift: false }, 'l': { key: 'VK_L', shift: false },
  'm': { key: 'VK_M', shift: false }, 'n': { key: 'VK_N', shift: false },
  'o': { key: 'VK_O', shift: false }, 'p': { key: 'VK_P', shift: false },
  'q': { key: 'VK_Q', shift: false }, 'r': { key: 'VK_R', shift: false },
  's': { key: 'VK_S', shift: false }, 't': { key: 'VK_T', shift: false },
  'u': { key: 'VK_U', shift: false }, 'v': { key: 'VK_V', shift: false },
  'w': { key: 'VK_W', shift: false }, 'x': { key: 'VK_X', shift: false },
  'y': { key: 'VK_Y', shift: false }, 'z': { key: 'VK_Z', shift: false },

  // Uppercase letters (with shift)
  'A': { key: 'VK_A', shift: true }, 'B': { key: 'VK_B', shift: true },
  'C': { key: 'VK_C', shift: true }, 'D': { key: 'VK_D', shift: true },
  'E': { key: 'VK_E', shift: true }, 'F': { key: 'VK_F', shift: true },
  'G': { key: 'VK_G', shift: true }, 'H': { key: 'VK_H', shift: true },
  'I': { key: 'VK_I', shift: true }, 'J': { key: 'VK_J', shift: true },
  'K': { key: 'VK_K', shift: true }, 'L': { key: 'VK_L', shift: true },
  'M': { key: 'VK_M', shift: true }, 'N': { key: 'VK_N', shift: true },
  'O': { key: 'VK_O', shift: true }, 'P': { key: 'VK_P', shift: true },
  'Q': { key: 'VK_Q', shift: true }, 'R': { key: 'VK_R', shift: true },
  'S': { key: 'VK_S', shift: true }, 'T': { key: 'VK_T', shift: true },
  'U': { key: 'VK_U', shift: true }, 'V': { key: 'VK_V', shift: true },
  'W': { key: 'VK_W', shift: true }, 'X': { key: 'VK_X', shift: true },
  'Y': { key: 'VK_Y', shift: true }, 'Z': { key: 'VK_Z', shift: true },

  // Numbers (no shift)
  '1': { key: 'VK_Num1', shift: false }, '2': { key: 'VK_Num2', shift: false },
  '3': { key: 'VK_Num3', shift: false }, '4': { key: 'VK_Num4', shift: false },
  '5': { key: 'VK_Num5', shift: false }, '6': { key: 'VK_Num6', shift: false },
  '7': { key: 'VK_Num7', shift: false }, '8': { key: 'VK_Num8', shift: false },
  '9': { key: 'VK_Num9', shift: false }, '0': { key: 'VK_Num0', shift: false },

  // Special characters (with shift)
  '!': { key: 'VK_Num1', shift: true }, '@': { key: 'VK_Num2', shift: true },
  '#': { key: 'VK_Num3', shift: true }, '$': { key: 'VK_Num4', shift: true },
  '%': { key: 'VK_Num5', shift: true }, '^': { key: 'VK_Num6', shift: true },
  '&': { key: 'VK_Num7', shift: true }, '*': { key: 'VK_Num8', shift: true },
  '(': { key: 'VK_Num9', shift: true }, ')': { key: 'VK_Num0', shift: true },

  // Symbols (no shift)
  ' ': { key: 'VK_Space', shift: false },
  '-': { key: 'VK_Minus', shift: false },
  '=': { key: 'VK_Equal', shift: false },
  '[': { key: 'VK_LeftBracket', shift: false },
  ']': { key: 'VK_RightBracket', shift: false },
  ';': { key: 'VK_Semicolon', shift: false },
  "'": { key: 'VK_Quote', shift: false },
  '`': { key: 'VK_Grave', shift: false },
  '\\': { key: 'VK_Backslash', shift: false },
  ',': { key: 'VK_Comma', shift: false },
  '.': { key: 'VK_Period', shift: false },
  '/': { key: 'VK_Slash', shift: false },

  // Symbols (with shift)
  '_': { key: 'VK_Minus', shift: true },
  '+': { key: 'VK_Equal', shift: true },
  '{': { key: 'VK_LeftBracket', shift: true },
  '}': { key: 'VK_RightBracket', shift: true },
  ':': { key: 'VK_Semicolon', shift: true },
  '"': { key: 'VK_Quote', shift: true },
  '~': { key: 'VK_Grave', shift: true },
  '|': { key: 'VK_Backslash', shift: true },
  '<': { key: 'VK_Comma', shift: true },
  '>': { key: 'VK_Period', shift: true },
  '?': { key: 'VK_Slash', shift: true },

  // Whitespace
  '\n': { key: 'VK_Enter', shift: false },
  '\t': { key: 'VK_Tab', shift: false },
};

/**
 * Options for text snippet conversion.
 */
export interface TextSnippetOptions {
  /** Delay between key presses in milliseconds (default: 10) */
  keyDelay?: number;
  /** Whether to optimize by removing redundant shift toggles */
  optimize?: boolean;
}


/**
 * Convert text string to macro steps.
 * Automatically handles shift state for uppercase and special characters.
 */
export function textToSteps(text: string, options: TextSnippetOptions = {}): MacroStep[] {
  const { keyDelay = 10, optimize = true } = options;
  const steps: MacroStep[] = [];
  let shiftPressed = false;

  for (let i = 0; i < text.length; i++) {
    const char = text[i];
    const mapping = CHAR_TO_KEY[char];

    if (!mapping) {
      // Skip unsupported characters
      continue;
    }

    // Press shift if needed and not already pressed
    if (mapping.shift && !shiftPressed) {
      steps.push({ type: 'press', key: 'VK_LeftShift' });
      shiftPressed = true;
    }

    // Release shift if it's pressed but not needed
    if (!mapping.shift && shiftPressed) {
      steps.push({ type: 'release', key: 'VK_LeftShift' });
      shiftPressed = false;
    }

    // Press and release the character key
    steps.push({ type: 'press', key: mapping.key });
    steps.push({ type: 'release', key: mapping.key });

    // Add delay between characters (except for last character)
    if (i < text.length - 1 && keyDelay > 0) {
      steps.push({ type: 'wait', duration: keyDelay });
    }
  }

  // Release shift if still pressed at the end
  if (shiftPressed) {
    steps.push({ type: 'release', key: 'VK_LeftShift' });
  }

  return optimize ? optimizeTextSteps(steps) : steps;
}

/**
 * Optimize text snippet steps by merging consecutive wait() calls.
 */
function optimizeTextSteps(steps: MacroStep[]): MacroStep[] {
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
 * Convert text snippet to MacroEvent array.
 * This allows text snippets to be used with the macro recorder UI.
 */
export function textToMacroEvents(text: string, options: TextSnippetOptions = {}): MacroEvent[] {
  const steps = textToSteps(text, options);
  const events: MacroEvent[] = [];
  let timestamp = 0;

  // Map Linux event codes for VK keys
  const VK_TO_CODE: Record<string, number> = {
    'VK_A': 30, 'VK_B': 48, 'VK_C': 46, 'VK_D': 32, 'VK_E': 18,
    'VK_F': 33, 'VK_G': 34, 'VK_H': 35, 'VK_I': 23, 'VK_J': 36,
    'VK_K': 37, 'VK_L': 38, 'VK_M': 50, 'VK_N': 49, 'VK_O': 24,
    'VK_P': 25, 'VK_Q': 16, 'VK_R': 19, 'VK_S': 31, 'VK_T': 20,
    'VK_U': 22, 'VK_V': 47, 'VK_W': 17, 'VK_X': 45, 'VK_Y': 21,
    'VK_Z': 44,
    'VK_Num1': 2, 'VK_Num2': 3, 'VK_Num3': 4, 'VK_Num4': 5,
    'VK_Num5': 6, 'VK_Num6': 7, 'VK_Num7': 8, 'VK_Num8': 9,
    'VK_Num9': 10, 'VK_Num0': 11,
    'VK_Space': 57, 'VK_Enter': 28, 'VK_Tab': 15,
    'VK_LeftShift': 42, 'VK_RightShift': 54,
    'VK_Minus': 12, 'VK_Equal': 13,
    'VK_LeftBracket': 26, 'VK_RightBracket': 27,
    'VK_Semicolon': 39, 'VK_Quote': 40,
    'VK_Grave': 41, 'VK_Backslash': 43,
    'VK_Comma': 51, 'VK_Period': 52, 'VK_Slash': 53,
  };

  for (const step of steps) {
    if (step.type === 'wait' && step.duration) {
      timestamp += step.duration * 1000; // Convert ms to us
    } else if (step.key) {
      const code = VK_TO_CODE[step.key] || 0;
      const value = step.type === 'press' ? 1 : 0;
      events.push({
        event: { code, value },
        relative_timestamp_us: timestamp,
      });
    }
  }

  return events;
}

/**
 * Get text snippet statistics.
 */
export function getTextSnippetStats(text: string, options: TextSnippetOptions = {}): {
  characters: number;
  supportedCharacters: number;
  unsupportedCharacters: number;
  steps: number;
  estimatedDurationMs: number;
} {
  const steps = textToSteps(text, options);
  const supportedCharacters = text.split('').filter((c) => CHAR_TO_KEY[c]).length;

  // Calculate estimated duration
  let durationMs = 0;
  for (const step of steps) {
    if (step.type === 'wait' && step.duration) {
      durationMs += step.duration;
    }
  }

  return {
    characters: text.length,
    supportedCharacters,
    unsupportedCharacters: text.length - supportedCharacters,
    steps: steps.length,
    estimatedDurationMs: durationMs,
  };
}

/**
 * Pre-built text snippet templates.
 */
export const TEXT_SNIPPET_TEMPLATES = {
  email: {
    name: 'Email Signature',
    template: 'Best regards,\nJohn Doe\njohn.doe@example.com',
  },
  greeting: {
    name: 'Greeting',
    template: 'Hello,\n\nThank you for reaching out. ',
  },
  closing: {
    name: 'Email Closing',
    template: '\n\nBest regards,\n',
  },
  code: {
    name: 'Code Comment',
    template: '// TODO: Implement this function\n',
  },
  date: {
    name: 'Current Date',
    template: new Date().toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'long',
      day: 'numeric',
    }),
  },
};
