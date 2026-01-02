import React, { useState, useCallback, useRef, useEffect } from 'react';
import { KeyboardVisualizer } from '../components/KeyboardVisualizer';
import { KeyMapping } from '../components/KeyButton';
import { Button } from '../components/Button';
import { Card } from '../components/Card';
import { useProfiles } from '../hooks/useProfiles';
import { useGetProfileConfig } from '../hooks/useProfileConfig';
import { useWasm } from '../hooks/useWasm';

interface SimulatorEvent {
  timestamp: string;
  type: 'press' | 'release' | 'wait' | 'output';
  key?: string;
  message: string;
}

interface SimulatorState {
  activeLayer: string;
  modifiers: {
    ctrl: boolean;
    shift: boolean;
    alt: boolean;
    gui: boolean;
  };
  locks: {
    capsLock: boolean;
    numLock: boolean;
    scrollLock: boolean;
  };
}

const MAX_EVENTS = 1000;
const AUTO_PAUSE_TIMEOUT = 60000; // 60 seconds

export const SimulatorPage: React.FC = () => {
  const [pressedKeys, setPressedKeys] = useState<Set<string>>(new Set());
  const [events, setEvents] = useState<SimulatorEvent[]>([]);
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
  const lastActivityRef = useRef<number>(Date.now());

  // Profile selection
  const [selectedProfile, setSelectedProfile] = useState<string>('');
  const { data: profiles, isLoading: isLoadingProfiles } = useProfiles();
  const { data: profileConfig, isLoading: isLoadingConfig } = useGetProfileConfig(selectedProfile);
  const { isWasmReady, isLoading: isLoadingWasm, validateConfig } = useWasm();
  const [configLoadError, setConfigLoadError] = useState<string | null>(null);
  const [isUsingProfileConfig, setIsUsingProfileConfig] = useState(false);

  // Set the first profile as selected when profiles load
  useEffect(() => {
    if (profiles && profiles.length > 0 && !selectedProfile) {
      setSelectedProfile(profiles[0].name);
    }
  }, [profiles, selectedProfile]);

  // Validate and load profile config when it changes
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
          const errorMsg = errors.map((e) => `Line ${e.line}: ${e.message}`).join('; ');
          setConfigLoadError(errorMsg);
          setIsUsingProfileConfig(false);
          console.error('Profile config validation failed:', errorMsg);
        } else {
          setConfigLoadError(null);
          setIsUsingProfileConfig(true);
          console.info(`Profile "${profileConfig.name}" loaded successfully`);
        }
      } catch (err) {
        const errorMsg = err instanceof Error ? err.message : String(err);
        setConfigLoadError(errorMsg);
        setIsUsingProfileConfig(false);
        console.error('Failed to load profile config:', err);
      }
    }

    loadProfileConfig();
  }, [profileConfig, isWasmReady, validateConfig]);

  // Mock key mappings for demonstration
  const keyMappings = new Map<string, KeyMapping>([
    [
      'CAPS',
      {
        type: 'tap_hold',
        tapAction: 'Escape',
        holdAction: 'Ctrl',
        threshold: 200,
      },
    ],
    [
      'SPACE',
      {
        type: 'tap_hold',
        tapAction: 'Space',
        holdAction: 'Layer_1',
        threshold: 150,
      },
    ],
  ]);

  const addEvent = useCallback((event: Omit<SimulatorEvent, 'timestamp'>) => {
    const timestamp = new Date().toLocaleTimeString('en-US', { hour12: false });
    setEvents((prev) => {
      const newEvents = [{ ...event, timestamp }, ...prev];
      return newEvents.slice(0, MAX_EVENTS);
    });
  }, []);

  const handleKeyPress = useCallback(
    (keyCode: string) => {
      if (isPaused) return;

      lastActivityRef.current = Date.now();

      // Add to pressed keys
      setPressedKeys((prev) => new Set(prev).add(keyCode));

      // Add press event
      addEvent({
        type: 'press',
        key: keyCode,
        message: `Press ${keyCode}`,
      });

      // Check if key has tap-hold configuration
      const mapping = keyMappings.get(keyCode);
      if (mapping?.type === 'tap_hold' && mapping.threshold) {
        // Start hold timer
        const timerId = window.setTimeout(() => {
          addEvent({
            type: 'wait',
            key: keyCode,
            message: `→Wait ${mapping.threshold}ms (hold)`,
          });
          addEvent({
            type: 'output',
            key: keyCode,
            message: `Output ${mapping.holdAction} (hold)`,
          });

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
        addEvent({
          type: 'output',
          key: keyCode,
          message: `Output ${mapping?.tapAction || keyCode}`,
        });
      }
    },
    [isPaused, keyMappings, addEvent]
  );

  const handleKeyRelease = useCallback(
    (keyCode: string) => {
      if (isPaused) return;

      lastActivityRef.current = Date.now();

      // Remove from pressed keys
      setPressedKeys((prev) => {
        const next = new Set(prev);
        next.delete(keyCode);
        return next;
      });

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
          addEvent({
            type: 'output',
            key: keyCode,
            message: `Output ${mapping.tapAction} (tap)`,
          });
        }
      }

      // Add release event
      addEvent({
        type: 'release',
        key: keyCode,
        message: `Release ${keyCode}`,
      });

      // Update modifiers if applicable
      const mapping = keyMappings.get(keyCode);
      if (mapping?.type === 'tap_hold' && mapping.holdAction === 'Ctrl') {
        setState((prev) => ({
          ...prev,
          modifiers: { ...prev.modifiers, ctrl: false },
        }));
      }
    },
    [isPaused, holdTimers, keyMappings, addEvent]
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
    setIsPaused(false);
    holdTimers.forEach((timerId) => clearTimeout(timerId));
    setHoldTimers(new Map());
    lastActivityRef.current = Date.now();
    addEvent({
      type: 'output',
      message: 'Simulator reset',
    });
  }, [holdTimers, addEvent]);

  const handleCopyLog = useCallback(() => {
    const logText = events
      .map((e) => `${e.timestamp}  ${e.type.padEnd(8)}  ${e.message}`)
      .join('\n');
    navigator.clipboard.writeText(logText);
  }, [events]);

  // Auto-pause after 60 seconds of inactivity
  useEffect(() => {
    autoPauseTimerRef.current = setInterval(() => {
      const now = Date.now();
      if (now - lastActivityRef.current > AUTO_PAUSE_TIMEOUT && !isPaused) {
        setIsPaused(true);
        addEvent({
          type: 'output',
          message: 'Auto-paused after 60 seconds of inactivity',
        });
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
      <div className="flex flex-col lg:flex-row lg:items-start lg:justify-between gap-4">
        <div>
          <h1 className="text-xl md:text-2xl lg:text-3xl font-bold text-slate-100">
            Keyboard Simulator
          </h1>
          <p className="text-sm md:text-base text-slate-400 mt-2">
            Test your configuration by clicking keys or typing. Changes are not
            saved to your keyboard.
          </p>
        </div>
        <div className="flex flex-col sm:flex-row gap-2">
          <Button
            variant="secondary"
            size="md"
            onClick={handleCopyLog}
            aria-label="Copy event log to clipboard"
            disabled={events.length === 0}
            className="w-full sm:w-auto min-h-[44px] sm:min-h-0"
          >
            Copy Event Log
          </Button>
          <Button
            variant="danger"
            size="md"
            onClick={handleReset}
            aria-label="Reset simulator state"
            className="w-full sm:w-auto min-h-[44px] sm:min-h-0"
          >
            Reset Simulator
          </Button>
        </div>
      </div>

      {/* Profile Selector */}
      <Card>
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
              value={selectedProfile}
              onChange={(e) => setSelectedProfile(e.target.value)}
              disabled={isLoadingProfiles || !profiles || profiles.length === 0}
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
              <span className="text-green-400">✓ Profile config active</span>
            )}
            {!isLoadingConfig && profileConfig && !isUsingProfileConfig && !configLoadError && (
              <span className="text-yellow-400">⚠ Using mock data (WASM not ready)</span>
            )}
            {!isWasmReady && !isLoadingWasm && (
              <span className="text-yellow-400">
                ⚠ WASM not available (run build:wasm)
              </span>
            )}
          </div>
        </div>
      </Card>

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
            The simulator is using mock key mappings. Fix the configuration to use
            real profile logic.
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
        <Card className="lg:col-span-1">
          <div className="space-y-3 md:space-y-4">
            <div>
              <h3 className="text-base md:text-lg font-semibold text-slate-100 mb-3">
                State
              </h3>
              <div className="space-y-2">
                <div className="flex justify-between items-center">
                  <span className="text-sm text-slate-400">Active Layer:</span>
                  <span className="text-sm font-mono text-slate-100">
                    {state.activeLayer}
                  </span>
                </div>
              </div>
            </div>

            <div>
              <h4 className="text-sm font-medium text-slate-300 mb-2">
                Modifiers
              </h4>
              <div className="grid grid-cols-2 gap-2">
                {Object.entries(state.modifiers).map(([key, active]) => (
                  <div
                    key={key}
                    className={`px-3 py-2 rounded text-xs font-mono text-center transition-colors ${
                      active
                        ? 'bg-green-500 text-white'
                        : 'bg-slate-700 text-slate-400'
                    }`}
                  >
                    {key.charAt(0).toUpperCase() + key.slice(1)}{' '}
                    {active ? '✓' : ''}
                  </div>
                ))}
              </div>
            </div>

            <div>
              <h4 className="text-sm font-medium text-slate-300 mb-2">
                Locks
              </h4>
              <div className="grid grid-cols-1 gap-2">
                {Object.entries(state.locks).map(([key, active]) => (
                  <div
                    key={key}
                    className={`px-3 py-2 rounded text-xs font-mono text-center transition-colors ${
                      active
                        ? 'bg-blue-500 text-white'
                        : 'bg-slate-700 text-slate-400'
                    }`}
                  >
                    {key
                      .replace(/([A-Z])/g, ' $1')
                      .trim()
                      .split(' ')
                      .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
                      .join(' ')}{' '}
                    {active ? '✓' : ''}
                  </div>
                ))}
              </div>
            </div>
          </div>
        </Card>

        {/* Event Log */}
        <Card className="lg:col-span-2">
          <h3 className="text-base md:text-lg font-semibold text-slate-100 mb-3">
            Event Log
            <span className="text-xs md:text-sm font-normal text-slate-400 ml-2">
              (last {Math.min(events.length, MAX_EVENTS)} events)
            </span>
          </h3>
          <div className="bg-slate-900 rounded-md p-3 md:p-4 h-48 md:h-64 overflow-y-auto font-mono text-xs">
            {events.length === 0 ? (
              <div className="flex items-center justify-center h-full text-slate-500">
                No events yet. Click a key to start.
              </div>
            ) : (
              <div className="space-y-1">
                {events.map((event, index) => (
                  <div
                    key={`${event.timestamp}-${index}`}
                    className="flex items-start gap-3"
                  >
                    <span className="text-slate-500 shrink-0">
                      {event.timestamp}
                    </span>
                    <span
                      className={`shrink-0 ${
                        event.type === 'press'
                          ? 'text-green-400'
                          : event.type === 'release'
                            ? 'text-red-400'
                            : event.type === 'wait'
                              ? 'text-yellow-400'
                              : 'text-blue-400'
                      }`}
                    >
                      {event.type.toUpperCase().padEnd(8)}
                    </span>
                    <span className="text-slate-300">{event.message}</span>
                  </div>
                ))}
              </div>
            )}
          </div>
        </Card>
      </div>

      {/* Keyboard Visualizer */}
      <Card>
        <h3 className="text-base md:text-lg font-semibold text-slate-100 mb-4">
          Interactive Keyboard
        </h3>
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

export default SimulatorPage;
