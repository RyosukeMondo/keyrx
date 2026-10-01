import type { Device } from '@/components/DeviceSelector';
import { isGlobPattern, matchesAnyIdentity } from '@/utils/devicePattern';

/** Minimal shape of a connected device (a subset of `DeviceEntry`). */
export interface ConnectedDevice {
  id: string;
  name: string;
  serial?: string | null;
  path?: string | null;
}

const identitiesOf = (d: ConnectedDevice) => [d.id, d.name, d.path, d.serial];

/**
 * Build the list of selectable "scopes" for the visual editor.
 *
 * A scope is either a pattern used by a `device_start(...)` block in the
 * profile (which may match zero, one or many devices, resolved with the
 * daemon's glob rule) or a connected device that no block mentions yet.
 * Wildcard-only `device_start("*")` is the Global scope and is not listed.
 */
export function buildScopes(
  patterns: readonly string[],
  connected: readonly ConnectedDevice[]
): Device[] {
  const scopes: Device[] = [];
  const claimed = new Set<string>();
  const seen = new Set<string>();

  for (const pattern of patterns) {
    if (pattern === '*' || seen.has(pattern)) continue;
    seen.add(pattern);

    const matched = connected.filter((d) =>
      matchesAnyIdentity(identitiesOf(d), pattern)
    );
    matched.forEach((d) => claimed.add(d.id));

    if (matched.length === 1 && !isGlobPattern(pattern)) {
      const [d] = matched;
      scopes.push({
        id: d.id,
        name: d.name,
        serial: d.serial || undefined,
        connected: true,
        pattern,
      });
    } else if (matched.length === 0) {
      scopes.push({
        id: `disconnected-${pattern}`,
        name: pattern,
        serial: pattern,
        connected: false,
        pattern,
        matchedNames: [],
      });
    } else {
      scopes.push({
        id: `pattern-${pattern}`,
        name: pattern,
        serial: pattern,
        connected: true,
        pattern,
        matchedNames: matched.map((d) => d.name),
      });
    }
  }

  for (const d of connected) {
    if (claimed.has(d.id)) continue;
    scopes.push({
      id: d.id,
      name: d.name,
      serial: d.serial || undefined,
      connected: true,
    });
  }
  return scopes;
}

/**
 * Does a `device_start(blockPattern)` block apply to the selected scope?
 * Pattern scopes are keyed by their pattern; plain devices are resolved with
 * the daemon's glob rule.
 */
export function scopeMatchesBlock(
  scope: Device,
  connected: readonly ConnectedDevice[],
  blockPattern: string
): boolean {
  if (scope.pattern !== undefined) return scope.pattern === blockPattern;
  const device = connected.find((d) => d.id === scope.id);
  return device
    ? matchesAnyIdentity(identitiesOf(device), blockPattern)
    : false;
}
