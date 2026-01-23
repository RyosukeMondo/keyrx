/**
 * Daemon Test Fixture
 *
 * Provides reliable daemon lifecycle management for automated tests.
 * Handles startup, health checking, log collection, and graceful shutdown.
 */

import { spawn, ChildProcess } from 'child_process';
import * as fs from 'fs';
import * as net from 'net';

export interface DaemonConfig {
  daemonPath: string;
  port?: number;
  testMode?: boolean;
  env?: Record<string, string>;
  healthCheckPath?: string;
  startupTimeout?: number;
  shutdownTimeout?: number;
}

export interface DaemonHealthStatus {
  healthy: boolean;
  status?: string;
  running?: boolean;
  error?: string;
}

/**
 * Test fixture for managing daemon lifecycle
 */
export class DaemonFixture {
  private process?: ChildProcess;
  private readonly config: Required<DaemonConfig>;
  private readonly stdoutLines: string[] = [];
  private readonly stderrLines: string[] = [];
  private startTime?: number;

  constructor(config: DaemonConfig) {
    this.config = {
      daemonPath: config.daemonPath,
      port: config.port ?? this.findAvailablePort(9867),
      testMode: config.testMode ?? false,
      env: config.env ?? {},
      healthCheckPath: config.healthCheckPath ?? '/api/status',
      startupTimeout: config.startupTimeout ?? 30000,
      shutdownTimeout: config.shutdownTimeout ?? 5000,
    };
  }

  /**
   * Find an available port, starting from the preferred port
   */
  private findAvailablePort(preferredPort: number): number {
    // For simplicity, just return the preferred port
    // In a real implementation, we'd check if the port is available
    // and try the next one if it's taken
    return preferredPort;
  }

  /**
   * Check if a port is available
   */
  private async isPortAvailable(port: number): Promise<boolean> {
    return new Promise((resolve) => {
      const server = net.createServer();

      server.once('error', (err: NodeJS.ErrnoException) => {
        if (err.code === 'EADDRINUSE') {
          resolve(false);
        } else {
          resolve(false);
        }
      });

      server.once('listening', () => {
        server.close();
        resolve(true);
      });

      server.listen(port);
    });
  }

  /**
   * Start the daemon process with test profile
   */
  async start(retryOnPortConflict = true): Promise<void> {
    // Check if daemon binary exists
    if (!fs.existsSync(this.config.daemonPath)) {
      throw new Error(`Daemon binary not found: ${this.config.daemonPath}`);
    }

    // Check if port is available
    const portAvailable = await this.isPortAvailable(this.config.port);
    if (!portAvailable) {
      if (retryOnPortConflict) {
        // Try next port
        this.config.port += 1;
        return this.start(false); // Don't retry again
      } else {
        throw new Error(`Port ${this.config.port} is already in use`);
      }
    }

    // Build command arguments - use 'run' subcommand
    const args = ['run'];

    // Add test mode flag if enabled
    if (this.config.testMode) {
      args.push('--test-mode');
    }

    // Merge environment variables
    const env = {
      ...process.env,
      RUST_LOG: 'debug',
      ...this.config.env,
    };

    // Spawn daemon process
    this.process = spawn(this.config.daemonPath, args, {
      stdio: ['ignore', 'pipe', 'pipe'],
      env,
    });

    this.startTime = Date.now();

    // Capture stdout
    this.process.stdout?.on('data', (data) => {
      const lines = data.toString().split('\n');
      this.stdoutLines.push(...lines.filter((line: string) => line.trim()));
    });

    // Capture stderr
    this.process.stderr?.on('data', (data) => {
      const lines = data.toString().split('\n');
      this.stderrLines.push(...lines.filter((line: string) => line.trim()));
    });

    // Handle process errors
    this.process.on('error', (error) => {
      throw new Error(`Failed to start daemon: ${error.message}`);
    });

    // Handle unexpected exits
    this.process.on('exit', (code, signal) => {
      if (code !== null && code !== 0) {
        const errorMsg = `Daemon exited unexpectedly with code ${code}`;
        console.error(errorMsg);
        console.error('Stderr:', this.stderrLines.join('\n'));
      }
    });

    // Wait for daemon to be ready
    await this.waitUntilReady(this.config.startupTimeout);
  }

