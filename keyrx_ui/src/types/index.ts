// Shared TypeScript types for the KeyRx UI
// This file re-exports generated types and defines types that can't be auto-generated

// Re-export generated types from typeshare (SSOT)
// Note: RPC protocol types (ClientMessage, ServerMessage, RpcError) are
// canonically exported from ./rpc which provides stricter typing.
export type {
  DeviceEntry,
  DeviceRpcInfo,
  ProfileMetadata,
  ProfileRpcInfo,
  ProfileConfigRpc,
  ActivationResult,
  ActivationRpcResult,
  ProfileTemplate,
  DaemonState,
  KeyEventData,
  LatencyStats,
  ErrorData,
  LatencyRpcStats,
  EventRpcEntry,
  RestartResult,
} from './generated';

// Re-export enums
export { ProfileTemplate } from './generated';

// Additional types for DeviceEntry that aren't in Rust
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
  // New fields for enhanced event info
  input?: string;
  output?: string;
  deviceId?: string;
  deviceName?: string;
  mappingType?: string;
  mappingTriggered?: boolean;
}

// WebSocket Message Types
// Note: DaemonEvent uses serde(flatten) which typeshare doesn't support,
// so we maintain the manual definition here for compatibility
export type WSMessage =
  | { type: 'event'; payload: KeyEventData; seq: number }
  | { type: 'state'; payload: DaemonState; seq: number }
  | { type: 'latency'; payload: LatencyStats; seq: number }
  | { type: 'error'; payload: ErrorData; seq: number };

// Raw key event payload from daemon (alias to generated type)
export type KeyEventPayload = KeyEventData;
