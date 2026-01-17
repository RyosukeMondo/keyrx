import { useMemo } from 'react';
import type { KeyMapping } from '@/types';

/**
 * Hook providing mock key mappings for simulator demonstration
 */
export function useMockKeyMappings() {
  const keyMappings = useMemo(
    () =>
      new Map<string, KeyMapping>([
        [
          'CAPS',
          {
            type: 'tap_hold',
            tapAction: 'Escape',
            holdAction: 'Ctrl',
            threshold: 200,
          },
        ],
        [
          'SPACE',
          {
            type: 'tap_hold',
            tapAction: 'Space',
            holdAction: 'Layer_1',
            threshold: 150,
          },
        ],
      ]),
    []
  );

  return keyMappings;
}
