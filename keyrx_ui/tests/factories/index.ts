/**
 * Test Data Factories
 *
 * This module provides factory functions for generating realistic test data
 * using @faker-js/faker and the factory pattern. All factories support:
 * - Default values with sensible randomization
 * - Partial overrides for specific test scenarios
 * - Deterministic seeding for reproducible tests
 *
 * Usage:
 *   import { createProfile, createDevice, seed } from '../factories';
 *
 *   // Random data
 *   const profile = createProfile();
 *
 *   // Partial override
 *   const activeProfile = createProfile({ isActive: true });
 *
 *   // Deterministic (same data every time)
 *   seed(12345);
 *   const profile1 = createProfile();
 *   seed(12345);
 *   const profile2 = createProfile(); // Identical to profile1
 */

import { faker } from '@faker-js/faker';
import type {
  ProfileMetadata,
  DeviceEntry,
  DeviceScope,
  LayoutPreset,
  LatencyStats,
  EventRecord,
  DaemonState,
  KeyMapping,
  MacroStep,
  ActivationResult,
} from '../../src/types';
import type {
  KeyEvent,
  LatencyMetrics,
  DaemonState as RpcDaemonState,
  RpcError,
  ClientMessage,
  ServerMessage,
} from '../../src/types/rpc';

// =============================================================================
// Seeding for Deterministic Tests
// =============================================================================

/**
 * Set a deterministic seed for faker.
 * Call this at the start of tests that need reproducible data.
 *
 * @example
 *   beforeEach(() => {
 *     seed(12345);
 *   });
 */
export function seed(value: number): void {
  faker.seed(value);
}

/**
 * Reset faker to random mode (undo seeding).
 */
export function resetSeed(): void {
  faker.seed();
}

// =============================================================================
// Profile Factories
// =============================================================================

/**
 * Create a realistic ProfileMetadata object.
 *
 * @example
 *   const profile = createProfile();
 *   const activeProfile = createProfile({ isActive: true });
 */
export function createProfile(overrides?: Partial<ProfileMetadata>): ProfileMetadata {
  return {
    name: overrides?.name ?? faker.word.adjective() + '-' + faker.word.noun(),
    createdAt: overrides?.createdAt ?? faker.date.past().toISOString(),
    modifiedAt: overrides?.modifiedAt ?? faker.date.recent().toISOString(),
    deviceCount: overrides?.deviceCount ?? faker.number.int({ min: 1, max: 5 }),
    keyCount: overrides?.keyCount ?? faker.number.int({ min: 0, max: 104 }),
    isActive: overrides?.isActive ?? false,
  };
}

/**
 * Active Profile type for ActiveProfileCard component.
 */
export interface ActiveProfile {
  name: string;
  layers: number;
  mappings: number;
  modifiedAt: string;
}

/**
 * Create a realistic ActiveProfile object.
 *
 * @example
 *   const profile = createActiveProfile();
 *   const customProfile = createActiveProfile({ layers: 3 });
 */
export function createActiveProfile(overrides?: Partial<ActiveProfile>): ActiveProfile {
  return {
    name: overrides?.name ?? faker.word.adjective() + '-' + faker.word.noun(),
    layers: overrides?.layers ?? faker.number.int({ min: 1, max: 5 }),
    mappings: overrides?.mappings ?? faker.number.int({ min: 0, max: 104 }),
    modifiedAt: overrides?.modifiedAt ?? faker.date.recent().toISOString(),
  };
}

/**
 * Create multiple profiles.
 *
 * @example
 *   const profiles = createProfiles(5);
 *   const profiles = createProfiles(3, { isActive: false });
 */
export function createProfiles(
  count: number,
  overrides?: Partial<ProfileMetadata>
): ProfileMetadata[] {
  return Array.from({ length: count }, () => createProfile(overrides));
}

/**
 * Create an ActivationResult object.
 */
export function createActivationResult(
  overrides?: Partial<ActivationResult>
): ActivationResult {
  return {
    success: overrides?.success ?? true,
    profile: overrides?.profile ?? faker.word.noun(),
    compiledSize: overrides?.compiledSize ?? faker.number.int({ min: 1024, max: 65536 }),
    compileTimeMs: overrides?.compileTimeMs ?? faker.number.int({ min: 10, max: 500 }),
    errors: overrides?.errors,
  };
}

// =============================================================================
// Device Factories
// =============================================================================

const DEVICE_SCOPES: DeviceScope[] = ['global', 'profile'];
const LAYOUT_PRESETS: LayoutPreset[] = ['ANSI_104', 'ISO_105', 'JIS_109', 'HHKB', 'NUMPAD'];

