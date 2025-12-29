/**
 * macroTemplates - Pre-built macro templates library
 *
 * This module provides a collection of commonly-used macro templates
 * for quick macro creation without manual recording.
 */

import type { MacroEvent } from '../hooks/useMacroRecorder';
import { textToMacroEvents } from './textSnippetTemplate';

/**
 * Template category for organization.
 */
export type TemplateCategory = 'text' | 'development' | 'productivity' | 'communication' | 'custom';

/**
 * Macro template definition.
 */
export interface MacroTemplate {
  id: string;
  name: string;
  description: string;
  category: TemplateCategory;
  events: MacroEvent[];
  tags: string[];
  /** If true, this is a text snippet template */
  isTextSnippet?: boolean;
  /** Original text for text snippet templates */
  text?: string;
}

/**
 * Template metadata for display.
 */
export interface TemplateMetadata {
  id: string;
  name: string;
  description: string;
  category: TemplateCategory;
  tags: string[];
  eventCount: number;
  estimatedDurationMs: number;
}

/**
 * Pre-built macro templates.
 */
export const MACRO_TEMPLATES: Record<string, MacroTemplate> = {
  // Text snippets
  emailSignature: {
    id: 'emailSignature',
    name: 'Email Signature',
    description: 'Professional email signature with name and contact',
    category: 'communication',
    tags: ['email', 'signature', 'text'],
    isTextSnippet: true,
    text: 'Best regards,\nJohn Doe\njohn.doe@example.com\n+1 (555) 123-4567',
    events: textToMacroEvents('Best regards,\nJohn Doe\njohn.doe@example.com\n+1 (555) 123-4567'),
  },

  greeting: {
    id: 'greeting',
    name: 'Professional Greeting',
    description: 'Formal greeting for emails or messages',
    category: 'communication',
    tags: ['email', 'greeting', 'text'],
    isTextSnippet: true,
    text: 'Hello,\n\nThank you for reaching out. ',
    events: textToMacroEvents('Hello,\n\nThank you for reaching out. '),
  },

  meetingRequest: {
    id: 'meetingRequest',
    name: 'Meeting Request',
    description: 'Standard meeting request template',
    category: 'communication',
    tags: ['meeting', 'email', 'text'],
    isTextSnippet: true,
    text: 'I would like to schedule a meeting to discuss this further. Please let me know your availability.\n\n',
    events: textToMacroEvents('I would like to schedule a meeting to discuss this further. Please let me know your availability.\n\n'),
  },

  thankYou: {
    id: 'thankYou',
    name: 'Thank You',
    description: 'Polite thank you message',
    category: 'communication',
    tags: ['email', 'thanks', 'text'],
    isTextSnippet: true,
    text: 'Thank you for your assistance with this matter.\n\n',
    events: textToMacroEvents('Thank you for your assistance with this matter.\n\n'),
  },

  // Development templates
  todoComment: {
    id: 'todoComment',
    name: 'TODO Comment',
    description: 'Standard TODO comment for code',
    category: 'development',
    tags: ['code', 'comment', 'todo'],
    isTextSnippet: true,
    text: '// TODO: Implement this function\n',
    events: textToMacroEvents('// TODO: Implement this function\n'),
  },

  fixmeComment: {
    id: 'fixmeComment',
    name: 'FIXME Comment',
    description: 'FIXME comment for known issues',
    category: 'development',
    tags: ['code', 'comment', 'fixme'],
    isTextSnippet: true,
    text: '// FIXME: This needs to be refactored\n',
    events: textToMacroEvents('// FIXME: This needs to be refactored\n'),
  },

  functionDocComment: {
    id: 'functionDocComment',
    name: 'Function Documentation',
    description: 'JSDoc/TypeDoc style function documentation',
    category: 'development',
    tags: ['code', 'documentation', 'comment'],
    isTextSnippet: true,
    text: '/**\n * Description of function.\n * @param {type} paramName - Parameter description\n * @returns {type} Return value description\n */\n',
    events: textToMacroEvents('/**\n * Description of function.\n * @param {type} paramName - Parameter description\n * @returns {type} Return value description\n */\n'),
  },

  consoleLog: {
    id: 'consoleLog',
    name: 'Console Log',
    description: 'Quick console.log statement',
    category: 'development',
    tags: ['code', 'debug', 'javascript'],
    isTextSnippet: true,
    text: 'console.log();',
    events: textToMacroEvents('console.log();'),
  },

  tryGatch: {
    id: 'tryCatch',
    name: 'Try-Catch Block',
    description: 'Error handling try-catch block',
    category: 'development',
    tags: ['code', 'error-handling', 'javascript'],
    isTextSnippet: true,
    text: 'try {\n  \n} catch (err) {\n  console.error(err);\n}\n',
    events: textToMacroEvents('try {\n  \n} catch (err) {\n  console.error(err);\n}\n'),
  },

  // Productivity templates
  currentDate: {
    id: 'currentDate',
    name: 'Current Date',
    description: 'Insert current date in readable format',
    category: 'productivity',
    tags: ['date', 'text', 'utility'],
    isTextSnippet: true,
    get text() {
      return new Date().toLocaleDateString('en-US', {
        year: 'numeric',
        month: 'long',
        day: 'numeric',
      });
    },
    get events() {
      return textToMacroEvents(this.text);
    },
  },

  currentDateTime: {
    id: 'currentDateTime',
    name: 'Current Date & Time',
    description: 'Insert current date and time',
    category: 'productivity',
    tags: ['date', 'time', 'text', 'utility'],
    isTextSnippet: true,
    get text() {
      return new Date().toLocaleString('en-US', {
        year: 'numeric',
        month: 'long',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit',
      });
    },
    get events() {
      return textToMacroEvents(this.text);
    },
  },

  isoTimestamp: {
    id: 'isoTimestamp',
    name: 'ISO Timestamp',
    description: 'Insert current time in ISO 8601 format',
    category: 'productivity',
    tags: ['date', 'time', 'iso', 'utility'],
    isTextSnippet: true,
    get text() {
      return new Date().toISOString();
    },
    get events() {
      return textToMacroEvents(this.text);
    },
  },

  loremIpsum: {
    id: 'loremIpsum',
    name: 'Lorem Ipsum',
    description: 'Standard Lorem Ipsum placeholder text',
    category: 'productivity',
    tags: ['text', 'placeholder', 'lorem'],
    isTextSnippet: true,
    text: 'Lorem ipsum dolor sit amet, consectetur adipiscing elit. ',
    events: textToMacroEvents('Lorem ipsum dolor sit amet, consectetur adipiscing elit. '),
  },
};

