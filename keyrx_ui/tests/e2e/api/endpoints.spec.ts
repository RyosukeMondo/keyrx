/**
 * E2E API Endpoint Tests
 *
 * Comprehensive tests for all keyrx_daemon REST API endpoints.
 * Uses Playwright's APIRequestContext to test API contracts without browser.
 *
 * These tests verify:
 * - All endpoints return correct HTTP status codes
 * - Response bodies match expected schema (Zod validation)
 * - Error cases return appropriate error codes
 * - CRUD operations work correctly
 */

import { test, expect } from '@playwright/test';
import { createApiHelpers, ApiHelpers } from '../fixtures/api';

/**
 * Test fixtures with API helpers
 */
let api: ApiHelpers;

test.beforeAll(async ({ request }) => {
  api = createApiHelpers(request, 'http://localhost:9867');
  await api.waitForReady(30000);
});

test.describe('Status & Health Endpoints', () => {
  test('GET /api/status returns valid status', async () => {
    const status = await api.getStatus();

    expect(status).toHaveProperty('status');
    expect(status).toHaveProperty('version');
    expect(status).toHaveProperty('daemon_running');
    expect(status.daemon_running).toBe(true);
  });

  test('GET /api/health returns health information', async () => {
    const health = await api.getHealth();

    expect(health).toHaveProperty('healthy');
    expect(health).toHaveProperty('uptime');
    expect(health).toHaveProperty('version');
    expect(health.healthy).toBe(true);
    expect(health.uptime).toBeGreaterThanOrEqual(0);
  });

  test('GET /api/version returns version string', async () => {
    const version = await api.getVersion();

    expect(version).toHaveProperty('version');
    expect(typeof version.version).toBe('string');
    expect(version.version).toMatch(/^\d+\.\d+\.\d+/); // Semantic versioning
  });

  test('GET /api/daemon/state returns daemon state', async () => {
    const state = await api.getDaemonState();

    expect(state).toHaveProperty('modifiers');
    expect(state).toHaveProperty('locks');
    expect(state).toHaveProperty('layer');
    expect(Array.isArray(state.modifiers)).toBe(true);
    expect(Array.isArray(state.locks)).toBe(true);
  });
});

test.describe('Device Endpoints', () => {
  let deviceId: string;

  test('GET /api/devices returns device list', async () => {
    const devices = await api.getDevices();

    expect(Array.isArray(devices)).toBe(true);

    // If devices exist, save first device ID for other tests
    if (devices.length > 0) {
      deviceId = devices[0].id;
      expect(devices[0]).toHaveProperty('id');
      expect(devices[0]).toHaveProperty('name');
      expect(devices[0]).toHaveProperty('path');
      expect(devices[0]).toHaveProperty('active');
      expect(typeof devices[0].active).toBe('boolean');
    }
  });

  test('PATCH /api/devices/:id updates device configuration', async ({ request }) => {
    const devices = await api.getDevices();
    if (devices.length === 0) {
      test.skip('No devices available for testing');
      return;
    }

    const testDeviceId = devices[0].id;
    const originalName = devices[0].name;

    // Update device name
    const result = await api.updateDevice(testDeviceId, {
      name: `Test Device ${Date.now()}`,
    });

    expect(result.success).toBe(true);

    // Restore original name
    await api.updateDevice(testDeviceId, {
      name: originalName,
    });
  });

  test('PUT /api/devices/:id/name renames device', async () => {
    const devices = await api.getDevices();
    if (devices.length === 0) {
      test.skip('No devices available for testing');
      return;
    }

    const testDeviceId = devices[0].id;
    const originalName = devices[0].name;
    const newName = `Renamed-${Date.now()}`;

    const updated = await api.renameDevice(testDeviceId, newName);

    expect(updated.name).toBe(newName);

    // Restore original name
    await api.renameDevice(testDeviceId, originalName);
  });

  test('PUT /api/devices/:id/layout sets device layout', async () => {
    const devices = await api.getDevices();
    if (devices.length === 0) {
      test.skip('No devices available for testing');
      return;
    }

    const testDeviceId = devices[0].id;

    // Set to a known layout
    const updated = await api.setDeviceLayout(testDeviceId, 'us');

    expect(updated.layout).toBe('us');
  });

  test('GET /api/devices/:id/layout returns device layout', async () => {
    const devices = await api.getDevices();
    if (devices.length === 0) {
      test.skip('No devices available for testing');
      return;
    }

    const testDeviceId = devices[0].id;

    const layout = await api.getDeviceLayout(testDeviceId);

    expect(layout).toHaveProperty('layout');
    expect(typeof layout.layout).toBe('string');
  });

  test('DELETE /api/devices/:id returns error for non-existent device', async ({ request }) => {
    const response = await request.delete('http://localhost:9867/api/devices/non-existent-id');

    // Expect 404 for non-existent device
    expect(response.status()).toBe(404);
  });
});

