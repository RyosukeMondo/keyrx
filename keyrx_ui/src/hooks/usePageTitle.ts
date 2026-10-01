import { useEffect } from 'react';

export const APP_NAME = 'keyrx';

/** `<Page> – keyrx`, so every route has a distinct, meaningful tab title. */
export function formatPageTitle(page: string): string {
  return `${page} – ${APP_NAME}`;
}

/** Set `document.title` for the lifetime of the calling page. */
export function usePageTitle(page: string): void {
  useEffect(() => {
    document.title = formatPageTitle(page);
  }, [page]);
}
