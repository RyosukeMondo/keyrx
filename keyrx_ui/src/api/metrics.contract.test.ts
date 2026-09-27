/**
 * Contract tests for the monitoring REST endpoints.
 *
 * These pin the frontend's api/metrics.ts functions against the real response
 * bodies captured in src/test/contract/*.json (regenerated from a Rust test
 * against the actual daemon handlers — see keyrx_daemon/tests/api_contract_test.rs).
 *
 * The compile-time half of the contract (fixtures `satisfies` the generated
 * types) lives in src/types/contract.check.ts, which `npm run type-check`
 * covers; this file checks the runtime behaviour of the API functions.
 */

import { describe, it, expect } from 'vitest';
import { http, HttpResponse } from 'msw';
import { server } from '../test/mocks/server';
import {
  fetchLatencyStats,
  fetchEventLog,
  fetchDaemonState,
  clearEventLog,
} from './metrics';
import type {
  LatencyStats,
  KeyEventData,
  ClearEventsResult,
  DaemonState,
} from '../types';

import latencyFixture from '../test/contract/metrics_latency.json';
import eventsFixture from '../test/contract/metrics_events.json';
import clearEventsFixture from '../test/contract/metrics_events_clear.json';
import daemonStateFixture from '../test/contract/daemon_state.json';

describe('metrics API contract', () => {
  describe('fixtures satisfy the generated types', () => {
    it('metrics_latency.json satisfies LatencyStats', () => {
      const typed: LatencyStats = latencyFixture;
      expect(typed.p50).toBeTypeOf('number');
      expect(typed.samples).toBeTypeOf('number');
    });

    it('metrics_events.json satisfies KeyEventData[]', () => {
      const typed: KeyEventData[] = eventsFixture;
      expect(typed.length).toBeGreaterThan(0);
      expect(typed[0].keyCode).toBeTypeOf('string');
      expect(typed[0].mappingTriggered).toBeTypeOf('boolean');
    });

    it('metrics_events_clear.json satisfies ClearEventsResult', () => {
      const typed: ClearEventsResult = clearEventsFixture;
      expect(typed.cleared).toBeTypeOf('number');
    });

    it('daemon_state.json satisfies DaemonState', () => {
      const typed: DaemonState = daemonStateFixture;
      expect(Array.isArray(typed.modifiers)).toBe(true);
      expect(Array.isArray(typed.locks)).toBe(true);
    });
  });

  describe('fetchLatencyStats', () => {
    it('GET /api/metrics/latency returns the fixture, fields mapped 1:1', async () => {
      const stats = await fetchLatencyStats();
      expect(stats).toEqual(latencyFixture);
    });
  });

  describe('fetchEventLog', () => {
    it('GET /api/metrics/events returns the fixture reversed to newest-first', async () => {
      const records = await fetchEventLog();
      const fixtureOldestFirst = eventsFixture as KeyEventData[];

      expect(records).toHaveLength(fixtureOldestFirst.length);

      // The fixture is oldest-first; the UI's event log is newest-first
      // everywhere, so fetchEventLog must reverse it.
      const expectedNewestFirst = [...fixtureOldestFirst].reverse();
      records.forEach((record, i) => {
        const source = expectedNewestFirst[i];
        expect(record.keyCode).toBe(source.keyCode.replace(/^KEY_/, ''));
        expect(record.input).toBe(source.input);
        expect(record.output).toBe(source.output);
        expect(record.latencyUs).toBe(source.latency);
        expect(record.deviceId).toBe(source.deviceId);
        expect(record.deviceName).toBe(source.deviceName);
        expect(record.mappingType).toBe(source.mappingType);
        expect(record.mappingTriggered).toBe(source.mappingTriggered);
      });

      // Newest-first: timestamps must be non-increasing across the array.
      for (let i = 1; i < records.length; i++) {
        const prev = new Date(records[i - 1].timestamp).getTime();
        const curr = new Date(records[i].timestamp).getTime();
        expect(prev).toBeGreaterThanOrEqual(curr);
      }
    });
  });

  describe('fetchDaemonState', () => {
    it('calls GET /api/daemon/state and returns the fixture', async () => {
      let requestedUrl = '';
      server.use(
        http.get('/api/daemon/state', ({ request }) => {
          requestedUrl = request.url;
          return HttpResponse.json(daemonStateFixture);
        })
      );

      const state = await fetchDaemonState();

      expect(new URL(requestedUrl).pathname).toBe('/api/daemon/state');
      expect(state).toEqual(daemonStateFixture);
    });
  });

  describe('clearEventLog', () => {
    it('DELETE /api/metrics/events returns the ClearEventsResult fixture', async () => {
      const result = await clearEventLog();
      expect(result).toEqual(clearEventsFixture);
    });
  });
});
