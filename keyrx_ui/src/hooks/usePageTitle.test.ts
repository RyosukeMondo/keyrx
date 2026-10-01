import { describe, it, expect } from 'vitest';
import { renderHook } from '@testing-library/react';
import { formatPageTitle, usePageTitle } from './usePageTitle';

describe('usePageTitle', () => {
  it('formats "<Page> – keyrx"', () => {
    expect(formatPageTitle('Devices')).toBe('Devices – keyrx');
  });

  it('sets document.title while the page is mounted', () => {
    renderHook(() => usePageTitle('Monitor'));
    expect(document.title).toBe('Monitor – keyrx');
  });
});
