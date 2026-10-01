import { describe, it, expect, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { GlobalDebugPanel } from './GlobalDebugPanel';

describe('GlobalDebugPanel visibility', () => {
  beforeEach(() => {
    window.localStorage.clear();
    window.history.replaceState({}, '', '/');
  });

  it('is hidden by default so it never covers page content', () => {
    render(<GlobalDebugPanel />);
    expect(screen.queryByRole('button', { name: /debug panel/i })).toBeNull();
  });

  it('is toggled (and remembered) with Ctrl+Shift+D', () => {
    render(<GlobalDebugPanel />);
    fireEvent.keyDown(window, { key: 'D', ctrlKey: true, shiftKey: true });
    expect(
      screen.getByRole('button', { name: /open debug panel/i })
    ).toBeInTheDocument();
    expect(window.localStorage.getItem('keyrx.debug')).toBe('1');

    fireEvent.keyDown(window, { key: 'D', ctrlKey: true, shiftKey: true });
    expect(screen.queryByRole('button', { name: /debug panel/i })).toBeNull();
    expect(window.localStorage.getItem('keyrx.debug')).toBeNull();
  });

  it('can be enabled with ?debug=1', () => {
    window.history.replaceState({}, '', '/?debug=1');
    render(<GlobalDebugPanel />);
    expect(
      screen.getByRole('button', { name: /open debug panel/i })
    ).toBeInTheDocument();
  });
});
