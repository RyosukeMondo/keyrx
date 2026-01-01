# Production Build Guide

This document describes the production build configuration for the KeyRx UI v2 application.

## Environment Configuration

### Environment Variables

The application uses environment variables for configuration. These are defined in `.env.*` files:

- `.env.development` - Development configuration (used by default with `npm run dev`)
- `.env.production` - Production configuration (used with `npm run build:production`)
- `.env.example` - Example configuration template

#### Available Variables

| Variable | Description | Development Default | Production Default |
|----------|-------------|---------------------|-------------------|
| `VITE_API_URL` | KeyRx daemon HTTP API base URL | `http://localhost:9867` | _(same origin)_ |
| `VITE_WS_URL` | KeyRx daemon WebSocket URL | `ws://localhost:9867/ws` | _(same origin with ws/wss)_ |
| `VITE_DEBUG` | Enable debug logging | `true` | `false` |
| `VITE_ENV` | Environment identifier | `development` | `production` |

#### Runtime Configuration

The `src/config/env.ts` module provides type-safe access to environment variables:

```typescript
import { env } from '@/config/env';

// Use configuration
const apiUrl = env.apiUrl;    // Auto-detects production vs development
const wsUrl = env.wsUrl;       // Auto-detects ws:// vs wss://
const debug = env.debug;       // Boolean flag
```

**Production behavior:**
- If `VITE_API_URL` is empty, uses same origin as the UI (for embedded deployment)
- If `VITE_WS_URL` is empty, uses same origin with `ws://` or `wss://` based on page protocol
- Debug logging is disabled by default

## Build Scripts

### Development

```bash
npm run dev                # Start development server (http://localhost:5173)
```

### Production Build

```bash
npm run build              # Standard production build
npm run build:production   # Explicit production mode build
npm run build:analyze      # Build and analyze bundle size
npm run preview            # Preview production build locally
```

### Type Checking

```bash
npm run type-check         # Run TypeScript compiler in check-only mode
```

## Build Optimizations

### Code Splitting

The build configuration uses manual chunk splitting for optimal loading:

| Chunk | Contents | Size (gzipped) |
|-------|----------|----------------|
| `react-core` | React, ReactDOM | ~54KB |
| `vendor` | Other node_modules | ~52KB |
| `recharts` | Chart library | ~52KB |
| `framer-motion` | Animation library | ~33KB |
| `ui-libs` | Headless UI, Floating UI | ~18KB |
| `lucide-icons` | Icon library | ~1KB |
| Page chunks | Lazy-loaded routes | ~2-3KB each |

### Minification

- **Tool:** Terser
- **Options:**
  - `drop_console: true` - Remove all `console.log` statements
  - `drop_debugger: true` - Remove all `debugger` statements
  - `passes: 2` - Multiple compression passes for better results
  - `safari10: true` - Safari 10+ compatibility
  - `comments: false` - Strip all comments

### Compression

Generated files include both gzip and brotli compressed versions:

- `.gz` files - Gzip compression (widely supported)
- `.br` files - Brotli compression (better compression ratio, modern browsers)

Server should be configured to serve pre-compressed files when available.

### Source Maps

Source maps are generated in production for debugging:

- Files: `dist/assets/*.js.map`
- Format: Separate files (not inline)
- Usage: Load in browser DevTools for debugging production issues

## Bundle Size Budget

Target bundle sizes (gzipped):

| Asset Type | Budget | Actual | Status |
|------------|--------|--------|--------|
| JavaScript (total) | ≤250KB | ~268KB | ⚠️ Slightly over |
| CSS (total) | ≤50KB | ~6KB | ✅ Well under |
| Individual JS chunks | ≤100KB | ✅ All under | ✅ Pass |

**Notes:**
- Total JS is slightly over budget due to large dependencies (Recharts, Framer Motion)
- Individual chunks are all well under limits
- Lazy loading ensures only needed code is loaded per page
- Future optimization: Consider removing or replacing heavy dependencies

## Build Output Structure

```
dist/
├── index.html                    # Main HTML file
├── assets/
│   ├── index-[hash].css         # Main stylesheet (~6KB gzipped)
│   ├── index-[hash].css.gz      # Gzipped CSS
│   ├── index-[hash].css.br      # Brotli CSS
│   ├── [chunk]-[hash].js        # JavaScript chunks
│   ├── [chunk]-[hash].js.gz     # Gzipped JS
│   ├── [chunk]-[hash].js.br     # Brotli JS
│   └── [chunk]-[hash].js.map    # Source maps
└── stats.html                    # Bundle analysis visualization
```

## Deployment

### Embedded in Daemon (Recommended)

The UI is designed to be embedded in the KeyRx daemon binary:

1. Build the production bundle: `npm run build:production`
2. Copy `dist/*` to daemon's static files directory
3. Daemon serves UI from `/` route

See Task 39 in the spec for daemon integration details.

### Standalone Server

For development or testing:

```bash
npm run preview                    # Serve on http://localhost:4173
npm run preview:production         # Preview with production env vars
```

### CDN/Static Hosting

The `dist/` directory can be deployed to any static file host:

- Set `VITE_API_URL` and `VITE_WS_URL` to point to daemon
- Configure server to:
  - Serve pre-compressed files (.gz, .br) when available
  - Set proper MIME types for source maps
  - Enable CORS if daemon is on different origin

## Performance Metrics

Target Web Vitals (from spec):

| Metric | Target | Description |
|--------|--------|-------------|
| LCP | <2.5s | Largest Contentful Paint |
| FCP | <1.5s | First Contentful Paint |
| TTI | <3.0s | Time to Interactive |
| CLS | <0.1 | Cumulative Layout Shift |
| FID | <100ms | First Input Delay |

Test with:
```bash
npm run test:performance          # Run Playwright performance tests
npm run test:lighthouse           # Run Lighthouse audit
```

## Troubleshooting

### Build Fails with TypeScript Errors

The `tsc -b` step may fail due to test file type issues. Use direct Vite build:

```bash
npx vite build --mode production
```

### Bundle Size Exceeds Budget

1. Run bundle analyzer: `npm run build:analyze`
2. Open `dist/stats.html` in browser
3. Identify large dependencies
4. Consider:
   - Lazy loading heavy components
   - Replacing large dependencies with lighter alternatives
   - Tree-shaking unused exports

### Environment Variables Not Working

1. Ensure `.env.production` exists
2. Variable names must start with `VITE_` prefix
3. Restart dev server after changing `.env` files
4. Check `src/config/env.ts` for variable access

### Source Maps Not Loading

1. Ensure `sourcemap: true` in `vite.config.ts`
2. Check browser DevTools settings (source maps enabled)
3. Verify `.map` files exist in `dist/assets/`

## CI/CD Integration

Production build should be automated in CI/CD pipeline (Task 40):

```yaml
- name: Build production bundle
  run: npm run build:production

- name: Check bundle size
  run: node scripts/analyze-bundle.js

- name: Upload build artifacts
  uses: actions/upload-artifact@v3
  with:
    name: ui-dist
    path: dist/
```

## Related Documentation

- [Task 38: Configure production build](../.spec-workflow/specs/web-ui-configuration-editor/tasks.md#L1138)
- [Task 39: Embed UI in daemon](../.spec-workflow/specs/web-ui-configuration-editor/tasks.md#L1147)
- [Task 40: Add CI/CD pipeline](../.spec-workflow/specs/web-ui-configuration-editor/tasks.md#L1156)
- [Vite Build Documentation](https://vitejs.dev/guide/build.html)
- [Environment Variables](https://vitejs.dev/guide/env-and-mode.html)
