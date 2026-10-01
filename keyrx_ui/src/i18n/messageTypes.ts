/** A catalog entry: one string, or one/other plural forms (Intl.PluralRules). */
export type Message = string | { one: string; other: string };
