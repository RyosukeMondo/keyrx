/**
 * API test helpers for Playwright E2E tests
 *
 * Provides typed API request functions using Playwright's APIRequestContext.
 * All responses are validated against Zod schemas for type safety.
 */

import type { APIRequestContext } from '@playwright/test';
import { validateApiResponse } from '../../../src/api/schemas';
import * as Schemas from '../../../src/api/schemas';
import type {
  DeviceEntry,
  ProfileMetadata,
  ActivationResult,
  LatencyStats,
  EventRecord,
  DaemonState,
  DeviceScope,
  Template
} from '../../../src/types';

/**
 * Simulation-related type definitions
 */

/** Event type for simulation */
export type EventType = 'press' | 'release';

/** Output event from simulation */
export interface OutputEvent {
  /** Output key identifier */
  key: string;
  /** Event type (press or release) */
  event_type: EventType;
  /** Timestamp when event was generated (microseconds) */
  timestamp_us: number;
}

/** Response from simulation endpoints */
export interface SimulationResponse {
  success: boolean;
  outputs: OutputEvent[];
}

/** Response from load-profile endpoint */
export interface SimulationLoadResponse {
  success: boolean;
  message: string;
}

/** Result for a single scenario */
export interface ScenarioResult {
  /** Scenario name */
  scenario: string;
  /** Whether the scenario passed */
  passed: boolean;
  /** Input events */
  input: Array<{
    device_id?: string | null;
    timestamp_us: number;
    key: string;
    event_type: EventType;
  }>;
  /** Output events generated */
  output: OutputEvent[];
  /** Optional error message if failed */
  error?: string | null;
}

/** Response from all scenarios endpoint */
export interface AllScenariosResponse {
  success: boolean;
  /** Results for all scenarios */
  scenarios: ScenarioResult[];
  /** Total number of scenarios */
  total: number;
  /** Number of passed scenarios */
  passed: number;
  /** Number of failed scenarios */
  failed: number;
}

/**
 * API helper class for E2E tests
 * Uses Playwright's APIRequestContext for HTTP requests
 */
export class ApiHelpers {
  private request: APIRequestContext;
  private baseUrl: string;

  constructor(request: APIRequestContext, baseUrl: string) {
    this.request = request;
    this.baseUrl = baseUrl;
  }

  /**
   * GET /api/status - Fetch daemon status
   */
  async getStatus(): Promise<{
    status: string;
    version: string;
    daemon_running: boolean;
    uptime_secs?: number | null;
    active_profile?: string | null;
    device_count?: number | null;
  }> {
    const response = await this.request.get(`${this.baseUrl}/api/status`);
    if (!response.ok()) {
      throw new Error(`GET /api/status failed: ${response.status()} ${response.statusText()}`);
    }
    const data = await response.json();
    return validateApiResponse(Schemas.StatusResponseSchema, data, 'GET /api/status');
  }

  /**
   * GET /api/health - Fetch health status
   */
  async getHealth(): Promise<{ healthy: boolean; uptime: number; version: string }> {
    const response = await this.request.get(`${this.baseUrl}/api/health`);
    if (!response.ok()) {
      throw new Error(`GET /api/health failed: ${response.status()} ${response.statusText()}`);
    }
    return response.json();
  }

  /**
   * GET /api/version - Fetch daemon version
   */
  async getVersion(): Promise<{ version: string }> {
    const response = await this.request.get(`${this.baseUrl}/api/version`);
    if (!response.ok()) {
      throw new Error(`GET /api/version failed: ${response.status()} ${response.statusText()}`);
    }
    return response.json();
  }

