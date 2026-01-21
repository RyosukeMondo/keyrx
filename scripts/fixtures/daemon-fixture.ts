/**
 * Daemon Fixture
 *
 * Manages daemon lifecycle for automated testing with health checks,
 * graceful shutdown, and log collection for debugging.
 */

import { spawn, ChildProcess } from 'child_process';
import * as path from 'path';
import * as fs from 'fs';

export interface DaemonConfig {
  daemonPath: string;
  port: number;
  configPath?: string;
  debug?: boolean;
}

export interface DaemonLogs {
  stdout: string;
  stderr: string;
}

/**
 * DaemonFixture manages the lifecycle of a daemon process for testing
 */
export class DaemonFixture {
  private process?: ChildProcess;
  private config: DaemonConfig;
  private stdoutBuffer: string[] = [];
  private stderrBuffer: string[] = [];
  private isReady = false;

  constructor(config: DaemonConfig) {
    this.config = config;
  }

  /**
   * Start the daemon process
   */
  async start(): Promise<void> {
    if (this.process) {
      throw new Error('Daemon is already running');
    }

    // Verify daemon executable exists
    if (!fs.existsSync(this.config.daemonPath)) {
      throw new Error(`Daemon executable not found: ${this.config.daemonPath}`);
    }

    // Build command arguments
    const args: string[] = ['run'];

    if (this.config.configPath) {
      args.push('--config', this.config.configPath);
    }

    if (this.config.debug) {
      args.push('--debug');
    }

    // Note: On Windows, the daemon requires admin privileges
    // For testing, we may need to run in validation/simulation mode
    // or use a test-specific non-privileged mode

    console.log(`Starting daemon: ${this.config.daemonPath} ${args.join(' ')}`);

    // Spawn daemon process
    this.process = spawn(this.config.daemonPath, args, {
      stdio: ['ignore', 'pipe', 'pipe'],
      detached: false,
      env: {
        ...process.env,
        KEYRX_PORT: this.config.port.toString(),
      },
    });

    // Capture stdout
    if (this.process.stdout) {
      this.process.stdout.on('data', (data: Buffer) => {
        const text = data.toString();
        this.stdoutBuffer.push(text);
        if (this.config.debug) {
          process.stdout.write(`[daemon stdout] ${text}`);
        }
      });
    }

    // Capture stderr
    if (this.process.stderr) {
      this.process.stderr.on('data', (data: Buffer) => {
        const text = data.toString();
        this.stderrBuffer.push(text);
        if (this.config.debug) {
          process.stderr.write(`[daemon stderr] ${text}`);
        }
      });
    }

    // Handle process exit
    this.process.on('exit', (code, signal) => {
      console.log(`Daemon exited: code=${code}, signal=${signal}`);
      this.isReady = false;
      this.process = undefined;
    });

    // Handle process errors
    this.process.on('error', (error) => {
      console.error(`Daemon process error: ${error.message}`);
      this.isReady = false;
    });
  }

  /**
   * Wait until daemon is ready by polling health check endpoint
   */
  async waitUntilReady(timeoutMs: number = 30000): Promise<void> {
    if (!this.process) {
      throw new Error('Daemon is not running');
    }

    const startTime = Date.now();
    const pollInterval = 100; // Poll every 100ms
    const baseUrl = `http://localhost:${this.config.port}`;

    while (Date.now() - startTime < timeoutMs) {
      try {
        // Try to fetch status endpoint
        const response = await fetch(`${baseUrl}/api/status`, {
          signal: AbortSignal.timeout(1000),
        });

        if (response.ok) {
          const data = await response.json();

          // Check if daemon reports as running
          // Status may have different states: "starting", "running", "stopping"
          if (data.status === 'running' || data.running === true) {
            this.isReady = true;
            console.log(`✓ Daemon is ready (took ${Date.now() - startTime}ms)`);
            return;
          }
        }
      } catch (error) {
        // Ignore connection errors during startup
        // These are expected while daemon is still initializing
      }

      // Check if process died
      if (!this.process || this.process.exitCode !== null) {
        const logs = this.getLogs();
        throw new Error(
          `Daemon process died during startup\n` +
          `Exit code: ${this.process?.exitCode}\n` +
          `Stderr: ${logs.stderr}`
        );
      }

      // Wait before next poll
      await new Promise(resolve => setTimeout(resolve, pollInterval));
    }

    // Timeout reached
    const logs = this.getLogs();
    throw new Error(
      `Daemon failed to become ready within ${timeoutMs}ms\n` +
      `Stderr: ${logs.stderr}`
    );
  }

  /**
   * Stop the daemon gracefully
   */
  async stop(): Promise<void> {
    if (!this.process) {
      return; // Already stopped
    }

    const proc = this.process;
    const pid = proc.pid;

    if (!pid) {
      this.process = undefined;
      return;
    }

    // Try graceful shutdown first (SIGTERM)
    console.log(`Stopping daemon (PID ${pid})...`);

    try {
      if (process.platform === 'win32') {
        // Windows: use taskkill
        spawn('taskkill', ['/pid', pid.toString(), '/f'], {
          stdio: 'ignore',
        });
      } else {
        // Unix: send SIGTERM
        process.kill(pid, 'SIGTERM');
      }
    } catch (error) {
      console.warn(`Failed to send termination signal: ${error}`);
    }

    // Wait for process to exit (up to 5 seconds)
    const exitTimeout = 5000;
    const startTime = Date.now();

    while (proc.exitCode === null && Date.now() - startTime < exitTimeout) {
      await new Promise(resolve => setTimeout(resolve, 100));
    }

    // If still running, force kill
    if (proc.exitCode === null) {
      console.warn('Daemon did not exit gracefully, force killing...');
      try {
        if (process.platform === 'win32') {
          spawn('taskkill', ['/pid', pid.toString(), '/f', '/t'], {
            stdio: 'ignore',
          });
        } else {
          process.kill(pid, 'SIGKILL');
        }
      } catch (error) {
        console.error(`Failed to force kill daemon: ${error}`);
      }
    } else {
      console.log('✓ Daemon stopped gracefully');
    }

    this.process = undefined;
    this.isReady = false;
  }

  /**
   * Get captured logs
   */
  getLogs(): DaemonLogs {
    return {
      stdout: this.stdoutBuffer.join(''),
      stderr: this.stderrBuffer.join(''),
    };
  }

  /**
   * Check if daemon is ready
   */
  isRunning(): boolean {
    return this.isReady && this.process !== undefined && this.process.exitCode === null;
  }

  /**
   * Get base URL for API requests
   */
  getBaseUrl(): string {
    return `http://localhost:${this.config.port}`;
  }

  /**
   * Get daemon process (for advanced usage)
   */
  getProcess(): ChildProcess | undefined {
    return this.process;
  }
}

/**
 * Utility function to find an available port
 */
export async function findAvailablePort(startPort: number, maxAttempts: number = 10): Promise<number> {
  const net = await import('net');

  for (let i = 0; i < maxAttempts; i++) {
    const port = startPort + i;

    try {
      await new Promise<void>((resolve, reject) => {
        const server = net.createServer();

        server.once('error', (err: NodeJS.ErrnoException) => {
          if (err.code === 'EADDRINUSE') {
            reject(err);
          } else {
            reject(err);
          }
        });

        server.once('listening', () => {
          server.close(() => resolve());
        });

        server.listen(port);
      });

      return port; // Port is available
    } catch (error) {
      // Port is in use, try next
      continue;
    }
  }

  throw new Error(`No available port found in range ${startPort}-${startPort + maxAttempts - 1}`);
}
