import { describe, it, expect, afterEach } from 'vitest';
import { detectLocale, setLocale, t, getLocale } from './index';
import { en, ja } from './messages';

describe('i18n', () => {
  afterEach(() => setLocale('en'));

  it('detects Japanese from BCP-47 tags and falls back to English', () => {
    expect(detectLocale('ja')).toBe('ja');
    expect(detectLocale('ja-JP')).toBe('ja');
    expect(detectLocale('en-US')).toBe('en');
    expect(detectLocale('de-DE')).toBe('en');
    expect(detectLocale(undefined)).toBe('en');
  });

  it('has a Japanese message for every English key', () => {
    expect(Object.keys(ja).sort()).toEqual(Object.keys(en).sort());
  });

  it('interpolates placeholders and picks plural forms', () => {
    expect(t('status.active', { profile: 'work' })).toBe('Active: work');
    expect(t('status.keyboards', { count: 1 })).toBe('1 keyboard grabbed');
    expect(t('status.keyboards', { count: 2 })).toBe('2 keyboards grabbed');
  });

  it('switches language and updates <html lang>', () => {
    setLocale('ja');
    expect(getLocale()).toBe('ja');
    expect(document.documentElement.lang).toBe('ja');
    expect(t('status.keyboards', { count: 1 })).toBe('キーボード 1 台を制御中');
    setLocale('en');
    expect(document.documentElement.lang).toBe('en');
  });
});
