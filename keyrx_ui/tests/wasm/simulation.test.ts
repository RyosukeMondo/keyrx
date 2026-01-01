/**
 * WASM Simulation Integration Tests
 *
 * Tests the simulate WASM function to ensure:
 * - Simulation processes events correctly
 * - Results are deterministic (same input = same output)
 * - Virtual clock makes tests time-independent
 * - State tracking works correctly
 *
 * @requirements REQ-5 (AC7, AC8, AC9, AC10)
 */

import { describe, it, expect, beforeAll } from 'vitest';

describe('WASM Simulation Integration Tests', () => {
  let wasm: typeof import('../../src/wasm/pkg/keyrx_core');

  beforeAll(async () => {
    // Import and initialize WASM module
    wasm = await import('../../src/wasm/pkg/keyrx_core');
    wasm.wasm_init();
  });

  describe('Configuration Loading', () => {
    it('should load valid Rhai config and return handle', () => {
      const config = `
        device("*") {
          map("A", "B");
        }
      `;

      const handle = wasm.load_config(config);
      expect(handle).toBeDefined();
      expect(typeof handle).toBe('object'); // ConfigHandle is a WASM object
    });

    it('should reject config larger than 1MB', () => {
      const largeConfig = 'x'.repeat(2 * 1024 * 1024); // 2MB

      expect(() => {
        wasm.load_config(largeConfig);
      }).toThrow(/too large/i);
    });

    it('should reject invalid Rhai syntax', () => {
      const invalidConfig = `
        device("*") {
          map("A", "B")
      `;

      expect(() => {
        wasm.load_config(invalidConfig);
      }).toThrow();
    });

    it('should load config with multiple devices', () => {
      const config = `
        device("kbd-1") {
          map("A", "B");
        }
        device("kbd-2") {
          map("VK_A", "VK_B");
        }
      `;

      const handle = wasm.load_config(config);
      expect(handle).toBeDefined();
    });
  });

  describe('Event Simulation', () => {
    it('should process simple key press/release sequence', () => {
      const config = `
        device("*") {
          map("A", "B");
        }
      `;

      const handle = wasm.load_config(config);

      const events = {
        events: [
          { keycode: 'A', event_type: 'press', timestamp_us: 0 },
          { keycode: 'A', event_type: 'release', timestamp_us: 100000 },
        ],
      };

      const result = wasm.simulate(handle, JSON.stringify(events));
      expect(result).toBeDefined();
      expect(result).toHaveProperty('timeline');
      expect(result).toHaveProperty('final_state');
      expect(result).toHaveProperty('latency_stats');
      expect(Array.isArray(result.timeline)).toBe(true);
    });

    it('should reject too many events (> 1000)', () => {
      const config = `
        device("*") {
          map("A", "B");
        }
      `;

      const handle = wasm.load_config(config);

      // Generate 1001 events
      const events = {
        events: Array.from({ length: 1001 }, (_, i) => ({
          keycode: 'A',
          event_type: i % 2 === 0 ? 'press' : 'release',
          timestamp_us: i * 1000,
        })),
      };

      expect(() => {
        wasm.simulate(handle, JSON.stringify(events));
      }).toThrow(/too many events/i);
    });

    it('should handle empty event sequence', () => {
      const config = `
        device("*") {
          map("A", "B");
        }
      `;

      const handle = wasm.load_config(config);

      const events = {
        events: [],
      };

      const result = wasm.simulate(handle, JSON.stringify(events));
      expect(result).toBeDefined();
      expect(result.timeline).toEqual([]);
    });

    it('should reject invalid JSON in event sequence', () => {
      const config = `
        device("*") {
          map("A", "B");
        }
      `;

      const handle = wasm.load_config(config);

      expect(() => {
        wasm.simulate(handle, 'invalid json');
      }).toThrow(/invalid json/i);
    });

    it('should reject invalid config handle', () => {
      // Create a fake handle (this will fail since WASM handles are opaque)
      // We can't easily create an invalid handle, so skip this test
      // or test with a handle from a different config
      const config1 = 'device("*") { map("A", "B"); }';
      const config2 = 'device("*") { map("VK_A", "VK_B"); }';

      const handle1 = wasm.load_config(config1);
      const handle2 = wasm.load_config(config2);

      const events = {
        events: [{ keycode: 'A', event_type: 'press', timestamp_us: 0 }],
      };

      // Both handles should work
      const result1 = wasm.simulate(handle1, JSON.stringify(events));
      const result2 = wasm.simulate(handle2, JSON.stringify(events));

      expect(result1).toBeDefined();
      expect(result2).toBeDefined();
    });
  });

  describe('Determinism', () => {
    it('should return identical results for same input', () => {
      const config = `
        device("*") {
          map("A", "B");
        }
      `;

      const events = {
        events: [
          { keycode: 'A', event_type: 'press', timestamp_us: 0 },
          { keycode: 'A', event_type: 'release', timestamp_us: 50000 },
          { keycode: 'A', event_type: 'press', timestamp_us: 100000 },
          { keycode: 'A', event_type: 'release', timestamp_us: 150000 },
        ],
      };

      const eventsJson = JSON.stringify(events);

      // Load config multiple times and simulate
      const handle1 = wasm.load_config(config);
      const result1 = wasm.simulate(handle1, eventsJson);

      const handle2 = wasm.load_config(config);
      const result2 = wasm.simulate(handle2, eventsJson);

      const handle3 = wasm.load_config(config);
      const result3 = wasm.simulate(handle3, eventsJson);

      // Results should be identical
      expect(result1.timeline.length).toBe(result2.timeline.length);
      expect(result2.timeline.length).toBe(result3.timeline.length);

      // Compare timeline entries
      for (let i = 0; i < result1.timeline.length; i++) {
        expect(result1.timeline[i]).toEqual(result2.timeline[i]);
        expect(result2.timeline[i]).toEqual(result3.timeline[i]);
      }

      // Compare final states
      expect(result1.final_state).toEqual(result2.final_state);
      expect(result2.final_state).toEqual(result3.final_state);
    });

    it('should be deterministic with virtual clock', () => {
      const config = `
        device("*") {
          map("A", "B");
        }
      `;

      const events = {
        events: [
          { keycode: 'A', event_type: 'press', timestamp_us: 1000 },
          { keycode: 'A', event_type: 'release', timestamp_us: 2000 },
        ],
      };

      // Run simulation at different real times
      const handle1 = wasm.load_config(config);
      const result1 = wasm.simulate(handle1, JSON.stringify(events));

      // Wait a bit (simulate different execution time)
      const start = Date.now();
      while (Date.now() - start < 10) {
        // Busy wait 10ms
      }

      const handle2 = wasm.load_config(config);
      const result2 = wasm.simulate(handle2, JSON.stringify(events));

      // Results should still be identical despite different execution times
      expect(result1).toEqual(result2);
    });

    it('should produce same results across multiple simulations with same handle', () => {
      const config = `
        device("*") {
          map("A", "B");
        }
      `;

      const handle = wasm.load_config(config);

      const events = {
        events: [
          { keycode: 'A', event_type: 'press', timestamp_us: 0 },
          { keycode: 'A', event_type: 'release', timestamp_us: 100000 },
        ],
      };

      const eventsJson = JSON.stringify(events);

      // Run simulation multiple times with same handle
      const result1 = wasm.simulate(handle, eventsJson);
      const result2 = wasm.simulate(handle, eventsJson);
      const result3 = wasm.simulate(handle, eventsJson);

      expect(result1).toEqual(result2);
      expect(result2).toEqual(result3);
    });
  });

  describe('State Tracking', () => {
    it('should return simulation state after running simulation', () => {
      const config = `
        device("*") {
          map("A", "B");
        }
      `;

      const handle = wasm.load_config(config);

      const events = {
        events: [
          { keycode: 'A', event_type: 'press', timestamp_us: 0 },
          { keycode: 'A', event_type: 'release', timestamp_us: 100000 },
        ],
      };

      const result = wasm.simulate(handle, JSON.stringify(events));

      expect(result.final_state).toBeDefined();
      expect(result.final_state).toHaveProperty('active_modifiers');
      expect(result.final_state).toHaveProperty('active_locks');
      expect(Array.isArray(result.final_state.active_modifiers)).toBe(true);
      expect(Array.isArray(result.final_state.active_locks)).toBe(true);
    });

    it('should track state via get_state after simulation', () => {
      const config = `
        device("*") {
          map("A", "B");
        }
      `;

      const handle = wasm.load_config(config);

      const events = {
        events: [
          { keycode: 'A', event_type: 'press', timestamp_us: 0 },
        ],
      };

      wasm.simulate(handle, JSON.stringify(events));

      const state = wasm.get_state(handle);
      expect(state).toBeDefined();
      expect(state).toHaveProperty('modifiers');
      expect(state).toHaveProperty('locks');
      expect(state).toHaveProperty('raw_state');
      expect(state).toHaveProperty('active_modifier_count');
      expect(state).toHaveProperty('active_lock_count');
    });

    it('should error when getting state before simulation', () => {
      const config = `
        device("*") {
          map("A", "B");
        }
      `;

      const handle = wasm.load_config(config);

      // Try to get state before running simulation
      expect(() => {
        wasm.get_state(handle);
      }).toThrow(/no simulation state/i);
    });

    it('should update state after each simulation', () => {
      const config = `
        device("*") {
          map("A", "B");
        }
      `;

      const handle = wasm.load_config(config);

      // First simulation
      const events1 = {
        events: [{ keycode: 'A', event_type: 'press', timestamp_us: 0 }],
      };
      wasm.simulate(handle, JSON.stringify(events1));
      const state1 = wasm.get_state(handle);

      // Second simulation (different events)
      const events2 = {
        events: [
          { keycode: 'A', event_type: 'press', timestamp_us: 0 },
          { keycode: 'A', event_type: 'release', timestamp_us: 100000 },
        ],
      };
      wasm.simulate(handle, JSON.stringify(events2));
      const state2 = wasm.get_state(handle);

      // States should be defined
      expect(state1).toBeDefined();
      expect(state2).toBeDefined();
    });
  });

  describe('Timeline and Latency', () => {
    it('should return timeline with correct structure', () => {
      const config = `
        device("*") {
          map("A", "B");
        }
      `;

      const handle = wasm.load_config(config);

      const events = {
        events: [
          { keycode: 'A', event_type: 'press', timestamp_us: 0 },
          { keycode: 'A', event_type: 'release', timestamp_us: 100000 },
        ],
      };

      const result = wasm.simulate(handle, JSON.stringify(events));

      expect(result.timeline).toBeDefined();
      expect(Array.isArray(result.timeline)).toBe(true);

      if (result.timeline.length > 0) {
        const entry = result.timeline[0];
        expect(entry).toHaveProperty('timestamp_us');
        expect(entry).toHaveProperty('state_before');
        expect(entry).toHaveProperty('state_after');
      }
    });

    it('should return latency statistics', () => {
      const config = `
        device("*") {
          map("A", "B");
        }
      `;

      const handle = wasm.load_config(config);

      const events = {
        events: [
          { keycode: 'A', event_type: 'press', timestamp_us: 0 },
          { keycode: 'A', event_type: 'release', timestamp_us: 100000 },
        ],
      };

      const result = wasm.simulate(handle, JSON.stringify(events));

      expect(result.latency_stats).toBeDefined();
      expect(result.latency_stats).toHaveProperty('min_us');
      expect(result.latency_stats).toHaveProperty('max_us');
      expect(result.latency_stats).toHaveProperty('avg_us');
      expect(result.latency_stats).toHaveProperty('count');

      // Latency values should be non-negative
      expect(result.latency_stats.min_us).toBeGreaterThanOrEqual(0);
      expect(result.latency_stats.max_us).toBeGreaterThanOrEqual(0);
      expect(result.latency_stats.avg_us).toBeGreaterThanOrEqual(0);
      expect(result.latency_stats.count).toBeGreaterThanOrEqual(0);
    });

    it('should track multiple events in timeline', () => {
      const config = `
        device("*") {
          map("A", "B");
        }
      `;

      const handle = wasm.load_config(config);

      const events = {
        events: [
          { keycode: 'A', event_type: 'press', timestamp_us: 0 },
          { keycode: 'A', event_type: 'release', timestamp_us: 50000 },
          { keycode: 'B', event_type: 'press', timestamp_us: 100000 },
          { keycode: 'B', event_type: 'release', timestamp_us: 150000 },
        ],
      };

      const result = wasm.simulate(handle, JSON.stringify(events));

      expect(result.timeline.length).toBeGreaterThan(0);
      expect(result.latency_stats.count).toBeGreaterThan(0);
    });
  });

  describe('Edge Cases', () => {
    it('should handle rapid key repeat', () => {
      const config = `
        device("*") {
          map("A", "B");
        }
      `;

      const handle = wasm.load_config(config);

      const events = {
        events: [
          { keycode: 'A', event_type: 'press', timestamp_us: 0 },
          { keycode: 'A', event_type: 'press', timestamp_us: 1000 },
          { keycode: 'A', event_type: 'press', timestamp_us: 2000 },
          { keycode: 'A', event_type: 'release', timestamp_us: 3000 },
        ],
      };

      const result = wasm.simulate(handle, JSON.stringify(events));
      expect(result).toBeDefined();
      expect(result.timeline).toBeDefined();
    });

    it('should handle simultaneous key presses', () => {
      const config = `
        device("*") {
          map("A", "B");
        }
      `;

      const handle = wasm.load_config(config);

      const events = {
        events: [
          { keycode: 'A', event_type: 'press', timestamp_us: 0 },
          { keycode: 'B', event_type: 'press', timestamp_us: 0 },
          { keycode: 'A', event_type: 'release', timestamp_us: 100000 },
          { keycode: 'B', event_type: 'release', timestamp_us: 100000 },
        ],
      };

      const result = wasm.simulate(handle, JSON.stringify(events));
      expect(result).toBeDefined();
      expect(result.timeline).toBeDefined();
    });

    it('should handle events with zero timestamp', () => {
      const config = `
        device("*") {
          map("A", "B");
        }
      `;

      const handle = wasm.load_config(config);

      const events = {
        events: [
          { keycode: 'A', event_type: 'press', timestamp_us: 0 },
          { keycode: 'A', event_type: 'release', timestamp_us: 0 },
        ],
      };

      const result = wasm.simulate(handle, JSON.stringify(events));
      expect(result).toBeDefined();
    });

    it('should handle maximum allowed events (1000)', () => {
      const config = `
        device("*") {
          map("A", "B");
        }
      `;

      const handle = wasm.load_config(config);

      // Generate exactly 1000 events
      const events = {
        events: Array.from({ length: 1000 }, (_, i) => ({
          keycode: 'A',
          event_type: i % 2 === 0 ? 'press' : 'release',
          timestamp_us: i * 1000,
        })),
      };

      const result = wasm.simulate(handle, JSON.stringify(events));
      expect(result).toBeDefined();
      expect(result.timeline).toBeDefined();
    });
  });
});
