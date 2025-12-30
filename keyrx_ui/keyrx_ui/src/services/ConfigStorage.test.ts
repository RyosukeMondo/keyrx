import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest';
import { LocalStorageImpl } from './LocalStorageImpl';
import { MockStorageImpl } from './MockStorageImpl';
import type { ConfigStorage } from './ConfigStorage';

describe('ConfigStorage Interface', () => {
  describe('MockStorageImpl', () => {
    let storage: MockStorageImpl;

    beforeEach(() => {
      storage = new MockStorageImpl();
    });

    describe('save', () => {
      it('should save content successfully', async () => {
        await storage.save('testKey', 'testValue');
        const result = await storage.load('testKey');
        expect(result).toBe('testValue');
      });

      it('should overwrite existing content', async () => {
        await storage.save('key', 'value1');
        await storage.save('key', 'value2');
        const result = await storage.load('key');
        expect(result).toBe('value2');
      });

      it('should handle empty string content', async () => {
        await storage.save('empty', '');
        const result = await storage.load('empty');
        expect(result).toBe('');
      });

      it('should handle large content', async () => {
        const largeContent = 'x'.repeat(100000);
        await storage.save('large', largeContent);
        const result = await storage.load('large');
        expect(result).toBe(largeContent);
      });

      it('should handle special characters in content', async () => {
        const specialContent = '{"key": "value", "emoji": "🎉", "newline": "line1\\nline2"}';
        await storage.save('special', specialContent);
        const result = await storage.load('special');
        expect(result).toBe(specialContent);
      });

      it('should throw error for empty key', async () => {
        await expect(storage.save('', 'value')).rejects.toThrow('Storage key cannot be empty');
      });

      it('should throw error for whitespace-only key', async () => {
        await expect(storage.save('   ', 'value')).rejects.toThrow('Storage key cannot be empty');
      });

      it('should handle keys with special characters', async () => {
        await storage.save('key-with-dashes', 'value');
        await storage.save('key.with.dots', 'value');
        await storage.save('key_with_underscores', 'value');
        expect(await storage.load('key-with-dashes')).toBe('value');
        expect(await storage.load('key.with.dots')).toBe('value');
        expect(await storage.load('key_with_underscores')).toBe('value');
      });
    });

    describe('load', () => {
      it('should load existing content', async () => {
        await storage.save('testKey', 'testValue');
        const result = await storage.load('testKey');
        expect(result).toBe('testValue');
      });

      it('should return null for non-existent key', async () => {
        const result = await storage.load('nonExistent');
        expect(result).toBeNull();
      });

      it('should throw error for empty key', async () => {
        await expect(storage.load('')).rejects.toThrow('Storage key cannot be empty');
      });

      it('should throw error for whitespace-only key', async () => {
        await expect(storage.load('   ')).rejects.toThrow('Storage key cannot be empty');
      });
    });

    describe('delete', () => {
      it('should delete existing content', async () => {
        await storage.save('testKey', 'testValue');
        await storage.delete('testKey');
        const result = await storage.load('testKey');
        expect(result).toBeNull();
      });

      it('should not throw error when deleting non-existent key', async () => {
        await expect(storage.delete('nonExistent')).resolves.not.toThrow();
      });

      it('should throw error for empty key', async () => {
        await expect(storage.delete('')).rejects.toThrow('Storage key cannot be empty');
      });

      it('should throw error for whitespace-only key', async () => {
        await expect(storage.delete('   ')).rejects.toThrow('Storage key cannot be empty');
      });
    });

    describe('clear', () => {
      it('should clear all storage', async () => {
        await storage.save('key1', 'value1');
        await storage.save('key2', 'value2');
        storage.clear();
        expect(await storage.load('key1')).toBeNull();
        expect(await storage.load('key2')).toBeNull();
        expect(storage.size()).toBe(0);
      });
    });

    describe('size', () => {
      it('should return correct size', async () => {
        expect(storage.size()).toBe(0);
        await storage.save('key1', 'value1');
        expect(storage.size()).toBe(1);
        await storage.save('key2', 'value2');
        expect(storage.size()).toBe(2);
        await storage.delete('key1');
        expect(storage.size()).toBe(1);
      });
    });

    describe('async behavior', () => {
      it('should handle concurrent operations', async () => {
        const promises = [
          storage.save('key1', 'value1'),
          storage.save('key2', 'value2'),
          storage.save('key3', 'value3'),
        ];
        await Promise.all(promises);
        expect(await storage.load('key1')).toBe('value1');
        expect(await storage.load('key2')).toBe('value2');
        expect(await storage.load('key3')).toBe('value3');
      });
    });
  });

  describe('LocalStorageImpl', () => {
    let storage: LocalStorageImpl;
    let localStorageMock: { [key: string]: string };

    beforeEach(() => {
      // Create a mock localStorage
      localStorageMock = {};

      global.localStorage = {
        getItem: vi.fn((key: string) => localStorageMock[key] ?? null),
        setItem: vi.fn((key: string, value: string) => {
          localStorageMock[key] = value;
        }),
        removeItem: vi.fn((key: string) => {
          delete localStorageMock[key];
        }),
        clear: vi.fn(() => {
          localStorageMock = {};
        }),
        key: vi.fn((index: number) => Object.keys(localStorageMock)[index] ?? null),
        get length() {
          return Object.keys(localStorageMock).length;
        },
      } as Storage;

      storage = new LocalStorageImpl();
    });

    afterEach(() => {
      vi.restoreAllMocks();
    });

    describe('save', () => {
      it('should save content to localStorage', async () => {
        await storage.save('testKey', 'testValue');
        expect(localStorage.setItem).toHaveBeenCalledWith('testKey', 'testValue');
        expect(localStorageMock['testKey']).toBe('testValue');
      });

      it('should overwrite existing content', async () => {
        await storage.save('key', 'value1');
        await storage.save('key', 'value2');
        expect(localStorageMock['key']).toBe('value2');
      });

      it('should handle empty string content', async () => {
        await storage.save('empty', '');
        expect(localStorageMock['empty']).toBe('');
      });

      it('should handle large content', async () => {
        const largeContent = 'x'.repeat(100000);
        await storage.save('large', largeContent);
        expect(localStorageMock['large']).toBe(largeContent);
      });

      it('should handle special characters in content', async () => {
        const specialContent = '{"key": "value", "emoji": "🎉"}';
        await storage.save('special', specialContent);
        expect(localStorageMock['special']).toBe(specialContent);
      });

      it('should throw error for empty key', async () => {
        await expect(storage.save('', 'value')).rejects.toThrow('Storage key cannot be empty');
      });

      it('should throw error for whitespace-only key', async () => {
        await expect(storage.save('   ', 'value')).rejects.toThrow('Storage key cannot be empty');
      });

      it('should handle quota exceeded error', async () => {
        const quotaError = new Error('QuotaExceededError');
        quotaError.name = 'QuotaExceededError';

        // Mock setItem to throw quota error on the actual save, but not on availability check
        let callCount = 0;
        vi.mocked(localStorage.setItem).mockImplementation((key: string, value: string) => {
          callCount++;
          if (key === '__localStorage_test__') {
            // Allow the availability check to pass
            return;
          }
          // Throw quota error on actual save
          throw quotaError;
        });

        await expect(storage.save('key', 'value')).rejects.toThrow('localStorage quota exceeded');
      });

      it('should throw error when localStorage is unavailable', async () => {
        // Mock localStorage to throw on test access
        vi.mocked(localStorage.setItem).mockImplementationOnce(() => {
          throw new Error('localStorage not available');
        });
        vi.mocked(localStorage.removeItem).mockImplementationOnce(() => {
          throw new Error('localStorage not available');
        });

        await expect(storage.save('key', 'value')).rejects.toThrow();
      });
    });

    describe('load', () => {
      it('should load existing content from localStorage', async () => {
        localStorageMock['testKey'] = 'testValue';
        const result = await storage.load('testKey');
        expect(localStorage.getItem).toHaveBeenCalledWith('testKey');
        expect(result).toBe('testValue');
      });

      it('should return null for non-existent key', async () => {
        const result = await storage.load('nonExistent');
        expect(result).toBeNull();
      });

      it('should throw error for empty key', async () => {
        await expect(storage.load('')).rejects.toThrow('Storage key cannot be empty');
      });

      it('should throw error for whitespace-only key', async () => {
        await expect(storage.load('   ')).rejects.toThrow('Storage key cannot be empty');
      });

      it('should handle localStorage errors', async () => {
        vi.mocked(localStorage.getItem).mockImplementationOnce(() => {
          throw new Error('Read error');
        });
        await expect(storage.load('key')).rejects.toThrow('Failed to load from localStorage');
      });
    });

    describe('delete', () => {
      it('should delete existing content from localStorage', async () => {
        localStorageMock['testKey'] = 'testValue';
        await storage.delete('testKey');
        expect(localStorage.removeItem).toHaveBeenCalledWith('testKey');
        expect(localStorageMock['testKey']).toBeUndefined();
      });

      it('should not throw error when deleting non-existent key', async () => {
        await expect(storage.delete('nonExistent')).resolves.not.toThrow();
      });

      it('should throw error for empty key', async () => {
        await expect(storage.delete('')).rejects.toThrow('Storage key cannot be empty');
      });

      it('should throw error for whitespace-only key', async () => {
        await expect(storage.delete('   ')).rejects.toThrow('Storage key cannot be empty');
      });

      it('should handle localStorage errors', async () => {
        // Mock removeItem to throw error on actual delete, but not on availability check
        vi.mocked(localStorage.removeItem).mockImplementation((key: string) => {
          if (key === '__localStorage_test__') {
            // Allow the availability check to pass
            return;
          }
          // Throw error on actual delete
          throw new Error('Delete error');
        });
        await expect(storage.delete('key')).rejects.toThrow('Failed to delete from localStorage');
      });
    });

    describe('async behavior', () => {
      it('should handle concurrent operations', async () => {
        const promises = [
          storage.save('key1', 'value1'),
          storage.save('key2', 'value2'),
          storage.save('key3', 'value3'),
        ];
        await Promise.all(promises);
        expect(localStorageMock['key1']).toBe('value1');
        expect(localStorageMock['key2']).toBe('value2');
        expect(localStorageMock['key3']).toBe('value3');
      });
    });
  });

  describe('Interface compliance', () => {
    const testImplementations: Array<{ name: string; factory: () => ConfigStorage }> = [
      { name: 'MockStorageImpl', factory: () => new MockStorageImpl() },
      { name: 'LocalStorageImpl', factory: () => new LocalStorageImpl() },
    ];

    // Set up localStorage mock for LocalStorageImpl tests
    beforeEach(() => {
      const localStorageMock: { [key: string]: string } = {};

      global.localStorage = {
        getItem: vi.fn((key: string) => localStorageMock[key] ?? null),
        setItem: vi.fn((key: string, value: string) => {
          localStorageMock[key] = value;
        }),
        removeItem: vi.fn((key: string) => {
          delete localStorageMock[key];
        }),
        clear: vi.fn(),
        key: vi.fn(),
        get length() {
          return Object.keys(localStorageMock).length;
        },
      } as Storage;
    });

    testImplementations.forEach(({ name, factory }) => {
      describe(`${name} interface compliance`, () => {
        let storage: ConfigStorage;

        beforeEach(() => {
          storage = factory();
        });

        it('should implement save method', () => {
          expect(storage.save).toBeDefined();
          expect(typeof storage.save).toBe('function');
        });

        it('should implement load method', () => {
          expect(storage.load).toBeDefined();
          expect(typeof storage.load).toBe('function');
        });

        it('should implement delete method', () => {
          expect(storage.delete).toBeDefined();
          expect(typeof storage.delete).toBe('function');
        });

        it('should return promises from all methods', async () => {
          const saveResult = storage.save('key', 'value');
          expect(saveResult).toBeInstanceOf(Promise);
          await saveResult;

          const loadResult = storage.load('key');
          expect(loadResult).toBeInstanceOf(Promise);
          await loadResult;

          const deleteResult = storage.delete('key');
          expect(deleteResult).toBeInstanceOf(Promise);
          await deleteResult;
        });
      });
    });
  });
});
