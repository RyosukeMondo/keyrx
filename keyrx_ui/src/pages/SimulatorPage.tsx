import React, {
  useState,
  useCallback,
  useRef,
  useEffect,
  useMemo,
} from 'react';
import { KeyboardVisualizer } from '../components/KeyboardVisualizer';
import type { KeyMapping } from '@/types';
import { StateIndicatorPanel } from '../components/StateIndicatorPanel';
import { Button } from '../components/Button';
import { Card } from '../components/Card';
import { MonacoEditor } from '../components/MonacoEditor';
import { WasmStatusBadge } from '../components/WasmStatusBadge';
import { WasmProvider } from '../contexts/WasmContext';
import { useProfiles } from '../hooks/useProfiles';
import { useGetProfileConfig } from '../hooks/useProfileConfig';
import { useWasm, type SimulationInput } from '../hooks/useWasm';
import { useProfileConfigLoader } from '../hooks/useProfileConfigLoader';
import { useMockKeyMappings } from '../hooks/useMockKeyMappings';
import { getErrorMessage } from '../utils/errorUtils';
import type { DaemonState, KeyEvent, SimulatorState } from '../types/rpc';
import type { ValidationError } from '../hooks/useWasm';
import { EventList } from '../components/simulator/EventList';
import { ConfigurationModeCard } from '../components/simulator/ConfigurationModeCard';
import { SimulatorHeader } from '../components/simulator/SimulatorHeader';
import { StateInspectorCard } from '../components/simulator/StateInspectorCard';

const MAX_EVENTS = 1000;
const AUTO_PAUSE_TIMEOUT = 60000; // 60 seconds

