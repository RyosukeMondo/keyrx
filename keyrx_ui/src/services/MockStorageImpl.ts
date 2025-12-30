import type { ConfigStorage } from './ConfigStorage';

/**
 * In-memory mock implementation of ConfigStorage interface for testing.
 *
 * Provides a fully deterministic storage implementation using a Map,
 * suitable for unit tests where browser localStorage is not available
 * or should not be used.
 *
 * @example
 * ```typescript
 * const storage = new MockStorageImpl();
 * await storage.save('test', 'value');
 * const result = await storage.load('test');
 * expect(result).toBe('value');
 * ```
 */
export class MockStorageImpl implements ConfigStorage {
  private storage: Map<string, string> = new Map();

  /**
   * Validates a storage key.
   *
   * @param key - The key to validate
   * @throws {Error} If key is empty or invalid
   */
  private validateKey(key: string): void {
    if (!key || key.trim().length === 0) {
      throw new Error('Storage key cannot be empty');
    }
  }

  /**
   * Saves content to in-memory storage under the specified key.
   *
   * @param key - The storage key
   * @param content - The content to save
   * @returns Promise that resolves when save is complete
   * @throws {Error} If key is invalid
   */
  async save(key: string, content: string): Promise<void> {
    this.validateKey(key);
    this.storage.set(key, content);
  }

  /**
   * Loads content from in-memory storage for the specified key.
   *
   * @param key - The storage key
   * @returns Promise that resolves to the stored content, or null if not found
   * @throws {Error} If key is invalid
   */
  async load(key: string): Promise<string | null> {
    this.validateKey(key);
    return this.storage.get(key) ?? null;
  }

  /**
   * Deletes content from in-memory storage for the specified key.
   *
   * @param key - The storage key
   * @returns Promise that resolves when deletion is complete
   * @throws {Error} If key is invalid
   */
  async delete(key: string): Promise<void> {
    this.validateKey(key);
    this.storage.delete(key);
  }

  /**
   * Clears all data from in-memory storage.
   * Useful for resetting state between tests.
   */
  clear(): void {
    this.storage.clear();
  }

  /**
   * Returns the number of items in storage.
   * Useful for testing and debugging.
   *
   * @returns The number of stored items
   */
  size(): number {
    return this.storage.size;
  }
}
