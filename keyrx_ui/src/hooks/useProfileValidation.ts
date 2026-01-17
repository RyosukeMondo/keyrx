/**
 * useProfileValidation - React hook for profile validation status
 *
 * This hook provides validation status for a specific profile using React Query.
 * It fetches validation results from the backend and caches them for 1 minute.
 *
 * Features:
 * - Fetches validation status via POST /api/profiles/:name/validate
 * - Caches results for 1 minute to avoid excessive validation requests
 * - Returns structured validation errors with line numbers
 * - Automatically refetches when profile content changes
 *
 * @example
 * ```tsx
 * function ProfileCard({ profileName }: Props) {
 *   const { data, isLoading, isError } = useProfileValidation(profileName);
 *
 *   if (isLoading) return <Spinner />;
 *   if (data && !data.valid) {
 *     return <Badge variant="warning">⚠️ Invalid Configuration</Badge>;
 *   }
 *   return <Badge variant="success">✓ Valid</Badge>;
 * }
 * ```
 */

import { useQuery } from '@tanstack/react-query';
import { apiClient } from '../api/client';

/**
 * Validation error structure from backend
 */
export interface ValidationError {
  line: number;
  column?: number;
  message: string;
}

/**
 * Validation result from backend
 */
export interface ValidationResult {
  valid: boolean;
  errors: ValidationError[];
}

/**
 * Validate a profile by name
 *
 * @param profileName - Name of the profile to validate
 * @returns Validation result with { valid: boolean, errors: ValidationError[] }
 */
async function validateProfile(profileName: string): Promise<ValidationResult> {
  return apiClient.post<ValidationResult>(
    `/api/profiles/${profileName}/validate`
  );
}

/**
 * Hook options
 */
export interface UseProfileValidationOptions {
  /** Enable or disable the query (default: true) */
  enabled?: boolean;
  /** Stale time in milliseconds (default: 60000 = 1 minute) */
  staleTime?: number;
}

/**
 * Fetch validation status for a profile
 *
 * Uses React Query for caching and automatic refetching.
 * Results are cached for 1 minute by default to avoid excessive validation requests.
 *
 * @param profileName - Name of the profile to validate
 * @param options - Query options (enabled, staleTime)
 * @returns Query result with:
 *   - data: Validation result { valid: boolean, errors: ValidationError[] }
 *   - isLoading: Loading state
 *   - isError: Error state
 *   - error: Error object if request failed
 *   - refetch: Function to manually refetch validation
 */
export function useProfileValidation(
  profileName: string,
  options?: UseProfileValidationOptions
) {
  const { enabled = true, staleTime = 60000 } = options || {};

  return useQuery({
    queryKey: ['profile-validation', profileName],
    queryFn: () => validateProfile(profileName),
    enabled: enabled && !!profileName,
    staleTime, // Cache for 1 minute by default
    retry: 1, // Only retry once on failure
    refetchOnWindowFocus: false, // Don't refetch on window focus
  });
}
