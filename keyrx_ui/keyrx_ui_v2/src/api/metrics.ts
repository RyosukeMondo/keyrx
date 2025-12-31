/**
 * Metrics and monitoring API client
 */

import { apiClient } from './client';
import type { LatencyStats, EventRecord, DaemonState } from '../types';

/**
 * Fetch latency statistics
 */
export async function fetchLatencyStats(): Promise<LatencyStats> {
  return apiClient.get<LatencyStats>('/api/metrics/latency');
}

/**
 * Fetch event log
 */
export async function fetchEventLog(): Promise<EventRecord[]> {
  return apiClient.get<EventRecord[]>('/api/metrics/events');
}

/**
 * Fetch current daemon state
 */
export async function fetchDaemonState(): Promise<DaemonState> {
  return apiClient.get<DaemonState>('/api/state');
}

/**
 * Clear event log
 */
export async function clearEventLog(): Promise<{ success: boolean }> {
  return apiClient.delete<{ success: boolean }>('/api/metrics/events');
}

/**
 * Get system health status
 */
export async function fetchHealthStatus(): Promise<{
  healthy: boolean;
  uptime: number;
  version: string;
}> {
  return apiClient.get<{
    healthy: boolean;
    uptime: number;
    version: string;
  }>('/api/health');
}
