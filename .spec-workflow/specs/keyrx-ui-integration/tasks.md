# Tasks Document

## Phase 1: Unified WebSocket RPC API

- [x] 1. Define RPC Message Types in Rust
  - File: keyrx_daemon/src/web/rpc_types.rs
  - Create ClientMessage enum with Query, Command, Subscribe, Unsubscribe variants
  - Create ServerMessage enum with Response, Event, Connected variants
  - Create RpcError struct with code, message, data fields
  - Define JSON-RPC error code constants
  - Purpose: Establish type-safe message protocol for WebSocket RPC
  - _Leverage: serde for serialization, JSON-RPC error codes specification_
  - _Requirements: REQ-1 (AC1, AC5)_
  - _Prompt: Role: Rust Backend Developer specializing in type systems and serialization | Task: Create keyrx_daemon/src/web/rpc_types.rs with RPC message type definitions using serde's tag attribute for type discrimination, implement ClientMessage (Query, Command, Subscribe, Unsubscribe) and ServerMessage (Response, Event, Connected) enums, define standard JSON-RPC error codes (PARSE_ERROR=-32700, INVALID_REQUEST=-32600, METHOD_NOT_FOUND=-32601, INVALID_PARAMS=-32602, INTERNAL_ERROR=-32603) | Restrictions: Must use serde's tag-based enum serialization, all types must derive Serialize/Deserialize, error codes must match JSON-RPC 2.0 specification exactly | Success: All types serialize/deserialize correctly to/from JSON, unit tests verify round-trip serialization for all message types, error codes are constants and match specification_

- [x] 2. Implement WebSocket RPC Handler
  - File: keyrx_daemon/src/web/ws_rpc.rs, keyrx_daemon/src/web/mod.rs
  - Create ws_handler function for WebSocket upgrade
  - Implement message loop with request/response correlation
  - Add handshake with Connected message
  - Handle parse errors and unknown methods
  - Purpose: Core WebSocket RPC server implementation with message routing
  - _Leverage: axum WebSocket upgrade, tokio async runtime_
  - _Requirements: REQ-1 (AC1, AC2, AC3, AC6, AC7)_
  - _Prompt: Role: Rust WebSocket Developer with expertise in Axum and async programming | Task: Implement WebSocket RPC handler in keyrx_daemon/src/web/ws_rpc.rs that accepts Axum WebSocket upgrade, sends Connected handshake with version and timestamp, splits socket into sender/receiver, loops on incoming messages deserializing to ClientMessage, routes to handler functions (handle_query, handle_command, handle_subscribe, handle_unsubscribe), correlates responses via UUID tracking in HashMap, handles parse errors with PARSE_ERROR code, unknown methods with METHOD_NOT_FOUND code, cleans up on disconnect | Restrictions: Must use split socket pattern, must not block on message send, UUID correlation is required for all requests, cleanup all pending requests on disconnect | Success: Handshake sent immediately on connect, messages routed correctly to handlers, invalid JSON returns PARSE_ERROR, unknown methods return METHOD_NOT_FOUND, concurrent requests correlated by UUID, disconnect cleans up resources_

- [x] 3. Implement Profile RPC Methods
  - File: keyrx_daemon/src/web/handlers/profile.rs, keyrx_daemon/src/web/handlers/mod.rs
  - Implement get_profiles, create_profile, activate_profile, delete_profile, duplicate_profile, rename_profile
  - Add parameter validation and error handling
  - Connect to profile manager
  - Purpose: Profile CRUD operations via RPC
  - _Leverage: existing profile manager, serde for param deserialization_
  - _Requirements: REQ-1 (AC2, AC3)_
  - _Prompt: Role: Backend Developer with expertise in CRUD operations and validation | Task: Create keyrx_daemon/src/web/handlers/profile.rs with 6 RPC method implementations (get_profiles, create_profile, activate_profile, delete_profile, duplicate_profile, rename_profile), each accepting AppState and serde_json::Value params, deserializing to typed structs, validating profile names (no '..' or '/' for path traversal), calling profile manager methods, returning Result<serde_json::Value, RpcError> | Restrictions: Must validate all inputs before processing, profile names must not allow path traversal, must use existing profile manager without modification, all errors must be RpcError with appropriate codes | Success: All 6 methods work correctly, invalid profile names rejected, integration tests verify each method with valid/invalid inputs, error codes are appropriate_

- [x] 4. Implement Device RPC Methods
  - File: keyrx_daemon/src/web/handlers/device.rs
  - Implement get_devices, rename_device, set_scope_device, forget_device
  - Validate serial numbers to prevent injection
  - Connect to device manager
  - Purpose: Device management operations via RPC
  - _Leverage: existing device manager_
  - _Requirements: REQ-1 (AC2, AC3)_
  - _Prompt: Role: Backend Developer with expertise in device management and security | Task: Create keyrx_daemon/src/web/handlers/device.rs with 4 RPC method implementations (get_devices, rename_device, set_scope_device, forget_device), validate serial numbers to prevent path traversal or injection attacks, call device manager methods, return typed results | Restrictions: Serial number validation is mandatory, must not allow special characters that could cause injection, use existing device manager interface | Success: All 4 methods implemented and tested, serial number validation prevents attacks, device manager integration works correctly_

- [x] 5. Implement Config RPC Methods
  - File: keyrx_daemon/src/web/handlers/config.rs
  - Implement get_config, update_config, set_key_mapping, delete_key_mapping, get_layers
  - Validate configurations with keyrx_compiler
  - Enforce 1MB config size limit
  - Purpose: Configuration CRUD operations with validation
  - _Leverage: keyrx_compiler for validation_
  - _Requirements: REQ-1 (AC2, AC3)_
  - _Prompt: Role: Backend Developer with expertise in configuration management and validation | Task: Create keyrx_daemon/src/web/handlers/config.rs with 5 RPC methods (get_config returns code+hash, update_config validates with keyrx_compiler::parse and enforces 1MB limit, set_key_mapping updates single key, delete_key_mapping removes key, get_layers returns layer list), all methods validate inputs and use config manager | Restrictions: update_config MUST validate with keyrx_compiler before saving, 1MB size limit is mandatory, configuration validation must match daemon behavior exactly | Success: All 5 methods work correctly, invalid configs rejected before save, size limit enforced, validation matches compiler behavior, hash verification works_

- [x] 6. Implement Metrics and Simulator RPC Methods
  - File: keyrx_daemon/src/web/handlers/metrics.rs
  - Implement get_latency, get_events (with pagination), clear_events, simulate, reset_simulator
  - Add pagination support for get_events (default limit 100, max 1000)
  - Connect to simulator from keyrx_core
  - Purpose: Metrics collection and simulation operations
  - _Leverage: keyrx_core simulator_
  - _Requirements: REQ-1 (AC2, AC3)_
  - _Prompt: Role: Backend Developer with expertise in metrics and simulation | Task: Create keyrx_daemon/src/web/handlers/metrics.rs with 5 RPC methods (get_latency returns current metrics, get_events with limit/offset pagination defaults to 100 max 1000, clear_events empties history, simulate uses keyrx_core to run config with input events, reset_simulator clears state), validate all parameters | Restrictions: Pagination limits are mandatory (default 100, max 1000), simulate must use keyrx_core's actual simulator not a mock, results must be deterministic | Success: All methods implemented, pagination works correctly, simulation uses real keyrx_core engine, results are deterministic_

- [x] 7. Implement Subscription Channel Manager
  - File: keyrx_daemon/src/web/subscriptions.rs, keyrx_daemon/src/web/ws_rpc.rs
  - Create SubscriptionManager with channel tracking
  - Implement subscribe, unsubscribe, unsubscribe_all, broadcast methods
  - Support daemon-state, events, latency channels
  - Purpose: Pub/sub infrastructure for real-time updates
  - _Leverage: tokio broadcast channels or custom HashMap_
  - _Requirements: REQ-1 (AC4, AC8, AC10)_
  - _Prompt: Role: Rust Systems Developer with expertise in pub/sub patterns and concurrency | Task: Create keyrx_daemon/src/web/subscriptions.rs with SubscriptionManager that tracks client subscriptions using HashMap<String, HashSet<ClientId>>, implements subscribe (add client to channel), unsubscribe (remove from channel), unsubscribe_all (remove all subscriptions for client), broadcast (send Event message to all subscribed clients on channel), integrate into WebSocket handler for automatic cleanup on disconnect | Restrictions: Must be thread-safe (use Arc<Mutex> or similar), broadcast must not block, disconnect must automatically clean up all subscriptions | Success: Multiple clients can subscribe to same channel, broadcasts reach all subscribed clients, unsubscribe stops events, disconnect cleans up automatically, integration tests verify pub/sub behavior_

