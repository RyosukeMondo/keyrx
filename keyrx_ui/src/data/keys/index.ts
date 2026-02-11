/**
 * Key definitions index - aggregates all key categories
 * Maintains backward compatibility with original keyDefinitions.ts API
 */

export type { KeyDefinition } from './types';

import { LETTERS } from './letters';
import { NUMBERS, NUMPAD } from './numbers';
import { FUNCTION_KEYS } from './function';
import { MODIFIERS } from './modifiers';
import { NAVIGATION, PUNCTUATION } from './navigation';
import { MEDIA } from './media';
import { MACROS, LOCKS, ALL_LAYER_MODIFIERS } from './special';

/**
 * Complete key definitions database (250+ keys)
 * Combines all key categories in a single array
 */
export const KEY_DEFINITIONS = [
  ...LETTERS,
  ...NUMBERS,
  ...FUNCTION_KEYS,
  ...NAVIGATION,
  ...PUNCTUATION,
  ...NUMPAD,
  ...MODIFIERS,
  ...MEDIA,
  ...MACROS,
  ...ALL_LAYER_MODIFIERS,
  ...LOCKS,
];

/**
 * Build reverse map from all aliases to key IDs for fast lookup
 */
function buildKeyCodeMap() {
  const map = new Map<string, string>();
  KEY_DEFINITIONS.forEach((key) => {
    map.set(key.id, key.id);
    key.aliases.forEach((alias) => {
      map.set(alias, key.id);
    });
  });
  return map;
}

export const KEY_CODE_MAP = buildKeyCodeMap();

/**
 * Get keys by category
 */
export function getKeysByCategory(
  category: string
) {
  return KEY_DEFINITIONS.filter((k) => k.category === category);
}

/**
 * Get keys by subcategory
 */
export function getKeysBySubcategory(subcategory: string) {
  return KEY_DEFINITIONS.filter((k) => k.subcategory === subcategory);
}

/**
 * Get key by ID
 */
export function getKeyById(id: string) {
  return KEY_DEFINITIONS.find((k) => k.id === id);
}

/**
 * Search keys by query (fuzzy search across id, label, description, aliases)
 */
export function searchKeys(query: string) {
  if (!query.trim()) return KEY_DEFINITIONS;

  const lowerQuery = query.toLowerCase();

  return KEY_DEFINITIONS.filter((key) => {
    // Check ID
    if (key.id.toLowerCase().includes(lowerQuery)) return true;

    // Check label
    if (key.label.toLowerCase().includes(lowerQuery)) return true;

    // Check description
    if (key.description.toLowerCase().includes(lowerQuery)) return true;

    // Check aliases
    if (key.aliases.some((alias) => alias.toLowerCase().includes(lowerQuery)))
      return true;

    return false;
  });
}

/**
 * Get all unique categories
 */
export function getCategories() {
  return ['basic', 'modifiers', 'media', 'macro', 'layers', 'special', 'any'] as const;
}

/**
 * Get all unique subcategories for a category
 */
export function getSubcategories(category: string) {
  const subcats = KEY_DEFINITIONS.filter(
    (k) => k.category === category && k.subcategory
  ).map((k) => k.subcategory as string);
  return Array.from(new Set(subcats));
}
