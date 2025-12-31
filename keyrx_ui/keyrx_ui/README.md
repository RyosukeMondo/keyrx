# KeyRX UI

React-based web interface for KeyRX keyboard remapping with WASM integration.

## Tech Stack

- **React 18+** - Modern UI framework
- **TypeScript 5+** - Type-safe development
- **Vite** - Fast build tool with HMR
- **vite-plugin-wasm** - WebAssembly integration for keyrx_core

## Directory Structure

```
src/
├── components/     # Reusable React components
├── wasm/          # WASM integration and bindings
├── hooks/         # Custom React hooks
└── App.tsx        # Main application component
```

## Development

### Prerequisites

- Node.js 18+ and npm

### Setup

```bash
npm install
```

### Development Server

```bash
npm run dev
```

The dev server will start at http://localhost:5173 with hot module replacement.

### Build

```bash
npm run build
```

Builds the application for production to the `dist/` directory.

### Preview Production Build

```bash
npm run preview
```

## WASM Integration

The UI integrates with `keyrx_core` compiled to WebAssembly. The WASM module is configured in `vite.config.ts` using `vite-plugin-wasm`.

WASM bindings will be added in the `src/wasm/` directory.

## Code Style

- Use TypeScript for all new files
- Follow React 18+ best practices (functional components, hooks)
- Component files: PascalCase (e.g., `KeyMapper.tsx`)
- Utility files: camelCase (e.g., `wasmLoader.ts`)

## Integration with Daemon

The UI communicates with the keyrx_daemon backend via:
- REST API for configuration
- WebSocket for real-time updates
- WASM for client-side key mapping simulation
