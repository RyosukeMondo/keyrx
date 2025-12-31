import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { SimulatorPage } from './SimulatorPage';

// Mock the KeyboardVisualizer component
vi.mock('../components/KeyboardVisualizer', () => ({
  KeyboardVisualizer: ({
    onKeyClick,
    pressedKeys,
  }: {
    onKeyClick: (key: string) => void;
    pressedKeys: Set<string>;
  }) => (
    <div data-testid="keyboard-visualizer">
      <button onClick={() => onKeyClick('A')} data-testid="key-A">
        A {pressedKeys.has('A') ? '(pressed)' : ''}
      </button>
      <button onClick={() => onKeyClick('CAPS')} data-testid="key-CAPS">
        CAPS {pressedKeys.has('CAPS') ? '(pressed)' : ''}
      </button>
    </div>
  ),
}));

// Mock clipboard API
Object.assign(navigator, {
  clipboard: {
    writeText: vi.fn(),
  },
});

describe('SimulatorPage', () => {
  beforeEach(() => {
    vi.clearAllTimers();
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.restoreAllMocks();
    vi.useRealTimers();
  });

  it('renders simulator page with title and description', () => {
    render(<SimulatorPage />);

    expect(screen.getByText('Keyboard Simulator')).toBeInTheDocument();
    expect(
      screen.getByText(/Test your configuration by clicking keys/)
    ).toBeInTheDocument();
  });

  it('renders state display with initial values', () => {
    render(<SimulatorPage />);

    expect(screen.getByText('State')).toBeInTheDocument();
    expect(screen.getByText('MD_00 (Base)')).toBeInTheDocument();
    expect(screen.getByText('Modifiers')).toBeInTheDocument();
    expect(screen.getByText(/Ctrl/)).toBeInTheDocument();
    expect(screen.getByText(/Shift/)).toBeInTheDocument();
  });

  it('renders event log section', () => {
    render(<SimulatorPage />);

    expect(screen.getByText('Event Log')).toBeInTheDocument();
    expect(
      screen.getByText('No events yet. Click a key to start.')
    ).toBeInTheDocument();
  });

  it('adds press event when key is clicked', () => {
    render(<SimulatorPage />);

    const keyA = screen.getByTestId('key-A');
    fireEvent.click(keyA);

    expect(screen.getByText(/Press A/)).toBeInTheDocument();
    expect(screen.getByText(/Output A/)).toBeInTheDocument();
  });

  it('shows key as pressed after click', () => {
    render(<SimulatorPage />);

    const keyA = screen.getByTestId('key-A');
    fireEvent.click(keyA);

    expect(screen.getByText(/A \(pressed\)/)).toBeInTheDocument();
  });

  it('removes key from pressed state on second click', () => {
    render(<SimulatorPage />);

    const keyA = screen.getByTestId('key-A');

    // First click - press
    fireEvent.click(keyA);
    expect(screen.getByText(/A \(pressed\)/)).toBeInTheDocument();

    // Second click - release
    fireEvent.click(keyA);
    expect(screen.queryByText(/A \(pressed\)/)).not.toBeInTheDocument();
  });

  it('adds release event when pressed key is clicked again', () => {
    render(<SimulatorPage />);

    const keyA = screen.getByTestId('key-A');

    // Press
    fireEvent.click(keyA);

    // Release
    fireEvent.click(keyA);

    expect(screen.getByText(/Release A/)).toBeInTheDocument();
  });

  it('resets simulator state when reset button is clicked', () => {
    render(<SimulatorPage />);

    // Press a key
    const keyA = screen.getByTestId('key-A');
    fireEvent.click(keyA);

    expect(screen.getByText(/Press A/)).toBeInTheDocument();

    // Reset
    const resetButton = screen.getByRole('button', {
      name: /Reset simulator state/,
    });
    fireEvent.click(resetButton);

    expect(screen.getByText(/Simulator reset/)).toBeInTheDocument();
    expect(screen.queryByText(/A \(pressed\)/)).not.toBeInTheDocument();
  });

  it('copies event log to clipboard', () => {
    render(<SimulatorPage />);

    // Create some events
    const keyA = screen.getByTestId('key-A');
    fireEvent.click(keyA);

    // Copy log
    const copyButton = screen.getByRole('button', {
      name: /Copy event log/,
    });
    fireEvent.click(copyButton);

    expect(navigator.clipboard.writeText).toHaveBeenCalled();
  });

  it('disables copy button when event log is empty', () => {
    render(<SimulatorPage />);

    const copyButton = screen.getByRole('button', {
      name: /Copy event log/,
    });

    expect(copyButton).toBeDisabled();
  });

  it('shows press event for tap-hold configured key', () => {
    render(<SimulatorPage />);

    const capsKey = screen.getByTestId('key-CAPS');

    // Press CAPS (has tap-hold config with 200ms threshold)
    fireEvent.click(capsKey);

    expect(screen.getByText(/Press CAPS/)).toBeInTheDocument();
  });

  it('shows initial modifier state as inactive', () => {
    render(<SimulatorPage />);

    // Check all modifiers start as inactive (no checkmark)
    const modifiers = screen.getAllByText(/Ctrl|Shift|Alt|Gui/);
    expect(modifiers.length).toBeGreaterThan(0);

    // All modifiers should have slate background (inactive)
    modifiers.forEach((modifier) => {
      expect(modifier.className).toContain('bg-slate-700');
    });
  });

  it('shows release event when key is clicked twice', () => {
    render(<SimulatorPage />);

    const capsKey = screen.getByTestId('key-CAPS');

    // Press CAPS
    fireEvent.click(capsKey);
    expect(screen.getByText(/Press CAPS/)).toBeInTheDocument();

    // Release - click again
    fireEvent.click(capsKey);
    expect(screen.getByText(/Release CAPS/)).toBeInTheDocument();
  });

  it('renders keyboard visualizer component', () => {
    render(<SimulatorPage />);

    expect(screen.getByTestId('keyboard-visualizer')).toBeInTheDocument();
  });

  it('shows interactive keyboard heading', () => {
    render(<SimulatorPage />);

    expect(screen.getByText('Interactive Keyboard')).toBeInTheDocument();
  });
});
