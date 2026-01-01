import { ReactElement } from 'react';
import { render, RenderOptions } from '@testing-library/react';
import { BrowserRouter } from 'react-router-dom';
import { vi } from 'vitest';

// Mock WebSocket before components import it
global.WebSocket = vi.fn(() => ({
  addEventListener: vi.fn(),
  removeEventListener: vi.fn(),
  send: vi.fn(),
  close: vi.fn(),
  readyState: WebSocket.CONNECTING,
  CONNECTING: 0,
  OPEN: 1,
  CLOSING: 2,
  CLOSED: 3,
})) as any;

/**
 * Custom render function that wraps components with necessary providers for accessibility tests
 */
export function renderForA11y(
  ui: ReactElement,
  options?: Omit<RenderOptions, 'wrapper'>
) {
  function Wrapper({ children }: { children: React.ReactNode }) {
    return <BrowserRouter>{children}</BrowserRouter>;
  }

  return render(ui, { wrapper: Wrapper, ...options });
}