- [x] 8. Integrate Daemon Event Broadcasting
  - File: keyrx_daemon/src/main.rs, keyrx_daemon/src/daemon/event_broadcaster.rs
  - Broadcast daemon state changes to "daemon-state" channel
  - Broadcast key events to "events" channel
  - Broadcast latency metrics to "latency" channel every 1 second
  - Purpose: Connect daemon core to WebSocket subscribers
  - _Leverage: SubscriptionManager from task 7_
  - _Requirements: REQ-1 (AC8)_
  - _Prompt: Role: Systems Integration Developer with expertise in event-driven architecture | Task: Modify keyrx_daemon/src/processor.rs to broadcast daemon state (modifiers, locks, layer) to "daemon-state" channel after each state change, broadcast key events (timestamp, keyCode, eventType, latency, layer) to "events" channel after processing, add 1-second periodic task in main.rs to broadcast latency metrics (min, avg, max, p50, p95, p99, count) to "latency" channel, use SubscriptionManager::broadcast | Restrictions: Broadcasts must not block event processing, use async spawn for non-blocking sends, broadcasts only when subscribers exist (check first) | Success: State changes broadcast immediately, key events broadcast in real-time, latency metrics broadcast every 1 second, broadcasts don't affect processing latency, integration tests verify events are received_

- [x] 9. Add RPC Integration Tests
  - File: keyrx_daemon/tests/integration/rpc_api_test.rs
  - Test all RPC methods with real WebSocket client
  - Test subscription and event broadcasting
  - Test error scenarios (invalid JSON, unknown methods, timeouts)
  - Purpose: Verify complete RPC API implementation
  - _Leverage: tokio-tungstenite for WebSocket client_
  - _Requirements: REQ-1 (AC1-AC10)_
  - _Prompt: Role: QA Engineer with expertise in integration testing and WebSocket protocols | Task: Create comprehensive integration tests in keyrx_daemon/tests/integration/rpc_api_test.rs using tokio-tungstenite WebSocket client, test scenarios: (1) connection receives Connected handshake, (2) query with ID receives response with matching ID, (3) command executes and returns success, (4) subscribe to channel receives broadcast events, (5) unsubscribe stops events, (6) invalid JSON returns PARSE_ERROR, (7) unknown method returns METHOD_NOT_FOUND, (8) concurrent requests from multiple clients are correlated correctly, (9) request timeout behavior, (10) disconnect cleans up subscriptions | Restrictions: Must use real WebSocket connections not mocks, must test actual daemon not test doubles, all 10 REQ-1 acceptance criteria must be verified | Success: All acceptance criteria verified, tests run reliably, error scenarios handled correctly, concurrent requests work_

- [x] 10. Define TypeScript RPC Types
  - File: keyrx_ui_v2/src/types/rpc.ts
  - Create RpcMethod type union with all method names
  - Create ClientMessage and ServerMessage interfaces
  - Create data structure interfaces (DaemonState, KeyEvent, LatencyMetrics)
  - Purpose: Type-safe RPC communication on frontend
  - _Leverage: TypeScript discriminated unions_
  - _Requirements: REQ-1 (AC1, AC2, AC3, AC4)_
  - _Prompt: Role: TypeScript Developer specializing in type systems and API contracts | Task: Create keyrx_ui_v2/src/types/rpc.ts with TypeScript types matching Rust RPC types exactly, define RpcMethod as string union of all 20+ method names, SubscriptionChannel as "daemon-state" | "events" | "latency", ClientMessage and ServerMessage as discriminated unions using type field, data structure interfaces (DaemonState with modifiers/locks/layer, KeyEvent with timestamp/keyCode/eventType/latency/layer, LatencyMetrics with min/avg/max/p50/p95/p99/count), RpcError interface with code/message/data | Restrictions: Field names must match Rust types exactly (camelCase in TS, snake_case in Rust conversions handled by serde), all types must be exported, use discriminated unions for type safety | Success: All types compile, field names match Rust types, discriminated unions provide type narrowing, no 'any' types used_

- [x] 11. Implement useUnifiedApi Hook
  - File: keyrx_ui_v2/src/hooks/useUnifiedApi.ts
  - Create hook with query, command, subscribe, unsubscribe methods
  - Implement request/response correlation via UUID
  - Add 30-second timeout for requests
  - Support auto-reconnect (3s interval, 10 attempts)
  - Purpose: React hook for WebSocket RPC communication
  - _Leverage: react-use-websocket library, uuid for IDs_
  - _Requirements: REQ-1 (AC1, AC2, AC3, AC4, AC6, AC9, AC10)_
  - _Prompt: Role: React Developer with expertise in hooks and WebSocket communication | Task: Implement keyrx_ui_v2/src/hooks/useUnifiedApi.ts using react-use-websocket with auto-reconnect (3s, 10 attempts), create query/command methods that generate UUIDs, track pending requests in useRef Map with {resolve, reject, timeout}, send ClientMessage, return Promise that resolves when matching Response received, implement subscribe/unsubscribe that track handlers in useRef Map and call on Event messages, add 30s timeout that rejects promises, handle Connected message to set isConnected, cleanup subscriptions on unmount | Restrictions: Must use useRef for pending requests/subscriptions (not useState), timeouts are mandatory, must cleanup on unmount, auto-reconnect is required | Success: Requests return Promises that resolve with results, timeouts reject after 30s, subscriptions call handlers on events, auto-reconnects on disconnect, cleanup works correctly, isConnected and readyState exposed_

- [x] 12. Create Type-Safe RPC Client Wrapper
  - File: keyrx_ui_v2/src/api/rpc.ts, keyrx_ui_v2/src/api/types.ts
  - Create RpcClient class with typed methods for all RPC operations
  - Add JSDoc comments for all methods
  - Purpose: Type-safe API client with intellisense
  - _Leverage: useUnifiedApi hook from task 11_
  - _Requirements: REQ-1 (AC2, AC3, AC4)_
  - _Prompt: Role: TypeScript API Developer with expertise in type-safe client libraries | Task: Create keyrx_ui_v2/src/api/rpc.ts with RpcClient class that wraps useUnifiedApi hook, implement all 20+ RPC methods with proper TypeScript types (e.g., async getProfiles(): Promise<Profile[]> calls this.api.query('getProfiles')), subscription methods like onDaemonState(handler: (state: DaemonState) => void): void call this.api.subscribe('daemon-state', handler), add comprehensive JSDoc comments describing parameters and return types | Restrictions: All methods must have proper TypeScript types, no 'any' types allowed, must not duplicate logic from useUnifiedApi, JSDoc required for all public methods | Success: All RPC methods implemented with correct types, intellisense works in IDE, JSDoc appears in hover tooltips, methods are thin wrappers over useUnifiedApi_

- [x] 13. Write useUnifiedApi Tests
  - File: keyrx_ui_v2/src/hooks/useUnifiedApi.test.ts
  - Test connection, query/response, command, subscribe/unsubscribe
  - Test timeout and error scenarios
  - Use mock WebSocket server
  - Purpose: Verify hook implementation
  - _Leverage: @testing-library/react-hooks, mock-socket_
  - _Requirements: REQ-1 (AC1, AC2, AC3, AC6, AC9)_
  - _Prompt: Role: React Testing Specialist with expertise in hook testing and WebSocket mocking | Task: Create comprehensive unit tests in keyrx_ui_v2/src/hooks/useUnifiedApi.test.ts using @testing-library/react-hooks and mock-socket, test scenarios: (1) connection and Connected handshake, (2) query sends message and resolves with response, (3) command executes, (4) subscribe receives events, (5) unsubscribe stops events, (6) 30s timeout rejects promise, (7) concurrent requests with different IDs correlated correctly, (8) auto-reconnect on disconnect, (9) server errors reject promises, (10) cleanup on unmount | Restrictions: Must use mock WebSocket not real connections, all assertions must use testing-library best practices, test isolation is required | Success: All test scenarios pass, coverage >= 90%, mocks used correctly, tests are reliable and fast_

