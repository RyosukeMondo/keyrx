import { beforeEach, describe, expect, it } from 'vitest';
import {
  MAX_VERSIONS,
  listVersions,
  previousVersion,
  recordSavedVersion,
} from './profileVersions';

describe('profileVersions', () => {
  beforeEach(() => window.localStorage.clear());

  it('keeps the replaced source newest-first and skips repeats and blanks', () => {
    recordSavedVersion('p', 'one', 1);
    recordSavedVersion('p', 'two', 2);
    recordSavedVersion('p', 'two', 3);
    recordSavedVersion('p', '  ', 4);
    expect(listVersions('p').map((v) => v.source)).toEqual(['two', 'one']);
  });

  it('is per profile and bounded', () => {
    for (let i = 0; i < MAX_VERSIONS + 5; i++) recordSavedVersion('p', `v${i}`);
    expect(listVersions('p')).toHaveLength(MAX_VERSIONS);
    expect(listVersions('other')).toEqual([]);
  });

  it('previousVersion skips the version equal to the current source', () => {
    recordSavedVersion('p', 'old');
    recordSavedVersion('p', 'current');
    expect(previousVersion('p', 'current')?.source).toBe('old');
    expect(previousVersion('p', 'zzz')?.source).toBe('current');
    expect(previousVersion('none', 'x')).toBeNull();
  });

  it('survives corrupt storage', () => {
    window.localStorage.setItem('keyrx.versions.p', '{not json');
    expect(listVersions('p')).toEqual([]);
  });
});
