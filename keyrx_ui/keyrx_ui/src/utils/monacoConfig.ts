/**
 * Monaco Editor language configuration for Rhai DSL
 *
 * This file configures syntax highlighting and editor features for Rhai
 * configuration files used in keyrx.
 */

import * as monaco from 'monaco-editor';

/**
 * Keywords used in the keyrx Rhai DSL
 */
const KEYWORDS = [
  // Device management
  'device_start',
  'device_end',

  // Basic mapping functions
  'map',
  'tap_hold',

  // Conditional functions
  'when',
  'when_not',

  // Modifier functions
  'with_shift',
  'with_ctrl',
  'with_alt',
  'with_mods',

  // Standard Rhai control flow
  'if',
  'else',
  'while',
  'for',
  'in',
  'loop',
  'break',
  'continue',
  'return',
  'fn',
  'let',
  'const',

  // Boolean literals
  'true',
  'false',
];

/**
 * Key code prefixes used in keyrx
 */
const KEY_PREFIXES = ['VK_', 'MD_', 'LK_'];

/**
 * Register Rhai language with Monaco Editor
 */
export function registerRhaiLanguage(): void {
  // Register the language
  monaco.languages.register({
    id: 'rhai',
    extensions: ['.rhai'],
    aliases: ['Rhai', 'rhai'],
  });

  // Set language configuration
  monaco.languages.setLanguageConfiguration('rhai', {
    comments: {
      lineComment: '//',
      blockComment: ['/*', '*/'],
    },
    brackets: [
      ['{', '}'],
      ['[', ']'],
      ['(', ')'],
    ],
    autoClosingPairs: [
      { open: '{', close: '}' },
      { open: '[', close: ']' },
      { open: '(', close: ')' },
      { open: '"', close: '"' },
      { open: "'", close: "'" },
    ],
    surroundingPairs: [
      { open: '{', close: '}' },
      { open: '[', close: ']' },
      { open: '(', close: ')' },
      { open: '"', close: '"' },
      { open: "'", close: "'" },
    ],
    indentationRules: {
      increaseIndentPattern: /^.*\{[^}]*$/,
      decreaseIndentPattern: /^\s*\}/,
    },
  });

  // Set syntax highlighting rules
  monaco.languages.setMonarchTokensProvider('rhai', {
    keywords: KEYWORDS,
    keyPrefixes: KEY_PREFIXES,

    tokenizer: {
      root: [
        // Comments
        [/\/\/.*$/, 'comment'],
        [/\/\*/, 'comment', '@comment'],

        // Strings
        [/"([^"\\]|\\.)*$/, 'string.invalid'], // non-terminated string
        [/"/, 'string', '@string_double'],
        [/'([^'\\]|\\.)*$/, 'string.invalid'], // non-terminated string
        [/'/, 'string', '@string_single'],

        // Numbers
        [/\d+\.\d+([eE][-+]?\d+)?/, 'number.float'],
        [/0[xX][0-9a-fA-F]+/, 'number.hex'],
        [/\d+/, 'number'],

        // Key codes with prefixes (VK_, MD_, LK_)
        [/\b(VK_|MD_|LK_)[A-Za-z0-9_]+\b/, 'type.identifier'],

        // Keywords
        [/\b(device_start|device_end|map|tap_hold|when|when_not|with_shift|with_ctrl|with_alt|with_mods)\b/, 'keyword'],
        [/\b(if|else|while|for|in|loop|break|continue|return|fn|let|const)\b/, 'keyword.control'],
        [/\b(true|false)\b/, 'constant.language'],

        // Identifiers
        [/[a-zA-Z_]\w*/, 'identifier'],

        // Operators
        [/[{}()\[\]]/, '@brackets'],
        [/[<>]=?|[!=]=|&&|\|\||[+\-*\/%!~&|^]/, 'operator'],
        [/[;,.]/, 'delimiter'],

        // Whitespace
        [/\s+/, 'white'],
      ],

      comment: [
        [/[^\/*]+/, 'comment'],
        [/\*\//, 'comment', '@pop'],
        [/[\/*]/, 'comment'],
      ],

      string_double: [
        [/[^\\"]+/, 'string'],
        [/\\./, 'string.escape'],
        [/"/, 'string', '@pop'],
      ],

      string_single: [
        [/[^\\']+/, 'string'],
        [/\\./, 'string.escape'],
        [/'/, 'string', '@pop'],
      ],
    },
  });
}

/**
 * Configure theme for Rhai syntax highlighting
 */
export function configureRhaiTheme(): void {
  // Define custom token colors for Rhai
  monaco.editor.defineTheme('rhai-dark', {
    base: 'vs-dark',
    inherit: true,
    rules: [
      { token: 'keyword', foreground: 'C586C0', fontStyle: 'bold' },
      { token: 'keyword.control', foreground: 'C586C0', fontStyle: 'bold' },
      { token: 'type.identifier', foreground: '4EC9B0' }, // Key codes (VK_, MD_, LK_)
      { token: 'string', foreground: 'CE9178' },
      { token: 'string.escape', foreground: 'D7BA7D' },
      { token: 'string.invalid', foreground: 'F44747' },
      { token: 'comment', foreground: '6A9955', fontStyle: 'italic' },
      { token: 'number', foreground: 'B5CEA8' },
      { token: 'number.float', foreground: 'B5CEA8' },
      { token: 'number.hex', foreground: 'B5CEA8' },
      { token: 'constant.language', foreground: '569CD6' },
      { token: 'operator', foreground: 'D4D4D4' },
    ],
    colors: {},
  });

  monaco.editor.defineTheme('rhai-light', {
    base: 'vs',
    inherit: true,
    rules: [
      { token: 'keyword', foreground: 'AF00DB', fontStyle: 'bold' },
      { token: 'keyword.control', foreground: 'AF00DB', fontStyle: 'bold' },
      { token: 'type.identifier', foreground: '267F99' }, // Key codes (VK_, MD_, LK_)
      { token: 'string', foreground: 'A31515' },
      { token: 'string.escape', foreground: 'EE0000' },
      { token: 'string.invalid', foreground: 'CD3131' },
      { token: 'comment', foreground: '008000', fontStyle: 'italic' },
      { token: 'number', foreground: '098658' },
      { token: 'number.float', foreground: '098658' },
      { token: 'number.hex', foreground: '098658' },
      { token: 'constant.language', foreground: '0000FF' },
      { token: 'operator', foreground: '000000' },
    ],
    colors: {},
  });
}

/**
 * Initialize Rhai language support for Monaco Editor
 * Call this function before creating any Monaco editor instances
 */
export function initializeRhaiSupport(): void {
  registerRhaiLanguage();
  configureRhaiTheme();
}
