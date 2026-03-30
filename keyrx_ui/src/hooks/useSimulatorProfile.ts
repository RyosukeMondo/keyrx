import { useState, useEffect, useCallback } from 'react';
import { toast } from 'sonner';
import { getErrorMessage } from '../utils/errorUtils';

interface SimulatorProfileState {
  isLoaded: boolean;
  isLoading: boolean;
  error: string | null;
  profileName: string | null;
}

interface LoadProfileResponse {
  success: boolean;
  message: string;
}

/**
 * Hook for loading profiles into the daemon's simulator via REST API.
 *
 * This bypasses WASM validation and uses the daemon's compiled .krx files,
 * which support all Rhai functions (device_start, tap_hold, when_start, etc.)
 */
export function useSimulatorProfile() {
  const [state, setState] = useState<SimulatorProfileState>({
    isLoaded: false,
    isLoading: false,
    error: null,
    profileName: null,
  });

  const loadProfile = useCallback(
    async (profileName: string): Promise<boolean> => {
      if (!profileName) {
        setState((prev) => ({
          ...prev,
          error: 'No profile name provided',
          isLoaded: false,
        }));
        return false;
      }

      setState((prev) => ({ ...prev, isLoading: true, error: null }));

      try {
        const response = await fetch('/api/simulator/load-profile', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ name: profileName }),
        });

        if (!response.ok) {
          const errorText = await response.text();
          throw new Error(
            errorText || `Failed to load profile: ${response.status}`
          );
        }

        const data: LoadProfileResponse = await response.json();

        if (data.success) {
          setState({
            isLoaded: true,
            isLoading: false,
            error: null,
            profileName,
          });
          return true;
        } else {
          throw new Error(data.message || 'Failed to load profile');
        }
      } catch (err) {
        const errorMsg = getErrorMessage(
          err,
          'Failed to load simulator profile'
        );
        setState({
          isLoaded: false,
          isLoading: false,
          error: errorMsg,
          profileName: null,
        });
        toast.error(errorMsg);
        return false;
      }
    },
    []
  );

  const reset = useCallback(async (): Promise<boolean> => {
    try {
      const response = await fetch('/api/simulator/reset', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
      });

      if (!response.ok) {
        toast.error('Failed to reset simulator');
        return false;
      }

      setState({
        isLoaded: false,
        isLoading: false,
        error: null,
        profileName: null,
      });
      return true;
    } catch (err) {
      toast.error(getErrorMessage(err, 'Failed to reset simulator'));
      return false;
    }
  }, []);

  return {
    ...state,
    loadProfile,
    reset,
  };
}

/**
 * Hook that automatically loads a profile when the profile name changes.
 */
export function useAutoLoadSimulatorProfile(profileName: string | undefined) {
  const simulator = useSimulatorProfile();

  useEffect(() => {
    if (profileName) {
      simulator.loadProfile(profileName);
    }
  }, [profileName]); // eslint-disable-line react-hooks/exhaustive-deps

  return simulator;
}
