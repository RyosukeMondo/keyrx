import { describe, it, expect } from 'vitest';
import { render } from '@testing-library/react';
import { axe } from 'vitest-axe';
import {
  EventLogList,
  type EventLogEntry,
} from '../../src/components/metrics/EventLogList';
import { StateSnapshot } from '../../src/components/metrics/StateSnapshot';
import { MetricsStatsCards } from '../../src/components/metrics/MetricsStatsCards';

// The Monitor event log is a virtualised table; row/cell roles must nest
// correctly (axe aria-required-children / aria-required-parent were critical).
const events: EventLogEntry[] = [
  {
    id: '1',
    timestamp: 1_700_000_000_000,
    type: 'press',
    keyCode: 'KEY_A',
    input: 'A',
    output: 'B',
    deviceName: 'Test Keyboard',
    latency: 0.03,
    mappingTriggered: true,
  },
  {
    id: '2',
    timestamp: 1_700_000_000_100,
    type: 'release',
    keyCode: 'KEY_A',
    input: 'A',
    output: 'A',
    deviceName: 'Test Keyboard',
    latency: 0.02,
  },
];

describe('Monitor accessibility', () => {
  it('event log table has valid row/cell structure', async () => {
    const { container, getAllByRole } = render(
      <EventLogList events={events} height={200} />
    );
    // rows exist and every cell sits directly in a row
    expect(getAllByRole('row').length).toBeGreaterThan(1);
    container.querySelectorAll('[role="cell"]').forEach((cell) => {
      expect(cell.closest('[role="row"]')).toBe(cell.parentElement);
    });
    expect(await axe(container)).toHaveNoViolations();
  });

  it('event rows expose a labelled expander button, not an interactive row', async () => {
    const { container } = render(<EventLogList events={events} height={200} />);
    const expander = container.querySelector('button[aria-expanded]');
    expect(expander).toBeTruthy();
    expect(container.querySelector('[role="row"][aria-expanded]')).toBeNull();
    expect(expander?.getAttribute('aria-label')).toMatch(/Details/);
  });

  it('stats cards and state snapshot have no violations', async () => {
    const { container } = render(
      <>
        <MetricsStatsCards
          latencyStats={
            { min: 20, avg: 30, max: 40, p95: 39, p99: 40, count: 2 } as never
          }
          eventCount={2}
          connected
        />
        <StateSnapshot
          state={{
            activeLayer: 'Base',
            modifiers: [],
            locks: [],
            tapHoldTimers: 0,
            queuedEvents: 0,
          }}
        />
      </>
    );
    expect(await axe(container)).toHaveNoViolations();
  });
});
