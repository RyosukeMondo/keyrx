import { useEffect, useState } from 'react';
import { useOptionalWasmContext } from '@/contexts/WasmContext';
import type { ValidationError } from '@/hooks/useWasm';

const DEBOUNCE_MS = 300;

/**
 * Validate Rhai source with the same WASM validator the code editor uses, so
 * "the editor shows errors" and "Save is blocked" are the same fact — even
 * while the code panel is closed. Returns [] while WASM is unavailable
 * (graceful degradation: the daemon still validates on save).
 */
export function useConfigValidation(code: string): ValidationError[] {
  const wasm = useOptionalWasmContext();
  const validateConfig = wasm?.validateConfig;
  const isWasmReady = wasm?.isWasmReady ?? false;
  const [errors, setErrors] = useState<ValidationError[]>([]);

  useEffect(() => {
    if (!isWasmReady || !validateConfig || !code.trim()) {
      // eslint-disable-next-line react-hooks/set-state-in-effect
      setErrors([]);
      return;
    }
    let cancelled = false;
    const timer = setTimeout(() => {
      validateConfig(code)
        .then((result) => {
          if (!cancelled) setErrors(result);
        })
        .catch(() => {
          if (!cancelled) setErrors([]);
        });
    }, DEBOUNCE_MS);
    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  }, [code, isWasmReady, validateConfig]);

  return errors;
}
