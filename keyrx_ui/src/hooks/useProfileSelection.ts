import { useState } from 'react';
import { useSearchParams, useParams } from 'react-router-dom';
import { useActiveProfileQuery } from './useProfiles';

/**
 * Custom hook for managing profile selection with fallback priority
 *
 * Priority order:
 * 1. Manual selection (user explicitly selected a profile)
 * 2. Prop (profileName passed as component prop)
 * 3. Route parameter (profile name from URL route)
 * 4. Query parameter (profile from URL query string)
 * 5. Active profile (from daemon)
 * 6. Default profile ('Default')
 *
 * @param propProfileName - Optional profile name passed as prop
 * @returns Object containing selectedProfileName and setSelectedProfileName
 *
 * @example
 * ```tsx
 * const { selectedProfileName, setSelectedProfileName } = useProfileSelection();
 *
 * // Use in component
 * <select value={selectedProfileName} onChange={(e) => setSelectedProfileName(e.target.value)}>
 *   {profiles.map(p => <option key={p.name} value={p.name}>{p.name}</option>)}
 * </select>
 * ```
 */
export function useProfileSelection(propProfileName?: string) {
  const [searchParams] = useSearchParams();
  const { name: profileNameFromRoute } = useParams<{ name: string }>();
  const profileNameFromQuery = searchParams.get('profile');

  // Get active profile from daemon - used as fallback when no profile specified
  const { data: activeProfileName } = useActiveProfileQuery();

  // Track user's manual selection separately from the computed default
  const [manualSelection, setManualSelection] = useState<string | null>(null);

  // Effective profile: manual selection > explicit URL/prop > active profile > 'Default'
  // Use || instead of ?? to treat empty strings as falsy
  // Note: fallback is 'Default' (capitalized) to match UI convention
  const selectedProfileName =
    manualSelection ||
    propProfileName ||
    profileNameFromRoute ||
    profileNameFromQuery ||
    activeProfileName ||
    'Default';

  /**
   * Set the selected profile name (tracks manual selection)
   * @param name - The profile name to select
   */
  const setSelectedProfileName = (name: string) => {
    setManualSelection(name);
  };

  return {
    selectedProfileName,
    setSelectedProfileName,
  };
}
