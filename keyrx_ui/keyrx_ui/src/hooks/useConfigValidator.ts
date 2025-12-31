/**
 * React hook for configuration validation with debounced execution.
 *
 * This hook provides a convenient interface for React components to validate
 * Rhai configuration source code with automatic debouncing and state management.
 */

import { useState, useCallback, useEffect, useMemo, useRef } from 'react';
import debounce from 'lodash.debounce';
import { validator } from '@/utils/validator';
import { wasmCore } from '@/wasm/core';
import type { ValidationResult, ValidationOptions } from '@/types/validation';
import { DEFAULT_VALIDATION_OPTIONS } from '@/types/validation';

/**
 * Return type for the useConfigValidator hook.
 */
export interface UseConfigValidatorReturn {
  /** Current validation result (null if not yet validated) */
  validationResult: ValidationResult | null;

  /** True if validation is currently in progress */
  isValidating: boolean;

  /** Whether WASM module is available and initialized */
  wasmAvailable: boolean;

  /**
   * Trigger validation for the given configuration source.
   * This call is debounced by 500ms.
   */
  validate: (source: string, options?: ValidationOptions) => void;

  /**
   * Clear the current validation result.
   */
  clearValidation: () => void;
}

/**
 * Custom hook for configuration validation with debouncing.
 *
 * This hook manages validation state and provides a debounced validate function
 * that automatically calls the ConfigValidator service after 500ms of idle time.
 *
 * @param debounceMs - Debounce delay in milliseconds (default: 500ms)
 * @returns Validation state and control functions
 *
 * @example
 * ```tsx
 * function ConfigEditor() {
 *   const { validationResult, isValidating, validate } = useConfigValidator();
 *
 *   const handleChange = (newSource: string) => {
 *     validate(newSource); // Debounced automatically
 *   };
 *
 *   return (
 *     <div>
 *       {isValidating && <Spinner />}
 *       {validationResult?.errors.map(err => (
 *         <ErrorMessage key={err.line} error={err} />
 *       ))}
 *     </div>
 *   );
 * }
 * ```
 */
export function useConfigValidator(
  debounceMs: number = 500
): UseConfigValidatorReturn {
  const [validationResult, setValidationResult] = useState<ValidationResult | null>(null);
  const [isValidating, setIsValidating] = useState(false);
  const [wasmAvailable, setWasmAvailable] = useState(false);

  // Track if component is mounted to avoid state updates after unmount
  const isMountedRef = useRef(true);

  // Check WASM availability on mount
  useEffect(() => {
    let mounted = true;

    const checkWasm = async () => {
      try {
        await wasmCore.init();
        if (mounted) {
          setWasmAvailable(true);
        }
      } catch (error) {
        console.error('WASM initialization failed:', error);
        if (mounted) {
          setWasmAvailable(false);
          setValidationResult({
            errors: [
              {
                line: 1,
                column: 1,
                message: 'Validation unavailable: WASM module failed to load',
                code: 'WASM_UNAVAILABLE',
              },
            ],
            warnings: [],
            hints: [],
            timestamp: new Date().toISOString(),
          });
        }
      }
    };

    checkWasm();

    return () => {
      mounted = false;
    };
  }, []);

  // Core validation function (not debounced)
  const performValidation = useCallback(
    async (source: string, options: ValidationOptions = DEFAULT_VALIDATION_OPTIONS) => {
      if (!isMountedRef.current) return;

      setIsValidating(true);

      try {
        const result = await validator.validate(source, options);
        if (isMountedRef.current) {
          setValidationResult(result);
        }
      } catch (error) {
        console.error('Validation error:', error);
        if (isMountedRef.current) {
          setValidationResult({
            errors: [
              {
                line: 1,
                column: 1,
                message: error instanceof Error ? error.message : 'Unknown validation error',
                code: 'VALIDATION_ERROR',
              },
            ],
            warnings: [],
            hints: [],
            timestamp: new Date().toISOString(),
          });
        }
      } finally {
        if (isMountedRef.current) {
          setIsValidating(false);
        }
      }
    },
    []
  );

  // Debounced validation function
  const debouncedValidate = useMemo(
    () =>
      debounce(
        (source: string, options?: ValidationOptions) => {
          performValidation(source, options);
        },
        debounceMs,
        { leading: false, trailing: true }
      ),
    [debounceMs, performValidation]
  );

  // Cleanup debounce timer on unmount
  useEffect(() => {
    return () => {
      isMountedRef.current = false;
      debouncedValidate.cancel();
    };
  }, [debouncedValidate]);

  // Public validate function (debounced)
  const validate = useCallback(
    (source: string, options?: ValidationOptions) => {
      debouncedValidate(source, options);
    },
    [debouncedValidate]
  );

  // Clear validation result
  const clearValidation = useCallback(() => {
    debouncedValidate.cancel();
    setValidationResult(null);
    setIsValidating(false);
  }, [debouncedValidate]);

  return {
    validationResult,
    isValidating,
    wasmAvailable,
    validate,
    clearValidation,
  };
}
