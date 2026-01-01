/**
 * Integration Test Harness
 *
 * Provides utilities for starting and stopping the daemon for integration tests.
 * This allows tests to run against a real daemon instance in test mode.
 *
 * Usage:
 * ```typescript
 * import { setupDaemon, teardownDaemon } from './test-harness';
 *
 * beforeAll(async () => {
 *   await setupDaemon();
 * });
 *
 * afterAll(async () => {
 *   await teardownDaemon();
 * });
 * ```
 */

import { spawn, ChildProcess } from 'child_process';
import { join } from 'path';
import { setTimeout as sleep } from 'timers/promises';

// Test daemon configuration
export const DAEMON_TEST_PORT = 13030;
export const DAEMON_WS_URL = `ws://127.0.0.1:${DAEMON_TEST_PORT}/ws-rpc`;
export const DAEMON_API_URL = `http://127.0.0.1:${DAEMON_TEST_PORT}`;
export const DAEMON_HEALTH_URL = `${DAEMON_API_URL}/health`;

// Daemon process handle
let daemonProcess: ChildProcess | null = null;
let daemonStartTime = 0;

/**
 * Check if daemon is already running on the test port.
 */
export async function isDaemonRunning(): Promise<boolean> {
  try {
    const response = await fetch(DAEMON_HEALTH_URL, {
      method: 'GET',
      signal: AbortSignal.timeout(2000),
    });
    return response.ok;
  } catch {
    return false;
  }
}

/**
 * Wait for daemon to be ready by polling the health endpoint.
 */
async function waitForDaemon(timeoutMs = 10000): Promise<void> {
  const startTime = Date.now();

  while (Date.now() - startTime < timeoutMs) {
    if (await isDaemonRunning()) {
      console.log(`✓ Daemon ready in ${Date.now() - daemonStartTime}ms`);
      return;
    }
    await sleep(100);
  }

  throw new Error(`Daemon failed to start within ${timeoutMs}ms`);
}

/**
 * Start the daemon in test mode.
 *
 * This function:
 * 1. Checks if daemon is already running
 * 2. Spawns a new daemon process if not running
 * 3. Waits for the daemon to be ready
 * 4. Stores the process handle for cleanup
 *
 * Options:
 * - autoStart: If false, only checks if daemon is running (default: true)
 * - headless: Run daemon without UI (default: true)
 * - port: Port to run daemon on (default: DAEMON_TEST_PORT)
 */
export async function setupDaemon(options: {
  autoStart?: boolean;
  headless?: boolean;
  port?: number;
} = {}): Promise<void> {
  const { autoStart = true, headless = true, port = DAEMON_TEST_PORT } = options;

  // Check if daemon is already running
  const alreadyRunning = await isDaemonRunning();

  if (alreadyRunning) {
    console.log('✓ Daemon already running');
    return;
  }

  if (!autoStart) {
    throw new Error(
      `Daemon is not running on port ${port}.\n` +
      `Start the daemon before running integration tests:\n` +
      `  cd keyrx_daemon && cargo run -- --port ${port} --headless`
    );
  }

  // Start daemon process
  console.log(`Starting daemon on port ${port}...`);
  daemonStartTime = Date.now();

  const daemonBinary = join(__dirname, '../../target/debug/keyrx_daemon');
  const args = ['--port', port.toString()];
  if (headless) {
    args.push('--headless');
  }

  daemonProcess = spawn('cargo', ['run', '-p', 'keyrx_daemon', '--', ...args], {
    cwd: join(__dirname, '../..'),
    stdio: 'pipe',
    env: {
      ...process.env,
      RUST_LOG: 'error', // Only show errors to reduce noise
    },
  });

  // Capture output for debugging
  daemonProcess.stdout?.on('data', (data) => {
    if (process.env.DEBUG_DAEMON) {
      console.log('[daemon stdout]', data.toString());
    }
  });

  daemonProcess.stderr?.on('data', (data) => {
    if (process.env.DEBUG_DAEMON) {
      console.error('[daemon stderr]', data.toString());
    }
  });

  daemonProcess.on('error', (error) => {
    console.error('Daemon process error:', error);
  });

  daemonProcess.on('exit', (code, signal) => {
    if (process.env.DEBUG_DAEMON || (code !== 0 && code !== null)) {
      console.log(`Daemon exited with code ${code}, signal ${signal}`);
    }
  });

  // Wait for daemon to be ready
  try {
    await waitForDaemon();
  } catch (error) {
    // Cleanup on failure
    if (daemonProcess) {
      daemonProcess.kill('SIGTERM');
      daemonProcess = null;
    }
    throw error;
  }
}

/**
 * Stop the daemon process.
 *
 * This function:
 * 1. Sends SIGTERM to the daemon process
 * 2. Waits for graceful shutdown (max 5s)
 * 3. Sends SIGKILL if still running after 5s
 * 4. Cleans up the process handle
 */
export async function teardownDaemon(): Promise<void> {
  if (!daemonProcess) {
    return;
  }

  console.log('Stopping daemon...');

  // Send SIGTERM for graceful shutdown
  daemonProcess.kill('SIGTERM');

  // Wait for process to exit
  const exitPromise = new Promise<void>((resolve) => {
    daemonProcess?.on('exit', () => {
      resolve();
    });
  });

  // Force kill after 5 seconds
  const timeoutPromise = sleep(5000).then(() => {
    if (daemonProcess && !daemonProcess.killed) {
      console.warn('Daemon did not exit gracefully, sending SIGKILL');
      daemonProcess.kill('SIGKILL');
    }
  });

  await Promise.race([exitPromise, timeoutPromise]);

  daemonProcess = null;
  console.log('✓ Daemon stopped');
}

/**
 * Create a test profile for integration tests.
 *
 * Returns the profile name which should be cleaned up after the test.
 */
export function createTestProfileName(): string {
  return `test-profile-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`;
}

/**
 * Simple configuration for testing.
 */
export const SIMPLE_TEST_CONFIG = `
// Simple test configuration
layer("default") {
  bind("a", action::tap("b"));
  bind("c", action::tap("d"));
}
`;

/**
 * Configuration with multiple layers for testing.
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
