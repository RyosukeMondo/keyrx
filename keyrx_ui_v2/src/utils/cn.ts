/**
 * Utility function to combine class names
 * Filters out falsy values and joins remaining strings
 */
export function cn(...classes: (string | boolean | undefined | null)[]): string {
  return classes.filter(Boolean).join(' ');
}
