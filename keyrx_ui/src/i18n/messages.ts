/**
 * UI message catalogs.
 *
 * English (catalog.en.ts) is the source of truth: `MessageKey` is derived from
 * it and the Japanese catalog (catalog.ja.ts) is typed against it, so a
 * missing translation is a compile error rather than a silent English string.
 * Other languages: add a catalog typed `Record<MessageKey, Message>` and
 * register it in i18n/index.ts.
 */
import { en } from './catalog.en';

export type { Message } from './messageTypes';
export type MessageKey = keyof typeof en;
export { en };
export { ja } from './catalog.ja';
