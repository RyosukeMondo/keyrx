/**
 * Key definition types - shared across all key modules
 */

export interface KeyDefinition {
  id: string;
  label: string;
  category:
    | 'basic'
    | 'modifiers'
    | 'media'
    | 'macro'
    | 'layers'
    | 'special'
    | 'any';
  subcategory?: string;
  description: string;
  aliases: string[];
  icon?: string;
}
