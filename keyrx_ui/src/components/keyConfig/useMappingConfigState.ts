import { useState, useCallback, useEffect } from 'react';
import type {
  MappingType,
  MappingConfig,
  ValidationResult,
} from './mappingConfigTypes';

export interface UseMappingConfigStateArgs {
  mappingType: MappingType;
  currentConfig: MappingConfig;
  onChange: (config: MappingConfig) => void;
  onValidate?: (config: MappingConfig) => ValidationResult;
}

export type ListeningTarget = 'tap' | 'hold' | null;

export interface MappingFormState {
  tapAction: string;
  holdAction: string;
  threshold: number;
  modifierKey: string;
  lockKey: string;
  targetLayer: string;
  useKeyboard: boolean;
  setUseKeyboard: (value: boolean) => void;
  isListening: boolean;
  listeningFor: ListeningTarget;
  errors: Record<string, string>;
  handleTapActionChange: (value: string) => void;
  handleHoldActionChange: (value: string) => void;
  handleThresholdChange: (value: number) => void;
  handleModifierKeyChange: (value: string) => void;
  handleLockKeyChange: (value: string) => void;
  handleTargetLayerChange: (value: string) => void;
  startListening: (target: 'tap' | 'hold') => void;
  stopListening: () => void;
}

/**
 * Owns all form state, validation and key-listening behavior for
 * {@link MappingConfigForm}. Extracted so the mapping-type-specific
 * form components can stay presentational.
 */
export function useMappingConfigState({
  mappingType,
  currentConfig,
  onChange,
  onValidate,
}: UseMappingConfigStateArgs): MappingFormState {
  // Form state
  const [tapAction, setTapAction] = useState(currentConfig.tapAction || '');
  const [holdAction, setHoldAction] = useState(currentConfig.holdAction || '');
  const [threshold, setThreshold] = useState(currentConfig.threshold || 200);
  const [modifierKey, setModifierKey] = useState(
    currentConfig.modifierKey || ''
  );
  const [lockKey, setLockKey] = useState(currentConfig.lockKey || '');
  const [targetLayer, setTargetLayer] = useState(
    currentConfig.targetLayer || ''
  );

  // UI state
  const [useKeyboard, setUseKeyboard] = useState(false);
  const [isListening, setIsListening] = useState(false);
  const [listeningFor, setListeningFor] = useState<ListeningTarget>(null);
  const [errors, setErrors] = useState<Record<string, string>>({});

  // Update form when currentConfig changes
  // This is needed to sync form state when parent changes the config (e.g., editing existing mapping)
  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect
    setTapAction(currentConfig.tapAction || '');
    // eslint-disable-next-line react-hooks/set-state-in-effect
    setHoldAction(currentConfig.holdAction || '');
    // eslint-disable-next-line react-hooks/set-state-in-effect
    setThreshold(currentConfig.threshold || 200);
    // eslint-disable-next-line react-hooks/set-state-in-effect
    setModifierKey(currentConfig.modifierKey || '');
    // eslint-disable-next-line react-hooks/set-state-in-effect
    setLockKey(currentConfig.lockKey || '');
    // eslint-disable-next-line react-hooks/set-state-in-effect
    setTargetLayer(currentConfig.targetLayer || '');
  }, [currentConfig]);

  // Build current config object
  const buildConfig = useCallback((): MappingConfig => {
    switch (mappingType) {
      case 'simple':
        return { type: 'simple', tapAction };
      case 'modifier':
        return { type: 'modifier', modifierKey };
      case 'lock':
        return { type: 'lock', lockKey };
      case 'tap_hold':
        return { type: 'tap_hold', tapAction, holdAction, threshold };
      case 'layer_active':
        return { type: 'layer_active', targetLayer };
      default:
        return { type: 'simple', tapAction };
    }
  }, [
    mappingType,
    tapAction,
    holdAction,
    threshold,
    modifierKey,
    lockKey,
    targetLayer,
  ]);

  // Validate and notify onChange
  const notifyChange = useCallback(() => {
    const config = buildConfig();
    if (onValidate) {
      const result = onValidate(config);
      setErrors(result.errors);
    }
    onChange(config);
  }, [buildConfig, onChange, onValidate]);

  // Key listening
  const handleKeyCapture = useCallback(
    (event: KeyboardEvent) => {
      if (!isListening) return;

      event.preventDefault();
      event.stopPropagation();

      if (event.key === 'Escape') {
        setIsListening(false);
        setListeningFor(null);
        return;
      }

      let vkCode = event.code;
      if (vkCode.startsWith('Key')) {
        vkCode = 'VK_' + vkCode.substring(3);
      } else if (vkCode.startsWith('Digit')) {
        vkCode = 'VK_' + vkCode.substring(5);
      } else {
        vkCode = 'VK_' + vkCode.toUpperCase();
      }

      if (listeningFor === 'tap') {
        setTapAction(vkCode);
      } else if (listeningFor === 'hold') {
        setHoldAction(vkCode);
      }

      setIsListening(false);
      setListeningFor(null);
    },
    [isListening, listeningFor]
  );

  useEffect(() => {
    if (isListening) {
      document.addEventListener('keydown', handleKeyCapture);
      return () => document.removeEventListener('keydown', handleKeyCapture);
    }
  }, [isListening, handleKeyCapture]);

  const startListening = (target: 'tap' | 'hold') => {
    setIsListening(true);
    setListeningFor(target);
  };

  const stopListening = () => {
    setIsListening(false);
    setListeningFor(null);
  };

  // Update handlers
  const handleTapActionChange = (value: string) => {
    setTapAction(value);
    setTimeout(notifyChange, 0);
  };

  const handleHoldActionChange = (value: string) => {
    setHoldAction(value);
    setTimeout(notifyChange, 0);
  };

  const handleThresholdChange = (value: number) => {
    setThreshold(value);
    setTimeout(notifyChange, 0);
  };

  const handleModifierKeyChange = (value: string) => {
    setModifierKey(value);
    setTimeout(notifyChange, 0);
  };

  const handleLockKeyChange = (value: string) => {
    setLockKey(value);
    setTimeout(notifyChange, 0);
  };

  const handleTargetLayerChange = (value: string) => {
    setTargetLayer(value);
    setTimeout(notifyChange, 0);
  };

  return {
    tapAction,
    holdAction,
    threshold,
    modifierKey,
    lockKey,
    targetLayer,
    useKeyboard,
    setUseKeyboard,
    isListening,
    listeningFor,
    errors,
    handleTapActionChange,
    handleHoldActionChange,
    handleThresholdChange,
    handleModifierKeyChange,
    handleLockKeyChange,
    handleTargetLayerChange,
    startListening,
    stopListening,
  };
}
