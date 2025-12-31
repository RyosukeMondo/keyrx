# Tasks Document

## Phase 1: Daemon WebSocket Event Broadcasting (Rust)

- [x] 1. Add broadcast channel to daemon main loop in keyrx_daemon/src/main.rs
  - Create tokio::sync::broadcast::channel(1000) for event broadcasting
  - Clone sender to share across event loop and WebSocket handler
  - Broadcast state changes, key events, latency stats
  - _Prompt: Role: Rust Async Developer | Task: Add broadcast channel to daemon main loop for WebSocket event streaming | Restrictions: File ≤500 lines, use tokio::sync::broadcast, broadcast on state changes only (not every event), batch if >100 events/sec | Success: ✅ Channel created, ✅ Events broadcast, ✅ No performance impact_

- [x] 2. Implement WebSocket event streaming in keyrx_daemon/src/web/ws.rs
  - Subscribe to broadcast channel in handle_websocket
  - Forward events to WebSocket clients as JSON
  - Implement batching (50ms intervals when high-frequency)
  - _Prompt: Role: Rust WebSocket Developer | Task: Implement event streaming in ws.rs, subscribing to broadcast channel and forwarding to clients | Restrictions: File ≤500 lines, batch updates if >100 events/sec, handle client disconnect gracefully, JSON serialize with serde_json | Success: ✅ Events forwarded to clients, ✅ Batching works, ✅ No memory leaks_

## Phase 2: React WebSocket Integration

- [x] 3. Install WebSocket and state management dependencies
  - Add react-use-websocket@^4.8.1, zustand@^4.5.0, recharts@^2.10.0, react-window@^1.8.10
  - _Prompt: Role: Frontend Build Engineer | Task: Add WebSocket and visualization dependencies to package.json | Success: ✅ Dependencies installed, ✅ No conflicts_

- [x] 4. Create Zustand dashboard store in keyrx_ui/src/store/dashboardStore.ts
  - Define store with currentState, events[], metrics
  - Actions: updateState, addEvent, updateMetrics
  - _Prompt: Role: React State Management Expert | Task: Create Zustand store for dashboard state | Restrictions: File ≤200 lines, TypeScript strict mode, persist last 100 events only | Success: ✅ Store created, ✅ Actions work, ✅ FIFO for events_

- [x] 5. Create useDaemonWebSocket hook in keyrx_ui/src/hooks/useDaemonWebSocket.ts
  - Wrap react-use-websocket with auto-reconnect
  - Parse JSON messages and update Zustand store
  - Handle connection state (connected, connecting, disconnected)
  - _Prompt: Role: React Hooks Developer | Task: Create WebSocket hook with auto-reconnect and message parsing | Restrictions: File ≤300 lines, TypeScript strict mode, retry on disconnect (exponential backoff), update store on message | Success: ✅ Auto-reconnects, ✅ Messages parsed, ✅ Store updated_

## Phase 3: Dashboard UI Components

- [x] 6. Create DashboardPage component in keyrx_ui/src/pages/DashboardPage.tsx
  - Main dashboard layout with state panel, metrics chart, event timeline
  - Connect to useDaemonWebSocket hook and Zustand store
  - _Prompt: Role: React Developer | Task: Create dashboard page with real-time data display | Restrictions: File ≤400 lines, use hooks for WebSocket and store, show connection status, display all 3 panels | Success: ✅ Layout matches design, ✅ Real-time updates work_

- [x] 7. Create StateIndicatorPanel component in keyrx_ui/src/components/StateIndicatorPanel.tsx
  - Display active modifiers/locks/layer as color-coded badges
  - Animate badge changes
  - _Prompt: Role: React UI Developer | Task: Create state indicator with badges | Restrictions: File ≤200 lines, badges color-coded (blue=mod, orange=lock, green=layer), ARIA labels for accessibility | Success: ✅ Badges display correctly, ✅ Animations smooth, ✅ 0 axe violations_

- [x] 8. Create MetricsChart component in keyrx_ui/src/components/MetricsChart.tsx
  - Line chart showing latency over 60-second window
  - Highlight values >5ms in red
  - _Prompt: Role: React Visualization Developer | Task: Create latency metrics chart with recharts | Restrictions: File ≤250 lines, use recharts LineChart, 60-second rolling window, red line when >5ms | Success: ✅ Chart renders, ✅ Data updates in real-time, ✅ Red highlighting works_

- [x] 9. Create EventTimeline component in keyrx_ui/src/components/DashboardEventTimeline.tsx
  - Virtualized list (react-window) of last 100 events
  - Pause/resume functionality
  - Tooltips on hover
  - _Prompt: Role: React Performance Engineer | Task: Create virtualized event timeline | Restrictions: File ≤300 lines, use react-window for virtualization, FIFO (100 max), pause buffers events, tooltips show full details | Success: ✅ Timeline renders 100+ events smoothly, ✅ Pause works, ✅ Tooltips display_

## Phase 4: Testing & Documentation

- [x] 10. Write unit tests for WebSocket broadcasting (Rust)
  - Test broadcast channel publishes events
  - Test batching when >100 events/sec
  - _Prompt: Role: Rust Test Engineer | Task: Write unit tests for WebSocket broadcasting | Success: ✅ All tests pass, ✅ Batching verified_

- [x] 11. Write unit tests for useDaemonWebSocket hook
  - Test auto-reconnect on disconnect
  - Test message parsing
  - _Prompt: Role: React Test Engineer | Task: Write hook tests with mocked WebSocket | Success: ✅ Auto-reconnect tested, ✅ Parsing verified_

- [x] 12. Write E2E test for dashboard workflow
  - Test dashboard loads → WebSocket connects → events appear
  - _Prompt: Role: QA Automation Engineer | Task: Write E2E test for full dashboard | Success: ✅ Dashboard displays real-time data, ✅ Test passes in CI_

- [x] 13. Create dashboard documentation in docs/real-time-dashboard.md
  - Explain how to use dashboard
  - Document WebSocket protocol
  - _Prompt: Role: Technical Writer | Task: Document dashboard usage and WebSocket protocol | Success: ✅ Documentation complete_

- [x] 14. Log implementation artifacts
  - Use spec-workflow log-implementation tool
  - Document all components, hooks, WebSocket protocol
  - _Prompt: Role: Documentation Engineer | Task: Log all dashboard implementation artifacts | Success: ✅ All artifacts documented_
