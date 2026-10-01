import { describe, expect, it } from 'vitest';
import { en, ja } from './messages';
import { fitText } from '@/utils/keycap';
import { detectLayout } from '@/utils/layoutPreference';
import type { DeviceEntry } from '@/types';

const text = (m: string | { one: string; other: string }): string =>
  typeof m === 'string' ? m : `${m.one}\n${m.other}`;
const placeholders = (m: string | { one: string; other: string }): string[] =>
  [...new Set([...text(m).matchAll(/\{(\w+)\}/g)].map((x) => x[1]))].sort();

describe('message catalogs', () => {
  it('ja has every key of en with the same placeholders', () => {
    for (const key of Object.keys(en) as (keyof typeof en)[]) {
      expect(ja[key], `ja is missing ${key}`).toBeDefined();
      expect(placeholders(ja[key]), key).toEqual(placeholders(en[key]));
    }
  });

  it('plural messages are plural in both languages', () => {
    for (const key of Object.keys(en) as (keyof typeof en)[]) {
      expect(typeof ja[key] === 'string', key).toBe(
        typeof en[key] === 'string'
      );
    }
  });

  it('ja strings are actually translated (contain Japanese) unless they are names', () => {
    const untranslated = (Object.keys(en) as (keyof typeof en)[]).filter(
      (k) => !/[぀-ヿ一-鿿]/.test(text(ja[k]))
    );
    // Allowed: pure symbols / product names / identical technical labels
    expect(
      untranslated.filter(
        (k) => /[A-Za-z]{4,}/.test(text(ja[k])) && text(ja[k]) === text(en[k])
      )
    ).toEqual(['swap.preset', 'save.added']);
  });
});

describe('fitText with full-width characters', () => {
  it('counts CJK glyphs as a whole em so JIS legends do not overflow', () => {
    const fit = fitText('半角/全角', 44, 13, 10);
    expect(fit.text.endsWith('…')).toBe(true);
    expect(fitText('半/全', 44, 13, 10).text).toBe('半/全');
    expect(fitText('Esc', 44, 13, 10).text).toBe('Esc');
  });
});

describe('JIS detection from device data', () => {
  const dev = (over: Partial<DeviceEntry>): DeviceEntry => ({
    id: 'd',
    name: 'Board',
    path: '/dev/input/event1',
    serial: null,
    active: true,
    scope: 'global',
    layout: null,
    isVirtual: false,
    ...over,
  });

  it('prefers the daemon flag, falls back to the name', () => {
    expect(detectLayout('map("A","B")', [dev({ hasJisKeys: true })])).toBe(
      'JIS_109'
    );
    expect(
      detectLayout('map("A","B")', [dev({ name: 'Topre Realforce 109 JP' })])
    ).toBe('JIS_109');
    expect(detectLayout('map("A","B")', [dev({ name: 'Logitech K120' })])).toBe(
      'ANSI_104'
    );
  });
});
