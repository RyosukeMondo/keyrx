/**
 * E2E Test API Mocks
 *
 * Provides mock API responses for E2E tests, allowing tests to run
 * without a real backend daemon.
 */

import type { Page, Route } from '@playwright/test';

export interface MockProfile {
  name: string;
  rhaiPath: string;
  krxPath: string;
  isActive: boolean;
  modifiedAt: string;
  createdAt: string;
  layerCount: number;
  deviceCount: number;
  keyCount: number;
  // For E2E test internal tracking
  valid?: boolean;
  errors?: Array<{ line: number; column: number; message: string }>;
}

export interface MockDevice {
  id: string;
  name: string;
  path: string;
  layout: string;
  active: boolean;
  serial?: string;
}

export interface MockDaemonState {
  modifiers: number[];
  locks: number[];
  layer: number;
}

/**
 * Default mock data for tests
 */
export const defaultMockData = {
  profiles: [
    {
      name: 'default',
      rhaiPath: '/home/user/.keyrx/profiles/default.rhai',
      krxPath: '/home/user/.keyrx/profiles/default.krx',
      isActive: true,
      modifiedAt: new Date().toISOString(),
      createdAt: new Date(Date.now() - 86400000).toISOString(),
      layerCount: 1,
      deviceCount: 0,
      keyCount: 0,
      valid: true,
    },
    {
      name: 'gaming',
      rhaiPath: '/home/user/.keyrx/profiles/gaming.rhai',
      krxPath: '/home/user/.keyrx/profiles/gaming.krx',
      isActive: false,
      modifiedAt: new Date(Date.now() - 86400000).toISOString(),
      createdAt: new Date(Date.now() - 172800000).toISOString(),
      layerCount: 2,
      deviceCount: 1,
      keyCount: 10,
      valid: true,
    },
  ] as MockProfile[],

  devices: [
    {
      id: 'device-1',
      name: 'Main Keyboard',
      path: '/dev/input/event0_VID_1234_PID_5678',
      layout: 'ANSI_104',
      active: true,
      serial: 'ABC123',
    },
    {
      id: 'device-2',
      name: 'Secondary Keyboard',
      path: '/dev/input/event1_VID_9ABC_PID_DEF0',
      layout: 'ISO_105',
      active: true,
    },
  ] as MockDevice[],

  daemonState: {
    modifiers: [],
    locks: [],
    layer: 0,
  } as MockDaemonState,

  globalLayout: 'ANSI_104',

  activeProfile: 'default',
};

/**
 * Setup API mocks for a Playwright page
 *
 * @param page - Playwright page instance
 * @param options - Custom mock data options
 */