- [x] 14. End-to-End RPC Communication Test
  - File: keyrx_ui_v2/tests/integration/rpc-communication.test.ts
  - Test complete RPC stack from React to Rust daemon
  - Execute full profile workflow (create, activate, delete)
  - Test subscription and real-time updates
  - Purpose: Verify entire RPC system works together
  - _Leverage: Vitest, real daemon in test mode_
  - _Requirements: REQ-1 (AC1-AC10)_
  - _Prompt: Role: Integration Testing Engineer with expertise in full-stack testing | Task: Create end-to-end integration test in keyrx_ui_v2/tests/integration/rpc-communication.test.ts that starts daemon in test mode (beforeAll), creates test React component using useUnifiedApi and RpcClient, executes workflow: (1) connect and verify handshake, (2) create new profile, (3) subscribe to daemon-state, (4) activate profile, (5) verify state change event received, (6) update config, (7) delete profile, (8) disconnect, stops daemon (afterAll) | Restrictions: Must use real daemon not mocks, must test actual WebSocket communication, all REQ-1 acceptance criteria must be verified end-to-end | Success: Complete workflow executes successfully, state changes broadcast correctly, all acceptance criteria verified, test runs reliably in CI_

## Phase 2: Monaco Code Editor Integration

- [x] 15. Create Monaco Editor Component
  - File: keyrx_ui_v2/src/components/MonacoEditor.tsx
  - Register Rhai language with syntax highlighting
  - Implement F8 keybinding for next error navigation
  - Add 500ms debounced validation
  - Display error markers with tooltips
  - Purpose: Professional code editor for Rhai configuration
  - _Leverage: @monaco-editor/react, keyrx_core WASM for validation_
  - _Requirements: REQ-2 (AC1, AC2, AC3, AC4, AC5, AC6, AC7, AC8, AC9, AC10)_
  - _Prompt: Role: Frontend Developer specializing in Monaco editor and code editing features | Task: Create keyrx_ui_v2/src/components/MonacoEditor.tsx using @monaco-editor/react, implement beforeMount to register Rhai language with monarch tokenizer (keywords: let/const/if/else/while/for/loop/break/continue/return/fn, operators, strings, numbers, comments), define rhai-dark theme, implement onMount to configure editor (minimap disabled, fontSize 14, tabSize 2, rulers at 80/120), add F8 keybinding to jump to next error, use useEffect with 500ms debounce to run WASM validation, convert errors to Monaco markers, display status showing error count or success message, handle WASM unavailable gracefully | Restrictions: Must use Monaco's language registration API, validation must be debounced exactly 500ms, F8 must jump to next error and center line, must work without WASM (show unavailable status) | Success: Rhai syntax highlighted correctly, F8 navigates errors, validation runs after 500ms, error markers appear at correct lines, tooltips show error messages, graceful fallback without WASM, all REQ-2 acceptance criteria verified_

