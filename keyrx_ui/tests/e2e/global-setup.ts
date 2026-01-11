/**
 * Playwright Global Setup
 *
 * Runs once before all E2E tests to start the keyrx_daemon.
 * The daemon will remain running for the entire test suite and be
 * stopped by global-teardown.ts.
 *
 * This avoids the overhead of starting/stopping the daemon for every test,
 * while still providing isolation through individual test profiles.
 */

import { spawn, ChildProcess } from 'child_process';
import { join } from 'path';
import { writeFile } from 'fs/promises';
import { setTimeout as sleep } from 'timers/promises';

// Daemon configuration (must match daemon fixture)
const DAEMON_PORT = 9867;
const DAEMON_API_URL = `http://127.0.0.1:${DAEMON_PORT}`;
const PID_FILE = join(__dirname, '../../.daemon-e2e.pid');

/**
 * Check if daemon is responding on the health endpoint
 */
async function isDaemonReady(): Promise<boolean> {
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
async function waitForDaemonReady(timeoutMs = 30000): Promise<void> {
  const startTime = Date.now();

  while (Date.now() - startTime < timeoutMs) {
    if (await isDaemonReady()) {
      const elapsed = Date.now() - startTime;
      console.log(`✓ Daemon ready in ${elapsed}ms`);
      return;
    }
    await sleep(100);
  }

  throw new Error(`Daemon failed to start within ${timeoutMs}ms`);
}

/**
 * Start daemon process
 */
async function startDaemon(): Promise<ChildProcess> {
  console.log(`Starting daemon on port ${DAEMON_PORT}...`);
  const startTime = Date.now();

  // Determine daemon binary path
  // In CI, use release build; in dev, use debug build
  const buildType = process.env.CI ? 'release' : 'debug';
  const daemonBinary = join(__dirname, `../../../target/${buildType}/keyrx_daemon`);

  // Try to use pre-built binary first, fall back to cargo run
  const useBinary = process.env.E2E_USE_BINARY === 'true';

  let daemonProcess: ChildProcess;

  if (useBinary) {
    // Direct binary execution (faster)
    console.log(`Using binary: ${daemonBinary}`);
    daemonProcess = spawn(daemonBinary, ['--port', DAEMON_PORT.toString()], {
      cwd: join(__dirname, '../../..'),
      stdio: 'pipe',
      env: {
        ...process.env,
        RUST_LOG: 'error,keyrx_daemon=info',
      },
      detached: false,
    });
  } else {
    // Cargo run (slower but works without pre-built binary)
    console.log('Using cargo run (set E2E_USE_BINARY=true to use pre-built binary)');
    daemonProcess = spawn(
      'cargo',
      ['run', '-p', 'keyrx_daemon', '--', '--port', DAEMON_PORT.toString()],
      {
        cwd: join(__dirname, '../../..'),
        stdio: 'pipe',
        env: {
          ...process.env,
          RUST_LOG: 'error,keyrx_daemon=info',
        },
        detached: false,
      }
    );
  }

  // Capture output for debugging
  daemonProcess.stdout?.on('data', (data) => {
    if (process.env.DEBUG_DAEMON) {
      console.log('[daemon stdout]', data.toString().trim());
    }
  });

  daemonProcess.stderr?.on('data', (data) => {
    const msg = data.toString().trim();
    // Always show errors, show info only if DEBUG_DAEMON or CI
    if (msg.includes('ERROR') || process.env.DEBUG_DAEMON || process.env.CI) {
      console.error('[daemon stderr]', msg);
    }
  });

  daemonProcess.on('error', (error) => {
    console.error('❌ Daemon process error:', error);
  });

  daemonProcess.on('exit', (code, signal) => {
    if (code !== 0 && code !== null) {
      console.error(`❌ Daemon exited unexpectedly with code ${code}, signal ${signal}`);
    }
  });

  // Store PID for global teardown
  if (!daemonProcess.pid) {
    throw new Error('Failed to get daemon PID');
  }

  await writeFile(PID_FILE, daemonProcess.pid.toString(), 'utf-8');
  console.log(`✓ Daemon PID ${daemonProcess.pid} written to ${PID_FILE}`);

  return daemonProcess;
}

/**
 * Global setup function - executed once before all tests
 */
export default async function globalSetup() {
  console.log('\n=== E2E Global Setup ===\n');

  try {
    // Check if daemon is already running
    if (await isDaemonReady()) {
      console.log('✓ Daemon already running, skipping startup');
      return;
    }

    // Start daemon
    const daemonProcess = await startDaemon();

    // Wait for daemon to be ready
    await waitForDaemonReady();

    console.log('\n✓ Global setup complete - daemon ready for tests\n');
  } catch (error) {
    console.error('\n❌ Global setup failed:', error);
    throw error;
  }
}