/**
 * Get all templates.
 */
export function getAllTemplates(): MacroTemplate[] {
  return Object.values(MACRO_TEMPLATES);
}

/**
 * Get templates by category.
 */
export function getTemplatesByCategory(category: TemplateCategory): MacroTemplate[] {
  return getAllTemplates().filter((t) => t.category === category);
}

/**
 * Get templates by tag.
 */
export function getTemplatesByTag(tag: string): MacroTemplate[] {
  return getAllTemplates().filter((t) => t.tags.includes(tag));
}

/**
 * Search templates by name or description.
 */
export function searchTemplates(query: string): MacroTemplate[] {
  const lowerQuery = query.toLowerCase();
  return getAllTemplates().filter(
    (t) =>
      t.name.toLowerCase().includes(lowerQuery) ||
      t.description.toLowerCase().includes(lowerQuery) ||
      t.tags.some((tag) => tag.toLowerCase().includes(lowerQuery))
  );
}

/**
 * Get template by ID.
 */
export function getTemplateById(id: string): MacroTemplate | undefined {
  return MACRO_TEMPLATES[id];
}

/**
 * Get template metadata (without full event data).
 */
export function getTemplateMetadata(template: MacroTemplate): TemplateMetadata {
  const events = template.events;
  const durationUs = events.length > 0 ? events[events.length - 1].relative_timestamp_us : 0;

  return {
    id: template.id,
    name: template.name,
    description: template.description,
    category: template.category,
    tags: template.tags,
    eventCount: events.length,
    estimatedDurationMs: Math.round(durationUs / 1000),
  };
}

/**
 * Get all available categories.
 */
export function getCategories(): TemplateCategory[] {
  return ['text', 'development', 'productivity', 'communication', 'custom'];
}

/**
 * Get all unique tags from templates.
 */
export function getAllTags(): string[] {
  const tags = new Set<string>();
  getAllTemplates().forEach((t) => t.tags.forEach((tag) => tags.add(tag)));
  return Array.from(tags).sort();
}

/**
 * Category display names and descriptions.
 */
export const CATEGORY_INFO: Record<TemplateCategory, { name: string; description: string }> = {
  text: {
    name: 'Text Snippets',
    description: 'Common text snippets and phrases',
  },
  development: {
    name: 'Development',
    description: 'Code snippets and programming templates',
  },
  productivity: {
    name: 'Productivity',
    description: 'Date/time and utility templates',
  },
  communication: {
    name: 'Communication',
    description: 'Email and messaging templates',
  },
  custom: {
    name: 'Custom',
    description: 'User-created custom templates',
  },
};

/**
 * Create a custom template from macro events.
 */
export function createCustomTemplate(
  name: string,
  description: string,
  events: MacroEvent[],
  tags: string[] = []
): MacroTemplate {
  const id = `custom_${Date.now()}`;
  return {
    id,
    name,
    description,
    category: 'custom',
    tags: ['custom', ...tags],
    events,
  };
}
