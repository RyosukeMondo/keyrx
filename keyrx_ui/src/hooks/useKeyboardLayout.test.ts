import { renderHook, act } from '@testing-library/react';
import { useKeyboardLayout } from './useKeyboardLayout';
import type { LayoutType } from '@/components/KeyboardVisualizer';

describe('useKeyboardLayout', () => {
  it('should initialize with default layout ANSI_104', () => {
    const { result } = renderHook(() => useKeyboardLayout());

    expect(result.current.layout).toBe('ANSI_104');
    expect(result.current.layoutKeys).toBeDefined();
    expect(Array.isArray(result.current.layoutKeys)).toBe(true);
    expect(result.current.layoutKeys.length).toBeGreaterThan(0);
  });

  it('should initialize with custom layout', () => {
    const { result } = renderHook(() => useKeyboardLayout('ISO_105'));

    expect(result.current.layout).toBe('ISO_105');
    expect(result.current.layoutKeys).toBeDefined();
    expect(result.current.layoutKeys.length).toBeGreaterThan(0);
  });

  it('should update layout when setLayout is called', () => {
    const { result } = renderHook(() => useKeyboardLayout('ANSI_104'));

    expect(result.current.layout).toBe('ANSI_104');
    const initialKeys = result.current.layoutKeys;

    act(() => {
      result.current.setLayout('ISO_105');
    });

    expect(result.current.layout).toBe('ISO_105');
    expect(result.current.layoutKeys).not.toBe(initialKeys); // Should be different reference
  });

  it('should memoize layoutKeys - same reference when layout unchanged', () => {
    const { result, rerender } = renderHook(() =>
      useKeyboardLayout('ANSI_104')
    );

    const firstKeys = result.current.layoutKeys;

    // Re-render without changing layout
    rerender();

    const secondKeys = result.current.layoutKeys;

    // Should be the same reference (memoized)
    expect(secondKeys).toBe(firstKeys);
  });

  it('should recalculate layoutKeys when layout changes', () => {
    const { result } = renderHook(() => useKeyboardLayout('ANSI_104'));

    const ansiKeys = result.current.layoutKeys;

    act(() => {
      result.current.setLayout('ISO_105');
    });

    const isoKeys = result.current.layoutKeys;

    // Should be different references
    expect(isoKeys).not.toBe(ansiKeys);

    // Should have different content (different layouts have different keys)
    expect(isoKeys.length).not.toBe(ansiKeys.length);
  });

  it('should return properly formatted SVG key data', () => {
    const { result } = renderHook(() => useKeyboardLayout('ANSI_104'));

    expect(result.current.layoutKeys.length).toBeGreaterThan(0);

    // Check that first key has required properties
    const firstKey = result.current.layoutKeys[0];
    expect(firstKey).toHaveProperty('code');
    expect(firstKey).toHaveProperty('label');
    expect(firstKey).toHaveProperty('x');
    expect(firstKey).toHaveProperty('y');
    expect(firstKey).toHaveProperty('w');
    expect(firstKey).toHaveProperty('h');
    expect(firstKey).toHaveProperty('shape');

    // Validate types
    expect(typeof firstKey.code).toBe('string');
    expect(typeof firstKey.label).toBe('string');
    expect(typeof firstKey.x).toBe('number');
    expect(typeof firstKey.y).toBe('number');
    expect(typeof firstKey.w).toBe('number');
    expect(typeof firstKey.h).toBe('number');
    expect(['iso-enter', 'standard']).toContain(firstKey.shape);
  });

  it('should support all layout types', () => {
    const layouts: LayoutType[] = [
      'ANSI_104',
      'ANSI_87',
      'ISO_105',
      'ISO_88',
      'JIS_109',
      'COMPACT_60',
      'COMPACT_65',
      'COMPACT_75',
      'COMPACT_96',
      'HHKB',
      'NUMPAD',
    ];

    layouts.forEach((layoutType) => {
      const { result } = renderHook(() => useKeyboardLayout(layoutType));

      expect(result.current.layout).toBe(layoutType);
      expect(result.current.layoutKeys).toBeDefined();
      expect(result.current.layoutKeys.length).toBeGreaterThan(0);
    });
  });

  it('should handle rapid layout changes', () => {
    const { result } = renderHook(() => useKeyboardLayout('ANSI_104'));

    act(() => {
      result.current.setLayout('ISO_105');
      result.current.setLayout('JIS_109');
      result.current.setLayout('COMPACT_60');
    });

    // Should end up with the last layout
    expect(result.current.layout).toBe('COMPACT_60');
    expect(result.current.layoutKeys).toBeDefined();
    expect(result.current.layoutKeys.length).toBeGreaterThan(0);
  });

  it('should provide stable setLayout reference', () => {
    const { result, rerender } = renderHook(() =>
      useKeyboardLayout('ANSI_104')
    );

    const firstSetLayout = result.current.setLayout;

    rerender();

    const secondSetLayout = result.current.setLayout;

    // setLayout should be a stable reference
    expect(secondSetLayout).toBe(firstSetLayout);
  });

  it('should handle switching between different layout sizes', () => {
    const { result } = renderHook(() => useKeyboardLayout('ANSI_104'));

    const fullSizeKeys = result.current.layoutKeys;
    expect(fullSizeKeys.length).toBeGreaterThan(80); // 104-key layout

    act(() => {
      result.current.setLayout('COMPACT_60');
    });

    const compactKeys = result.current.layoutKeys;
    expect(compactKeys.length).toBeLessThan(fullSizeKeys.length); // 60% layout has fewer keys
    expect(compactKeys.length).toBeGreaterThan(50);
    expect(compactKeys.length).toBeLessThan(70);
  });

  it('should parse layout keys correctly for NUMPAD', () => {
    const { result } = renderHook(() => useKeyboardLayout('NUMPAD'));

    expect(result.current.layout).toBe('NUMPAD');
    expect(result.current.layoutKeys.length).toBeGreaterThan(15); // Numpad has ~17 keys
    expect(result.current.layoutKeys.length).toBeLessThan(25);
  });
});
