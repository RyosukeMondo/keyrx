import { useEffect, useState } from 'react';
import { getErrorMessage } from '../utils/errorUtils';

interface ProfileConfig {
  source: string;
}

interface UseProfileConfigLoaderProps {
  profileConfig: ProfileConfig | undefined;
  isWasmReady: boolean;
  validateConfig: (source: string) => Promise<{ line: number; message: string }[]>;
}

/**
 * Hook for loading and validating profile configuration
 */
export function useProfileConfigLoader({
  profileConfig,
  isWasmReady,
  validateConfig,
}: UseProfileConfigLoaderProps) {
  const [isUsingProfileConfig, setIsUsingProfileConfig] = useState(false);
  const [configLoadError, setConfigLoadError] = useState<string | null>(null);

  useEffect(() => {
    async function loadProfileConfig() {
      if (!profileConfig || !isWasmReady) {
        setIsUsingProfileConfig(false);
        setConfigLoadError(null);
        return;
      }

      try {
        // Validate the config
        const errors = await validateConfig(profileConfig.source);
        if (errors.length > 0) {
          const errorMsg = errors
            .map((e) => `Line ${e.line}: ${e.message}`)
            .join('; ');
          setConfigLoadError(errorMsg);
          setIsUsingProfileConfig(false);
          console.error('Profile config validation failed:', errorMsg);
        } else {
          setConfigLoadError(null);
          setIsUsingProfileConfig(true);
        }
      } catch (err) {
        const errorMsg = getErrorMessage(err, 'Failed to load profile config');
        setConfigLoadError(errorMsg);
        setIsUsingProfileConfig(false);
        console.error('Failed to load profile config:', err);
      }
    }

    loadProfileConfig();
  }, [profileConfig, isWasmReady, validateConfig]);

  return { isUsingProfileConfig, configLoadError };
}