export const SimulatorPage: React.FC = () => {
  const [pressedKeys, setPressedKeys] = useState<Set<string>>(new Set());
  const [events, setEvents] = useState<KeyEvent[]>([]);
  const [state, setState] = useState<SimulatorState>({
    activeLayer: 'MD_00 (Base)',
    modifiers: {
      ctrl: false,
      shift: false,
      alt: false,
      gui: false,
    },
    locks: {
      capsLock: false,
      numLock: false,
      scrollLock: false,
    },
  });
  const [isPaused, setIsPaused] = useState(false);
  const [holdTimers, setHoldTimers] = useState<Map<string, number>>(new Map());
  const autoPauseTimerRef = useRef<NodeJS.Timeout>();
  const lastActivityRef = useRef<number>(0);

  // Initialize lastActivityRef on mount
  useEffect(() => {
    lastActivityRef.current = Date.now();
  }, []);

  // Profile selection
  const { data: profiles, isLoading: isLoadingProfiles } = useProfiles();
  const [selectedProfile, setSelectedProfile] = useState<string>('');

  // Compute effective profile (use selected or default to first profile)
  const effectiveProfile = useMemo(() => {
    if (selectedProfile) {
      return selectedProfile;
    }
    if (profiles && profiles.length > 0) {
      return profiles[0].name;
    }
    return '';
  }, [selectedProfile, profiles]);

  const { data: profileConfig, isLoading: isLoadingConfig } =
    useGetProfileConfig(effectiveProfile);
  const {
    isWasmReady,
    isLoading: isLoadingWasm,
    error,
    validateConfig,
    runSimulation,
  } = useWasm();
  const { isUsingProfileConfig, configLoadError } = useProfileConfigLoader({
    profileConfig,
    isWasmReady,
    validateConfig,
  });
  const [wasmState, setWasmState] = useState<DaemonState | null>(null);

  // Custom code editor mode
  const [useCustomCode, setUseCustomCode] = useState(false);
  const [customCode, setCustomCode] = useState<string>(
    '// Write your Rhai configuration here\n'
  );
  const [validationErrors, setValidationErrors] = useState<ValidationError[]>(
    []
  );


  // Mock key mappings for demonstration
  const keyMappings = useMockKeyMappings();

  const addEvent = useCallback(
    (
      keyCode: string,
      eventType: 'press' | 'release',
      input: string,
      output: string
    ) => {
      const timestamp = Date.now() * 1000; // Convert to microseconds
      const event: KeyEvent = {
        timestamp,
        keyCode,
        eventType,
        input,
        output,
        latency: 0, // Simulated events have no real latency
      };
      setEvents((prev) => {
        const newEvents = [event, ...prev];
        return newEvents.slice(0, MAX_EVENTS);
      });
    },
    []
  );

  const clearEvents = useCallback(() => {
    setEvents([]);
  }, []);

  const handleKeyPress = useCallback(
    async (keyCode: string) => {
      if (isPaused) return;

      lastActivityRef.current = Date.now();

      // Add to pressed keys
      setPressedKeys((prev) => new Set(prev).add(keyCode));

      // Add press event
      addEvent(keyCode, 'press', keyCode, keyCode);

      // If WASM is ready and we have a valid profile config, use WASM simulation
      if (
        isUsingProfileConfig &&
        profileConfig &&
        isWasmReady &&
        runSimulation
      ) {
        try {
          const input: SimulationInput = {
            events: [
              {
                keycode: keyCode,
                event_type: 'press',
                timestamp_us: Date.now() * 1000,
              },
            ],
          };

          const result = await runSimulation(profileConfig.source, input);

          if (result) {
            // Update state from WASM result
            setWasmState({
              modifiers: result.final_state.active_modifiers.map(
                (id) => `MD_${id.toString().padStart(2, '0')}`
              ),
              locks: result.final_state.active_locks.map(
                (id) => `LK_${id.toString().padStart(2, '0')}`
              ),
              layer: result.final_state.active_layer || 'Base',
            });

            // Add output events to log
            result.outputs.forEach((output) => {
              addEvent(
                output.keycode,
                output.event_type as 'press' | 'release',
                keyCode,
                output.keycode
              );
            });

            // Update state display
            setState({
              activeLayer: result.final_state.active_layer || 'MD_00 (Base)',
              modifiers: {
                ctrl: result.final_state.active_modifiers.includes(0),
                shift: result.final_state.active_modifiers.includes(1),
                alt: result.final_state.active_modifiers.includes(2),
                gui: result.final_state.active_modifiers.includes(3),
              },
              locks: {
                capsLock: result.final_state.active_locks.includes(0),
                numLock: result.final_state.active_locks.includes(1),
                scrollLock: result.final_state.active_locks.includes(2),
              },
            });
          }
        } catch (err) {
          console.error('WASM simulation error:', err);
          // For errors, create a synthetic event
          addEvent(
            'ERROR',
            'press',
            keyCode,
            `Error: ${getErrorMessage(err, 'Simulation failed')}`
          );
        }
      } else {
        // Fallback to mock simulation
        const mapping = keyMappings.get(keyCode);
        if (mapping?.type === 'tap_hold' && mapping.threshold) {
          // Start hold timer
          const timerId = window.setTimeout(() => {
            // For hold actions, create output event
            const holdAction = mapping.holdAction || keyCode;
            addEvent(keyCode, 'press', keyCode, holdAction);

            // Update modifiers if applicable
            if (mapping.holdAction === 'Ctrl') {
              setState((prev) => ({
                ...prev,
                modifiers: { ...prev.modifiers, ctrl: true },
              }));
            }
          }, mapping.threshold);

          setHoldTimers((prev) => new Map(prev).set(keyCode, timerId));
        } else {
          // Simple key press output
          const output = mapping?.tapAction || keyCode;
          addEvent(keyCode, 'press', keyCode, output);
        }
      }
    },
    [
      isPaused,
      keyMappings,
      addEvent,
      isUsingProfileConfig,
      profileConfig,
      isWasmReady,
      runSimulation,
    ]
  );

  const handleKeyRelease = useCallback(
    async (keyCode: string) => {
      if (isPaused) return;

      lastActivityRef.current = Date.now();

      // Remove from pressed keys
      setPressedKeys((prev) => {
        const next = new Set(prev);
        next.delete(keyCode);
        return next;
      });

      // Add release event
      addEvent(keyCode, 'release', keyCode, keyCode);

      // If WASM is ready and we have a valid profile config, use WASM simulation
      if (
        isUsingProfileConfig &&
        profileConfig &&
        isWasmReady &&
        runSimulation
      ) {
        try {
          const input: SimulationInput = {
            events: [
              {
                keycode: keyCode,
                event_type: 'release',
                timestamp_us: Date.now() * 1000,
              },
            ],
          };

          const result = await runSimulation(profileConfig.source, input);

          if (result) {
            // Update state from WASM result
            setWasmState({
              modifiers: result.final_state.active_modifiers.map(
                (id) => `MD_${id.toString().padStart(2, '0')}`
              ),
              locks: result.final_state.active_locks.map(
                (id) => `LK_${id.toString().padStart(2, '0')}`
              ),
              layer: result.final_state.active_layer || 'Base',
            });

            // Add output events to log
            result.outputs.forEach((output) => {
              addEvent(
                output.keycode,
                output.event_type as 'press' | 'release',
                keyCode,
                output.keycode
              );
            });

            // Update state display
            setState({
              activeLayer: result.final_state.active_layer || 'MD_00 (Base)',
              modifiers: {
                ctrl: result.final_state.active_modifiers.includes(0),
                shift: result.final_state.active_modifiers.includes(1),
                alt: result.final_state.active_modifiers.includes(2),
                gui: result.final_state.active_modifiers.includes(3),
              },
              locks: {
                capsLock: result.final_state.active_locks.includes(0),
                numLock: result.final_state.active_locks.includes(1),
                scrollLock: result.final_state.active_locks.includes(2),
              },
            });
          }
        } catch (err) {
          console.error('WASM simulation error:', err);
          // For errors, create a synthetic event
          addEvent(
            'ERROR',
            'release',
            keyCode,
            `Error: ${getErrorMessage(err, 'Simulation failed')}`
          );
        }
      } else {
        // Fallback to mock simulation
        // Clear hold timer if exists
        const timerId = holdTimers.get(keyCode);
        if (timerId !== undefined) {
          clearTimeout(timerId);
          setHoldTimers((prev) => {
            const next = new Map(prev);
            next.delete(keyCode);
            return next;
          });

          // If timer was still running, it was a tap
          const mapping = keyMappings.get(keyCode);
          if (mapping?.type === 'tap_hold') {
            const tapAction = mapping.tapAction || keyCode;
            addEvent(keyCode, 'release', keyCode, tapAction);
          }
        }

        // Update modifiers if applicable
        const mapping = keyMappings.get(keyCode);
        if (mapping?.type === 'tap_hold' && mapping.holdAction === 'Ctrl') {
          setState((prev) => ({
            ...prev,
            modifiers: { ...prev.modifiers, ctrl: false },
          }));
        }
      }
    },
    [
      isPaused,
      holdTimers,
      keyMappings,
      addEvent,
      isUsingProfileConfig,
      profileConfig,
      isWasmReady,
      runSimulation,
    ]
  );

  const handleKeyClick = useCallback(
    (keyCode: string) => {
      if (pressedKeys.has(keyCode)) {
        handleKeyRelease(keyCode);
      } else {
        handleKeyPress(keyCode);
      }
    },
    [pressedKeys, handleKeyPress, handleKeyRelease]
  );

  const handleReset = useCallback(() => {
    setPressedKeys(new Set());
    setEvents([]);
    setState({
      activeLayer: 'MD_00 (Base)',
      modifiers: {
        ctrl: false,
        shift: false,
        alt: false,
        gui: false,
      },
      locks: {
        capsLock: false,
        numLock: false,
        scrollLock: false,
      },
    });
    setWasmState(null);
    setIsPaused(false);
    holdTimers.forEach((timerId) => clearTimeout(timerId));
    setHoldTimers(new Map());
    lastActivityRef.current = Date.now();
    addEvent('RESET', 'press', 'RESET', 'Simulator reset');
  }, [holdTimers, addEvent]);

  const handleCopyLog = useCallback(() => {
    const logText = events
      .map((e) => {
        const time = new Date(e.timestamp / 1000).toLocaleTimeString('en-US', {
          hour12: false,
        });
        return `${time}  ${e.eventType.padEnd(8).toUpperCase()}  ${e.input} → ${
          e.output
        }`;
      })
      .join('\n');
    navigator.clipboard.writeText(logText);
  }, [events]);

  // Auto-pause after 60 seconds of inactivity
  useEffect(() => {
    autoPauseTimerRef.current = setInterval(() => {
      const now = Date.now();
      if (now - lastActivityRef.current > AUTO_PAUSE_TIMEOUT && !isPaused) {
        setIsPaused(true);
        addEvent(
          'PAUSE',
          'press',
          'AUTO',
          'Auto-paused after 60 seconds of inactivity'
        );
      }
    }, 1000);

    return () => {
      if (autoPauseTimerRef.current) {
        clearInterval(autoPauseTimerRef.current);
      }
    };
  }, [isPaused, addEvent]);

  // Cleanup hold timers on unmount
  useEffect(() => {
    return () => {
      holdTimers.forEach((timerId) => clearTimeout(timerId));
    };
  }, [holdTimers]);

  return (
    <div className="flex flex-col gap-4 md:gap-6 p-4 md:p-6 lg:p-8">
      <SimulatorHeader
        isLoadingWasm={isLoadingWasm}
        isWasmReady={isWasmReady}
        error={error}
        eventCount={events.length}
        onCopyLog={handleCopyLog}
        onReset={handleReset}
      />

      {/* Configuration Mode Toggle */}
      <ConfigurationModeCard
        useCustomCode={useCustomCode}
        onToggleMode={setUseCustomCode}
        effectiveProfile={effectiveProfile}
        onProfileChange={setSelectedProfile}
        profiles={profiles}
        isLoadingProfiles={isLoadingProfiles}
        isLoadingConfig={isLoadingConfig}
        isUsingProfileConfig={isUsingProfileConfig}
        isWasmReady={isWasmReady}
        isLoadingWasm={isLoadingWasm}
        profileConfig={profileConfig}
        configLoadError={configLoadError}
        customCode={customCode}
        onCustomCodeChange={setCustomCode}
        validationErrors={validationErrors}
        onValidate={setValidationErrors}
      />

      {/* Config Load Error */}
      {configLoadError && (
        <div
          className="bg-red-500/10 border border-red-500 text-red-400 px-4 py-3 rounded-md"
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

      {isPaused && (
        <div
          className="bg-yellow-500/10 border border-yellow-500 text-yellow-500 px-4 py-3 rounded-md"
          role="alert"
        >
          <p className="font-medium">Simulator Paused</p>
          <p className="text-sm mt-1">
            The simulator has been paused after 60 seconds of inactivity. Click
            any key to resume.
          </p>
        </div>
      )}

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-4 md:gap-6">
        {/* State Display */}
        <StateInspectorCard
          isUsingProfileConfig={isUsingProfileConfig}
          wasmState={wasmState}
          state={state}
        />

        {/* Event Log */}
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

// Wrap with WasmProvider to enable WASM features (MonacoEditor validation, compilation, simulation)
const SimulatorPageWithWasm: React.FC = () => (
  <WasmProvider>
    <SimulatorPage />
  </WasmProvider>
);

export default SimulatorPageWithWasm;