/**
 * Create a realistic DeviceEntry object.
 *
 * @example
 *   const device = createDevice();
 *   const connectedDevice = createDevice({ isConnected: true });
 */
export function createDevice(overrides?: Partial<DeviceEntry>): DeviceEntry {
  return {
    id: overrides?.id ?? `device-${faker.string.uuid()}`,
    name:
      overrides?.name ??
      `${faker.company.name()} ${faker.commerce.productName()} Keyboard`,
    path: overrides?.path ?? `/dev/input/event${faker.number.int({ min: 0, max: 20 })}`,
    serial: overrides?.serial ?? faker.string.alphanumeric(8).toUpperCase(),
    active: overrides?.active ?? true,
    scope: overrides?.scope ?? faker.helpers.arrayElement(['global', 'device-specific', null]),
    layout: overrides?.layout ?? faker.helpers.arrayElement(['ANSI_104', 'ISO_105', 'JIS_109', null]),
  };
}

/**
 * Create multiple devices.
 */
export function createDevices(count: number, overrides?: Partial<DeviceEntry>): DeviceEntry[] {
  return Array.from({ length: count }, () => createDevice(overrides));
}

// =============================================================================
// Key Event Factories
// =============================================================================

const KEY_CODES = [
  'KEY_A',
  'KEY_B',
  'KEY_C',
  'KEY_D',
  'KEY_E',
  'KEY_SPACE',
  'KEY_ENTER',
  'KEY_ESC',
  'KEY_LEFTSHIFT',
  'KEY_LEFTCTRL',
];

/**
 * Create a realistic KeyEvent (RPC type).
 *
 * @example
 *   const event = createKeyEvent();
 *   const press = createKeyEvent({ eventType: 'press' });
 */
export function createKeyEvent(overrides?: Partial<KeyEvent>): KeyEvent {
  const keyCode = overrides?.keyCode ?? faker.helpers.arrayElement(KEY_CODES);
  return {
    timestamp: overrides?.timestamp ?? Date.now() * 1000, // microseconds
    keyCode,
    eventType: overrides?.eventType ?? faker.helpers.arrayElement(['press', 'release'] as const),
    input: overrides?.input ?? keyCode,
    output: overrides?.output ?? keyCode,
    latency: overrides?.latency ?? faker.number.int({ min: 50, max: 500 }),
  };
}

/**
 * Create an EventRecord (UI type).
 */
export function createEventRecord(overrides?: Partial<EventRecord>): EventRecord {
  return {
    id: overrides?.id ?? faker.string.uuid(),
    timestamp: overrides?.timestamp ?? faker.date.recent().toISOString(),
    type:
      overrides?.type ??
      faker.helpers.arrayElement([
        'key_press',
        'key_release',
        'tap',
        'hold',
        'macro',
        'layer_switch',
      ] as const),
    keyCode: overrides?.keyCode ?? faker.helpers.arrayElement(KEY_CODES),
    layer: overrides?.layer ?? 'base',
    latencyUs: overrides?.latencyUs ?? faker.number.int({ min: 50, max: 500 }),
    action: overrides?.action,
  };
}

/**
 * Create multiple key events.
 */
export function createKeyEvents(count: number, overrides?: Partial<KeyEvent>): KeyEvent[] {
  return Array.from({ length: count }, () => createKeyEvent(overrides));
}

// =============================================================================
// Latency & Metrics Factories
// =============================================================================

/**
 * Create realistic LatencyMetrics (RPC type).
 *
 * @example
 *   const metrics = createLatencyMetrics();
 */
export function createLatencyMetrics(overrides?: Partial<LatencyMetrics>): LatencyMetrics {
  const min = overrides?.min ?? faker.number.int({ min: 10, max: 50 });
  const max = overrides?.max ?? faker.number.int({ min: 300, max: 1000 });
  const avg = overrides?.avg ?? faker.number.int({ min: min, max: max });

  return {
    min,
    max,
    avg,
    p95: overrides?.p95 ?? faker.number.int({ min: avg, max: max }),
    p99: overrides?.p99 ?? faker.number.int({ min: avg, max: max }),
    timestamp: overrides?.timestamp ?? Date.now() * 1000, // microseconds
  };
}

/**
 * Create LatencyStats (UI type).
 */
export function createLatencyStats(overrides?: Partial<LatencyStats>): LatencyStats {
  const min = overrides?.min ?? faker.number.int({ min: 10, max: 50 });
  const max = overrides?.max ?? faker.number.int({ min: 300, max: 1000 });
  const avg = overrides?.avg ?? faker.number.int({ min: min, max: max });

  return {
    min,
    max,
    avg,
    p50: overrides?.p50 ?? faker.number.int({ min: avg, max: max }),
    p95: overrides?.p95 ?? faker.number.int({ min: avg, max: max }),
    p99: overrides?.p99 ?? faker.number.int({ min: avg, max: max }),
    samples: overrides?.samples ?? faker.number.int({ min: 100, max: 10000 }),
    timestamp: overrides?.timestamp ?? faker.date.recent().toISOString(),
  };
}

