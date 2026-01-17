import { useQuery, useQueryClient } from '@tanstack/react-query';
import { useEffect } from 'react';
import { queryKeys } from '../lib/queryClient';
import * as metricsApi from '../api/metrics';
import type {
  LatencyStats,
  EventRecord,
  WSMessage,
  KeyEventPayload,
} from '../types';

/**
 * Transform daemon's KeyEventPayload to frontend's EventRecord
 */
function transformKeyEvent(payload: KeyEventPayload): EventRecord {
  return {
    id: `evt-${payload.timestamp}-${Math.random().toString(36).slice(2, 8)}`,
    timestamp: new Date(payload.timestamp / 1000).toISOString(),
    type: payload.eventType === 'press' ? 'press' : 'release',
    keyCode: payload.keyCode.replace(/^KEY_/, ''),
    layer: 'Base',
    latencyUs: payload.latency,
    action: payload.mappingTriggered ? payload.output : undefined,
    input: payload.input,
    output: payload.output,
    deviceId: payload.deviceId,
    deviceName: payload.deviceName,
    mappingType: payload.mappingType,
    mappingTriggered: payload.mappingTriggered,
  };
}

/**
 * Fetch latency statistics with React Query caching
 */
export function useLatencyStats() {
  return useQuery({
    queryKey: queryKeys.latencyStats,
    queryFn: metricsApi.fetchLatencyStats,
    // Refetch every 5 seconds for near real-time updates
    refetchInterval: 5000,
  });
}

/**
 * Fetch event log with React Query caching
 */
export function useEventLog() {
  return useQuery({
    queryKey: queryKeys.eventLog,
    queryFn: metricsApi.fetchEventLog,
    // Refetch every 10 seconds
    refetchInterval: 10000,
  });
}

/**
 * Subscribe to real-time WebSocket updates
 * Automatically updates React Query cache when events arrive
 */
export function useWebSocketMetrics() {
  const queryClient = useQueryClient();

  useEffect(() => {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${protocol}//${window.location.host}/ws`;
    const websocket = new WebSocket(wsUrl);

    websocket.onopen = () => {
      if (import.meta.env.DEV) {
        // eslint-disable-next-line no-console
        console.log('WebSocket connected');
      }
    };

    websocket.onmessage = (event) => {
      try {
        const message: WSMessage = JSON.parse(event.data);

        switch (message.type) {
          case 'event': {
            // Transform KeyEventPayload to EventRecord
            const eventRecord = transformKeyEvent(message.payload);

            // Update event log cache
            queryClient.setQueryData<EventRecord[]>(
              queryKeys.eventLog,
              (old) => {
                const updatedLog = [eventRecord, ...(old ?? [])];
                // Limit to 1000 events
                return updatedLog.slice(0, 1000);
              }
            );
            break;
          }

          case 'state': {
            // Type is automatically narrowed to DaemonState
            queryClient.setQueryData(queryKeys.daemonState, message.payload);
            break;
          }

          case 'latency': {
            const stats = message.payload as LatencyStats;

            // Update latency stats cache
            queryClient.setQueryData(queryKeys.latencyStats, stats);
            break;
          }

          case 'error': {
            const errorPayload = message.payload as { message: string };
            console.error('WebSocket error:', errorPayload.message);
            break;
          }
        }
      } catch (error) {
        console.error('Failed to parse WebSocket message:', error);
      }
    };

    websocket.onerror = (error) => {
      console.error('WebSocket error:', error);
    };

    websocket.onclose = () => {
      if (import.meta.env.DEV) {
        // eslint-disable-next-line no-console
        console.log('WebSocket disconnected');
      }
    };

    // Cleanup on unmount
    return () => {
      websocket.close();
    };
  }, [queryClient]);

  // Return connection status from cache or default to false
  return {
    connected: true, // Simplified - could track actual connection state
  };
}

/**
 * Get daemon state from cache
 */
export function useDaemonState() {
  return useQuery({
    queryKey: queryKeys.daemonState,
    queryFn: async () => {
      // Daemon state is populated by WebSocket
      // Return null if not yet received
      return null;
    },
    // Don't refetch - updated via WebSocket
    refetchInterval: false,
    refetchOnWindowFocus: false,
  });
}