  /**
   * GET /api/devices - Fetch all devices
   */
  async getDevices(): Promise<DeviceEntry[]> {
    const response = await this.request.get(`${this.baseUrl}/api/devices`);
    if (!response.ok()) {
      throw new Error(`GET /api/devices failed: ${response.status()} ${response.statusText()}`);
    }
    const data = await response.json();
    const validated = validateApiResponse(Schemas.DeviceListResponseSchema, data, 'GET /api/devices');

    // Map validated response to DeviceEntry format
    return validated.devices.map((device) => ({
      id: device.id,
      name: device.name,
      path: device.path,
      serial: device.serial || null,
      active: device.active,
      scope: device.scope === 'DeviceSpecific' ? 'device-specific' :
             device.scope === 'Global' ? 'global' : 'global',
      layout: device.layout || null,
    }));
  }

  /**
   * PATCH /api/devices/:id - Update device configuration
   */
  async updateDevice(id: string, updates: { name?: string; scope?: DeviceScope; layout?: string }): Promise<{ success: boolean }> {
    const response = await this.request.patch(`${this.baseUrl}/api/devices/${id}`, {
      data: updates,
    });
    if (!response.ok()) {
      throw new Error(`PATCH /api/devices/${id} failed: ${response.status()} ${response.statusText()}`);
    }
    const data = await response.json();
    return validateApiResponse(Schemas.UpdateDeviceConfigResponseSchema, data, `PATCH /api/devices/${id}`);
  }

  /**
   * PUT /api/devices/:id/name - Rename device
   */
  async renameDevice(id: string, name: string): Promise<DeviceEntry> {
    const response = await this.request.put(`${this.baseUrl}/api/devices/${id}/name`, {
      data: { name },
    });
    if (!response.ok()) {
      throw new Error(`PUT /api/devices/${id}/name failed: ${response.status()} ${response.statusText()}`);
    }
    const data = await response.json();
    return validateApiResponse(Schemas.DeviceEntrySchema, data, `PUT /api/devices/${id}/name`);
  }

  /**
   * PUT /api/devices/:id/layout - Set device layout
   */
  async setDeviceLayout(id: string, layout: string): Promise<DeviceEntry> {
    const response = await this.request.put(`${this.baseUrl}/api/devices/${id}/layout`, {
      data: { layout },
    });
    if (!response.ok()) {
      throw new Error(`PUT /api/devices/${id}/layout failed: ${response.status()} ${response.statusText()}`);
    }
    const data = await response.json();
    return validateApiResponse(Schemas.DeviceEntrySchema, data, `PUT /api/devices/${id}/layout`);
  }

  /**
   * GET /api/devices/:id/layout - Get device layout
   */
  async getDeviceLayout(id: string): Promise<{ layout: string }> {
    const response = await this.request.get(`${this.baseUrl}/api/devices/${id}/layout`);
    if (!response.ok()) {
      throw new Error(`GET /api/devices/${id}/layout failed: ${response.status()} ${response.statusText()}`);
    }
    return response.json();
  }

  /**
   * DELETE /api/devices/:id - Forget device
   */
  async forgetDevice(id: string): Promise<DeviceEntry> {
    const response = await this.request.delete(`${this.baseUrl}/api/devices/${id}`);
    if (!response.ok()) {
      throw new Error(`DELETE /api/devices/${id} failed: ${response.status()} ${response.statusText()}`);
    }
    const data = await response.json();
    return validateApiResponse(Schemas.DeviceEntrySchema, data, `DELETE /api/devices/${id}`);
  }

  /**
   * GET /api/profiles - Fetch all profiles
   */
  async getProfiles(): Promise<ProfileMetadata[]> {
    const response = await this.request.get(`${this.baseUrl}/api/profiles`);
    if (!response.ok()) {
      throw new Error(`GET /api/profiles failed: ${response.status()} ${response.statusText()}`);
    }
    const data = await response.json();
    const validated = validateApiResponse(Schemas.ProfileListResponseSchema, data, 'GET /api/profiles');

    // Map response to ProfileMetadata format
    return validated.profiles.map((p) => ({
      name: p.name,
      rhaiPath: p.rhaiPath,
      krxPath: p.krxPath,
      createdAt: p.createdAt,
      modifiedAt: p.modifiedAt,
      deviceCount: p.deviceCount,
      keyCount: p.keyCount,
      isActive: p.isActive,
    }));
  }

