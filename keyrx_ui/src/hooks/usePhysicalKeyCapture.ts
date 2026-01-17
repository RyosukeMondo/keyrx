import { useEffect } from 'react';
import { mapDomCodeToKeyId } from '../utils/paletteHelpers.tsx';
import type { PaletteKey } from '../components/KeyPalette';

interface UsePhysicalKeyCaptureProps {
  isCapturingKey: boolean;
  onCapturedKey: (key: PaletteKey) => void;
  onCancel: () => void;
}

/**
 * Hook for capturing physical keyboard events
 */
export function usePhysicalKeyCapture({
  isCapturingKey,
  onCapturedKey,
  onCancel,
}: UsePhysicalKeyCaptureProps) {
  useEffect(() => {
    if (!isCapturingKey) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      e.preventDefault();
      e.stopPropagation();

      // Allow Escape to cancel
      if (e.code === 'Escape') {
        onCancel();
        return;
      }

      // Map the DOM code to our key ID
      const mappedKey = mapDomCodeToKeyId(e.code);
      if (mappedKey) {
        onCapturedKey(mappedKey);
      } else {
        // Unknown key - show error state
        console.warn('Unknown key code:', e.code);
      }
    };

    // Add listener at document level to capture all keys
    document.addEventListener('keydown', handleKeyDown, true);

    return () => {
      document.removeEventListener('keydown', handleKeyDown, true);
    };
  }, [isCapturingKey, onCapturedKey, onCancel]);
}
