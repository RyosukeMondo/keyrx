import { describe, it, expect } from 'vitest';
import { truncatePath, formatPathForDisplay } from './pathUtils';

describe('pathUtils', () => {
  describe('truncatePath', () => {
    it('should return path with home replaced if under max length', () => {
      const path = '/home/user/file.txt';
      expect(truncatePath(path, 50)).toBe('~/file.txt');
    });

    it('should replace home directory with ~', () => {
      const path = '/home/username/.config/keyrx/profiles/gaming.rhai';
      const result = truncatePath(path, 100);
      // Should replace /home/username with ~
      expect(result).toContain('~');
      expect(result).not.toContain('/home/username');
      expect(result).toContain('gaming.rhai');
    });

    it('should truncate long paths with ellipsis in middle', () => {
      const path = '/home/user/very/long/path/to/config/file.rhai';
      const result = truncatePath(path, 30);
      expect(result).toContain('...');
      expect(result).toContain('file.rhai');
      expect(result.length).toBeLessThanOrEqual(33); // Allow some flexibility
    });

    it('should handle very short max length', () => {
      const path = '/home/user/very/long/nested/path/to/file.txt';
      const result = truncatePath(path, 20);
      expect(result).toContain('...');
      expect(result.length).toBeLessThanOrEqual(23); // Allow some flexibility
    });

    it('should handle paths with few segments', () => {
      const path = '/file.txt';
      expect(truncatePath(path, 50)).toBe(path);
    });

    it('should use default max length of 50', () => {
      const longPath =
        '/home/user/very/long/path/with/many/segments/that/exceeds/fifty/characters/total.rhai';
      const result = truncatePath(longPath);
      expect(result.length).toBeLessThanOrEqual(53); // Allow ellipsis
    });
  });

  describe('formatPathForDisplay', () => {
    it('should replace home directory with ~', () => {
      const path = '/home/username/Documents/file.txt';
      expect(formatPathForDisplay(path)).toBe('~/Documents/file.txt');
    });

    it('should shorten config paths', () => {
      const path = '/home/user/.config/keyrx/profiles/gaming.rhai';
      const result = formatPathForDisplay(path);
      expect(result).toBe('~/gaming.rhai');
    });

    it('should handle paths without home directory', () => {
      const path = '/etc/keyrx/config.rhai';
      expect(formatPathForDisplay(path)).toBe(path);
    });

    it('should handle paths without config directory', () => {
      const path = '/home/user/custom/location/config.rhai';
      expect(formatPathForDisplay(path)).toBe('~/custom/location/config.rhai');
    });

    it('should handle already formatted paths', () => {
      const path = '~/Documents/file.txt';
      expect(formatPathForDisplay(path)).toBe(path);
    });

    it('should handle empty path', () => {
      expect(formatPathForDisplay('')).toBe('');
    });
  });
});
