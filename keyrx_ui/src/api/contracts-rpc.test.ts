/**
 * WebSocket RPC Contract Tests
 */

import { describe, it, expect } from 'vitest';
import { validateRpcMessage } from './schemas';

describe('WebSocket RPC Contracts', () => {
  describe('Client Messages (Outgoing)', () => {
    it('validates query message', () => {
      const queryMessage = {
        type: 'query',
        content: {
          id: 'req-1',
          method: 'get_devices',
          params: null,
        },
      };

      const validated = validateRpcMessage(queryMessage, 'client');

      expect(validated.type).toBe('query');
      expect(validated.content.method).toBe('get_devices');
    });

    it('validates command message', () => {
      const commandMessage = {
        type: 'command',
        content: {
          id: 'req-2',
          method: 'activate_profile',
          params: { name: 'gaming' },
        },
      };

      const validated = validateRpcMessage(commandMessage, 'client');

      expect(validated.type).toBe('command');
      expect(validated.content.params).toEqual({ name: 'gaming' });
    });

    it('validates subscribe message', () => {
      const subscribeMessage = {
        type: 'subscribe',
        content: {
          id: 'sub-1',
          channel: 'daemon_state',
        },
      };

      const validated = validateRpcMessage(subscribeMessage, 'client');

      expect(validated.type).toBe('subscribe');
      expect(validated.content.channel).toBe('daemon_state');
    });

    it('validates unsubscribe message', () => {
      const unsubscribeMessage = {
        type: 'unsubscribe',
        content: {
          id: 'unsub-1',
          channel: 'daemon_state',
        },
      };

      const validated = validateRpcMessage(unsubscribeMessage, 'client');

      expect(validated.type).toBe('unsubscribe');
    });

    it('rejects client message with invalid type', () => {
      const invalidMessage = {
        type: 'invalid_type',
        content: {
          id: 'req-1',
          method: 'get_devices',
        },
      };

      expect(() => {
        validateRpcMessage(invalidMessage, 'client');
      }).toThrow('Invalid client RPC message');
    });

    it('rejects client message with missing required fields', () => {
      const invalidMessage = {
        type: 'query',
        content: {},
      };

      expect(() => {
        validateRpcMessage(invalidMessage, 'client');
      }).toThrow('Invalid client RPC message');
    });

    it('rejects client message without content', () => {
      const invalidMessage = {
        type: 'query',
      };

      expect(() => {
        validateRpcMessage(invalidMessage, 'client');
      }).toThrow('Invalid client RPC message');
    });
  });

  describe('Server Messages (Incoming)', () => {
    it('validates response message with result', () => {
      const responseMessage = {
        type: 'response',
        content: {
          id: 'req-1',
          result: { devices: [] },
        },
      };

      const validated = validateRpcMessage(responseMessage, 'server');

      expect(validated.type).toBe('response');
      expect(validated.content.result).toEqual({ devices: [] });
    });

    it('validates response message with error', () => {
      const errorMessage = {
        type: 'response',
        content: {
          id: 'req-1',
          error: {
            code: 404,
            message: 'Profile not found',
          },
        },
      };

      const validated = validateRpcMessage(errorMessage, 'server');

      expect(validated.type).toBe('response');
      expect(validated.content.error?.code).toBe(404);
    });

    it('validates event message', () => {
      const eventMessage = {
        type: 'event',
        content: {
          channel: 'daemon_state',
          data: {
            modifiers: ['LCtrl'],
            locks: [],
            layer: 'base',
            active_profile: 'gaming',
          },
        },
      };

      const validated = validateRpcMessage(eventMessage, 'server');

      expect(validated.type).toBe('event');
      expect(validated.content.channel).toBe('daemon_state');
    });

    it('validates connected message', () => {
      const connectedMessage = {
        type: 'connected',
        content: {
          version: '0.1.0',
          timestamp: Date.now(),
        },
      };

      const validated = validateRpcMessage(connectedMessage, 'server');

      expect(validated.type).toBe('connected');
      expect(validated.content.version).toBe('0.1.0');
    });

    it('rejects server message with invalid type', () => {
      const invalidMessage = {
        type: 'unknown_type',
        content: {},
      };

      expect(() => {
        validateRpcMessage(invalidMessage, 'server');
      }).toThrow('Invalid server RPC message');
    });

    it('rejects server message with missing content', () => {
      const invalidMessage = {
        type: 'response',
      };

      expect(() => {
        validateRpcMessage(invalidMessage, 'server');
      }).toThrow('Invalid server RPC message');
    });

    it('rejects response without id', () => {
      const invalidMessage = {
        type: 'response',
        content: {
          result: {},
        },
      };

      expect(() => {
        validateRpcMessage(invalidMessage, 'server');
      }).toThrow('Invalid server RPC message');
    });

    it('rejects event without channel', () => {
      const invalidMessage = {
        type: 'event',
        content: {
          data: {},
        },
      };

      expect(() => {
        validateRpcMessage(invalidMessage, 'server');
      }).toThrow('Invalid server RPC message');
    });
  });
});
