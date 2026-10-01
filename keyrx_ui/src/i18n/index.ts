/**
 * Minimal i18n: a typed message catalog, `navigator.language` detection with
 * an English fallback, and `<html lang>` kept in sync.
 *
 * Deliberately not a framework. The locale is read once at startup (override
 * with `?lang=ja` or `localStorage['keyrx.lang']` for testing); components
 * call `t()` during render.
 */
import { en, ja, type Message, type MessageKey } from './messages';

export type Locale = 'en' | 'ja';
export type { MessageKey };

const catalogs: Record<Locale, Record<MessageKey, Message>> = { en, ja };

/** Pick a supported locale from a BCP-47 tag; unknown languages fall back to English. */
export function detectLocale(tag: string | undefined): Locale {
  const primary = (tag ?? '').toLowerCase().split('-')[0];
  return primary === 'ja' ? 'ja' : 'en';
}

function initialLocale(): Locale {
  try {
    const forced =
      new URLSearchParams(window.location.search).get('lang') ??
      window.localStorage.getItem('keyrx.lang');
    if (forced) return detectLocale(forced);
  } catch {
    // storage / URL access can throw in sandboxed contexts
  }
  return detectLocale(
    typeof navigator === 'undefined' ? undefined : navigator.language
  );
}

let current: Locale = initialLocale();

export function getLocale(): Locale {
  return current;
}

/** Switch locale (tests, future language picker) and update `<html lang>`. */
export function setLocale(locale: Locale): void {
  current = locale;
  applyDocumentLanguage();
}

export function applyDocumentLanguage(): void {
  if (typeof document !== 'undefined') {
    document.documentElement.lang = current;
  }
}

function pick(message: Message, count: number | undefined, locale: Locale) {
  if (typeof message === 'string') return message;
  const rule = new Intl.PluralRules(locale).select(count ?? 0);
  return rule === 'one' ? message.one : message.other;
}

/** Translate `key`, substituting `{name}` placeholders; `count` selects the plural form. */
export function t(
  key: MessageKey,
  vars: Record<string, string | number> = {}
): string {
  const message = pick(
    catalogs[current][key],
    typeof vars.count === 'number' ? vars.count : undefined,
    current
  );
  return message.replace(/\{(\w+)\}/g, (_, name: string) =>
    name in vars ? String(vars[name]) : `{${name}}`
  );
}