export async function setupApiMocks(
  page: Page,
  options: {
    profiles?: MockProfile[];
    devices?: MockDevice[];
    daemonState?: MockDaemonState;
    globalLayout?: string;
    failOnCreate?: boolean;
    failOnActivate?: boolean;
    failOnDelete?: boolean;
  } = {}
): Promise<void> {
  const profiles = options.profiles ?? [...defaultMockData.profiles];
  const devices = options.devices ?? [...defaultMockData.devices];
  let daemonState = options.daemonState ?? { ...defaultMockData.daemonState };
  let globalLayout = options.globalLayout ?? defaultMockData.globalLayout;

  // GET /api/profiles - List all profiles
  await page.route('**/api/profiles', async (route: Route) => {
    if (route.request().method() === 'GET') {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ profiles }),
      });
    } else if (route.request().method() === 'POST') {
      // Create profile
      if (options.failOnCreate) {
        await route.fulfill({
          status: 500,
          contentType: 'application/json',
          body: JSON.stringify({ error: 'Failed to create profile' }),
        });
        return;
      }

      const body = route.request().postDataJSON();
      const newProfile: MockProfile = {
        name: body.name,
        rhaiPath: `/home/user/.keyrx/profiles/${body.name}.rhai`,
        krxPath: `/home/user/.keyrx/profiles/${body.name}.krx`,
        isActive: false,
        modifiedAt: new Date().toISOString(),
        createdAt: new Date().toISOString(),
        layerCount: 1,
        deviceCount: 0,
        keyCount: 0,
        valid: true,
      };
      profiles.push(newProfile);

      await route.fulfill({
        status: 201,
        contentType: 'application/json',
        body: JSON.stringify(newProfile),
      });
    } else {
      await route.continue();
    }
  });

  // GET /api/profiles/:name - Get specific profile
  await page.route('**/api/profiles/*', async (route: Route) => {
    const url = route.request().url();
    const nameMatch = url.match(/\/api\/profiles\/([^/]+)$/);

    if (!nameMatch) {
      await route.continue();
      return;
    }

    const profileName = decodeURIComponent(nameMatch[1]);

    if (route.request().method() === 'GET') {
      const profile = profiles.find((p) => p.name === profileName);
      if (profile) {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify(profile),
        });
      } else {
        await route.fulfill({
          status: 404,
          contentType: 'application/json',
          body: JSON.stringify({ error: 'Profile not found' }),
        });
      }
    } else if (route.request().method() === 'DELETE') {
      if (options.failOnDelete) {
        await route.fulfill({
          status: 500,
          contentType: 'application/json',
          body: JSON.stringify({ error: 'Failed to delete profile' }),
        });
        return;
      }

      const index = profiles.findIndex((p) => p.name === profileName);
      if (index !== -1) {
        profiles.splice(index, 1);
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({ success: true }),
        });
      } else {
        await route.fulfill({
          status: 404,
          contentType: 'application/json',
          body: JSON.stringify({ error: 'Profile not found' }),
        });
      }
    } else {
      await route.continue();
    }
  });

  // POST /api/profiles/:name/activate - Activate profile
  await page.route('**/api/profiles/*/activate', async (route: Route) => {
    if (route.request().method() !== 'POST') {
      await route.continue();
      return;
    }

    if (options.failOnActivate) {
      await route.fulfill({
        status: 500,
        contentType: 'application/json',
        body: JSON.stringify({
          success: false,
          errors: ['Failed to activate profile'],
        }),
      });
      return;
    }

    const url = route.request().url();
    const nameMatch = url.match(/\/api\/profiles\/([^/]+)\/activate$/);

    if (nameMatch) {
      const profileName = decodeURIComponent(nameMatch[1]);
      const profile = profiles.find((p) => p.name === profileName);

      if (profile && !profile.valid) {
        await route.fulfill({
          status: 400,
          contentType: 'application/json',
          body: JSON.stringify({
            success: false,
            errors: profile.errors?.map((e) => e.message) ?? ['Invalid configuration'],
          }),
        });
        return;
      }

      // Deactivate current active profile
      profiles.forEach((p) => {
        p.isActive = p.name === profileName;
      });

      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ success: true, errors: [] }),
      });
    }
  });

  // GET /api/profiles/:name/validate - Validate profile
  await page.route('**/api/profiles/*/validate', async (route: Route) => {
    const url = route.request().url();
    const nameMatch = url.match(/\/api\/profiles\/([^/]+)\/validate$/);

    if (nameMatch) {
      const profileName = decodeURIComponent(nameMatch[1]);
      const profile = profiles.find((p) => p.name === profileName);

      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          valid: profile?.valid ?? true,
          errors: profile?.errors ?? [],
        }),
      });
    }
  });

  // GET /api/profiles/:name/config - Get profile configuration
  await page.route('**/api/profiles/*/config', async (route: Route) => {
    const url = route.request().url();
    const nameMatch = url.match(/\/api\/profiles\/([^/]+)\/config$/);

    if (nameMatch && route.request().method() === 'GET') {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          content: '// Blank configuration\n',
        }),
      });
    } else if (nameMatch && route.request().method() === 'PUT') {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ success: true }),
      });
    } else {
      await route.continue();
    }
  });

  // GET /api/devices - List devices
  await page.route('**/api/devices', async (route: Route) => {
    if (route.request().method() === 'GET') {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ devices }),
      });
    } else {
      await route.continue();
    }
  });

  // PUT /api/devices/:id - Update device
  await page.route('**/api/devices/*', async (route: Route) => {
    if (route.request().method() === 'PUT') {
      const body = route.request().postDataJSON();
      const url = route.request().url();
      const idMatch = url.match(/\/api\/devices\/([^/]+)$/);

      if (idMatch) {
        const deviceId = idMatch[1];
        const device = devices.find((d) => d.id === deviceId);
        if (device && body.layout) {
          device.layout = body.layout;
        }
        if (device && body.name) {
          device.name = body.name;
        }

        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({ success: true }),
        });
      }
    } else {
      await route.continue();
    }
  });

  // GET/PUT /api/settings/global-layout
  await page.route('**/api/settings/global-layout', async (route: Route) => {
    if (route.request().method() === 'GET') {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ layout: globalLayout }),
      });
    } else if (route.request().method() === 'PUT') {
      const body = route.request().postDataJSON();
      globalLayout = body.layout;

      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ success: true }),
      });
    } else {
      await route.continue();
    }
  });

  // GET /api/daemon/state - Get daemon state
  await page.route('**/api/daemon/state', async (route: Route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(daemonState),
    });
  });

  // GET /api/version - Get version info
  await page.route('**/api/version', async (route: Route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({ version: '0.1.0' }),
    });
  });

  // Mock WebSocket connections (return 404 to prevent connection attempts)
  await page.route('**/ws/**', async (route: Route) => {
    await route.abort('connectionrefused');
  });
}

/**
 * Create test profile helper
 */
export function createMockProfile(name: string, overrides: Partial<MockProfile> = {}): MockProfile {
  return {
    name,
    rhaiPath: `/home/user/.keyrx/profiles/${name}.rhai`,
    krxPath: `/home/user/.keyrx/profiles/${name}.krx`,
    isActive: false,
    modifiedAt: new Date().toISOString(),
    createdAt: new Date().toISOString(),
    layerCount: 1,
    deviceCount: 0,
    keyCount: 0,
    valid: true,
    ...overrides,
  };
}

/**
 * Create invalid profile helper
 */
export function createInvalidMockProfile(
  name: string,
  errors: Array<{ line: number; column: number; message: string }>
): MockProfile {
  return {
    name,
    rhaiPath: `/home/user/.keyrx/profiles/${name}.rhai`,
    krxPath: `/home/user/.keyrx/profiles/${name}.krx`,
    isActive: false,
    modifiedAt: new Date().toISOString(),
    createdAt: new Date().toISOString(),
    layerCount: 0,
    deviceCount: 0,
    keyCount: 0,
    valid: false,
    errors,
  };
}
