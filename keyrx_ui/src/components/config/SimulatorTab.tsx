import React, { useState } from 'react';
import { KeyboardVisualizer } from '@/components/KeyboardVisualizer';
import { Card } from '@/components/Card';
import { useWasm } from '@/hooks/useWasm';
import { useProfileConfigLoader } from '@/hooks/useProfileConfigLoader';
import { useMockKeyMappings } from '@/hooks/useMockKeyMappings';
import { useSimulatorState } from '@/hooks/useSimulatorState';
import { SimulatorHeader } from '@/components/simulator/SimulatorHeader';
import { StateInspectorCard } from '@/components/simulator/StateInspectorCard';
import { EventList } from '@/components/simulator/EventList';

const MAX_EVENTS = 1000;

interface SimulatorTabProps {
  profileName: string;
  profileConfig: { source: string } | undefined;
}

/**
 * Simulator tab for ConfigPage - tests profile configuration by simulating
 * key presses and visualizing output events, state changes, and mappings.
 *
 * Unlike SimulatorPage, this component:
 * - Does not include a profile selector (parent ConfigPage handles that)
 * - Does not wrap in WasmProvider (App.tsx provides it)
 * - Uses useSimulatorState hook for consolidated state management
 */
export const SimulatorTab: React.FC<SimulatorTabProps> = ({
  profileName,
  profileConfig,
}) => {
  const [useCustomCode, setUseCustomCode] = useState(false);
  const [customCode, setCustomCode] = useState(
    '// Write your Rhai configuration here\n'
  );

  const {
    isWasmReady,
    isLoading: isLoadingWasm,
    error,
    validateConfig,
    runSimulation,
  } = useWasm();

  const effectiveConfig = useCustomCode
    ? { source: customCode }
    : profileConfig;

  const { isUsingProfileConfig, configLoadError } = useProfileConfigLoader({
    profileConfig: effectiveConfig,
    isWasmReady,
    validateConfig,
  });

  const keyMappings = useMockKeyMappings();

  const {
    pressedKeys,
    events,
    state,
    wasmState,
    eventCount,
    handleKeyClick,
    handleReset,
    handleCopyLog,
    clearEvents,
  } = useSimulatorState({
    profileConfig: effectiveConfig,
    isUsingProfileConfig,
    isWasmReady,
    runSimulation,
    keyMappings,
  });

  return (
    <div className="flex flex-col gap-4 md:gap-6">
      <SimulatorHeader
        isLoadingWasm={isLoadingWasm}
        isWasmReady={isWasmReady}
        error={error}
        eventCount={eventCount}
        onCopyLog={handleCopyLog}
        onReset={handleReset}
      />

      {/* Custom Code Toggle */}
      <Card>
        <div className="flex items-center justify-between mb-3">
          <div>
            <h2 className="text-base font-semibold text-slate-100">
              Configuration Source
            </h2>
            <p className="text-xs text-slate-400 mt-1">
              {useCustomCode
                ? 'Using custom code for simulation'
                : `Using profile: ${profileName || 'none'}`}
            </p>
          </div>
          <label className="flex items-center gap-2 cursor-pointer">
            <span className="text-sm text-slate-300">Custom Code</span>
            <input
              type="checkbox"
              checked={useCustomCode}
              onChange={(e) => setUseCustomCode(e.target.checked)}
              className="w-4 h-4 rounded border-slate-600 bg-slate-700
                text-blue-500 focus:ring-blue-500 focus:ring-offset-0"
              aria-label="Toggle custom code mode"
            />
          </label>
        </div>
        {useCustomCode && (
          <textarea
            value={customCode}
            onChange={(e) => setCustomCode(e.target.value)}
            className="w-full h-48 bg-slate-900 text-slate-100 font-mono
              text-sm p-3 rounded-md border border-slate-700
              focus:border-blue-500 focus:outline-none resize-y"
            placeholder="// Enter Rhai configuration..."
            spellCheck={false}
            aria-label="Custom Rhai configuration code"
          />
        )}
      </Card>

      {/* Config Load Error */}
      {configLoadError && (
        <div
          className="bg-red-500/10 border border-red-500 text-red-400
            px-4 py-3 rounded-md"
          role="alert"
        >
          <p className="font-medium">Configuration Error</p>
          <p className="text-sm mt-1">
            Failed to load profile configuration: {configLoadError}
          </p>
          <p className="text-xs mt-2 text-red-300">
            The simulator is using mock key mappings. Fix the configuration to
            use real profile logic.
          </p>
        </div>
      )}

      {/* State Inspector + Event Log */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-4 md:gap-6">
        <StateInspectorCard
          isUsingProfileConfig={isUsingProfileConfig}
          wasmState={wasmState}
          state={state}
        />
        <Card
          className="lg:col-span-2 flex flex-col"
          aria-labelledby="simulator-event-log-heading"
        >
          <EventList
            events={events}
            maxEvents={MAX_EVENTS}
            onClear={clearEvents}
            virtualizeThreshold={100}
          />
        </Card>
      </div>

      {/* Keyboard Visualizer */}
      <Card aria-labelledby="interactive-keyboard-heading">
        <h2
          id="interactive-keyboard-heading"
          className="text-base md:text-lg font-semibold text-slate-100 mb-4"
        >
          Interactive Keyboard
        </h2>
        <div className="flex justify-center overflow-x-auto md:overflow-x-visible">
          <KeyboardVisualizer
            layout="ANSI_104"
            keyMappings={keyMappings}
            onKeyClick={handleKeyClick}
            simulatorMode={true}
            pressedKeys={pressedKeys}
          />
        </div>
        <p className="text-xs text-slate-500 mt-4 text-center">
          Click keys to simulate press/release. Hold-configured keys will show
          timer behavior.
        </p>
      </Card>
    </div>
  );
};
