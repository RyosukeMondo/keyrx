import type { ConfigStorage } from './ConfigStorage';

/**
 * localStorage-based implementation of ConfigStorage interface.
 *
 * Uses the browser's localStorage API to persist configuration data.
 * Handles common localStorage edge cases including quota exceeded,
 * storage disabled, and invalid keys.
 *
 * @example
 * ```typescript
 * const storage = new LocalStorageImpl();
 * await storage.save('myKey', 'myValue');
 * const value = await storage.load('myKey');
 * ```
 */
export class LocalStorageImpl implements ConfigStorage {
  /**
   * Checks if localStorage is available and functional.
   *
   * @returns true if localStorage is available, false otherwise
   */
  private isLocalStorageAvailable(): boolean {
    try {
      const testKey = '__localStorage_test__';
      localStorage.setItem(testKey, 'test');
      localStorage.removeItem(testKey);
      return true;
    } catch {
      return false;
    }
  }

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
   * Saves content to localStorage under the specified key.
   *
   * @param key - The storage key
   * @param content - The content to save
   * @returns Promise that resolves when save is complete
   * @throws {Error} If localStorage is unavailable, quota exceeded, or key is invalid
   */
  async save(key: string, content: string): Promise<void> {
    this.validateKey(key);

    if (!this.isLocalStorageAvailable()) {
      throw new Error('localStorage is not available');
    }

    try {
      localStorage.setItem(key, content);
    } catch (error) {
      // Handle quota exceeded error
      if (error instanceof Error && error.name === 'QuotaExceededError') {
        throw new Error('localStorage quota exceeded. Please clear some space.');
      }
      // Handle other storage errors
      throw new Error(`Failed to save to localStorage: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }

  /**
   * Loads content from localStorage for the specified key.
   *
   * @param key - The storage key
   * @returns Promise that resolves to the stored content, or null if not found
   * @throws {Error} If localStorage is unavailable or key is invalid
   */
  async load(key: string): Promise<string | null> {
    this.validateKey(key);

    if (!this.isLocalStorageAvailable()) {
      throw new Error('localStorage is not available');
    }

    try {
      return localStorage.getItem(key);
    } catch (error) {
      throw new Error(`Failed to load from localStorage: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }

  /**
   * Deletes content from localStorage for the specified key.
   *
   * @param key - The storage key
   * @returns Promise that resolves when deletion is complete
   * @throws {Error} If localStorage is unavailable or key is invalid
   */
  async delete(key: string): Promise<void> {
    this.validateKey(key);

    if (!this.isLocalStorageAvailable()) {
      throw new Error('localStorage is not available');
    }

    try {
      localStorage.removeItem(key);
    } catch (error) {
      throw new Error(`Failed to delete from localStorage: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }
}
