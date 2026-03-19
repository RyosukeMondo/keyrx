// Shared TypeScript types for the KeyRx UI
// This file re-exports generated types and defines frontend-specific types
// that extend what the REST API actually returns.

// Re-export generated types that match the frontend usage exactly
export type {
  DeviceRpcInfo,
  ProfileRpcInfo,
  ProfileConfigRpc,
  ActivationRpcResult,
  KeyEventData,
  ErrorData,
  LatencyRpcStats,
  EventRpcEntry,
  RestartResult,
} from './generated';

// Import types used locally in this file
import type { KeyEventData, ErrorData } from './generated';

// Re-export ProfileTemplate enum (both type and value)
export { ProfileTemplate } from './generated';

// Alias — some code imports as Template
export type Template = import('./generated').ProfileTemplate;

// Frontend DeviceEntry — matches what fetchDevices() returns after
// mapping the REST API /api/devices response
export interface DeviceEntry {
  id: string;
  name: string;
  path: string;
  serial: string | null;
  active: boolean;
  scope: string;
  layout: string | null;
  isVirtual: boolean;
  enabled: boolean;
  lastSeen?: number;
}

// Frontend ProfileMetadata — matches what fetchProfiles() returns after
// mapping the REST API /api/profiles response
export interface ProfileMetadata {
  name: string;
  rhaiPath: string;
  krxPath: string;
  createdAt: string;
  modifiedAt: string;
  layerCount?: number;
  deviceCount: number;
  keyCount: number;
  isActive: boolean;
  activatedAt?: string;
  activatedBy?: string;
}

// Frontend ActivationResult — matches what activateProfile() returns
export interface ActivationResult {
  success: boolean;
  profile: string;
  compiledSize: number;
  compileTimeMs: number;
  errors: string[];
}

// Frontend DaemonState — union of generated fields + REST /api/daemon/state extras
export interface DaemonState {
  modifiers: string[];
  locks: string[];
  layer: string;
  activeProfile?: string;
  activeLayer?: string;
  tapHoldPending?: boolean;
}

// Frontend LatencyStats — REST /api/metrics/latency response
export interface LatencyStats {
  min: number;
  avg: number;
  max: number;
  p50?: number;
  p95: number;
  p99: number;
  timestamp: number;
  samples?: number;
}

export type DeviceScope = 'global' | 'device-specific';

export type LayoutPreset =
  | 'ANSI_104'
  | 'ISO_105'
  | 'JIS_109'
  | 'HHKB'
  | 'NUMPAD';

// Configuration Types (not represented in Rust backend)
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

// Frontend-specific event record (transformed from KeyEventData)
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
  input?: string;
  output?: string;
  deviceId?: string;
  deviceName?: string;
  mappingType?: string;
  mappingTriggered?: boolean;
}

// WebSocket Message Types
export type WSMessage =
  | { type: 'event'; payload: KeyEventData; seq: number }
  | { type: 'state'; payload: DaemonState; seq: number }
  | { type: 'latency'; payload: LatencyStats; seq: number }
  | { type: 'error'; payload: ErrorData; seq: number };

// Raw key event payload from daemon (alias to generated type)
export type KeyEventPayload = KeyEventData;