// =============================================================================
// Daemon State Factories
// =============================================================================

/**
 * Create DaemonState (UI type).
 */
export function createDaemonState(overrides?: Partial<DaemonState>): DaemonState {
  return {
    activeLayer: overrides?.activeLayer ?? 'base',
    modifiers: overrides?.modifiers ?? [],
    locks: overrides?.locks ?? [],
    tapHoldPending: overrides?.tapHoldPending ?? false,
    uptime: overrides?.uptime ?? faker.number.int({ min: 0, max: 86400 }),
    activeProfile: overrides?.activeProfile ?? faker.word.noun(),
  };
}

/**
 * Create RpcDaemonState (RPC type).
 */
export function createRpcDaemonState(overrides?: Partial<RpcDaemonState>): RpcDaemonState {
  return {
    modifiers: overrides?.modifiers ?? [],
    locks: overrides?.locks ?? [],
    layer: overrides?.layer ?? 'base',
  };
}

// =============================================================================
// Configuration Factories
// =============================================================================

/**
 * Create a MacroStep.
 */
export function createMacroStep(overrides?: Partial<MacroStep>): MacroStep {
  const type = overrides?.type ?? faker.helpers.arrayElement(['press', 'release', 'delay'] as const);

  if (type === 'delay') {
    return {
      type: 'delay',
      delayMs: overrides?.delayMs ?? faker.number.int({ min: 10, max: 1000 }),
    };
  }

  return {
    type,
    key: overrides?.key ?? faker.helpers.arrayElement(KEY_CODES),
  };
}

/**
 * Create a KeyMapping.
 */
export function createKeyMapping(overrides?: Partial<KeyMapping>): KeyMapping {
  const type =
    overrides?.type ??
    faker.helpers.arrayElement(['simple', 'tap_hold', 'macro', 'layer_switch'] as const);

  const base: KeyMapping = { type };

  if (type === 'tap_hold') {
    return {
      ...base,
      tapAction: overrides?.tapAction ?? faker.helpers.arrayElement(KEY_CODES),
      holdAction: overrides?.holdAction ?? faker.helpers.arrayElement(KEY_CODES),
      threshold: overrides?.threshold ?? faker.number.int({ min: 100, max: 500 }),
    };
  }

  if (type === 'macro') {
    return {
      ...base,
      macroSteps: overrides?.macroSteps ?? [createMacroStep(), createMacroStep()],
    };
  }

  if (type === 'layer_switch') {
    return {
      ...base,
      targetLayer: overrides?.targetLayer ?? 'layer-1',
    };
  }

  return base;
}

// =============================================================================
// RPC Message Factories
// =============================================================================

/**
 * Create an RPC error.
 */
export function createRpcError(overrides?: Partial<RpcError>): RpcError {
  return {
    code: overrides?.code ?? -32603,
    message: overrides?.message ?? faker.lorem.sentence(),
    data: overrides?.data,
  };
}

/**
 * Create a query ClientMessage.
 */
export function createQueryMessage(
  method: string,
  params?: unknown,
  id?: string
): ClientMessage {
  return {
    type: 'query',
    id: id ?? faker.string.uuid(),
    method: method as any,
    params,
  };
}

/**
 * Create a command ClientMessage.
 */
export function createCommandMessage(
  method: string,
  params?: unknown,
  id?: string
): ClientMessage {
  return {
    type: 'command',
    id: id ?? faker.string.uuid(),
    method: method as any,
    params,
  };
}

/**
 * Create a response ServerMessage.
 */
export function createResponse(
  id: string,
  result?: unknown,
  error?: RpcError
): Extract<ServerMessage, { type: 'response' }> {
  return {
    type: 'response',
    id,
    result,
    error,
  };
}

/**
 * Create a connected ServerMessage.
 */
export function createConnectedMessage(
  sessionId?: string
): Extract<ServerMessage, { type: 'connected' }> {
  return {
    type: 'connected',
    version: '1.0.0',
    timestamp: Date.now() * 1000,
  };
}

/**
 * Create an event ServerMessage.
 */
export function createEventMessage(
  channel: string,
  data: unknown
): Extract<ServerMessage, { type: 'event' }> {
  return {
    type: 'event',
    channel: channel as any,
    data,
  };
}
