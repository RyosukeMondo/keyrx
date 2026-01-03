/**
 * useValidateConfig - React hook for profile configuration validation
 *
 * This hook provides backend validation for profile configurations using React Query mutation.
 * It replaces the WASM-based validation with a REST API call to the daemon backend.
 *
 * Features:
 * - Async validation via POST /api/profiles/validate
 * - Returns structured validation errors with line numbers
 * - Caching of validation results
 * - Loading and error state management
 *
 * @example
 * ```tsx
 * function ConfigEditor() {
 *   const { mutateAsync: validate, isPending } = useValidateConfig();
 *
 *   const handleValidate = async (code: string) => {
 *     const result = await validate(code);
 *     if (!result.valid) {
 *       console.error('Validation errors:', result.errors);
 *     }
 *   };
 * }
 * ```
 */

import { useMutation } from '@tanstack/react-query';
import * as profileApi from '../api/profiles';
import type { ValidationResult } from '../api/profiles';

/**
 * Validate profile configuration against backend compiler
 *
 * Uses React Query mutation for async validation with error handling.
 * The backend endpoint compiles the Rhai source and returns structured errors.
 *
 * @returns Mutation object with:
 *   - mutate/mutateAsync: Validate function (accepts config string)
 *   - isPending: Loading state
 *   - isError: Error state
 *   - data: Validation result with { valid: boolean, errors: ValidationError[] }
 */
export function useValidateConfig() {
  return useMutation<ValidationResult, Error, string>({
    mutationFn: (config: string) => profileApi.validateConfig(config),

    // Optional: You could add caching by mutation key if needed
    // mutationKey: ['validate-config'],

    // Error handling is done via the hook consumer (ConfigPage)
    // No need for onError/onSuccess handlers here
  });
}
