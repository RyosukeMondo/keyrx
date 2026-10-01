import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { StatusChip } from './StatusChip';
import * as statusApi from '../api/status';

function renderChip() {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return render(
    <QueryClientProvider client={client}>
      <StatusChip />
    </QueryClientProvider>
  );
}

const status = {
  status: 'running',
  daemon_running: true,
  version: '1.1.0',
  uptime_secs: 5,
  active_profile: 'work',
  device_count: 1,
};

describe('StatusChip', () => {
  beforeEach(() => vi.restoreAllMocks());

  it('shows the active profile and grabbed keyboards', async () => {
    vi.spyOn(statusApi, 'fetchDaemonStatus').mockResolvedValue(status);
    renderChip();
    expect(await screen.findByRole('status')).toHaveTextContent(
      'Active: work · 1 keyboard grabbed'
    );
  });

  it('pluralizes and handles no active profile', async () => {
    vi.spyOn(statusApi, 'fetchDaemonStatus').mockResolvedValue({
      ...status,
      active_profile: null,
      device_count: 0,
    });
    renderChip();
    expect(await screen.findByRole('status')).toHaveTextContent(
      'No active profile · 0 keyboards grabbed'
    );
  });

  it('reports the daemon as offline when the request fails', async () => {
    vi.spyOn(statusApi, 'fetchDaemonStatus').mockRejectedValue(
      new Error('down')
    );
    renderChip();
    expect(await screen.findByRole('status')).toHaveTextContent(
      'Daemon offline'
    );
  });
});
