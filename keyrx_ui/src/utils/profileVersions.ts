/**
 * Previously saved versions of a profile, kept client-side.
 *
 * Each successful Save first stores the source it is about to replace, so
 * "Revert to previous version" always has something to go back to. Versions
 * live in this browser only.
 *
 * TODO(daemon): replace with a server-side history so versions survive a
 * browser change and cover edits made outside the UI. Contract the UI would
 * consume (not implemented in the daemon yet):
 *   GET  /api/profiles/{name}/versions            -> [{ id, savedAt (ISO), source_sha256, bytes }]
 *   GET  /api/profiles/{name}/versions/{id}       -> { id, savedAt, source }
 * The daemon would append a version on every PUT /api/profiles/{name}/config.
 * Only `listVersions` / `recordSavedVersion` below would need to change.
 */

export interface ProfileVersion {
  source: string;
  /** Epoch milliseconds when this source was replaced by a newer save. */
  savedAt: number;
}

export const MAX_VERSIONS = 10;

const storageKey = (profile: string): string => `keyrx.versions.${profile}`;

function read(profile: string): ProfileVersion[] {
  try {
    const raw = window.localStorage.getItem(storageKey(profile));
    const parsed: unknown = raw ? JSON.parse(raw) : [];
    if (!Array.isArray(parsed)) return [];
    return parsed.filter(
      (v): v is ProfileVersion =>
        typeof v?.source === 'string' && typeof v?.savedAt === 'number'
    );
  } catch {
    return [];
  }
}

/** Saved versions, newest first. */
export function listVersions(profile: string): ProfileVersion[] {
  return read(profile);
}

/**
 * Remember `previousSource` (what the server held before this save). Empty
 * sources and repeats of the newest version are ignored.
 */
export function recordSavedVersion(
  profile: string,
  previousSource: string,
  now: number = Date.now()
): void {
  if (!profile || !previousSource.trim()) return;
  const versions = read(profile);
  if (versions[0]?.source === previousSource) return;
  const next = [{ source: previousSource, savedAt: now }, ...versions].slice(
    0,
    MAX_VERSIONS
  );
  try {
    window.localStorage.setItem(storageKey(profile), JSON.stringify(next));
  } catch {
    // storage full or blocked: history is a convenience, never fail a save
  }
}

/** Newest saved version whose source differs from `current`, if any. */
export function previousVersion(
  profile: string,
  current: string
): ProfileVersion | null {
  return read(profile).find((v) => v.source !== current) ?? null;
}
