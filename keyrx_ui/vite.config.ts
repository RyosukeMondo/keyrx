import { defineConfig, Plugin } from 'vite'
import react from '@vitejs/plugin-react'
import wasm from 'vite-plugin-wasm'
import topLevelAwait from 'vite-plugin-top-level-await'
import { exec } from 'child_process'
import { promisify } from 'util'
import path from 'path'

const execAsync = promisify(exec)

// Custom plugin to rebuild WASM when Rust files change
function wasmRebuildPlugin(): Plugin {
  return {
    name: 'wasm-rebuild',
    async handleHotUpdate({ file, server }) {
      // Check if the changed file is a Rust source file in keyrx_core
      const keyrxCorePath = path.resolve(__dirname, '../keyrx_core/src')
      if (file.startsWith(keyrxCorePath) && file.endsWith('.rs')) {
        console.log('Rust file changed, rebuilding WASM...')
        try {
          await execAsync('npm run build:wasm', { cwd: __dirname })
          console.log('WASM rebuild complete')
          // Trigger full page reload since WASM changed
          server.ws.send({ type: 'full-reload' })
        } catch (error) {
          console.error('WASM rebuild failed:', error)
        }
      }
    },
    configureServer(server) {
      // Watch keyrx_core/src directory
      const keyrxCorePath = path.resolve(__dirname, '../keyrx_core/src')
      server.watcher.add(keyrxCorePath + '/**/*.rs')
    }
  }
}

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    react(),
    wasm(),
    topLevelAwait(),
    wasmRebuildPlugin()
  ],
  optimizeDeps: {
    exclude: ['@keyrx/core'],
  },
})
