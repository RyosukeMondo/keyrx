/**
 * Sync port configuration from Rust source to UI configs
 * Ensures SSOT (Single Source of Truth) for port number
 *
 * SSOT: keyrx_daemon/src/services/settings_service.rs::DEFAULT_PORT
 */

import fs from 'fs';
import path from 'path';

const DAEMON_SOURCE = path.join(__dirname, '..', 'keyrx_daemon', 'src', 'services', 'settings_service.rs');
const VITE_CONFIG = path.join(__dirname, '..', 'keyrx_ui', 'vite.config.ts');
const ENV_DEV = path.join(__dirname, '..', 'keyrx_ui', '.env.development');
const ENV_EXAMPLE = path.join(__dirname, '..', 'keyrx_ui', '.env.example');

/**
 * Extract DEFAULT_PORT from Rust source code
 */
function extractDefaultPort(): number {
  if (!fs.existsSync(DAEMON_SOURCE)) {
    throw new Error(`Rust source not found: ${DAEMON_SOURCE}`);
  }

  const content = fs.readFileSync(DAEMON_SOURCE, 'utf8');
  const match = content.match(/pub const DEFAULT_PORT:\s*u16\s*=\s*(\d+);/);

  if (!match) {
    throw new Error('DEFAULT_PORT not found in Rust source. Expected: pub const DEFAULT_PORT: u16 = XXXX;');
  }

  return parseInt(match[1], 10);
}

/**
 * Update vite.config.ts proxy configuration
 */
function updateViteConfig(port: number): void {
  if (!fs.existsSync(VITE_CONFIG)) {
    console.warn(`⚠️  vite.config.ts not found: ${VITE_CONFIG}`);
    return;
  }

  let content = fs.readFileSync(VITE_CONFIG, 'utf8');

  // Update HTTP proxy target
  const httpRegex = /target:\s*['"]http:\/\/localhost:\d+['"]/g;
  content = content.replace(httpRegex, `target: 'http://localhost:${port}'`);

  // Update WebSocket proxy target
  const wsRegex = /target:\s*['"]ws:\/\/localhost:\d+['"]/g;
  content = content.replace(wsRegex, `target: 'ws://localhost:${port}'`);

  fs.writeFileSync(VITE_CONFIG, content, 'utf8');
}

/**
 * Update .env.development
 */
function updateEnvDev(port: number): void {
  const content = `# Auto-generated from DEFAULT_PORT in settings_service.rs
# DO NOT EDIT MANUALLY - Run: npm run sync-port
# Last synced: ${new Date().toISOString()}

# API Base URL - development daemon
VITE_API_URL=http://localhost:${port}

# WebSocket URL - development daemon (RPC endpoint)
VITE_WS_URL=ws://localhost:${port}/ws-rpc

# Enable debug logging
VITE_DEBUG=true

# Development mode flag
VITE_ENV=development
`;

  fs.writeFileSync(ENV_DEV, content, 'utf8');
}

/**
 * Update .env.example
 */
function updateEnvExample(port: number): void {
  const content = `# Example Environment Configuration
# Copy this file to .env.development or .env.production and configure

# API Base URL - KeyRx daemon HTTP endpoint
VITE_API_URL=http://localhost:${port}

# WebSocket URL - KeyRx daemon WebSocket RPC endpoint
VITE_WS_URL=ws://localhost:${port}/ws-rpc

# Enable debug logging (true/false)
VITE_DEBUG=true

# Environment (development/production)
VITE_ENV=development
`;

  fs.writeFileSync(ENV_EXAMPLE, content, 'utf8');
}

/**
 * Main execution
 */
try {
  console.log('========================================');
  console.log(' SSOT Port Configuration Sync');
  console.log('========================================');
  console.log('');

  const port = extractDefaultPort();
  console.log(`✓ Extracted DEFAULT_PORT from Rust: ${port}`);

  updateViteConfig(port);
  console.log(`✓ Updated vite.config.ts proxy: http://localhost:${port}`);

  updateEnvDev(port);
  console.log(`✓ Updated .env.development: http://localhost:${port}`);

  updateEnvExample(port);
  console.log(`✓ Updated .env.example: http://localhost:${port}`);

  console.log('');
  console.log('========================================');
  console.log('✓ SSOT enforced successfully!');
  console.log(`  All configs now use port ${port}`);
  console.log('========================================');
  console.log('');

  process.exit(0);
} catch (error) {
  console.error('');
  console.error('========================================');
  console.error('✗ SSOT sync failed!');
  console.error('========================================');
  console.error('');
  console.error(error instanceof Error ? error.message : String(error));
  console.error('');
  process.exit(1);
}
