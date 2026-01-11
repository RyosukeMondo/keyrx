/**
 * Playwright Global Teardown
 *
 * Runs once after all E2E tests to stop the keyrx_daemon.
 * Reads the daemon PID from the file written by global-setup.ts
 * and sends SIGTERM for graceful shutdown.
 */

import { readFile, unlink } from 'fs/promises';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';
import { setTimeout as sleep } from 'timers/promises';

// ES module equivalent of __dirname
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const PID_FILE = join(__dirname, '../../.daemon-e2e.pid');
const DAEMON_API_URL = 'http://127.0.0.1:9867';

/**
 * Check if daemon is still responding
 */
async function isDaemonRunning(): Promise<boolean> {
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
 * Check if process is running by PID
 */
function isProcessRunning(pid: number): boolean {
  try {
    // Signal 0 checks if process exists without actually sending a signal
    process.kill(pid, 0);
    return true;
  } catch {
    return false;
  }
}

/**
 * Stop daemon process
 */
async function stopDaemon(pid: number): Promise<void> {
  console.log(`Stopping daemon (PID: ${pid})...`);

  try {
    // Send SIGTERM for graceful shutdown
    process.kill(pid, 'SIGTERM');
    console.log('✓ Sent SIGTERM signal');

    // Wait for process to exit (max 5 seconds)
    const maxWaitMs = 5000;
    const startTime = Date.now();

    while (Date.now() - startTime < maxWaitMs) {
      if (!isProcessRunning(pid)) {
        console.log(`✓ Daemon stopped gracefully in ${Date.now() - startTime}ms`);
        return;
      }
      await sleep(100);
    }

    // If still running after 5 seconds, send SIGKILL
    if (isProcessRunning(pid)) {
      console.warn('⚠ Daemon did not exit gracefully, sending SIGKILL');
      process.kill(pid, 'SIGKILL');
      await sleep(500); // Brief wait for SIGKILL to take effect
      console.log('✓ Daemon force-stopped');
    }
  } catch (error: any) {
    // ESRCH means process doesn't exist (already stopped)
    if (error.code === 'ESRCH') {
      console.log('✓ Daemon already stopped');
      return;
    }
    throw error;
  }
}

/**
 * Clean up PID file
 */
async function cleanupPidFile(): Promise<void> {
  try {
    await unlink(PID_FILE);
    console.log(`✓ Removed PID file: ${PID_FILE}`);
  } catch (error: any) {
    if (error.code === 'ENOENT') {
      // File doesn't exist - that's fine
      console.log('✓ PID file already removed');
    } else {
      console.warn('⚠ Failed to remove PID file:', error.message);
    }
  }
}

/**
 * Global teardown function - executed once after all tests
 */
export default async function globalTeardown() {
  console.log('\n=== E2E Global Teardown ===\n');

  try {
    // Read PID from file
    let pid: number;
    try {
      const pidStr = await readFile(PID_FILE, 'utf-8');
      pid = parseInt(pidStr.trim(), 10);
      console.log(`Found daemon PID: ${pid}`);
    } catch (error: any) {
      if (error.code === 'ENOENT') {
        console.log('✓ No PID file found - daemon may not have started or already stopped');
        return;
      }
      throw error;
    }

    // Verify daemon is actually running
    if (!isProcessRunning(pid)) {
      console.log('✓ Daemon already stopped (process not found)');
      await cleanupPidFile();
      return;
    }

    // Double-check with API health check
    const isRunning = await isDaemonRunning();
    if (!isRunning) {
      console.log('✓ Daemon not responding to health check (likely already stopped)');
      // Still try to kill the process if it exists
      if (isProcessRunning(pid)) {
        await stopDaemon(pid);
      }
    } else {
      // Daemon is running - stop it
      await stopDaemon(pid);
    }

    // Clean up PID file
    await cleanupPidFile();

    console.log('\n✓ Global teardown complete - daemon stopped\n');
  } catch (error) {
    console.error('\n❌ Global teardown failed:', error);
    // Don't throw - allow tests to complete even if teardown fails
    // The process will clean up when it exits anyway
  }
}
