/**
 * Daemon Fixture for Playwright E2E Tests
 *
 * Provides a Playwright test fixture that manages daemon lifecycle for E2E tests.
 * This fixture extends the base Playwright test object with daemon management capabilities.
 *
 * Usage:
 * ```typescript
 * import { test } from './fixtures/daemon';
 *
 * test('my test', async ({ page, daemon }) => {
 *   // Daemon is automatically started and ready
 *   await page.goto('/');
 *   // ...
 * });
 * ```
 */

import { test as base, expect } from '@playwright/test';
import { spawn, ChildProcess } from 'child_process';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';
import { setTimeout as sleep } from 'timers/promises';
import { writeFile, unlink } from 'fs/promises';

// ES module equivalent of __dirname
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Daemon configuration for E2E tests
export const DAEMON_PORT = 9867; // Standard keyrx_daemon port
export const DAEMON_API_URL = `http://127.0.0.1:${DAEMON_PORT}`;
export const DAEMON_WS_URL = `ws://127.0.0.1:${DAEMON_PORT}/ws`;

/**
 * Daemon fixture interface
 */
export interface DaemonFixture {
  /** Daemon API base URL */
  apiUrl: string;
  /** Daemon WebSocket URL */
  wsUrl: string;
  /** Daemon port */
  port: number;
  /** Check if daemon is responding */
  isReady: () => Promise<boolean>;
  /** Create a test profile with given config */
  createTestProfile: (name: string, config: string) => Promise<void>;
  /** Delete a test profile */
  deleteTestProfile: (name: string) => Promise<void>;
}

/**
 * Daemon manager - handles daemon process lifecycle
 */
class DaemonManager {
  private process: ChildProcess | null = null;
  private startTime = 0;
  private pidFile: string;

  constructor() {
    this.pidFile = join(__dirname, '../../../.daemon-e2e.pid');
  }

  /**
   * Check if daemon is responding on the health endpoint
   */
  async isReady(): Promise<boolean> {
    try {
      const response = await fetch(`${DAEMON_API_URL}/api/status`, {
        method: 'GET',
        signal: AbortSignal.timeout(2000),
      });
      return response.ok;
    } catch {
      return false;
    }
  }

  /**
   * Wait for daemon to be ready by polling the status endpoint
   */
  private async waitForReady(timeoutMs = 30000): Promise<void> {
    const startTime = Date.now();

    while (Date.now() - startTime < timeoutMs) {
      if (await this.isReady()) {
        const elapsed = Date.now() - this.startTime;
        console.log(`✓ Daemon ready in ${elapsed}ms`);
        return;
      }
      await sleep(100);
    }

    throw new Error(`Daemon failed to start within ${timeoutMs}ms`);
  }

  /**
   * Start the daemon process
   */
  async start(): Promise<void> {
    // Check if daemon is already running
    if (await this.isReady()) {
      console.log('✓ Daemon already running');
      return;
    }

    console.log(`Starting daemon on port ${DAEMON_PORT}...`);
    this.startTime = Date.now();

    // Determine daemon binary path
    // In CI, use release build; in dev, use debug build
    const buildType = process.env.CI ? 'release' : 'debug';
    const daemonBinary = join(__dirname, `../../../../target/${buildType}/keyrx_daemon`);

    // Try to use pre-built binary first, fall back to cargo run
    const useBinary = process.env.E2E_USE_BINARY === 'true';

    if (useBinary) {
      // Direct binary execution
      this.process = spawn(daemonBinary, ['--port', DAEMON_PORT.toString()], {
        cwd: join(__dirname, '../../../..'),
        stdio: 'pipe',
        env: {
          ...process.env,
          RUST_LOG: 'error,keyrx_daemon=info',
        },
      });
    } else {
      // Cargo run (slower but works without pre-built binary)
      this.process = spawn(
        'cargo',
        ['run', '-p', 'keyrx_daemon', '--', '--port', DAEMON_PORT.toString()],
        {
          cwd: join(__dirname, '../../../..'),
          stdio: 'pipe',
          env: {
            ...process.env,
            RUST_LOG: 'error,keyrx_daemon=info',
          },
        }
      );
    }

    // Capture output for debugging
    this.process.stdout?.on('data', (data) => {
      if (process.env.DEBUG_DAEMON) {
        console.log('[daemon stdout]', data.toString());
      }
    });

    this.process.stderr?.on('data', (data) => {
      if (process.env.DEBUG_DAEMON || process.env.CI) {
        console.error('[daemon stderr]', data.toString());
      }
    });

    this.process.on('error', (error) => {
      console.error('Daemon process error:', error);
    });

    this.process.on('exit', (code, signal) => {
      if (process.env.DEBUG_DAEMON || (code !== 0 && code !== null)) {
        console.log(`Daemon exited with code ${code}, signal ${signal}`);
      }
    });

    // Store PID for global teardown
    if (this.process.pid) {
      await writeFile(this.pidFile, this.process.pid.toString(), 'utf-8');
    }

    // Wait for daemon to be ready
    try {
      await this.waitForReady();
    } catch (error) {
      // Cleanup on failure
      await this.stop();
      throw error;
    }
  }

