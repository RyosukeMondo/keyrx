import { useState, useCallback, useEffect, useRef } from 'react';
import type { DaemonState, KeyEvent, SimulatorState } from '@/types/rpc';
import type { SimulationInput } from '@/hooks/useWasm';
import { getErrorMessage } from '@/utils/errorUtils';

const MAX_EVENTS = 1000;

const INITIAL_STATE: SimulatorState = {
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
};

export interface UseSimulatorStateParams {
  profileConfig: { source: string } | undefined;
  isUsingProfileConfig: boolean;
  isWasmReady: boolean;
  runSimulation:
    | ((source: string, input: SimulationInput) => Promise<any>)
    | null;
  keyMappings: Map<string, any>;
}

export interface UseSimulatorStateReturn {
  pressedKeys: Set<string>;
  events: KeyEvent[];
  state: SimulatorState;
  wasmState: DaemonState | null;
  eventCount: number;
  handleKeyClick: (keyCode: string) => void;
  handleReset: () => void;
  handleCopyLog: () => void;
  clearEvents: () => void;
}

export function useSimulatorState(
  params: UseSimulatorStateParams
): UseSimulatorStateReturn {
  const {
    profileConfig,
    isUsingProfileConfig,
    isWasmReady,
    runSimulation,
    keyMappings,
  } = params;

  const [pressedKeys, setPressedKeys] = useState<Set<string>>(new Set());
  const [events, setEvents] = useState<KeyEvent[]>([]);
  const [state, setState] = useState<SimulatorState>({ ...INITIAL_STATE });
  const [holdTimers, setHoldTimers] = useState<Map<string, number>>(new Map());
  const [wasmState, setWasmState] = useState<DaemonState | null>(null);

  // Use ref to access holdTimers in cleanup without stale closures
  const holdTimersRef = useRef(holdTimers);
  holdTimersRef.current = holdTimers;

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

  const processWasmResult = useCallback(
    (result: any, keyCode: string, eventType: 'press' | 'release') => {
      setWasmState({
        modifiers: result.final_state.active_modifiers.map(
          (id: number) => `MD_${id.toString().padStart(2, '0')}`
        ),
        locks: result.final_state.active_locks.map(
          (id: number) => `LK_${id.toString().padStart(2, '0')}`
        ),
        layer: result.final_state.active_layer || 'Base',
      });

      result.outputs.forEach((output: any) => {
        addEvent(
          output.keycode,
          output.event_type as 'press' | 'release',
          keyCode,
          output.keycode
        );
      });

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
    },
    [addEvent]
  );

  const handleKeyPress = useCallback(
    async (keyCode: string) => {
      setPressedKeys((prev) => new Set(prev).add(keyCode));
      addEvent(keyCode, 'press', keyCode, keyCode);

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
            processWasmResult(result, keyCode, 'press');
          }
        } catch (err) {
          console.error('WASM simulation error:', err);
          addEvent(
            'ERROR',
            'press',
            keyCode,
            `Error: ${getErrorMessage(err, 'Simulation failed')}`
          );
        }
      } else {
        const mapping = keyMappings.get(keyCode);
        if (mapping?.type === 'tap_hold' && mapping.threshold) {
          const timerId = window.setTimeout(() => {
            const holdAction = mapping.holdAction || keyCode;
            addEvent(keyCode, 'press', keyCode, holdAction);
            if (mapping.holdAction === 'Ctrl') {
              setState((prev) => ({
                ...prev,
                modifiers: { ...prev.modifiers, ctrl: true },
              }));
            }
          }, mapping.threshold);
          setHoldTimers((prev) => new Map(prev).set(keyCode, timerId));
        } else {
          const output = mapping?.tapAction || keyCode;
          addEvent(keyCode, 'press', keyCode, output);
        }
      }
    },
    [
      keyMappings,
      addEvent,
      isUsingProfileConfig,
      profileConfig,
      isWasmReady,
      runSimulation,
      processWasmResult,
    ]
  );

  const handleKeyRelease = useCallback(
    async (keyCode: string) => {
      setPressedKeys((prev) => {
        const next = new Set(prev);
        next.delete(keyCode);
        return next;
      });
      addEvent(keyCode, 'release', keyCode, keyCode);

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
            processWasmResult(result, keyCode, 'release');
          }
        } catch (err) {
          console.error('WASM simulation error:', err);
          addEvent(
            'ERROR',
            'release',
            keyCode,
            `Error: ${getErrorMessage(err, 'Simulation failed')}`
          );
        }
      } else {
        const timerId = holdTimers.get(keyCode);
        if (timerId !== undefined) {
          clearTimeout(timerId);
          setHoldTimers((prev) => {
            const next = new Map(prev);
            next.delete(keyCode);
            return next;
          });
          const mapping = keyMappings.get(keyCode);
          if (mapping?.type === 'tap_hold') {
            const tapAction = mapping.tapAction || keyCode;
            addEvent(keyCode, 'release', keyCode, tapAction);
          }
        }
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
      holdTimers,
      keyMappings,
      addEvent,
      isUsingProfileConfig,
      profileConfig,
      isWasmReady,
      runSimulation,
      processWasmResult,
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
    setState({ ...INITIAL_STATE });
    setWasmState(null);
    holdTimers.forEach((timerId) => clearTimeout(timerId));
    setHoldTimers(new Map());
    addEvent('RESET', 'press', 'RESET', 'Simulator reset');
  }, [holdTimers, addEvent]);

  const handleCopyLog = useCallback(() => {
    const logText = events
      .map((e) => {
        const time = new Date(e.timestamp / 1000).toLocaleTimeString('en-US', {
          hour12: false,
        });
        return `${time}  ${e.eventType.padEnd(8).toUpperCase()}  ${e.input} → ${e.output}`;
      })
      .join('\n');
    navigator.clipboard.writeText(logText);
  }, [events]);

  // Cleanup hold timers on unmount
  useEffect(() => {
    return () => {
      holdTimersRef.current.forEach((timerId) => clearTimeout(timerId));
    };
  }, []);

  return {
    pressedKeys,
    events,
    state,
    wasmState,
    eventCount: events.length,
    handleKeyClick,
    handleReset,
    handleCopyLog,
    clearEvents,
  };
}