  /**
   * POST /api/profiles - Create new profile
   */
  async createProfile(name: string, template: Template = 'blank'): Promise<ProfileMetadata> {
    const response = await this.request.post(`${this.baseUrl}/api/profiles`, {
      data: { name, template },
    });
    if (!response.ok()) {
      throw new Error(`POST /api/profiles failed: ${response.status()} ${response.statusText()}`);
    }
    const data = await response.json();
    return validateApiResponse(Schemas.ProfileRpcInfoSchema, data, 'POST /api/profiles');
  }

  /**
   * GET /api/profiles/active - Get active profile
   */
  async getActiveProfile(): Promise<{ name: string | null }> {
    const response = await this.request.get(`${this.baseUrl}/api/profiles/active`);
    if (!response.ok()) {
      throw new Error(`GET /api/profiles/active failed: ${response.status()} ${response.statusText()}`);
    }
    return response.json();
  }

  /**
   * POST /api/profiles/:name/activate - Activate profile
   */
  async activateProfile(name: string): Promise<ActivationResult> {
    const response = await this.request.post(`${this.baseUrl}/api/profiles/${name}/activate`);
    if (!response.ok()) {
      throw new Error(`POST /api/profiles/${name}/activate failed: ${response.status()} ${response.statusText()}`);
    }
    const data = await response.json();
    const validated = validateApiResponse(Schemas.ActivationRpcResultSchema, data, `POST /api/profiles/${name}/activate`);

    // Map RPC activation result to ActivationResult format
    return {
      success: validated.success,
      profile: name,
      compiledSize: 0,
      compileTimeMs: validated.compile_time_ms,
      errors: validated.error ? [validated.error] : [],
    };
  }

  /**
   * GET /api/profiles/:name/config - Get profile configuration
   */
  async getProfileConfig(name: string): Promise<{ name: string; config: string }> {
    const response = await this.request.get(`${this.baseUrl}/api/profiles/${name}/config`);
    if (!response.ok()) {
      throw new Error(`GET /api/profiles/${name}/config failed: ${response.status()} ${response.statusText()}`);
    }
    const data = await response.json();
    return validateApiResponse(Schemas.ProfileConfigRpcSchema, data, `GET /api/profiles/${name}/config`);
  }

  /**
   * PUT /api/profiles/:name/config - Update profile configuration
   */
  async updateProfileConfig(name: string, config: string): Promise<{ success: boolean }> {
    const response = await this.request.put(`${this.baseUrl}/api/profiles/${name}/config`, {
      data: { config },
    });
    if (!response.ok()) {
      throw new Error(`PUT /api/profiles/${name}/config failed: ${response.status()} ${response.statusText()}`);
    }
    return response.json();
  }

  /**
   * DELETE /api/profiles/:name - Delete profile
   */
  async deleteProfile(name: string): Promise<{ success: boolean }> {
    const response = await this.request.delete(`${this.baseUrl}/api/profiles/${name}`);
    if (!response.ok()) {
      throw new Error(`DELETE /api/profiles/${name} failed: ${response.status()} ${response.statusText()}`);
    }
    return { success: true };
  }

  /**
   * POST /api/profiles/:name/duplicate - Duplicate profile
   */
  async duplicateProfile(sourceName: string, newName: string): Promise<ProfileMetadata> {
    const response = await this.request.post(`${this.baseUrl}/api/profiles/${sourceName}/duplicate`, {
      data: { newName },
    });
    if (!response.ok()) {
      throw new Error(`POST /api/profiles/${sourceName}/duplicate failed: ${response.status()} ${response.statusText()}`);
    }
    const data = await response.json();
    return validateApiResponse(Schemas.ProfileRpcInfoSchema, data, `POST /api/profiles/${sourceName}/duplicate`);
  }

