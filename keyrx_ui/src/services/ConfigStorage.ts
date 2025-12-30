/**
 * Storage abstraction interface for configuration data.
 *
 * Provides a unified interface for storing, loading, and deleting configuration
 * data with support for different storage backends (localStorage, in-memory, etc.).
 * All operations are promise-based for consistency and to support async storage
 * implementations in the future.
 *
 * @example
 * ```typescript
 * const storage: ConfigStorage = new LocalStorageImpl();
 * await storage.save('myConfig', '{ "key": "value" }');
 * const data = await storage.load('myConfig');
 * await storage.delete('myConfig');
 * ```
 */
export interface ConfigStorage {
  /**
   * Saves content to storage under the specified key.
   *
   * @param key - The storage key to save the content under
   * @param content - The string content to save
   * @returns Promise that resolves when save is complete
   * @throws {Error} If storage is unavailable, quota exceeded, or key is invalid
   *
   * @example
   * ```typescript
   * await storage.save('config', '{ "value": 42 }');
   * ```
   */
  save(key: string, content: string): Promise<void>;

  /**
   * Loads content from storage for the specified key.
   *
   * @param key - The storage key to load content from
   * @returns Promise that resolves to the stored content, or null if not found
   * @throws {Error} If storage is unavailable or key is invalid
   *
   * @example
   * ```typescript
   * const config = await storage.load('config');
   * if (config) {
   *   console.log('Config found:', config);
   * }
   * ```
   */
  load(key: string): Promise<string | null>;

  /**
   * Deletes content from storage for the specified key.
   *
   * @param key - The storage key to delete
   * @returns Promise that resolves when deletion is complete
   * @throws {Error} If storage is unavailable or key is invalid
   *
   * @example
   * ```typescript
   * await storage.delete('config');
   * ```
   */
  delete(key: string): Promise<void>;
}
