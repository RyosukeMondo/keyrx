import React from 'react';
import { Card } from '../Card';
import { MonacoEditor } from '../MonacoEditor';
import type { ValidationError } from '@/hooks/useWasm';

interface ConfigurationModeCardProps {
  useCustomCode: boolean;
  onToggleMode: (useCustom: boolean) => void;
  effectiveProfile: string;
  onProfileChange: (profile: string) => void;
  profiles: { name: string; isActive: boolean }[] | undefined;
  isLoadingProfiles: boolean;
  isLoadingConfig: boolean;
  isUsingProfileConfig: boolean;
  isWasmReady: boolean;
  isLoadingWasm: boolean;
  profileConfig: { source: string } | undefined;
  configLoadError: string | null;
  customCode: string;
  onCustomCodeChange: (code: string) => void;
  validationErrors: ValidationError[];
  onValidate: (errors: ValidationError[]) => void;
}

/**
 * Card component for selecting configuration mode (profile or custom code)
 */
export const ConfigurationModeCard: React.FC<ConfigurationModeCardProps> = ({
  useCustomCode,
  onToggleMode,
  effectiveProfile,
  onProfileChange,
  profiles,
  isLoadingProfiles,
  isLoadingConfig,
  isUsingProfileConfig,
  isWasmReady,
  isLoadingWasm,
  profileConfig,
  configLoadError,
  customCode,
  onCustomCodeChange,
  validationErrors,
  onValidate,
}) => {
  return (
    <Card aria-label="Configuration mode selector">
      <div className="flex flex-col gap-4">
        <div className="flex items-center gap-4">
          <span className="text-sm font-medium text-slate-300">
            Configuration Mode:
          </span>
          <div className="flex gap-2">
            <button
              onClick={() => onToggleMode(false)}
              className={`px-4 py-2 rounded-md text-sm font-medium transition-colors ${
                !useCustomCode
                  ? 'bg-primary-500 text-white'
                  : 'bg-slate-700 text-slate-300 hover:bg-slate-600'
              }`}
            >
              Use Profile
            </button>
            <button
              onClick={() => onToggleMode(true)}
              className={`px-4 py-2 rounded-md text-sm font-medium transition-colors ${
                useCustomCode
                  ? 'bg-primary-500 text-white'
                  : 'bg-slate-700 text-slate-300 hover:bg-slate-600'
              }`}
            >
              Edit Code (WASM)
            </button>
          </div>
        </div>

        {!useCustomCode ? (
          // Profile selector mode
          <div className="flex flex-col sm:flex-row sm:items-center gap-3">
            <label
              htmlFor="profile-selector"
              className="text-sm font-medium text-slate-300 shrink-0"
            >
              Select Profile:
            </label>
            <div className="flex-1">
              <select
                id="profile-selector"
                value={effectiveProfile}
                onChange={(e) => onProfileChange(e.target.value)}
                disabled={
                  isLoadingProfiles || !profiles || profiles.length === 0
                }
                className="w-full sm:w-auto min-w-[200px] px-3 py-2 bg-slate-700 border border-slate-600 rounded-md text-slate-100 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
                aria-label="Select profile for simulation"
              >
                {isLoadingProfiles ? (
                  <option>Loading profiles...</option>
                ) : profiles && profiles.length > 0 ? (
                  profiles.map((profile) => (
                    <option key={profile.name} value={profile.name}>
                      {profile.name}
                      {profile.isActive ? ' [Active]' : ''}
                    </option>
                  ))
                ) : (
                  <option>No profiles available</option>
                )}
              </select>
            </div>
            <div className="flex items-center gap-2 text-xs text-slate-400">
              {isLoadingConfig && (
                <span className="flex items-center gap-1">
                  <svg
                    className="animate-spin h-4 w-4"
                    xmlns="http://www.w3.org/2000/svg"
                    fill="none"
                    viewBox="0 0 24 24"
                  >
                    <circle
                      className="opacity-25"
                      cx="12"
                      cy="12"
                      r="10"
                      stroke="currentColor"
                      strokeWidth="4"
                    />
                    <path
                      className="opacity-75"
                      fill="currentColor"
                      d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                    />
                  </svg>
                  Loading config...
                </span>
              )}
              {!isLoadingConfig && isUsingProfileConfig && (
                <span className="text-green-400 font-medium">
                  ✓ WASM Simulator Active
                </span>
              )}
              {!isLoadingConfig &&
                profileConfig &&
                !isUsingProfileConfig &&
                !configLoadError && (
                  <span className="text-yellow-400">
                    ⚠ Using mock simulation (WASM not ready)
                  </span>
                )}
              {!isWasmReady && !isLoadingWasm && (
                <span className="text-yellow-400">
                  ⚠ WASM not available (run build:wasm)
                </span>
              )}
            </div>
          </div>
        ) : (
          // Custom code editor mode
          <div className="flex flex-col gap-2">
            <div className="flex items-center justify-between">
              <p className="text-sm text-slate-400">
                Edit Rhai configuration and test with WASM compilation +
                simulation
              </p>
              {validationErrors.length > 0 && (
                <span className="text-xs text-red-400">
                  {validationErrors.length} error
                  {validationErrors.length > 1 ? 's' : ''}
                </span>
              )}
            </div>
            <div className="h-[400px]">
              <MonacoEditor
                value={customCode}
                onChange={(value) => onCustomCodeChange(value)}
                onValidate={onValidate}
                height="400px"
              />
            </div>
          </div>
        )}
      </div>
    </Card>
  );
};