  /**
   * PUT /api/profiles/:name/rename - Rename profile
   */
  async renameProfile(oldName: string, newName: string): Promise<{ success: boolean }> {
    const response = await this.request.put(`${this.baseUrl}/api/profiles/${oldName}/rename`, {
      data: { newName },
    });
    if (!response.ok()) {
      throw new Error(`PUT /api/profiles/${oldName}/rename failed: ${response.status()} ${response.statusText()}`);
    }
    return response.json();
  }

  /**
   * POST /api/profiles/:name/validate - Validate profile configuration
   */
  async validateConfig(name: string, config: string): Promise<{ valid: boolean; errors: any[] }> {
    const response = await this.request.post(`${this.baseUrl}/api/profiles/${name}/validate`, {
      data: { config },
    });
    if (!response.ok()) {
      throw new Error(`POST /api/profiles/${name}/validate failed: ${response.status()} ${response.statusText()}`);
    }
    return response.json();
  }

  /**
   * GET /api/metrics/latency - Get latency statistics
   */
  async getLatencyStats(): Promise<LatencyStats> {
    const response = await this.request.get(`${this.baseUrl}/api/metrics/latency`);
    if (!response.ok()) {
      throw new Error(`GET /api/metrics/latency failed: ${response.status()} ${response.statusText()}`);
    }
    return response.json();
  }

  /**
   * GET /api/metrics/events - Get event log
   */
  async getEventLog(): Promise<EventRecord[]> {
    const response = await this.request.get(`${this.baseUrl}/api/metrics/events`);
    if (!response.ok()) {
      throw new Error(`GET /api/metrics/events failed: ${response.status()} ${response.statusText()}`);
    }
    return response.json();
  }

  /**
   * DELETE /api/metrics/events - Clear event log
   */
  async clearEventLog(): Promise<{ success: boolean }> {
    const response = await this.request.delete(`${this.baseUrl}/api/metrics/events`);
    if (!response.ok()) {
      throw new Error(`DELETE /api/metrics/events failed: ${response.status()} ${response.statusText()}`);
    }
    return response.json();
  }

  /**
   * GET /api/daemon/state - Get daemon state
   */
  async getDaemonState(): Promise<DaemonState> {
    const response = await this.request.get(`${this.baseUrl}/api/daemon/state`);
    if (!response.ok()) {
      throw new Error(`GET /api/daemon/state failed: ${response.status()} ${response.statusText()}`);
    }
    return response.json();
  }

  /**
   * GET /api/config - Get global configuration
   */
  async getConfig(): Promise<any> {
    const response = await this.request.get(`${this.baseUrl}/api/config`);
    if (!response.ok()) {
      throw new Error(`GET /api/config failed: ${response.status()} ${response.statusText()}`);
    }
    return response.json();
  }

  /**
   * PUT /api/config - Update global configuration
   */
  async updateConfig(config: any): Promise<{ success: boolean }> {
    const response = await this.request.put(`${this.baseUrl}/api/config`, {
      data: config,
    });
    if (!response.ok()) {
      throw new Error(`PUT /api/config failed: ${response.status()} ${response.statusText()}`);
    }
    return response.json();
  }

  /**
   * GET /api/layers - Get available layers
   */
  async getLayers(): Promise<string[]> {
    const response = await this.request.get(`${this.baseUrl}/api/layers`);
    if (!response.ok()) {
      throw new Error(`GET /api/layers failed: ${response.status()} ${response.statusText()}`);
    }
    return response.json();
  }

  /**
   * GET /api/layouts - Get available keyboard layouts
   */
  async getLayouts(): Promise<string[]> {
    const response = await this.request.get(`${this.baseUrl}/api/layouts`);
    if (!response.ok()) {
      throw new Error(`GET /api/layouts failed: ${response.status()} ${response.statusText()}`);
    }
    return response.json();
  }

