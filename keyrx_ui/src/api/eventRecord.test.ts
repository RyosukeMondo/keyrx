import { describe, it, expect } from 'vitest';
import { toEventRecord } from './eventRecord';
import type { KeyEventData } from '../types';

function makeKeyEventData(overrides?: Partial<KeyEventData>): KeyEventData {
  return {
    timestamp: 1_700_000_000_000_000,
    keyCode: 'KEY_A',
    eventType: 'press',
    input: 'KEY_A',
    output: 'KEY_B',
    latency: 250,
    mappingTriggered: true,
    ...overrides,
  };
}

describe('toEventRecord', () => {
  it('converts the timestamp from microseconds to an ISO string', () => {
    const record = toEventRecord(makeKeyEventData({ timestamp: 1_700_000_000_000_000 }));
    expect(record.timestamp).toBe(new Date(1_700_000_000_000).toISOString());
  });

  it('maps eventType press/release to type press/release', () => {
    expect(toEventRecord(makeKeyEventData({ eventType: 'press' })).type).toBe('press');
    expect(toEventRecord(makeKeyEventData({ eventType: 'release' })).type).toBe(
      'release'
    );
  });

  it('treats any non-press eventType as release', () => {
    expect(
      toEventRecord(makeKeyEventData({ eventType: 'something-else' })).type
    ).toBe('release');
  });

  it('strips the KEY_ prefix from keyCode for display', () => {
    expect(toEventRecord(makeKeyEventData({ keyCode: 'KEY_A' })).keyCode).toBe(
      'A'
    );
    expect(toEventRecord(makeKeyEventData({ keyCode: 'CapsLock' })).keyCode).toBe(
      'CapsLock'
    );
  });

  it('defaults layer to Base', () => {
    expect(toEventRecord(makeKeyEventData()).layer).toBe('Base');
  });

  it('maps latency to latencyUs', () => {
    expect(toEventRecord(makeKeyEventData({ latency: 999 })).latencyUs).toBe(999);
  });

  it('sets action to output when mappingTriggered is true', () => {
    const record = toEventRecord(
      makeKeyEventData({ mappingTriggered: true, output: 'Escape' })
    );
    expect(record.action).toBe('Escape');
  });

  it('leaves action undefined when mappingTriggered is false', () => {
    const record = toEventRecord(
      makeKeyEventData({ mappingTriggered: false, output: 'A' })
    );
    expect(record.action).toBeUndefined();
  });

  it('passes through device and mapping metadata', () => {
    const record = toEventRecord(
      makeKeyEventData({
        deviceId: 'dev-1',
        deviceName: 'Keyboard 1',
        mappingType: 'simple',
      })
    );
    expect(record.deviceId).toBe('dev-1');
    expect(record.deviceName).toBe('Keyboard 1');
    expect(record.mappingType).toBe('simple');
    expect(record.mappingTriggered).toBe(true);
  });

  it('generates a unique id per call', () => {
    const data = makeKeyEventData();
    const a = toEventRecord(data);
    const b = toEventRecord(data);
    expect(a.id).not.toBe(b.id);
    expect(a.id).toMatch(new RegExp(`^evt-${data.timestamp}-`));
  });
});
