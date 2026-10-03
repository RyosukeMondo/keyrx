/**
 * Layout-file helpers for "Load layout from file": which files the daemon
 * accepts, a valid profile name derived from the file name, and base64.
 *
 * Mirrors keyrx_daemon::services::layout_import (the daemon re-validates
 * everything; these checks only give an instant, friendly answer).
 */

/** Same limit as the daemon's MAX_LAYOUT_BYTES. */
export const MAX_LAYOUT_BYTES = 1024 * 1024;

/** Longest profile name the daemon accepts. */
export const MAX_PROFILE_NAME = 64;

export type LayoutFormat = 'krx' | 'rhai';

/** `.krx` / `.rhai` (case-insensitive), or null for anything else. */
export function detectLayoutFormat(fileName: string): LayoutFormat | null {
  const match = /\.(krx|rhai)$/i.exec(fileName);
  return match ? (match[1].toLowerCase() as LayoutFormat) : null;
}

export type LayoutFileProblem = 'type' | 'empty' | 'tooLarge';

/** Why a file cannot be imported, or null when it looks fine. */
export function layoutFileProblem(file: {
  name: string;
  size: number;
}): LayoutFileProblem | null {
  if (detectLayoutFormat(file.name) === null) return 'type';
  if (file.size === 0) return 'empty';
  if (file.size > MAX_LAYOUT_BYTES) return 'tooLarge';
  return null;
}

/** A profile name from a file name: `my layout.v2.krx` becomes `my-layout-v2`. */
export function profileNameFromFile(fileName: string): string {
  const stem = fileName.replace(/\.[^.]*$/, '');
  const cleaned = stem
    .replace(/[^a-zA-Z0-9_-]+/g, '-')
    .replace(/^-+|-+$/g, '')
    .slice(0, MAX_PROFILE_NAME);
  return cleaned || 'imported';
}

/** Whether `name` satisfies the daemon's profile-name rule. */
export function isValidProfileName(name: string): boolean {
  return /^[a-zA-Z0-9_-]{1,64}$/.test(name);
}

/** `name`, or `name-2`, `name-3`... — the first one not in `taken`. */
export function uniqueProfileName(
  name: string,
  taken: readonly string[]
): string {
  const used = new Set(taken);
  if (!used.has(name)) return name;
  for (let n = 2; ; n++) {
    const suffix = `-${n}`;
    const candidate = name.slice(0, MAX_PROFILE_NAME - suffix.length) + suffix;
    if (!used.has(candidate)) return candidate;
  }
}

/** Standard base64 of `bytes` (chunked so large files do not overflow the stack). */
export function bytesToBase64(bytes: Uint8Array): string {
  let binary = '';
  const chunk = 0x8000;
  for (let i = 0; i < bytes.length; i += chunk) {
    binary += String.fromCharCode(...bytes.subarray(i, i + chunk));
  }
  return btoa(binary);
}

/** The bytes of `file` (FileReader where `Blob.arrayBuffer` is missing). */
export function readFileBytes(file: Blob): Promise<Uint8Array> {
  if (typeof file.arrayBuffer === 'function') {
    return file.arrayBuffer().then((buffer) => new Uint8Array(buffer));
  }
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(new Uint8Array(reader.result as ArrayBuffer));
    reader.onerror = () => reject(reader.error);
    reader.readAsArrayBuffer(file);
  });
}
