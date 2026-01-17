import { useCallback, useState } from 'react';
import { validateCustomKeycode, saveViewMode, type ViewMode, type ValidationResult } from '../utils/paletteHelpers.tsx';
import type { PaletteKey } from '../components/KeyPalette';

interface UseKeyPaletteHandlersProps {
  onKeySelect: (key: PaletteKey) => void;
  addRecentKey: (keyId: string) => void;
}

/**
 * Hook for managing KeyPalette event handlers and state
 */
export function useKeyPaletteHandlers({
  onKeySelect,
  addRecentKey,
}: UseKeyPaletteHandlersProps) {
  // Custom keycode input state
  const [customKeycode, setCustomKeycode] = useState('');
  const [customValidation, setCustomValidation] = useState<ValidationResult>({ valid: false });

  // Physical key capture state
  const [isCapturingKey, setIsCapturingKey] = useState(false);
  const [capturedKey, setCapturedKey] = useState<PaletteKey | null>(null);

  // Handle key selection with recent tracking
  const handleKeySelect = useCallback(
    (key: PaletteKey) => {
      addRecentKey(key.id);
      onKeySelect(key);
    },
    [addRecentKey, onKeySelect]
  );

  // Handle custom keycode input change
  const handleCustomKeycodeChange = useCallback((value: string) => {
    setCustomKeycode(value);
    const validation = validateCustomKeycode(value);
    setCustomValidation(validation);
  }, []);

  // Apply custom keycode
  const handleApplyCustomKeycode = useCallback(() => {
    if (
      customValidation.valid &&
      customValidation.normalizedId &&
      customValidation.label
    ) {
      const customKey: PaletteKey = {
        id: customValidation.normalizedId,
        label: customValidation.label,
        category: 'any',
        description: `Custom keycode: ${customValidation.normalizedId}`,
      };
      handleKeySelect(customKey);
      setCustomKeycode('');
      setCustomValidation({ valid: false });
    }
  }, [customValidation, handleKeySelect]);

  // Start physical key capture mode
  const startKeyCapture = useCallback(() => {
    setIsCapturingKey(true);
    setCapturedKey(null);
  }, []);

  // Cancel key capture mode
  const cancelKeyCapture = useCallback(() => {
    setIsCapturingKey(false);
    setCapturedKey(null);
  }, []);

  // Confirm captured key
  const confirmCapturedKey = useCallback(() => {
    if (capturedKey) {
      handleKeySelect(capturedKey);
    }
    setIsCapturingKey(false);
    setCapturedKey(null);
  }, [capturedKey, handleKeySelect]);

  // Toggle view mode
  const toggleViewMode = useCallback((currentMode: ViewMode): ViewMode => {
    const newMode = currentMode === 'grid' ? 'list' : 'grid';
    saveViewMode(newMode);
    return newMode;
  }, []);

  return {
    customKeycode,
    customValidation,
    isCapturingKey,
    capturedKey,
    setCapturedKey,
    handleKeySelect,
    handleCustomKeycodeChange,
    handleApplyCustomKeycode,
    startKeyCapture,
    cancelKeyCapture,
    confirmCapturedKey,
    toggleViewMode,
  };
}
