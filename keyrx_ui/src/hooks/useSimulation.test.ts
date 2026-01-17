/**
 * Tests for useSimulation hook
 */

import { renderHook, act, waitFor } from '@testing-library/react';
import { vi, describe, it, expect, beforeEach, afterEach } from 'vitest';
import { useSimulation } from './useSimulation';
import { useUnifiedApi } from './useUnifiedApi';
import type { KeyEvent } from '../types/rpc';

// Mock useUnifiedApi
vi.mock('./useUnifiedApi');

const mockUseUnifiedApi = useUnifiedApi as ReturnType<typeof vi.fn>;

describe('useSimulation', () => {
  let mockSubscribe: ReturnType<typeof vi.fn>;
  let mockUnsubscribe: ReturnType<typeof vi.fn>;
  let eventHandlers: Map<string, (data: unknown) => void>;

  beforeEach(() => {
    eventHandlers = new Map();
    mockUnsubscribe = vi.fn();

    mockSubscribe = vi.fn((channel, handler) => {
      eventHandlers.set(channel, handler);
      return mockUnsubscribe;
    });

    mockUseUnifiedApi.mockReturnValue({
      subscribe: mockSubscribe,
      query: vi.fn(),
      command: vi.fn(),
      unsubscribe: vi.fn(),
      readyState: 1,
      isConnected: true,
      lastError: null,
    });
  });

  afterEach(() => {
    vi.clearAllMocks();
    eventHandlers.clear();
  });

  describe('initialization', () => {
    it('initializes with empty events and stopped state', () => {
      const { result } = renderHook(() => useSimulation());

      expect(result.current.events).toEqual([]);
      expect(result.current.isRunning).toBe(false);
      expect(result.current.statistics.total).toBe(0);
    });

    it('auto-starts when autoStart is true', () => {
      const { result } = renderHook(() => useSimulation({ autoStart: true }));

      expect(result.current.isRunning).toBe(true);
    });

    it('does not auto-start when autoStart is false', () => {
      const { result } = renderHook(() => useSimulation({ autoStart: false }));

      expect(result.current.isRunning).toBe(false);
    });

    it('respects maxEvents option', () => {
      const { result } = renderHook(() => useSimulation({ maxEvents: 5 }));

      const mockEvent: KeyEvent = {
        timestamp: Date.now() * 1000,
        keyCode: 'KEY_A',
        eventType: 'press',
        input: 'A',
        output: 'A',
        latency: 100,
      };

      // Add 10 events
      act(() => {
        for (let i = 0; i < 10; i++) {
          result.current.addEvent({ ...mockEvent, timestamp: i });
        }
      });

      // Should only keep 5 events (most recent)
      expect(result.current.events).toHaveLength(5);
    });
  });

  describe('event management', () => {
    it('adds events correctly', () => {
      const { result } = renderHook(() => useSimulation());

      const event1: KeyEvent = {
        timestamp: 1000,
        keyCode: 'KEY_A',
        eventType: 'press',
        input: 'A',
        output: 'A',
        latency: 100,
      };

      act(() => {
        result.current.addEvent(event1);
      });

      expect(result.current.events).toHaveLength(1);
      expect(result.current.events[0]).toEqual(event1);
    });

    it('maintains FIFO order when max events exceeded', () => {
      const { result } = renderHook(() => useSimulation({ maxEvents: 3 }));

      act(() => {
        result.current.addEvent({
          timestamp: 1,
          keyCode: 'KEY_A',
          eventType: 'press',
          input: 'A',
          output: 'A',
          latency: 100,
        });
        result.current.addEvent({
          timestamp: 2,
          keyCode: 'KEY_B',
          eventType: 'press',
          input: 'B',
          output: 'B',
          latency: 100,
        });
        result.current.addEvent({
          timestamp: 3,
          keyCode: 'KEY_C',
          eventType: 'press',
          input: 'C',
          output: 'C',
          latency: 100,
        });
        result.current.addEvent({
          timestamp: 4,
          keyCode: 'KEY_D',
          eventType: 'press',
          input: 'D',
          output: 'D',
          latency: 100,
        });
      });

      expect(result.current.events).toHaveLength(3);
      // Newest first (4, 3, 2)
      expect(result.current.events[0].timestamp).toBe(4);
      expect(result.current.events[1].timestamp).toBe(3);
      expect(result.current.events[2].timestamp).toBe(2);
    });

    it('clears all events', () => {
      const { result } = renderHook(() => useSimulation());

      act(() => {
        result.current.addEvent({
          timestamp: 1,
          keyCode: 'KEY_A',
          eventType: 'press',
          input: 'A',
          output: 'A',
          latency: 100,
        });
        result.current.addEvent({
          timestamp: 2,
          keyCode: 'KEY_B',
          eventType: 'press',
          input: 'B',
          output: 'B',
          latency: 100,
        });
      });

      expect(result.current.events).toHaveLength(2);

      act(() => {
        result.current.clearEvents();
      });

      expect(result.current.events).toHaveLength(0);
    });
  });

  describe('simulation control', () => {
    it('starts simulation', () => {
      const { result } = renderHook(() => useSimulation());

      expect(result.current.isRunning).toBe(false);

      act(() => {
        result.current.start();
      });

      expect(result.current.isRunning).toBe(true);
    });

    it('stops simulation', () => {
      const { result } = renderHook(() => useSimulation({ autoStart: true }));

      expect(result.current.isRunning).toBe(true);

      act(() => {
        result.current.stop();
      });

      expect(result.current.isRunning).toBe(false);
    });

    it('handles rapid start/stop transitions', () => {
      const { result } = renderHook(() => useSimulation());

      act(() => {
        result.current.start();
        result.current.stop();
        result.current.start();
        result.current.stop();
        result.current.start();
      });

      expect(result.current.isRunning).toBe(true);
    });
  });

  describe('WebSocket subscription', () => {
    it('subscribes to events channel when started', async () => {
      const { result } = renderHook(() => useSimulation());

      act(() => {
        result.current.start();
      });

      await waitFor(() => {
        expect(mockSubscribe).toHaveBeenCalledWith(
          'events',
          expect.any(Function)
        );
      });
    });

    it('does not subscribe when stopped', () => {
      renderHook(() => useSimulation());

      expect(mockSubscribe).not.toHaveBeenCalled();
    });

    it('unsubscribes when stopped', async () => {
      const { result } = renderHook(() => useSimulation());

      act(() => {
        result.current.start();
      });

      await waitFor(() => {
        expect(mockSubscribe).toHaveBeenCalled();
      });

      act(() => {
        result.current.stop();
      });

      await waitFor(() => {
        expect(mockUnsubscribe).toHaveBeenCalled();
      });
    });

    it('unsubscribes on unmount', async () => {
      const { result, unmount } = renderHook(() => useSimulation());

      act(() => {
        result.current.start();
      });

      await waitFor(() => {
        expect(mockSubscribe).toHaveBeenCalled();
      });

      unmount();

      expect(mockUnsubscribe).toHaveBeenCalled();
    });

    it('handles WebSocket events correctly', async () => {
      const { result } = renderHook(() => useSimulation());

      act(() => {
        result.current.start();
      });

      await waitFor(() => {
        expect(mockSubscribe).toHaveBeenCalled();
      });

      const mockEvent: KeyEvent = {
        timestamp: Date.now() * 1000,
        keyCode: 'KEY_A',
        eventType: 'press',
        input: 'A',
        output: 'A',
        latency: 100,
      };

      // Simulate WebSocket event
      act(() => {
        const handler = eventHandlers.get('events');
        handler?.(mockEvent);
      });

      expect(result.current.events).toHaveLength(1);
      expect(result.current.events[0]).toEqual(mockEvent);
    });

    it('ignores invalid WebSocket events', async () => {
      const { result } = renderHook(() => useSimulation());
      const consoleSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

      act(() => {
        result.current.start();
      });

      await waitFor(() => {
        expect(mockSubscribe).toHaveBeenCalled();
      });

      // Simulate invalid event
      act(() => {
        const handler = eventHandlers.get('events');
        handler?.({ invalid: 'data' });
      });

      expect(result.current.events).toHaveLength(0);
      expect(consoleSpy).toHaveBeenCalledWith(
        '[useSimulation] Received invalid event data:',
        { invalid: 'data' }
      );

      consoleSpy.mockRestore();
    });
  });

  describe('statistics computation', () => {
    it('computes total correctly', () => {
      const { result } = renderHook(() => useSimulation());

      act(() => {
        result.current.addEvent({
          timestamp: 1,
          keyCode: 'KEY_A',
          eventType: 'press',
          input: 'A',
          output: 'A',
          latency: 100,
        });
        result.current.addEvent({
          timestamp: 2,
          keyCode: 'KEY_B',
          eventType: 'release',
          input: 'B',
          output: 'B',
          latency: 100,
        });
      });

      expect(result.current.statistics.total).toBe(2);
    });

    it('computes press count correctly', () => {
      const { result } = renderHook(() => useSimulation());

      act(() => {
        result.current.addEvent({
          timestamp: 1,
          keyCode: 'KEY_A',
          eventType: 'press',
          input: 'A',
          output: 'A',
          latency: 100,
        });
        result.current.addEvent({
          timestamp: 2,
          keyCode: 'KEY_B',
          eventType: 'press',
          input: 'B',
          output: 'B',
          latency: 100,
        });
        result.current.addEvent({
          timestamp: 3,
          keyCode: 'KEY_C',
          eventType: 'release',
          input: 'C',
          output: 'C',
          latency: 100,
        });
      });

      expect(result.current.statistics.pressCount).toBe(2);
    });

    it('computes release count correctly', () => {
      const { result } = renderHook(() => useSimulation());

      act(() => {
        result.current.addEvent({
          timestamp: 1,
          keyCode: 'KEY_A',
          eventType: 'press',
          input: 'A',
          output: 'A',
          latency: 100,
        });
        result.current.addEvent({
          timestamp: 2,
          keyCode: 'KEY_B',
          eventType: 'release',
          input: 'B',
          output: 'B',
          latency: 100,
        });
        result.current.addEvent({
          timestamp: 3,
          keyCode: 'KEY_C',
          eventType: 'release',
          input: 'C',
          output: 'C',
          latency: 100,
        });
      });

      expect(result.current.statistics.releaseCount).toBe(2);
    });

    it('computes events per second correctly', () => {
      const { result } = renderHook(() => useSimulation());

      const now = Date.now() * 1000;

      act(() => {
        // Add events within the last second
        result.current.addEvent({
          timestamp: now - 500_000, // 0.5 seconds ago
          keyCode: 'KEY_A',
          eventType: 'press',
          input: 'A',
          output: 'A',
          latency: 100,
        });
        result.current.addEvent({
          timestamp: now - 200_000, // 0.2 seconds ago
          keyCode: 'KEY_B',
          eventType: 'press',
          input: 'B',
          output: 'B',
          latency: 100,
        });
        // Add event older than 1 second
        result.current.addEvent({
          timestamp: now - 1_500_000, // 1.5 seconds ago
          keyCode: 'KEY_C',
          eventType: 'press',
          input: 'C',
          output: 'C',
          latency: 100,
        });
      });

      // Should count only the 2 events within last second
      expect(result.current.statistics.eventsPerSecond).toBe(2);
    });

    it('returns zero statistics for empty events', () => {
      const { result } = renderHook(() => useSimulation());

      expect(result.current.statistics).toEqual({
        total: 0,
        pressCount: 0,
        releaseCount: 0,
        eventsPerSecond: 0,
      });
    });
  });
});