  /**
   * GET /api/layouts/:name - Get specific keyboard layout
   */
  async getLayout(name: string): Promise<any> {
    const response = await this.request.get(`${this.baseUrl}/api/layouts/${name}`);
    if (!response.ok()) {
      throw new Error(`GET /api/layouts/${name} failed: ${response.status()} ${response.statusText()}`);
    }
    return response.json();
  }

  /**
   * POST /api/simulator/events - Simulate key events
   */
  async simulateEvents(events: any[]): Promise<any> {
    const response = await this.request.post(`${this.baseUrl}/api/simulator/events`, {
      data: { events },
    });
    if (!response.ok()) {
      throw new Error(`POST /api/simulator/events failed: ${response.status()} ${response.statusText()}`);
    }
    return response.json();
  }

  /**
   * POST /api/simulator/reset - Reset simulator state
   */
  async resetSimulator(): Promise<{ success: boolean }> {
    const response = await this.request.post(`${this.baseUrl}/api/simulator/reset`);
    if (!response.ok()) {
      throw new Error(`POST /api/simulator/reset failed: ${response.status()} ${response.statusText()}`);
    }
    return response.json();
  }

  /**
   * POST /api/simulator/load-profile - Load a profile for simulation
   */
  async loadSimulatorProfile(name: string): Promise<SimulationLoadResponse> {
    const response = await this.request.post(`${this.baseUrl}/api/simulator/load-profile`, {
      data: { name },
    });
    if (!response.ok()) {
      throw new Error(`POST /api/simulator/load-profile failed: ${response.status()} ${response.statusText()}`);
    }
    return response.json();
  }

  /**
   * POST /api/simulator/events - Simulate events with DSL
   */
  async simulateEventsDsl(dsl: string, seed?: number): Promise<SimulationResponse> {
    const response = await this.request.post(`${this.baseUrl}/api/simulator/events`, {
      data: { dsl, seed },
    });
    if (!response.ok()) {
      throw new Error(`POST /api/simulator/events (dsl) failed: ${response.status()} ${response.statusText()}`);
    }
    return response.json();
  }

  /**
   * POST /api/simulator/events - Simulate events with scenario
   */
  async simulateScenario(scenario: string): Promise<SimulationResponse> {
    const response = await this.request.post(`${this.baseUrl}/api/simulator/events`, {
      data: { scenario },
    });
    if (!response.ok()) {
      throw new Error(`POST /api/simulator/events (scenario) failed: ${response.status()} ${response.statusText()}`);
    }
    return response.json();
  }

  /**
   * POST /api/simulator/scenarios/all - Run all built-in scenarios
   */
  async runAllScenarios(): Promise<AllScenariosResponse> {
    const response = await this.request.post(`${this.baseUrl}/api/simulator/scenarios/all`);
    if (!response.ok()) {
      throw new Error(`POST /api/simulator/scenarios/all failed: ${response.status()} ${response.statusText()}`);
    }
    return response.json();
  }

  /**
   * Wait for daemon to be ready
   * Polls /api/status until daemon responds or timeout
   */
  async waitForReady(timeoutMs: number = 30000, pollIntervalMs: number = 500): Promise<void> {
    const startTime = Date.now();

    while (Date.now() - startTime < timeoutMs) {
      try {
        const status = await this.getStatus();
        if (status.daemon_running) {
          return;
        }
      } catch (err) {
        // Daemon not ready yet, continue polling
      }

      await new Promise(resolve => setTimeout(resolve, pollIntervalMs));
    }

    throw new Error(`Daemon did not become ready within ${timeoutMs}ms`);
  }
}

/**
 * Create API helpers instance
 */
export function createApiHelpers(request: APIRequestContext, baseUrl: string = 'http://localhost:9867'): ApiHelpers {
  return new ApiHelpers(request, baseUrl);
}