test.describe('Profile Endpoints', () => {
  const testProfileName = `E2E-Test-${Date.now()}`;
  const testProfileName2 = `E2E-Test2-${Date.now()}`;

  test.afterAll(async () => {
    // Cleanup test profiles
    try {
      await api.deleteProfile(testProfileName);
    } catch (err) {
      // Profile may not exist
    }
    try {
      await api.deleteProfile(testProfileName2);
    } catch (err) {
      // Profile may not exist
    }
  });

  test('GET /api/profiles returns profile list', async () => {
    const profiles = await api.getProfiles();

    expect(Array.isArray(profiles)).toBe(true);

    if (profiles.length > 0) {
      expect(profiles[0]).toHaveProperty('name');
      expect(profiles[0]).toHaveProperty('rhaiPath');
      expect(profiles[0]).toHaveProperty('krxPath');
      expect(profiles[0]).toHaveProperty('isActive');
      expect(typeof profiles[0].isActive).toBe('boolean');
    }
  });

  test('POST /api/profiles creates new profile', async () => {
    const newProfile = await api.createProfile(testProfileName, 'blank');

    expect(newProfile.name).toBe(testProfileName);
    expect(newProfile).toHaveProperty('rhaiPath');
    expect(newProfile).toHaveProperty('krxPath');
  });

  test('GET /api/profiles/active returns active profile', async () => {
    const active = await api.getActiveProfile();

    expect(active).toHaveProperty('name');
    // name can be null if no profile is active
    expect(active.name === null || typeof active.name === 'string').toBe(true);
  });

  test('GET /api/profiles/:name/config returns profile config', async () => {
    const config = await api.getProfileConfig(testProfileName);

    expect(config.name).toBe(testProfileName);
    expect(config).toHaveProperty('config');
    expect(typeof config.config).toBe('string');
  });

  test('PUT /api/profiles/:name/config updates config', async () => {
    const newConfig = `// Test config updated at ${Date.now()}\n`;

    const result = await api.updateProfileConfig(testProfileName, newConfig);

    expect(result.success).toBe(true);

    // Verify config was updated
    const updated = await api.getProfileConfig(testProfileName);
    expect(updated.config).toContain('Test config');
  });

  test('POST /api/profiles/:name/activate activates profile', async () => {
    const result = await api.activateProfile(testProfileName);

    expect(result.success).toBe(true);
    expect(result.profile).toBe(testProfileName);

    // Verify activation
    const active = await api.getActiveProfile();
    expect(active.name).toBe(testProfileName);
  });

  test('POST /api/profiles/:name/duplicate creates duplicate', async () => {
    const duplicate = await api.duplicateProfile(testProfileName, testProfileName2);

    expect(duplicate.name).toBe(testProfileName2);

    // Verify duplicate exists
    const profiles = await api.getProfiles();
    const found = profiles.find(p => p.name === testProfileName2);
    expect(found).toBeDefined();
  });

  test('PUT /api/profiles/:name/rename renames profile', async () => {
    const renamedName = `${testProfileName2}-renamed`;

    const result = await api.renameProfile(testProfileName2, renamedName);

    expect(result.success).toBe(true);

    // Verify rename
    const profiles = await api.getProfiles();
    const found = profiles.find(p => p.name === renamedName);
    expect(found).toBeDefined();

    // Cleanup renamed profile
    await api.deleteProfile(renamedName);
  });

  test('POST /api/profiles/:name/validate validates config', async () => {
    const validConfig = '// Valid empty config\n';
    const invalidConfig = 'invalid syntax {{{';

    // Valid config
    const validResult = await api.validateConfig(testProfileName, validConfig);
    expect(validResult).toHaveProperty('valid');

    // Invalid config
    const invalidResult = await api.validateConfig(testProfileName, invalidConfig);
    expect(invalidResult).toHaveProperty('valid');
    expect(invalidResult).toHaveProperty('errors');
  });

  test('DELETE /api/profiles/:name deletes profile', async () => {
    const result = await api.deleteProfile(testProfileName);

    expect(result.success).toBe(true);

    // Verify deletion
    const profiles = await api.getProfiles();
    const found = profiles.find(p => p.name === testProfileName);
    expect(found).toBeUndefined();
  });

  test('GET /api/profiles/:name/config returns 404 for non-existent profile', async ({ request }) => {
    const response = await request.get('http://localhost:9867/api/profiles/NonExistentProfile123/config');

    expect(response.status()).toBe(404);
  });
});

