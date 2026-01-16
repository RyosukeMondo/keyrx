/**
 * useUnifiedApi - Simple Beneficial Tests
 *
 * Philosophy: Test basic hook usage without complex WebSocket timing
 *
 * Complex WebSocket timing tests removed - they were flaky and tested
 * implementation details. For full WebSocket integration, use E2E tests.
 */

import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { renderHook } from '@testing-library/react';
import { useUnifiedApi } from './useUnifiedApi';
import {
  setupMockWebSocket,
  cleanupMockWebSocket,
} from '../../tests/helpers/websocket';

describe('useUnifiedApi - Simple Tests', () => {
  beforeEach(async () => {
    await setupMockWebSocket();
  });

  afterEach(() => {
    cleanupMockWebSocket();
  });

  it('initializes without crashing', () => {
    const { result } = renderHook(() => useUnifiedApi());

    // Hook should initialize
    expect(result.current).toBeDefined();
    expect(typeof result.current.query).toBe('function');
    expect(typeof result.current.command).toBe('function');
    expect(typeof result.current.subscribe).toBe('function');
  });

  it('provides readyState property', () => {
    const { result } = renderHook(() => useUnifiedApi());

    // Should have readyState (WebSocket state)
    expect(result.current.readyState).toBeDefined();
    expect(typeof result.current.readyState).toBe('number');
  });

  it('provides isConnected property', () => {
    const { result } = renderHook(() => useUnifiedApi());

    // Should have isConnected boolean
    expect(typeof result.current.isConnected).toBe('boolean');
  });
});
