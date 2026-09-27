/**
 * Metrics and monitoring API client
 */

import { apiClient } from './client';
import { toEventRecord } from './eventRecord';
import { API_ENDPOINTS } from '../config/constants';
import type {
  LatencyStats,
  EventRecord,
  DaemonState,
  KeyEventData,
  ClearEventsResult,
} from '../types';

/**
 * Fetch latency statistics
 */
export async function fetchLatencyStats(): Promise<LatencyStats> {
  return apiClient.get<LatencyStats>(API_ENDPOINTS.metricsLatency);
}

/**
 * Fetch the event log.
 *
 * The REST endpoint returns KeyEventData[] oldest-first; this maps each entry
 * to the frontend's EventRecord and reverses it to newest-first, matching the
 * ordering used everywhere else in the UI.
 */
export async function fetchEventLog(): Promise<EventRecord[]> {
  const events = await apiClient.get<KeyEventData[]>(
    API_ENDPOINTS.metricsEvents
  );
  return events.map((event) => toEventRecord(event)).reverse();
}

/**
 * Fetch current daemon state
 */
export async function fetchDaemonState(): Promise<DaemonState> {
  return apiClient.get<DaemonState>(API_ENDPOINTS.daemonState);
}

/**
 * Clear event log
 */
export async function clearEventLog(): Promise<ClearEventsResult> {
  return apiClient.delete<ClearEventsResult>(API_ENDPOINTS.metricsEventsClear);
}