- [x] 16. Implement useWasm Hook
  - File: keyrx_ui_v2/src/hooks/useWasm.ts
  - Initialize WASM module on mount
  - Provide validateConfig and runSimulation functions
  - Track initialization state and errors
  - Purpose: WASM integration for browser-based validation and simulation
  - _Leverage: keyrx_core WASM module_
  - _Requirements: REQ-2 (AC8, AC9), REQ-5 (AC1, AC2, AC3, AC4, AC5, AC6)_
  - _Prompt: Role: Frontend Developer with expertise in WebAssembly integration and React hooks | Task: Create keyrx_ui_v2/src/hooks/useWasm.ts that uses useEffect to call WASM init() on mount and sets isWasmReady on success, implements validateConfig using useCallback that checks isWasmReady, calls WASM validate_config function, parses JSON result to ValidationError array, implements runSimulation similarly for simulate function, tracks error state if init fails, returns {isWasmReady, error, validateConfig, runSimulation} | Restrictions: Must handle init failure gracefully (log error, set error state, don't crash), validateConfig must return empty array if WASM not ready, functions must be memoized with useCallback | Success: WASM initializes on mount, validateConfig returns errors with line/column/message, runSimulation returns results, graceful handling of init failure, functions are properly memoized_

- [x] 17. Write Monaco Editor Tests
  - File: keyrx_ui_v2/src/components/MonacoEditor.test.tsx
  - Test rendering, syntax highlighting, validation, F8 navigation
  - Mock WASM module
  - Purpose: Verify editor functionality
  - _Leverage: @testing-library/react_
  - _Requirements: REQ-2 (AC1-AC10)_
  - _Prompt: Role: Frontend Testing Specialist with expertise in React component testing | Task: Create comprehensive tests in keyrx_ui_v2/src/components/MonacoEditor.test.tsx mocking Monaco and WASM, test scenarios: (1) component renders, (2) value prop displays in editor, (3) onChange fires when edited, (4) validation errors create markers at correct lines, (5) onValidate callback receives error array, (6) F8 keybinding simulated (call registered command), (7) debounce delays validation 500ms (use jest.useFakeTimers), (8) WASM unavailable shows fallback status, (9) readOnly prop disables editing, (10) syntax highlighting token classes present | Restrictions: Must mock Monaco editor and WASM module, use @testing-library/react best practices, test user-facing behavior not implementation details | Success: All REQ-2 acceptance criteria verified, tests reliable and fast, mocks used correctly, coverage >= 90%_

## Phase 3: Real-Time Dashboard

- [x] 18. Create Dashboard Page Component
  - File: keyrx_ui_v2/src/pages/DashboardPage.tsx
  - Connect to WebSocket and subscribe to all channels
  - Display connection status banner
  - Render StateIndicatorPanel, MetricsChart, EventTimeline
  - Manage state for daemon-state, events (max 100 FIFO), latency (max 60)
  - Purpose: Real-time monitoring dashboard
  - _Leverage: useUnifiedApi, RpcClient_
  - _Requirements: REQ-3 (AC1, AC2, AC3)_
  - _Prompt: Role: React Developer with expertise in real-time data visualization and state management | Task: Create keyrx_ui_v2/src/pages/DashboardPage.tsx that uses useUnifiedApi hook, creates RpcClient, uses useEffect to subscribe to daemon-state/events/latency on mount, tracks daemonState (latest), events array (max 100 FIFO), latencyHistory array (max 60) in useState, renders connection banner (green "Connected" if isConnected else red "Disconnected"), renders StateIndicatorPanel with daemonState, MetricsChart with latencyHistory, DashboardEventTimeline with events, unsubscribes on unmount, uses responsive layout (single column mobile, 2-column grid desktop >= 1024px) | Restrictions: Events array must enforce 100 max FIFO (slice on append), latency array must enforce 60 max, must unsubscribe on unmount to prevent leaks, connection banner must show current status | Success: Dashboard connects on mount, connection banner shows correct status, all three channels subscribed, state updates in real-time, FIFO limits enforced, cleanup on unmount, responsive layout works_

- [x] 19. Create State Indicator Panel Component
  - File: keyrx_ui_v2/src/components/StateIndicatorPanel.tsx
  - Display active modifiers as blue badges
  - Display active locks as orange badges
  - Display current layer as green badge
  - Show "None" when empty
  - Purpose: Visual display of daemon state
  - _Leverage: DaemonState type_
  - _Requirements: REQ-3 (AC4)_
  - _Prompt: Role: React Component Developer with expertise in data visualization and accessibility | Task: Create keyrx_ui_v2/src/components/StateIndicatorPanel.tsx accepting state: DaemonState | null prop, if null show loading message, otherwise render 3-column grid (single column mobile), Modifiers section maps state.modifiers to blue badges with "MOD_{id}" text (bg-blue-600), Locks section maps state.locks to orange badges with "LOCK_{id}" (bg-orange-600), Layer section shows green badge with "Layer {state.layer}" (bg-green-600), show "None" in gray if arrays empty, add ARIA labels for accessibility | Restrictions: Must handle null state gracefully, color scheme must match design (blue/orange/green), responsive grid required (1 col mobile, 3 col desktop), ARIA labels mandatory | Success: Modifiers display as blue badges, locks as orange, layer as green, "None" shown when empty, responsive grid works, ARIA labels present, updates immediately when state prop changes_

- [x] 20. Create Metrics Chart Component
  - File: keyrx_ui_v2/src/components/MetricsChart.tsx
  - Render line chart with avg, P95, P99 lines
  - Add 5ms reference line for performance target
  - Convert microseconds to milliseconds
  - Support responsive sizing
  - Purpose: Visualize latency metrics over time
  - _Leverage: Recharts library_
  - _Requirements: REQ-3 (AC5, AC9)_
  - _Prompt: Role: Data Visualization Developer with expertise in Recharts and performance metrics | Task: Create keyrx_ui_v2/src/components/MetricsChart.tsx accepting data: LatencyMetrics[] prop, transform data by converting microseconds to milliseconds (divide by 1000) and mapping to {index, avg, p95, p99}, render Recharts LineChart with ResponsiveContainer width 100% height 300px, add three lines (avg in blue, p95 in orange, p99 in red), add ReferenceLine at y=5 in red with label "Target (5ms)", configure dark theme colors matching TailwindCSS slate palette, add CartesianGrid, XAxis (index), YAxis (ms label), Tooltip, Legend | Restrictions: Must convert microseconds to milliseconds, reference line must be at exactly 5ms, chart must be responsive, dark theme colors required, height 300px fixed | Success: Chart renders three lines correctly, reference line at 5ms, values in milliseconds, responsive width, dark theme applied, tooltip shows values on hover, legend identifies lines_

- [x] 21. Create Event Timeline Component
  - File: keyrx_ui_v2/src/components/DashboardEventTimeline.tsx
  - Render virtualized list of events (react-window)
  - Support pause/resume and clear
  - Display keyCode, eventType, timestamp
  - Add tooltip with full details
  - Purpose: Real-time event log with virtualization
  - _Leverage: react-window for virtualization_
  - _Requirements: REQ-3 (AC6, AC7, AC8, AC9)_
  - _Prompt: Role: React Performance Specialist with expertise in virtualization and large lists | Task: Create keyrx_ui_v2/src/components/DashboardEventTimeline.tsx accepting events: KeyEvent[], isPaused: boolean, onTogglePause: () => void, onClear: () => void props, render Pause/Resume and Clear buttons above list, use react-window FixedSizeList with height 400px itemSize 50px, each row displays keyCode (formatted as readable label like "A"/"Enter"), eventType, timestamp (formatted as relative time like "2s ago"), add tooltip on hover showing full details (timestamp in microseconds, latency, layer), component is purely presentational (parent handles state) | Restrictions: Must use react-window for virtualization, list height 400px fixed, each item 50px, formatting functions required (keyCodeToLabel, formatTimestamp), component must not manage isPaused state | Success: Events render in virtualized list, newest first, pause stops updates (verified by parent), clear empties list, tooltip shows full details on hover, keyCode formatted as labels, timestamp as relative time, virtualization performs well with 1000+ events_

- [x] 22. Write Dashboard Component Tests
  - File: keyrx_ui_v2/src/pages/DashboardPage.test.tsx, keyrx_ui_v2/src/components/StateIndicatorPanel.test.tsx, keyrx_ui_v2/src/components/MetricsChart.test.tsx, keyrx_ui_v2/src/components/DashboardEventTimeline.test.tsx
  - Test all dashboard components
  - Mock WebSocket and subscriptions
  - Verify real-time updates
  - Purpose: Ensure dashboard reliability
  - _Leverage: @testing-library/react_
  - _Requirements: REQ-3 (AC1-AC10)_
  - _Prompt: Role: QA Engineer with expertise in React component testing and real-time systems | Task: Create comprehensive tests for all dashboard components, DashboardPage.test.tsx: mock useUnifiedApi, verify connects/subscribes on mount, unsubscribes on unmount, connection banner shows status, child components rendered with correct props; StateIndicatorPanel.test.tsx: verify badges render for modifiers/locks/layer with correct colors, "None" shown when empty; MetricsChart.test.tsx: verify lines and reference line render with mock data; DashboardEventTimeline.test.tsx: verify events render, pause/resume works, FIFO limit enforced, tooltip appears on hover | Restrictions: Must mock useUnifiedApi hook, use @testing-library/react best practices, test user-facing behavior, all REQ-3 acceptance criteria must be verified | Success: All tests pass, coverage >= 90%, subscriptions verified, real-time updates tested, FIFO limits enforced, all acceptance criteria verified_

## Phase 4: Tab-Based Configuration Editor

- [x] 23. Update ConfigPage with Tab Switching
  - File: keyrx_ui_v2/src/pages/ConfigPage.tsx
  - Add tab state (visual/code) with visual as default
  - Render tab buttons with active styling
  - Conditionally render KeyboardVisualizer or MonacoEditor
  - Share configCode state between tabs
  - Implement save with validation check and Ctrl+S shortcut
  - Purpose: Unified configuration editor with dual modes
  - _Leverage: MonacoEditor from task 15, existing KeyboardVisualizer, RpcClient_
  - _Requirements: REQ-4 (AC1, AC2, AC3, AC4, AC5, AC6, AC7, AC8, AC9, AC10)_
  - _Prompt: Role: React Developer with expertise in complex UI patterns and keyboard shortcuts | Task: Modify keyrx_ui_v2/src/pages/ConfigPage.tsx to add activeTab: 'visual' | 'code' state (default 'visual'), render tab buttons above editor with conditional styling (bg-primary-500 text-white when active, text-slate-400 hover when inactive), conditionally render KeyboardVisualizer when activeTab === 'visual' or MonacoEditor when 'code', both use same configCode state, track validationErrors from Monaco onValidate, implement handleSave that checks errors.length === 0 before calling rpc.updateConfig, add keyboard listener for Ctrl+S / Cmd+S to call handleSave, render validation status panel showing error count or success message | Restrictions: Both editors must share same configCode state (single source of truth), tab switching must not lose unsaved changes, save must be blocked if validation errors exist, keyboard shortcut must work in both tabs | Success: Visual tab active by default, clicking Code tab renders Monaco, clicking Visual renders KeyboardVisualizer, active tab has correct styling, changes persist across tabs, validation status shows in both tabs, save works from both tabs, Ctrl+S triggers save, validation errors prevent save_

- [x] 24. Write ConfigPage Tab Tests
  - File: keyrx_ui_v2/src/pages/ConfigPage.test.tsx
  - Test tab switching and state persistence
  - Test validation integration
  - Test save functionality
  - Purpose: Verify tab-based editor works correctly
  - _Leverage: @testing-library/react, userEvent_
  - _Requirements: REQ-4 (AC1-AC10)_
  - _Prompt: Role: Frontend Testing Engineer with expertise in user interaction testing | Task: Create comprehensive tests in keyrx_ui_v2/src/pages/ConfigPage.test.tsx using @testing-library/react and userEvent, test scenarios: (1) Visual tab active on mount, (2) clicking Code tab renders Monaco editor, (3) clicking Visual tab renders KeyboardVisualizer, (4) active tab has bg-primary-500 class, (5) typing in Code editor then switching to Visual preserves changes (verify configCode state), (6) validation status panel appears in both tabs, (7) save button calls updateConfig RPC method, (8) Ctrl+S keyboard event triggers save, (9) validation errors disable save button, (10) switching tabs with unsaved changes doesn't revert | Restrictions: Must use userEvent for interactions, verify state not implementation, all REQ-4 acceptance criteria must be verified, mock RPC client | Success: All tests pass, coverage >= 90%, tab switching verified, state persistence tested, validation integration works, save functionality verified, all acceptance criteria met_

## Phase 5: WASM Integration

- [x] 25. Configure WASM Build in keyrx_core
  - File: keyrx_core/Cargo.toml, keyrx_core/src/lib.rs
  - Add cdylib and rlib crate types
  - Add wasm feature with wasm-bindgen dependencies
  - Optimize release profile for size
  - Export validate_config and simulate functions with #[wasm_bindgen]
  - Purpose: Enable keyrx_core compilation to WebAssembly
  - _Leverage: wasm-bindgen for JS interop_
  - _Requirements: REQ-5 (AC1, AC2, AC9, AC10)_
  - _Prompt: Role: Rust WebAssembly Developer with expertise in wasm-bindgen and optimization | Task: Modify keyrx_core/Cargo.toml to add crate-type = ["cdylib", "rlib"], add wasm feature that enables wasm-bindgen and serde-wasm-bindgen dependencies, configure [profile.release] with opt-level="z" lto=true codegen-units=1 strip=true for size optimization, modify keyrx_core/src/lib.rs to add #[wasm_bindgen] pub fn validate_config(code: &str) -> JsValue and pub fn simulate(code: &str, input: JsValue) -> JsValue when wasm feature enabled, use serde-wasm-bindgen for Rust<->JS conversion | Restrictions: Must use cdylib for WASM, release profile must optimize for size not speed, WASM functions must use JsValue for JS interop, feature gating required (only export when wasm feature enabled) | Success: Builds with wasm-pack successfully, output in pkg/ directory, WASM file size < 1MB, functions callable from JavaScript, type conversions work correctly_

- [x] 26. Configure Vite for WASM
  - File: keyrx_ui_v2/vite.config.ts, keyrx_ui_v2/package.json
  - Install and configure vite-plugin-wasm and vite-plugin-top-level-await
  - Configure code splitting for Monaco and WASM
  - Exclude WASM from optimizeDeps
  - Purpose: Enable Vite to bundle WASM modules
  - _Leverage: Vite plugins for WASM support_
  - _Requirements: REQ-5 (AC9, AC10)_
  - _Prompt: Role: Frontend Build Engineer with expertise in Vite and module bundling | Task: Install vite-plugin-wasm and vite-plugin-top-level-await via npm, modify keyrx_ui_v2/vite.config.ts to add both plugins, set build.target to 'esnext', configure build.rollupOptions.output.manualChunks for 'vendor' (react/react-dom/react-router-dom), 'monaco' (@monaco-editor/react), 'charts' (recharts), add optimizeDeps.exclude for '@/wasm/keyrx_core', update package.json with build:wasm script: "cd ../keyrx_core && wasm-pack build --target web --out-dir ../keyrx_ui_v2/src/wasm/pkg --release", update build script to run build:wasm first | Restrictions: Must use esnext target for top-level await, WASM module must be excluded from Vite optimization, code splitting configuration is mandatory | Success: WASM builds and bundles correctly, separate chunks for vendor/monaco/charts, WASM loads in browser, top-level await works, hot reload rebuilds WASM on source changes_

- [x] 27. Implement WASM Validation
  - File: keyrx_core/src/lib.rs, keyrx_core/src/wasm/validation.rs
  - Implement validate_config using keyrx_compiler
  - Return JSON array of errors with line/column/message
  - Ensure deterministic validation matching daemon
  - Purpose: Browser-based configuration validation
  - _Leverage: keyrx_compiler parser_
  - _Requirements: REQ-5 (AC2, AC3, AC4, AC5, AC6)_
  - _Prompt: Role: Rust Developer with expertise in parsing and error handling | Task: Create keyrx_core/src/wasm/validation.rs that implements validate_config(code: &str) -> String using keyrx_compiler::parse to validate Rhai code, catches parse errors and converts to JSON array format [{line: number, column: number, length: number, message: string}], returns "[]" if valid, export from lib.rs with #[wasm_bindgen] when wasm feature enabled, ensure validation logic is IDENTICAL to daemon (same parser, same error messages) for determinism | Restrictions: Must use existing keyrx_compiler::parse not custom parser, validation must be deterministic (same input always produces same output), JSON format is mandatory, field names must match TypeScript ValidationError interface | Success: Valid configs return empty array, invalid configs return errors with correct line/column numbers, validation matches daemon behavior exactly, deterministic results verified, unit tests compare WASM vs daemon validation_

- [x] 28. Implement WASM Simulation
  - File: keyrx_core/src/lib.rs, keyrx_core/src/wasm/simulator.rs
  - Implement simulate using keyrx_core DFA and state machine
  - Accept JSON input events, return JSON results
  - Use virtual clock for determinism
  - Purpose: Browser-based configuration testing
  - _Leverage: keyrx_core simulator engine_
  - _Requirements: REQ-5 (AC7, AC8)_
  - _Prompt: Role: Rust Systems Developer with expertise in simulation and determinism | Task: Create keyrx_core/src/wasm/simulator.rs that implements simulate(code: &str, input_json: &str) -> String, parses config and compiles to executable form, parses input JSON as KeyEvent array, initializes simulator state, processes each event through keyrx_core DFA (SAME engine as daemon), records state transitions and output events, uses virtual clock (not wall time) for time-independent determinism, returns JSON with {states: StateTransition[], outputs: KeyEvent[], latency: number[]}, export from lib.rs with #[wasm_bindgen] | Restrictions: Must use keyrx_core's actual DFA not a mock, virtual clock is mandatory for determinism, same input must always produce same output, results must match daemon simulation byte-for-byte | Success: Simulation processes events correctly, results are deterministic (run twice with same input produces identical output), simulation matches daemon behavior exactly, virtual clock makes tests time-independent, unit tests compare WASM vs daemon simulation_

- [x] 29. Write WASM Integration Tests
  - File: keyrx_ui_v2/tests/wasm/validation.test.ts, keyrx_ui_v2/tests/wasm/simulation.test.ts
  - Test validation and simulation functions
  - Verify determinism and daemon parity
  - Test graceful failure
  - Purpose: Ensure WASM module works correctly
  - _Leverage: Vitest, daemon for comparison_
  - _Requirements: REQ-5 (AC1-AC10)_
  - _Prompt: Role: QA Engineer with expertise in WASM testing and cross-platform verification | Task: Create keyrx_ui_v2/tests/wasm/validation.test.ts that tests: (1) WASM init succeeds, (2) validate_config detects syntax errors with correct line/column, (3) valid config returns empty array, (4) validation results match daemon (compile same config with daemon and WASM, compare errors), create keyrx_ui_v2/tests/wasm/simulation.test.ts that tests: (1) simulate processes events correctly, (2) determinism (run twice with same input, compare outputs), (3) simulation results match daemon (run same input through daemon and WASM, compare), (4) virtual clock makes tests time-independent | Restrictions: Must import WASM module directly, must run actual daemon for comparison tests, determinism must be exact (byte-for-byte comparison), all REQ-5 acceptance criteria must be verified | Success: All tests pass, validation matches daemon exactly, simulation is deterministic, simulation matches daemon behavior, graceful failure tested, coverage >= 90%_

## Phase 6: Responsive UI

- [x] 30. Implement Responsive Navigation
  - File: keyrx_ui_v2/src/components/BottomNav.tsx, keyrx_ui_v2/src/components/Sidebar.tsx, keyrx_ui_v2/src/components/Layout.tsx
  - Create BottomNav for mobile (< 768px) with icon navigation
  - Create Sidebar for desktop (>= 768px) with fixed left navigation
  - Ensure 44px minimum tap targets on mobile
  - Add ARIA labels
  - Purpose: Responsive navigation for all screen sizes
  - _Leverage: React Router NavLink, TailwindCSS breakpoints_
  - _Requirements: REQ-6 (AC1, AC2)_
  - _Prompt: Role: Frontend Developer with expertise in responsive design and accessibility | Task: Create keyrx_ui_v2/src/components/BottomNav.tsx that renders fixed bottom navigation bar with icons for Profiles/Config/Dashboard/Devices (visible only < 768px using md:hidden), create keyrx_ui_v2/src/components/Sidebar.tsx that renders fixed left sidebar with text links (visible >= 768px using hidden md:block), both use React Router NavLink for active state styling, ensure all nav items have min-h-[44px] min-w-[44px] on mobile for touch accessibility, update keyrx_ui_v2/src/components/Layout.tsx to render both conditionally, add ARIA labels and nav landmarks | Restrictions: BottomNav must be hidden on desktop, Sidebar must be hidden on mobile, minimum tap targets mandatory, NavLink must indicate active page, ARIA labels required | Success: BottomNav shows on mobile only, Sidebar shows on desktop only, active page highlighted, tap targets >= 44px on mobile, smooth transition between breakpoints, ARIA labels present, navigation accessible via keyboard_

- [x] 31. Make ConfigPage Responsive
  - File: keyrx_ui_v2/src/pages/ConfigPage.tsx
  - Apply responsive classes for single-column mobile, multi-column desktop
  - Ensure 44px tap targets on mobile
  - Make KeyboardVisualizer horizontally scrollable on mobile
  - Purpose: ConfigPage works on all screen sizes
  - _Leverage: TailwindCSS responsive utilities_
  - _Requirements: REQ-6 (AC3, AC4)_
  - _Prompt: Role: Responsive UI Developer with expertise in TailwindCSS and mobile-first design | Task: Modify keyrx_ui_v2/src/pages/ConfigPage.tsx to add responsive classes: main container flex-col gap-4 md:gap-6, tab buttons grid grid-cols-2 sm:flex sm:gap-2 (stack 2-per-row on mobile, inline on tablet+), save button w-full md:w-auto (full-width mobile), KeyboardVisualizer wrapper overflow-x-auto md:overflow-x-visible (horizontal scroll mobile), validation status text-sm md:text-base, all buttons min-h-[44px] md:min-h-0 for touch targets | Restrictions: Mobile-first approach required, 44px tap targets mandatory on mobile, horizontal scroll must work smoothly, test at 3 breakpoints (375px, 768px, 1024px) | Success: Single-column layout on mobile, multi-column on desktop, tab buttons stack on mobile, save button full-width on mobile, KeyboardVisualizer scrolls horizontally on mobile, all tap targets >= 44px on mobile, responsive at all breakpoints_

- [ ] 32. Make DashboardPage Responsive
  - File: keyrx_ui_v2/src/pages/DashboardPage.tsx
  - Stack components vertically on mobile
  - Use 2-column grid on desktop for state indicators and chart
  - Ensure 44px tap targets
  - Purpose: Dashboard works on all screen sizes
  - _Leverage: TailwindCSS responsive grid_
  - _Requirements: REQ-6 (AC5, AC6)_
  - _Prompt: Role: Responsive Dashboard Developer with expertise in data visualization layouts | Task: Modify keyrx_ui_v2/src/pages/DashboardPage.tsx layout: main container flex-col gap-4 p-4 md:p-6, StateIndicatorPanel and MetricsChart in grid grid-cols-1 lg:grid-cols-2 gap-4 (stack mobile/tablet, side-by-side desktop), connection banner text-sm md:text-base, pause/clear buttons flex flex-col sm:flex-row gap-2 (stack mobile), all buttons min-h-[44px] md:min-h-0 | Restrictions: Components must stack vertically on mobile, 2-column grid only on desktop >= 1024px, MetricsChart uses ResponsiveContainer so auto-scales, 44px tap targets mandatory | Success: Components stack vertically on mobile, 2-column grid on desktop, connection banner responsive, pause/clear buttons stack on mobile, all tap targets >= 44px, MetricsChart scales to container width_

- [ ] 33. Create Visual Regression Tests
  - File: keyrx_ui_v2/e2e/visual/responsive.spec.ts
  - Capture screenshots at 375px, 768px, 1024px viewports
  - Test all pages at all breakpoints
  - Compare against baselines
  - Purpose: Prevent visual regressions across breakpoints
  - _Leverage: Playwright screenshot testing_
  - _Requirements: REQ-6 (AC8)_
  - _Prompt: Role: Visual Testing Engineer with expertise in Playwright and regression testing | Task: Create keyrx_ui_v2/e2e/visual/responsive.spec.ts using Playwright, test all pages (ConfigPage, DashboardPage, ProfilesPage, DevicesPage) at 3 viewports (375px mobile, 768px tablet, 1024px desktop), use page.setViewportSize({width, height}) before each screenshot, capture full page with page.screenshot(), store baseline images in e2e/visual/baselines/, configure Playwright to compare screenshots with threshold tolerance, test scenarios include both tabs on ConfigPage, dashboard with events, profiles list, navigation components | Restrictions: Must test exactly 3 viewports (375/768/1024), full page screenshots required, baseline images must be committed to repo, CI must fail on visual differences | Success: Screenshots captured for all pages at all breakpoints, baselines stored in repo, CI compares and fails on differences, tests run reliably, navigation components captured (BottomNav on mobile, Sidebar on desktop)_

- [ ] 34. Configure TailwindCSS Purge
  - File: keyrx_ui_v2/tailwind.config.js
  - Configure content paths for purging
  - Define custom theme colors and breakpoints
  - Enable dark mode
  - Verify CSS bundle < 50KB
  - Purpose: Optimize TailwindCSS for production
  - _Leverage: TailwindCSS purge feature_
  - _Requirements: REQ-6 (AC9, AC10)_
  - _Prompt: Role: CSS Optimization Engineer with expertise in TailwindCSS and build optimization | Task: Modify keyrx_ui_v2/tailwind.config.js to set content paths ["./src/**/*.{js,jsx,ts,tsx}", "./index.html"] for purge scanning, configure theme extend with custom colors (primary blue shades, secondary slate shades), add custom breakpoints if needed, enable darkMode with class strategy, add safelist for dynamically generated classes if any, build production bundle and verify dist/assets/*.css size < 50KB gzipped | Restrictions: Content paths must cover all source files, unused classes must be purged, dark mode class strategy required, final CSS bundle must be < 50KB gzipped | Success: Content paths configured correctly, custom theme colors defined, dark mode enabled, production build purges unused classes, CSS bundle < 50KB gzipped, no unused styles in production build_

## Phase 7: Testing Infrastructure

- [ ] 35. Setup Unit Test Infrastructure
  - File: keyrx_ui_v2/vitest.config.ts, keyrx_ui_v2/package.json, keyrx_core/tests/
  - Configure Vitest for React testing
  - Setup coverage reporting with 80% threshold
  - Add test scripts
  - Purpose: Unit testing foundation
  - _Leverage: Vitest, @testing-library/react_
  - _Requirements: REQ-7 (AC1, AC2, AC10)_
  - _Prompt: Role: Test Infrastructure Engineer with expertise in Vitest and Jest testing frameworks | Task: Create keyrx_ui_v2/vitest.config.ts with Vitest configuration (jsdom environment for React, setup files for @testing-library/jest-dom, coverage with v8 provider, thresholds lines/functions/branches 80%, exclude node_modules/dist from coverage), add package.json scripts ("test": "vitest run", "test:watch": "vitest", "test:coverage": "vitest run --coverage"), ensure all existing components (Card, Dropdown, KeyboardVisualizer, etc.) have .test.tsx files with > 80% coverage, for Rust configure cargo test --workspace and cargo tarpaulin with > 90% threshold for keyrx_core | Restrictions: Coverage thresholds are mandatory (80% overall, 90% keyrx_core), all existing components must have tests before merge, tests must use @testing-library/react best practices | Success: Vitest configured correctly, test scripts in package.json work, all components have unit tests, coverage >= 80% overall and >= 90% for keyrx_core, tests run in CI_

- [ ] 36. Setup Integration Test Infrastructure
  - File: keyrx_ui_v2/tests/integration/, keyrx_daemon/tests/integration/
  - Create test harness to start/stop daemon
  - Write integration tests for key workflows
  - Purpose: Integration testing foundation
  - _Leverage: Vitest for UI, tokio::test for daemon_
  - _Requirements: REQ-7 (AC3, AC10)_
  - _Prompt: Role: Integration Testing Specialist with expertise in test harnesses and workflows | Task: Create keyrx_ui_v2/tests/integration/ directory with test harness that starts daemon in test mode (headless, test port) before tests and stops after, create test files: config-editor.test.tsx (tests tab switching and validation flow), dashboard-updates.test.tsx (tests WebSocket subscription and state updates), profile-workflow.test.tsx (tests create/activate/delete profile), for daemon expand keyrx_daemon/tests/integration/ with WebSocket RPC tests and event broadcasting tests, configure CI to run integration tests after unit tests | Restrictions: Test harness must start real daemon not mock, daemon must run on different port for tests, cleanup required after each test, integration tests must not interfere with each other | Success: Test harness starts/stops daemon correctly, all workflow tests pass (tab switching, validation, WebSocket updates, profile CRUD), daemon integration tests verify RPC and broadcasting, tests run reliably in CI_

- [ ] 37. Setup E2E Tests with Playwright
  - File: keyrx_ui_v2/playwright.config.ts, keyrx_ui_v2/e2e/, keyrx_ui_v2/package.json
  - Configure Playwright for Chromium, Firefox, WebKit
  - Write E2E tests for full user workflows
  - Purpose: End-to-end testing with real browser
  - _Leverage: Playwright test runner_
  - _Requirements: REQ-7 (AC4, AC10)_
  - _Prompt: Role: E2E Testing Engineer with expertise in Playwright and browser automation | Task: Install Playwright (npm install --save-dev @playwright/test), create keyrx_ui_v2/playwright.config.ts with config (baseURL http://localhost:9867, webServer starts daemon before tests, browsers chromium/firefox/webkit, screenshots/videos on failure, retries on failure), create keyrx_ui_v2/e2e/ with test files: profile-crud.spec.ts (tests creating/activating/deleting profiles), config-editor.spec.ts (tests editing and saving config with Monaco), dashboard-monitoring.spec.ts (tests real-time dashboard updates), add package.json scripts ("test:e2e": "playwright test", "test:e2e:ui": "playwright test --ui"), configure CI to run E2E tests | Restrictions: Must test in real browsers not headless only, daemon must start automatically before tests, screenshots/videos on failure required, tests must be reliable (retries allowed) | Success: Playwright configured correctly, all 3 browsers tested (chromium/firefox/webkit), full workflows tested (profile CRUD, config editing, dashboard monitoring), screenshots/videos captured on failure, E2E tests run in CI_

- [ ] 38. Setup Accessibility Tests
  - File: keyrx_ui_v2/tests/a11y/, keyrx_ui_v2/package.json
  - Configure jest-axe for accessibility testing
  - Test all pages for WCAG AA compliance
  - Test keyboard navigation
  - Purpose: Ensure accessibility compliance
  - _Leverage: jest-axe, @testing-library/user-event_
  - _Requirements: REQ-7 (AC5, AC10)_
  - _Prompt: Role: Accessibility Testing Specialist with expertise in WCAG compliance and axe-core | Task: Install jest-axe (npm install --save-dev jest-axe), create keyrx_ui_v2/tests/a11y/ with test files for each page (config-page.test.tsx, dashboard-page.test.tsx, profiles-page.test.tsx, devices-page.test.tsx), each test renders component, runs axe(container) to check violations, asserts expect(results).toHaveNoViolations(), test keyboard navigation with userEvent.tab() to verify focus order, verify ARIA labels present on interactive elements, add package.json script ("test:a11y": "vitest run tests/a11y"), configure CI to run accessibility tests and fail build on violations | Restrictions: WCAG AA compliance mandatory, all pages must have zero axe violations, keyboard navigation must be fully functional, ARIA labels required on all interactive elements | Success: jest-axe configured, all pages tested with zero violations, keyboard navigation verified (tab order correct, focus visible), ARIA labels present and correct, accessibility tests run in CI and fail on violations_

- [ ] 39. Setup Performance Tests
  - File: keyrx_ui_v2/tests/performance/, keyrx_ui_v2/.lighthouserc.js, keyrx_ui_v2/package.json
  - Test bundle sizes (main < 500KB, Monaco < 2MB, WASM < 1MB)
  - Configure Lighthouse CI
  - Set performance budgets
  - Purpose: Enforce performance requirements
  - _Leverage: Lighthouse CI, fs/zlib for bundle size_
  - _Requirements: REQ-7 (AC6, AC10)_
  - _Prompt: Role: Performance Engineer with expertise in bundle optimization and Lighthouse | Task: Create keyrx_ui_v2/tests/performance/bundle-size.test.ts that uses fs and zlib to measure gzipped bundle sizes after build, verify main bundle < 500KB, Monaco chunk < 2MB, WASM module < 1MB, install Lighthouse CI (npm install --save-dev @lhci/cli), create .lighthouserc.js config with budgets and assertions (performance >= 90, accessibility >= 90, best-practices >= 90), configure URLs to test (/, /config/Default, /dashboard, /devices), add package.json scripts ("test:perf": "vitest run tests/performance", "test:lighthouse": "lhci autorun"), configure CI to run performance tests after build | Restrictions: Bundle size limits are hard requirements (500KB/2MB/1MB gzipped), Lighthouse scores >= 90 mandatory, tests must run after production build, CI must fail if budgets exceeded | Success: Bundle size tests verify limits, Lighthouse CI configured with budgets, all scores >= 90, performance tests run in CI, build fails if budgets exceeded or scores too low_

- [ ] 40. Configure Pre-Commit Hooks
  - File: .husky/pre-commit, package.json, .git/hooks/pre-commit
  - Install Husky and lint-staged
  - Configure hooks for Rust (clippy, fmt, test) and TypeScript (prettier, eslint, vitest)
  - Purpose: Catch issues before commit
  - _Leverage: Husky for Git hooks_
  - _Requirements: REQ-7 (AC8, AC9, AC10)_
  - _Prompt: Role: DevOps Engineer with expertise in Git hooks and code quality automation | Task: Install Husky and lint-staged (npm install --save-dev husky lint-staged), run npx husky install, create .husky/pre-commit script that runs lint-staged, configure lint-staged in package.json with rules ("*.rs": ["cargo fmt --check", "cargo clippy -- -D warnings"], "*.{ts,tsx}": ["prettier --check", "eslint", "vitest related --run"]), add package.json prepare script ("prepare": "husky install"), test that commit is blocked if checks fail, add documentation for bypassing with --no-verify | Restrictions: Hooks must run automatically on git commit, cargo clippy must treat warnings as errors (-D warnings), commit must be blocked if any check fails, hooks must run only on staged files (lint-staged) | Success: Husky installed and configured, pre-commit hook runs automatically, Rust checks run (clippy, fmt, test), TypeScript checks run (prettier, eslint, vitest), commit blocked on failure, documentation explains --no-verify bypass_

## Phase 8: Build Process Integration

- [ ] 41. Create WASM Build Script
  - File: scripts/build_wasm.sh
  - Build keyrx_core to WASM with wasm-pack
  - Output to keyrx_ui_v2/src/wasm/pkg/
  - Verify output files exist
  - Purpose: Automated WASM build
  - _Leverage: wasm-pack CLI_
  - _Requirements: REQ-8 (AC1, AC4, AC7)_
  - _Prompt: Role: Build Engineer with expertise in WebAssembly tooling and bash scripting | Task: Create scripts/build_wasm.sh that checks if wasm-pack is installed (error with install instructions if not), cd to keyrx_core, runs "wasm-pack build --target web --out-dir ../keyrx_ui_v2/src/wasm/pkg --release -- --features wasm", verifies output files exist (keyrx_core_bg.wasm, keyrx_core.js, keyrx_core.d.ts), prints build time and wasm file size, exits 1 on failure with clear error message, uses bash set -e for error handling, add header comment explaining script purpose and dependencies | Restrictions: Must check wasm-pack installed, must verify output files, must use --release flag, must exit 1 on any error, script must be executable (chmod +x) | Success: Script checks dependencies, builds WASM successfully, outputs to correct directory, verifies files exist, prints build stats, fails clearly on errors, executable and documented_

- [ ] 42. Create UI Build Script
  - File: scripts/build_ui.sh
  - Run WASM build first, then npm install and npm run build
  - Verify dist/index.html exists
  - Clear previous dist/ before build
  - Purpose: Automated UI build
  - _Leverage: build_wasm.sh from task 41_
  - _Requirements: REQ-8 (AC2, AC5, AC7)_
  - _Prompt: Role: Frontend Build Engineer with expertise in npm and build automation | Task: Create scripts/build_ui.sh that calls scripts/build_wasm.sh first (exit if fails), cd to keyrx_ui_v2, checks if node_modules exists (run npm install if not), removes previous dist/ directory (rm -rf dist), runs npm run build, verifies dist/index.html exists, prints bundle size summary (total size, gzipped size using du and gzip), exits 1 on failure, uses bash set -e and set -o pipefail for error handling, make executable | Restrictions: WASM must build before UI, WASM failure must prevent UI build, dist/ must be cleared before build, dist/index.html verification mandatory, bundle size reporting required | Success: Script builds WASM then UI, clears old dist/, installs dependencies if needed, verifies output, prints bundle stats, fails clearly on errors, WASM failures prevent UI build_

- [ ] 43. Integrate UI into Daemon Build
  - File: keyrx_daemon/build.rs, keyrx_daemon/src/web/static_files.rs, keyrx_daemon/Cargo.toml
  - Verify ../keyrx_ui_v2/dist exists in build.rs
  - Embed UI with include_dir macro
  - Add unit test verifying index.html embedded
  - Purpose: Embed UI into daemon binary
  - _Leverage: include_dir crate_
  - _Requirements: REQ-8 (AC3, AC6, AC9)_
  - _Prompt: Role: Rust Build Systems Developer with expertise in cargo build scripts and embedding | Task: Modify keyrx_daemon/build.rs to check if ../keyrx_ui_v2/dist directory exists, if not print error "UI directory not found. Run 'scripts/build_ui.sh' first" and panic!, add println!("cargo:rerun-if-changed=../keyrx_ui_v2/dist"), add include_dir to Cargo.toml dependencies, verify keyrx_daemon/src/web/static_files.rs correctly embeds UI_DIR with include_dir!("$CARGO_MANIFEST_DIR/../keyrx_ui_v2/dist") and serves files with SPA fallback to index.html, add unit test in static_files.rs that asserts UI_DIR.get_file("index.html").is_some() | Restrictions: build.rs must fail with clear error if dist missing, must trigger rebuild if UI changes, include_dir macro required for embedding, unit test verification mandatory | Success: build.rs verifies dist exists, fails with helpful error if missing, rebuilds on UI changes, UI_DIR embedded correctly, serves files including SPA routing, unit test verifies index.html present, daemon binary serves UI correctly_

- [ ] 44. Create Master Build Script
  - File: scripts/build.sh, Makefile
  - Build in sequence: WASM → UI → daemon
  - Print summary with build times and sizes
  - Add Makefile targets (build, clean)
  - Purpose: One-command build for entire project
  - _Leverage: build_wasm.sh, build_ui.sh from tasks 41-42_
  - _Requirements: REQ-8 (AC2, AC7, AC10)_
  - _Prompt: Role: Build Automation Engineer with expertise in build orchestration and Makefiles | Task: Modify scripts/build.sh to call scripts/build_wasm.sh, scripts/build_ui.sh, then cargo build --release -p keyrx_daemon, verify output binaries exist in target/release/, print summary with build times and file sizes, use bash set -e, add colors for success/error (green/red), update Makefile with build target calling scripts/build.sh and clean target removing target/, keyrx_ui_v2/dist/, keyrx_ui_v2/src/wasm/pkg/, test make build produces working binaries | Restrictions: Build sequence must be WASM → UI → daemon (order matters), any step failure must stop build, summary must show times and sizes, Makefile targets required, colored output for readability | Success: Master script builds entire project, correct sequence enforced, failures stop build, summary printed, Makefile targets work, make build produces working binaries, make clean removes all artifacts_

- [ ] 45. Create Verification Script
  - File: scripts/verify.sh, Makefile
  - Run all quality checks: fmt, clippy, test, coverage
  - Print summary table of results
  - Add Makefile verify target
  - Purpose: One-command verification for CI and pre-commit
  - _Leverage: cargo fmt, clippy, test, tarpaulin, npm test_
  - _Requirements: REQ-8 (AC8, AC10)_
  - _Prompt: Role: CI/CD Engineer with expertise in quality gates and test automation | Task: Modify scripts/verify.sh to run checks in order: (1) cargo fmt --check all Rust code, (2) cargo clippy --workspace -- -D warnings, (3) cargo test --workspace, (4) cargo tarpaulin --workspace --out Stdout --exclude-files 'keyrx_ui*/*' with >= 80% coverage, (5) cd keyrx_ui_v2 and npm test -- --coverage with >= 80%, (6) npm run test:e2e for Playwright, use colored output (green checkmark pass, red X fail), print summary table showing which checks passed/failed, exit 1 if any check fails, update Makefile with verify target, test script catches violations | Restrictions: All checks must run even if earlier ones fail (collect all results), coverage thresholds mandatory (80% overall, 90% keyrx_core), summary table required, exit 1 only after all checks complete | Success: Script runs all checks, collects all results, prints summary table, exits 1 if any fail, colored output works, Makefile verify target runs script, script catches clippy/fmt/test/coverage violations_

- [ ] 46. Configure CI/CD Pipeline
  - File: .github/workflows/ci.yml, .github/workflows/release.yml
  - Setup CI workflow (build, verify, upload coverage)
  - Setup release workflow (multi-platform builds, GitHub release)
  - Cache dependencies (cargo, npm)
  - Purpose: Automated testing and deployment
  - _Leverage: GitHub Actions_
  - _Requirements: REQ-8 (AC10)_
  - _Prompt: Role: DevOps Engineer with expertise in GitHub Actions and CI/CD pipelines | Task: Modify .github/workflows/ci.yml to: checkout code, setup Rust stable + wasm32-unknown-unknown target, setup Node.js 18, install wasm-pack, cache cargo and npm dependencies, run scripts/build.sh, run scripts/verify.sh, upload coverage reports to artifact, fail build if any step fails; modify .github/workflows/release.yml (triggered on tag push) to: build for Linux and Windows targets, create release artifacts (binaries, UI bundle), create GitHub release with changelog, upload artifacts to release, test CI by pushing to branch | Restrictions: CI must run on push and pull_request, dependencies must be cached for speed, all steps must be logged, coverage upload required, release only on tag push, multi-platform builds mandatory | Success: CI workflow runs on push/PR, builds WASM → UI → daemon, runs all verifications, uploads coverage, caches work correctly, release workflow builds for Linux/Windows, creates GitHub release with binaries, all steps logged, failures stop pipeline_

- [ ] 47. Migrate keyrx_ui_v2 to keyrx_ui
  - File: keyrx_ui/ (remove old), keyrx_ui_v2/ (rename to keyrx_ui), scripts/, keyrx_daemon/build.rs, Cargo.toml, .github/workflows/
  - Remove old keyrx_ui directory completely
  - Rename keyrx_ui_v2 to keyrx_ui
  - Update all references in build scripts, Cargo.toml, daemon build.rs, CI workflows
  - Purpose: Finalize migration by removing temporary _v2 suffix
  - _Leverage: git mv for rename with history preservation_
  - _Requirements: All (final cleanup)_
  - _Prompt: Role: DevOps Engineer with expertise in repository migrations and git workflows | Task: Remove old keyrx_ui directory (rm -rf keyrx_ui after backing up any needed files like Monaco integration code), rename keyrx_ui_v2 to keyrx_ui using git mv to preserve history, update all references: scripts/build_wasm.sh (change ../keyrx_ui_v2/src/wasm/pkg to ../keyrx_ui/src/wasm/pkg), scripts/build_ui.sh (cd keyrx_ui instead of keyrx_ui_v2), keyrx_daemon/build.rs (../keyrx_ui/dist instead of ../keyrx_ui_v2/dist), keyrx_daemon/src/web/static_files.rs (update include_dir path), CI workflows (.github/workflows/ update paths), verify build works after migration | Restrictions: Must use git mv not regular mv to preserve git history, must verify old keyrx_ui has no needed code before deletion, all build scripts must be updated, must test complete build after migration | Success: Old keyrx_ui removed, keyrx_ui_v2 renamed to keyrx_ui with git history preserved, all build scripts updated and working, daemon embeds from keyrx_ui/dist, CI passes, no references to keyrx_ui_v2 remain in codebase_

## Summary

**Total Tasks**: 47 atomic implementation tasks across 8 phases (+ final migration)

**Task Status Tracking**:
- Use `- [ ]` for pending tasks
- Use `- [-]` for in-progress tasks (mark ONE at a time)
- Use `- [x]` for completed tasks

**Critical Dependencies**:
- Tasks 1-9 (RPC Backend) must complete before 10-14 (RPC Frontend)
- Task 15 (Monaco) depends on 16 (WASM hook)
- Tasks 18-22 (Dashboard) depend on 10-14 (RPC Client)
- Task 23 (Config Tabs) depends on 15 (Monaco)
- Tasks 25-29 (WASM) can run in parallel with Phase 1
- Tasks 30-34 (Responsive) depend on components existing
- Tasks 35-40 (Testing) run throughout all phases
- Tasks 41-46 (Build) support all phases
- Task 47 (Migration) is the FINAL task after everything works

**Estimated Timeline**:
- Phase 1: 2 weeks (RPC API)
- Phase 2: 1 week (Monaco)
- Phase 3: 1 week (Dashboard)
- Phase 4: 0.5 weeks (Tabs)
- Phase 5: 1 week (WASM)
- Phase 6: 1 week (Responsive)
- Phase 7: 1.5 weeks (Testing)
- Phase 8: 1 week (Build + Migration)

**Total**: 9-10 weeks with 1 developer, or ~6 weeks with parallel development

**When to Remove keyrx_ui_v2**:
- Task 47 is the LAST task after all others complete
- Do NOT remove keyrx_ui_v2 until:
  - All 46 implementation tasks are done
  - All tests pass (unit, integration, E2E, visual, accessibility, performance)
  - CI/CD pipeline is green
  - Production build works
- Then execute Task 47 to finalize the migration
