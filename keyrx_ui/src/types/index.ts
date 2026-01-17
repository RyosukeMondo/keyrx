// Shared TypeScript types for the KeyRx UI

// Device Management Types
export interface DeviceEntry {
  id: string;
  name: string;
  path: string;
  serial: string | null;
  active: boolean;
  scope: string | null; // "global" | "device-specific"
  layout: string | null;
  isVirtual: boolean; // true if device is daemon-created (uinput), false if physical hardware
  enabled: boolean; // true if device is enabled (shown in UI), false if disabled (hidden)
}

export type DeviceScope = 'global' | 'device-specific';

export type LayoutPreset =
  | 'ANSI_104'
  | 'ISO_105'
  | 'JIS_109'
  | 'HHKB'
  | 'NUMPAD';

// Profile Management Types
export interface ProfileMetadata {
  name: string;
  rhaiPath: string;
  krxPath: string;
  createdAt: string;
  modifiedAt: string;
  deviceCount: number;
  keyCount: number;
  isActive: boolean;
}

export type Template =
  | 'blank'
  | 'simple_remap'
  | 'capslock_escape'
  | 'vim_navigation'
  | 'gaming';

export interface ActivationResult {
  success: boolean;
  profile: string;
  compiledSize: number;
  compileTimeMs: number;
  errors?: string[];
}

// Configuration Types
export interface KeyMapping {
  type:
    | 'simple'
    | 'modifier'
    | 'lock'
    | 'tap_hold'
    | 'layer_active'
    | 'macro'
    | 'layer_switch';
  tapAction?: string;
  holdAction?: string;
  threshold?: number;
  modifierKey?: string;
  lockKey?: string;
  macroSteps?: MacroStep[];
  targetLayer?: string;
}

export interface MacroStep {
  type: 'press' | 'release' | 'delay';
  key?: string;
  delayMs?: number;
}

// Metrics Types
export interface LatencyStats {
  min: number;
  max: number;
  avg: number;
  p50: number;
  p95: number;
  p99: number;
  samples: number;
  timestamp: string;
}

export interface EventRecord {
  id: string;
  timestamp: string;
  type:
    | 'key_press'
    | 'key_release'
    | 'press'
    | 'release'
    | 'tap'
    | 'hold'
    | 'macro'
    | 'layer_switch';
  keyCode: string;
  layer: string;
  latencyUs: number;
  action?: string;
  // New fields for enhanced event info
  input?: string;
  output?: string;
  deviceId?: string;
  deviceName?: string;
  mappingType?: string;
  mappingTriggered?: boolean;
}

export interface DaemonState {
  activeLayer: string;
  modifiers: string[];
  locks: string[];
  tapHoldPending: boolean;
  uptime: number;
  activeProfile?: string | null;
}

// WebSocket Message Types - Discriminated union for proper type narrowing
export type WSMessage =
  | { type: 'event'; payload: KeyEventPayload }
  | { type: 'state'; payload: DaemonState }
  | { type: 'latency'; payload: LatencyStats }
  | { type: 'error'; payload: { message: string } };

// Raw key event payload from daemon (before transformation to EventRecord)
export interface KeyEventPayload {
  timestamp: number;
  keyCode: string;
  eventType: string;
  input: string;
  output: string;
  latency: number;
  deviceId?: string;
  deviceName?: string;
  mappingType?: string;
  mappingTriggered?: boolean;
}
