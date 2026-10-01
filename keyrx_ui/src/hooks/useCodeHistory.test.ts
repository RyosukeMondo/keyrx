import { act, renderHook } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { hasNativeUndo, useCodeHistory } from './useCodeHistory';

function setup() {
  const box = { code: 'a' };
  const source = {
    state: 'idle',
    getCode: () => box.code,
    loadServerConfig: (c: string) => {
      box.code = c;
    },
  };
  const hook = renderHook(() => useCodeHistory(source, true));
  const edit = (c: string) => {
    box.code = c;
    hook.rerender();
  };
  return { hook, edit, box };
}

describe('useCodeHistory', () => {
  it('undoes and redoes document changes', () => {
    const { hook, edit, box } = setup();
    edit('b');
    edit('c');
    expect(hook.result.current.canUndo).toBe(true);

    act(() => hook.result.current.undo());
    expect(box.code).toBe('b');
    hook.rerender();
    act(() => hook.result.current.undo());
    expect(box.code).toBe('a');
    hook.rerender();
    expect(hook.result.current.canUndo).toBe(false);

    act(() => hook.result.current.redo());
    expect(box.code).toBe('b');
  });

  it('a new edit clears redo, reset clears everything', () => {
    const { hook, edit } = setup();
    edit('b');
    act(() => hook.result.current.undo());
    hook.rerender();
    expect(hook.result.current.canRedo).toBe(true);
    edit('x');
    expect(hook.result.current.canRedo).toBe(false);
    act(() => hook.result.current.reset('fresh'));
    expect(hook.result.current.canUndo).toBe(false);
  });

  it('Ctrl+Z on the page undoes, but not inside a text control', () => {
    const { hook, edit, box } = setup();
    edit('b');
    act(() => {
      document.body.dispatchEvent(
        new KeyboardEvent('keydown', { key: 'z', ctrlKey: true, bubbles: true })
      );
    });
    expect(box.code).toBe('a');
    hook.rerender();

    const input = document.createElement('input');
    document.body.appendChild(input);
    act(() => {
      input.dispatchEvent(
        new KeyboardEvent('keydown', {
          key: 'z',
          ctrlKey: true,
          shiftKey: true,
          bubbles: true,
        })
      );
    });
    expect(box.code).toBe('a'); // redo ignored while typing in an input
    expect(hasNativeUndo(input)).toBe(true);
    input.remove();
  });
});
