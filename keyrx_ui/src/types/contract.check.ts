/**
 * Compile-time UI ↔ API contract (checked by `npm run type-check`).
 *
 * The JSON files are real REST responses pinned by
 * keyrx_daemon/tests/api_contract_test.rs. If the daemon's response shape and
 * the typeshare-generated types drift apart, these `satisfies` checks fail.
 * Nothing imports this module at runtime.
 */
import latency from '../test/contract/metrics_latency.json';
import events from '../test/contract/metrics_events.json';
import clearEvents from '../test/contract/metrics_events_clear.json';
import daemonState from '../test/contract/daemon_state.json';
import type {
  ClearEventsResult,
  DaemonState,
  KeyEventData,
  LatencyStats,
} from './generated';

export const contractFixtures = {
  latency: latency satisfies LatencyStats,
  events: events satisfies KeyEventData[],
  clearEvents: clearEvents satisfies ClearEventsResult,
  daemonState: daemonState satisfies DaemonState,
};
