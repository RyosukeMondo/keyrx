/**
 * Daemon status API client (`GET /api/status`).
 */

import { apiClient } from './client';
import { API_ENDPOINTS } from '../config/constants';

export interface DaemonStatus {
  status: string;
  daemon_running: boolean;
  version: string;
  uptime_secs: number;
  active_profile: string | null;
  /** Keyboards currently grabbed by the daemon. */
  device_count: number;
}

export function fetchDaemonStatus(): Promise<DaemonStatus> {
  return apiClient.get<DaemonStatus>(API_ENDPOINTS.status);
}
