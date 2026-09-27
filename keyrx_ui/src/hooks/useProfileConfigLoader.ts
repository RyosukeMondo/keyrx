import { useEffect, useState } from 'react';
import { getErrorMessage } from '../utils/errorUtils';
import { logger } from '../utils/logger';

interface ProfileConfig {
  source: string;
}

interface UseProfileConfigLoaderProps {
  profileConfig: ProfileConfig | undefined;
  isWasmReady: boolean;
  validateConfig: (
    source: string
  ) => Promise<{ line: number; message: string }[]>;
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
          logger.error('profile_config_validation_failed', undefined, {
            errorMsg,
          });
        } else {
          setConfigLoadError(null);
          setIsUsingProfileConfig(true);
        }
      } catch (err) {
        const errorMsg = getErrorMessage(err, 'Failed to load profile config');
        setConfigLoadError(errorMsg);
        setIsUsingProfileConfig(false);
        logger.error(
          'profile_config_load_failed',
          err instanceof Error ? err : undefined
        );
      }
    }

    loadProfileConfig();
  }, [profileConfig, isWasmReady, validateConfig]);

  return { isUsingProfileConfig, configLoadError };
}