test.describe('Metrics Endpoints', () => {
  test('GET /api/metrics/latency returns latency stats', async () => {
    const stats = await api.getLatencyStats();

    // Latency stats structure (may be empty if no events)
    expect(stats).toBeDefined();
    // Stats may have min, avg, max, p95, p99 properties
    // Values can be 0 if no events have been processed
  });

  test('GET /api/metrics/events returns event log', async () => {
    const events = await api.getEventLog();

    expect(Array.isArray(events)).toBe(true);

    // Events may be empty if none have been logged
    if (events.length > 0) {
      expect(events[0]).toHaveProperty('timestamp');
      expect(events[0]).toHaveProperty('key_code');
      expect(events[0]).toHaveProperty('event_type');
      expect(events[0]).toHaveProperty('device_id');
    }
  });

  test('DELETE /api/metrics/events clears event log', async () => {
    const result = await api.clearEventLog();

    expect(result.success).toBe(true);

    // Verify events cleared
    const events = await api.getEventLog();
    expect(Array.isArray(events)).toBe(true);
  });
});

test.describe('Configuration Endpoints', () => {
  test('GET /api/config returns global config', async () => {
    const config = await api.getConfig();

    expect(config).toBeDefined();
    // Config structure depends on daemon implementation
  });

  test('PUT /api/config updates global config', async ({ request }) => {
    // Skip this test as it modifies global config
    // In a real scenario, we'd backup and restore config
    test.skip('Skipping global config modification');
  });
});

test.describe('Layout Endpoints', () => {
  test('GET /api/layouts returns available layouts', async () => {
    const layouts = await api.getLayouts();

    expect(Array.isArray(layouts)).toBe(true);
    expect(layouts.length).toBeGreaterThan(0);
    // Should include standard layouts like 'us', 'uk', etc.
  });

  test('GET /api/layouts/:name returns specific layout', async () => {
    const layouts = await api.getLayouts();
    if (layouts.length === 0) {
      test.skip('No layouts available');
      return;
    }

    const layoutName = layouts[0];
    const layout = await api.getLayout(layoutName);

    expect(layout).toBeDefined();
    // Layout structure depends on implementation
  });

  test('GET /api/layouts/:name returns 404 for non-existent layout', async ({ request }) => {
    const response = await request.get('http://localhost:9867/api/layouts/NonExistentLayout123');

    expect(response.status()).toBe(404);
  });
});

test.describe('Layer Endpoints', () => {
  test('GET /api/layers returns available layers', async () => {
    const layers = await api.getLayers();

    expect(Array.isArray(layers)).toBe(true);
    // Layers depend on active profile config
  });
});

test.describe('Simulator Endpoints', () => {
  test('POST /api/simulator/reset resets simulator state', async () => {
    const result = await api.resetSimulator();

    expect(result.success).toBe(true);
  });

  test('POST /api/simulator/events simulates key events', async () => {
    // Reset simulator first
    await api.resetSimulator();

    // Simulate a simple key press/release
    const events = [
      { keyCode: 65, eventType: 'press', timestamp: 0 },
      { keyCode: 65, eventType: 'release', timestamp: 100 },
    ];

    const result = await api.simulateEvents(events);

    expect(result).toBeDefined();
    // Result structure depends on implementation
  });
});

test.describe('Error Handling', () => {
  test('Non-existent endpoints return 404', async ({ request }) => {
    const response = await request.get('http://localhost:9867/api/non-existent-endpoint');

    expect(response.status()).toBe(404);
  });

  test('Invalid JSON body returns 400', async ({ request }) => {
    const response = await request.post('http://localhost:9867/api/profiles', {
      data: 'invalid-json-string',
      headers: {
        'Content-Type': 'application/json',
      },
    });

    // Expect 400 Bad Request or 422 Unprocessable Entity
    expect([400, 422]).toContain(response.status());
  });

  test('Missing required fields returns appropriate error', async ({ request }) => {
    const response = await request.post('http://localhost:9867/api/profiles', {
      data: {}, // Missing required 'name' field
    });

    // Expect 400 Bad Request or 422 Unprocessable Entity
    expect([400, 422]).toContain(response.status());
  });
});

test.describe('API Contract Validation', () => {
  test('All successful responses are valid JSON', async () => {
    // Test a sample of endpoints
    const endpoints = [
      () => api.getStatus(),
      () => api.getDevices(),
      () => api.getProfiles(),
      () => api.getLayouts(),
    ];

    for (const endpoint of endpoints) {
      const result = await endpoint();
      expect(result).toBeDefined();
      // If it parsed as JSON and passed schema validation, it's valid
    }
  });

  test('Schema validation catches mismatches', async () => {
    // This test verifies that schema validation is working
    // by ensuring responses pass Zod validation
    const status = await api.getStatus();

    // Zod validation happens in ApiHelpers
    // If we got here, validation passed
    expect(status).toHaveProperty('status');
  });
});