  /**
   * Wait until daemon is healthy and ready to accept requests
   */
  async waitUntilReady(timeoutMs: number): Promise<void> {
    const startTime = Date.now();
    const checkInterval = 100; // Check every 100ms
    let lastError: Error | undefined;

    while (Date.now() - startTime < timeoutMs) {
      try {
        const health = await this.checkHealth();
        if (health.healthy) {
          return;
        }
        lastError = new Error(`Daemon not healthy: ${health.error ?? 'unknown'}`);
      } catch (error) {
        lastError = error as Error;
      }

      // Check if process has died
      if (this.process && this.process.exitCode !== null) {
        throw new Error(
          `Daemon process exited during startup with code ${this.process.exitCode}`
        );
      }

      await new Promise((resolve) => setTimeout(resolve, checkInterval));
    }

    // Timeout reached
    throw new Error(
      `Daemon failed to become ready within ${timeoutMs}ms. Last error: ${lastError?.message ?? 'unknown'}`
    );
  }

  /**
   * Check daemon health via HTTP endpoint
   */
  async checkHealth(): Promise<DaemonHealthStatus> {
    const url = `http://localhost:${this.config.port}${this.config.healthCheckPath}`;

    try {
      const response = await fetch(url, {
        method: 'GET',
        headers: { 'Content-Type': 'application/json' },
      });

      if (!response.ok) {
        return {
          healthy: false,
          error: `HTTP ${response.status}: ${response.statusText}`,
        };
      }

      const data = await response.json();

      // Check for various health indicators
      const healthy =
        data.status === 'ready' ||
        data.status === 'running' ||
        data.running === true ||
        data.state === 'running' ||
        data.healthy === true;

      return {
        healthy,
        status: data.status,
        running: data.running,
        error: healthy ? undefined : 'Daemon not ready',
      };
    } catch (error) {
      return {
        healthy: false,
        error: error instanceof Error ? error.message : String(error),
      };
    }
  }

  /**
   * Stop the daemon gracefully
   */
  async stop(): Promise<void> {
    if (!this.process || this.process.exitCode !== null) {
      return;
    }

    return new Promise((resolve, reject) => {
      if (!this.process) {
        resolve();
        return;
      }

      const { shutdownTimeout } = this.config;
      let resolved = false;

      // Set up force kill timeout
      const forceKillTimer = setTimeout(() => {
        if (this.process && this.process.exitCode === null) {
          console.warn('Daemon did not stop gracefully, force killing...');
          this.process.kill('SIGKILL');
        }
      }, shutdownTimeout);

      // Handle process exit
      this.process.once('exit', (code, signal) => {
        clearTimeout(forceKillTimer);
        if (!resolved) {
          resolved = true;
          resolve();
        }
      });

      // Handle process error during shutdown
      this.process.once('error', (error) => {
        clearTimeout(forceKillTimer);
        if (!resolved) {
          resolved = true;
          reject(error);
        }
      });

      // Send termination signal
      try {
        if (process.platform === 'win32') {
          // Windows: use taskkill for graceful shutdown
          const { exec } = require('child_process');
          exec(`taskkill /pid ${this.process.pid} /t /f`, (error: Error | null) => {
            if (error && !resolved) {
              // Process might have already exited
              resolved = true;
              resolve();
            }
          });
        } else {
          // Unix: send SIGTERM
          this.process.kill('SIGTERM');
        }
      } catch (error) {
        if (!resolved) {
          resolved = true;
          reject(error);
        }
      }
    });
  }

  /**
   * Restart the daemon
   */
  async restart(): Promise<void> {
    await this.stop();
    await new Promise((resolve) => setTimeout(resolve, 500)); // Brief pause
    await this.start();
  }

  /**
   * Get collected stdout logs
   */
  getStdoutLogs(): string[] {
    return [...this.stdoutLines];
  }

  /**
   * Get collected stderr logs
   */
  getStderrLogs(): string[] {
    return [...this.stderrLines];
  }

  /**
   * Get all logs (stdout + stderr)
   */
  getLogs(): string[] {
    return [...this.stdoutLines, ...this.stderrLines];
  }

  /**
   * Get daemon port
   */
  getPort(): number {
    return this.config.port;
  }

  /**
   * Get base URL for API requests
   */
  getBaseUrl(): string {
    return `http://localhost:${this.config.port}`;
  }

  /**
   * Check if daemon process is running
   */
  isRunning(): boolean {
    return this.process !== undefined && this.process.exitCode === null;
  }

  /**
   * Get daemon uptime in milliseconds
   */
  getUptime(): number | undefined {
    if (!this.startTime) {
      return undefined;
    }
    return Date.now() - this.startTime;
  }

  /**
   * Clear collected logs
   */
  clearLogs(): void {
    this.stdoutLines.length = 0;
    this.stderrLines.length = 0;
  }
}