  /**
   * Stop the daemon process
   */
  async stop(): Promise<void> {
    if (!this.process) {
      return;
    }

    console.log('Stopping daemon...');

    // Send SIGTERM for graceful shutdown
    this.process.kill('SIGTERM');

    // Wait for process to exit
    const exitPromise = new Promise<void>((resolve) => {
      this.process?.on('exit', () => {
        resolve();
      });
    });

    // Force kill after 5 seconds
    const timeoutPromise = sleep(5000).then(() => {
      if (this.process && !this.process.killed) {
        console.warn('Daemon did not exit gracefully, sending SIGKILL');
        this.process.kill('SIGKILL');
      }
    });

    await Promise.race([exitPromise, timeoutPromise]);

    this.process = null;

    // Clean up PID file
    try {
      await unlink(this.pidFile);
    } catch {
      // Ignore errors - file might not exist
    }

    console.log('✓ Daemon stopped');
  }

  /**
   * Create a test profile with given configuration
   */
  async createTestProfile(name: string, config: string): Promise<void> {
    const response = await fetch(`${DAEMON_API_URL}/api/profiles`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ name }),
    });

    if (!response.ok) {
      throw new Error(`Failed to create profile ${name}: ${response.status}`);
    }

    // Save config for the profile
    const configResponse = await fetch(`${DAEMON_API_URL}/api/profiles/${name}/config`, {
      method: 'PUT',
      headers: {
        'Content-Type': 'text/plain',
      },
      body: config,
    });

    if (!configResponse.ok) {
      throw new Error(`Failed to save config for ${name}: ${configResponse.status}`);
    }

    console.log(`✓ Created test profile: ${name}`);
  }

  /**
   * Delete a test profile
   */
  async deleteTestProfile(name: string): Promise<void> {
    const response = await fetch(`${DAEMON_API_URL}/api/profiles/${name}`, {
      method: 'DELETE',
    });

    if (!response.ok && response.status !== 404) {
      console.warn(`Failed to delete profile ${name}: ${response.status}`);
    } else {
      console.log(`✓ Deleted test profile: ${name}`);
    }
  }
}

// Global daemon manager instance
const daemonManager = new DaemonManager();

/**
 * Extended test with daemon fixture
 */
export const test = base.extend<{ daemon: DaemonFixture }>({
  daemon: async ({}, use) => {
    // Setup: Start daemon before test
    await daemonManager.start();

    // Provide fixture to test
    const fixture: DaemonFixture = {
      apiUrl: DAEMON_API_URL,
      wsUrl: DAEMON_WS_URL,
      port: DAEMON_PORT,
      isReady: () => daemonManager.isReady(),
      createTestProfile: (name, config) => daemonManager.createTestProfile(name, config),
      deleteTestProfile: (name) => daemonManager.deleteTestProfile(name),
    };

    // Run test with fixture
    await use(fixture);

    // Teardown: Cleanup is handled by global teardown
    // Individual tests should clean up their own test profiles
  },
});

// Export expect from Playwright
export { expect };

/**
 * Simple test configuration for basic tests
 */
export const SIMPLE_TEST_CONFIG = `
// Simple test configuration
layer("default") {
  bind("a", action::tap("b"));
  bind("c", action::tap("d"));
}
`;

/**
 * Multi-layer test configuration
 */
export const MULTILAYER_TEST_CONFIG = `
// Multi-layer test configuration
layer("default") {
  bind("a", action::tap("b"));
  bind("space", action::layer_toggle("symbols"));
}

layer("symbols") {
  bind("a", action::tap("1"));
  bind("b", action::tap("2"));
}
`;

/**
 * Generate unique test profile name
 */
export function generateTestProfileName(prefix = 'test'): string {
  return `${prefix}-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`;
}
