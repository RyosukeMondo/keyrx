// Shared TypeScript types for the KeyRx UI

// Device Management Types
export interface DeviceEntry {
  id: string;
  name: string;
  vendorId: string;
  productId: string;
  scope: DeviceScope;
  layoutPreset: LayoutPreset;
  isConnected: boolean;
}

export type DeviceScope = 'global' | 'profile';

export type LayoutPreset = 'ANSI_104' | 'ISO_105' | 'JIS_109' | 'HHKB' | 'NUMPAD';

// Profile Management Types
export interface ProfileMetadata {
  name: string;
  createdAt: string;
  modifiedAt: string;
  deviceCount: number;
  keyCount: number;
  isActive: boolean;
}

export type Template = 'blank' | 'qwerty' | 'dvorak' | 'colemak';

export interface ActivationResult {
  success: boolean;
  profile: string;
  compiledSize: number;
  compileTimeMs: number;
  errors?: string[];
}

// Configuration Types
export interface KeyMapping {
  type: 'simple' | 'tap_hold' | 'macro' | 'layer_switch';
  tapAction?: string;
  holdAction?: string;
  threshold?: number;
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
  type: 'key_press' | 'key_release' | 'tap' | 'hold' | 'macro' | 'layer_switch';
  keyCode: string;
  layer: string;
  latencyUs: number;
  action?: string;
}

export interface DaemonState {
  activeLayer: string;
  modifiers: string[];
  locks: string[];
  tapHoldPending: boolean;
  uptime: number;
}

// WebSocket Message Types
export interface WSMessage {
  type: 'event' | 'state' | 'latency' | 'error';
  payload: EventRecord | DaemonState | LatencyStats | { message: string };
}
