import { describe, it, expect, beforeEach } from 'vitest';
import { useMetricsStore } from './metricsStore';

/**
 * fetchMetrics seeds the Monitor page from the REST read model. The MSW
 * handlers serve the contract fixtures (real daemon responses), so this also
 * exercises the UI's handling of the actual wire shapes.
 */
describe('metricsStore.fetchMetrics', () => {
  beforeEach(() => {
    useMetricsStore.setState({
      latencyStats: null,
      eventLog: [],
      currentState: null,
      error: null,
    });
  });

  it('loads latency, event history (newest first) and daemon state', async () => {
    await useMetricsStore.getState().fetchMetrics();
    const { latencyStats, eventLog, currentState, error } =
      useMetricsStore.getState();

    expect(error).toBeNull();
    expect(latencyStats).toMatchObject({ min: 30, max: 200, samples: 5 });
    expect(eventLog.map((e) => `${e.type} ${e.input}`)).toEqual([
      'press A',
      'release CapsLock',
      'press CapsLock',
    ]);
    expect(eventLog[1]).toMatchObject({
      output: 'Escape',
      mappingTriggered: true,
    });
    expect(currentState).toEqual({
      modifiers: ['MD_00', 'MD_0A'],
      locks: ['LK_01'],
      layer: 'MD_0A',
      activeProfile: 'default',
    });
  });
});
