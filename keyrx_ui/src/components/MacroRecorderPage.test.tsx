/**
 * Component tests for MacroRecorderPage.
 *
 * Tests cover:
 * - Recording controls (start, stop, clear)
 * - Event display and formatting
 * - Rhai code generation preview
 * - Export functionality
 * - Text snippet conversion
 * - Template library integration
 * - Macro testing with simulator
 * - Event timeline editing
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, waitFor, fireEvent, act } from '@testing-library/react';
import type { MacroEvent } from '../hooks/useMacroRecorder';

// Mock CSS import
vi.mock('./MacroRecorderPage.css', () => ({}));

// Mock macroGenerator utilities
vi.mock('../utils/macroGenerator', () => ({
  eventCodeToVK: vi.fn((code: number) => `VK_KEY_${code}`),
  generateRhaiMacro: vi.fn(() => 'layer "default" {\n  // Generated macro\n}'),
  generateMacroJSON: vi.fn(() => '{}'),
  getMacroStats: vi.fn(() => ({ totalEvents: 0, duration: 0 })),
}));

// Mock textSnippetTemplate utilities
vi.mock('../utils/textSnippetTemplate', () => ({
  textToMacroEvents: vi.fn(() => []),
  getTextSnippetStats: vi.fn(() => ({
    characters: 0,
    supportedCharacters: 0,
    unsupportedCharacters: 0,
    steps: 0,
    estimatedDurationMs: 0,
  })),
  TEXT_SNIPPET_TEMPLATES: {
    email: { name: 'Email', template: 'test@example.com' },
  },
}));

// Mock the useMacroRecorder hook
vi.mock('../hooks/useMacroRecorder', () => ({
  useMacroRecorder: vi.fn(),
}));

// Mock the useSimulator hook
vi.mock('../hooks/useSimulator', () => ({
  useSimulator: vi.fn(),
}));

// Mock EventTimeline component
vi.mock('./EventTimeline', () => ({
  EventTimeline: ({ events, onEventsChange, editable }: any) => (
    <div data-testid="event-timeline">
      <div>Events: {events.length}</div>
      <div>Editable: {String(editable)}</div>
      <button
        onClick={() => {
          // Simulate event editing
          const updated = [...events];
          if (updated.length > 0) {
            updated[0] = { ...updated[0], relative_timestamp_us: 999999 };
          }
          onEventsChange(updated);
        }}
      >
        Edit Event
      </button>
    </div>
  ),
}));

// Mock TemplateLibrary component
vi.mock('./TemplateLibrary', () => ({
  TemplateLibrary: ({ onSelectTemplate, onClose }: any) => (
    <div data-testid="template-library">
      <div>Template Library</div>
      <button
        onClick={() => {
          const mockEvents: MacroEvent[] = [
            { event: { code: 30, value: 1 }, relative_timestamp_us: 0 },
            { event: { code: 30, value: 0 }, relative_timestamp_us: 50000 },
          ];
          onSelectTemplate(mockEvents, 'Test Template');
        }}
      >
        Select Template
      </button>
      <button onClick={onClose}>Close Library</button>
    </div>
  ),
}));

import { useMacroRecorder } from '../hooks/useMacroRecorder';
import { useSimulator } from '../hooks/useSimulator';
import { MacroRecorderPage } from './MacroRecorderPage';

describe('MacroRecorderPage', () => {
  const mockStartRecording = vi.fn();
  const mockStopRecording = vi.fn();
  const mockClearEvents = vi.fn();
  const mockClearError = vi.fn();
  const mockLoadConfig = vi.fn();
  const mockSimulate = vi.fn();

  // Sample macro events for testing
  const sampleEvents: MacroEvent[] = [
    { event: { code: 30, value: 1 }, relative_timestamp_us: 0 },
    { event: { code: 30, value: 0 }, relative_timestamp_us: 50000 },
    { event: { code: 31, value: 1 }, relative_timestamp_us: 100000 },
    { event: { code: 31, value: 0 }, relative_timestamp_us: 150000 },
  ];

  beforeEach(() => {
    vi.clearAllMocks();

    // Default mock return values for useMacroRecorder
    vi.mocked(useMacroRecorder).mockReturnValue({
      state: {
        recordingState: 'idle',
        events: [],
        error: null,
        isLoading: false,
      },
      startRecording: mockStartRecording,
      stopRecording: mockStopRecording,
      fetchEvents: vi.fn(),
      clearEvents: mockClearEvents,
      clearError: mockClearError,
    });

    // Default mock return values for useSimulator
    vi.mocked(useSimulator).mockReturnValue({
      state: {
        loadingState: 'idle',
        result: null,
        error: null,
        wasmAvailable: true,
      },
      loadConfig: mockLoadConfig,
      simulate: mockSimulate,
      clearResults: vi.fn(),
    });

    // Mock clipboard API
    Object.assign(navigator, {
      clipboard: {
        writeText: vi.fn().mockResolvedValue(undefined),
      },
    });

    // Mock document.createElement for export functionality
    const mockLinkElement = {
      setAttribute: vi.fn(),
      click: vi.fn(),
    };
    vi.spyOn(document, 'createElement').mockReturnValue(mockLinkElement as any);
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe('rendering', () => {
    it('should render page header', () => {
      render(<MacroRecorderPage />);

      expect(screen.getByText('Macro Recorder')).toBeInTheDocument();
      expect(screen.getByText('Record keyboard events and generate Rhai macro code')).toBeInTheDocument();
    });

    it('should render recording control buttons', () => {
      render(<MacroRecorderPage />);

      expect(screen.getByRole('button', { name: /start recording/i })).toBeInTheDocument();
      expect(screen.getByRole('button', { name: /stop recording/i })).toBeInTheDocument();
      expect(screen.getByRole('button', { name: /clear events/i })).toBeInTheDocument();
    });

    it('should render empty state when no events', () => {
      render(<MacroRecorderPage />);

      expect(screen.getByText('No events recorded yet')).toBeInTheDocument();
      expect(screen.getByText(/click "start recording" to begin/i)).toBeInTheDocument();
    });

    it('should render events table with correct count', () => {
      vi.mocked(useMacroRecorder).mockReturnValue({
        state: {
          recordingState: 'stopped',
          events: sampleEvents,
          error: null,
          isLoading: false,
        },
        startRecording: mockStartRecording,
        stopRecording: mockStopRecording,
        fetchEvents: vi.fn(),
        clearEvents: mockClearEvents,
        clearError: mockClearError,
      });

      render(<MacroRecorderPage />);

      expect(screen.getByText(/recorded events \(4\)/i)).toBeInTheDocument();
    });

    it('should display status indicator for idle state', () => {
      render(<MacroRecorderPage />);

      expect(screen.getByText('Ready')).toBeInTheDocument();
    });

    it('should display status indicator for recording state', () => {
      vi.mocked(useMacroRecorder).mockReturnValue({
        state: {
          recordingState: 'recording',
          events: [],
          error: null,
          isLoading: false,
        },
        startRecording: mockStartRecording,
        stopRecording: mockStopRecording,
        fetchEvents: vi.fn(),
        clearEvents: mockClearEvents,
        clearError: mockClearError,
      });

      render(<MacroRecorderPage />);

      const recordingIndicators = screen.getAllByText('Recording...');
      expect(recordingIndicators.length).toBeGreaterThan(0);
    });

    it('should display status indicator for stopped state', () => {
      vi.mocked(useMacroRecorder).mockReturnValue({
        state: {
          recordingState: 'stopped',
          events: [],
          error: null,
          isLoading: false,
        },
        startRecording: mockStartRecording,
        stopRecording: mockStopRecording,
        fetchEvents: vi.fn(),
        clearEvents: mockClearEvents,
        clearError: mockClearError,
      });

      render(<MacroRecorderPage />);

      expect(screen.getByText('Stopped')).toBeInTheDocument();
    });
  });

  describe('error handling', () => {
    it('should display error banner when error exists', () => {
      vi.mocked(useMacroRecorder).mockReturnValue({
        state: {
          recordingState: 'idle',
          events: [],
          error: 'Failed to start recording',
          isLoading: false,
        },
        startRecording: mockStartRecording,
        stopRecording: mockStopRecording,
        fetchEvents: vi.fn(),
        clearEvents: mockClearEvents,
        clearError: mockClearError,
      });

      render(<MacroRecorderPage />);

      expect(screen.getByText('Failed to start recording')).toBeInTheDocument();
    });

    it('should clear error when close button clicked', () => {
      vi.mocked(useMacroRecorder).mockReturnValue({
        state: {
          recordingState: 'idle',
          events: [],
          error: 'Test error',
          isLoading: false,
        },
        startRecording: mockStartRecording,
        stopRecording: mockStopRecording,
        fetchEvents: vi.fn(),
        clearEvents: mockClearEvents,
        clearError: mockClearError,
      });

      render(<MacroRecorderPage />);

      const closeButton = screen.getByRole('button', { name: '×' });
      fireEvent.click(closeButton);

      expect(mockClearError).toHaveBeenCalled();
    });
  });

  describe('recording controls', () => {
    it('should call startRecording when Start Recording button clicked', async () => {
      render(<MacroRecorderPage />);

      const startButton = screen.getByRole('button', { name: /start recording/i });

      await act(async () => {
        fireEvent.click(startButton);
      });

      expect(mockStartRecording).toHaveBeenCalled();
    });

    it('should disable Start button when recording', () => {
      vi.mocked(useMacroRecorder).mockReturnValue({
        state: {
          recordingState: 'recording',
          events: [],
          error: null,
          isLoading: false,
        },
        startRecording: mockStartRecording,
        stopRecording: mockStopRecording,
        fetchEvents: vi.fn(),
        clearEvents: mockClearEvents,
        clearError: mockClearError,
      });

      render(<MacroRecorderPage />);

      const startButton = screen.getByRole('button', { name: /recording/i });
      expect(startButton).toBeDisabled();
    });

    it('should call stopRecording when Stop Recording button clicked', async () => {
      vi.mocked(useMacroRecorder).mockReturnValue({
        state: {
          recordingState: 'recording',
          events: [],
          error: null,
          isLoading: false,
        },
        startRecording: mockStartRecording,
        stopRecording: mockStopRecording,
        fetchEvents: vi.fn(),
        clearEvents: mockClearEvents,
        clearError: mockClearError,
      });

      render(<MacroRecorderPage />);

      const stopButton = screen.getByRole('button', { name: /stop recording/i });

      await act(async () => {
        fireEvent.click(stopButton);
      });

      expect(mockStopRecording).toHaveBeenCalled();
    });

    it('should disable Stop button when not recording', () => {
      render(<MacroRecorderPage />);

      const stopButton = screen.getByRole('button', { name: /stop recording/i });
      expect(stopButton).toBeDisabled();
    });

    it('should call clearEvents when Clear Events button clicked', async () => {
      vi.mocked(useMacroRecorder).mockReturnValue({
        state: {
          recordingState: 'stopped',
          events: sampleEvents,
          error: null,
          isLoading: false,
        },
        startRecording: mockStartRecording,
        stopRecording: mockStopRecording,
        fetchEvents: vi.fn(),
        clearEvents: mockClearEvents,
        clearError: mockClearError,
      });

      render(<MacroRecorderPage />);

      const clearButton = screen.getByRole('button', { name: /clear events/i });

      await act(async () => {
        fireEvent.click(clearButton);
      });

      expect(mockClearEvents).toHaveBeenCalled();
    });

    it('should disable Clear button when no events', () => {
      render(<MacroRecorderPage />);

      const clearButton = screen.getByRole('button', { name: /clear events/i });
      expect(clearButton).toBeDisabled();
    });
  });

  describe('event display', () => {
    it('should display events in table format', () => {
      vi.mocked(useMacroRecorder).mockReturnValue({
        state: {
          recordingState: 'stopped',
          events: sampleEvents,
          error: null,
          isLoading: false,
        },
        startRecording: mockStartRecording,
        stopRecording: mockStopRecording,
        fetchEvents: vi.fn(),
        clearEvents: mockClearEvents,
        clearError: mockClearError,
      });

      render(<MacroRecorderPage />);

      expect(screen.getByText('0.05ms')).toBeInTheDocument();
      expect(screen.getByText('0.10ms')).toBeInTheDocument();
      expect(screen.getByText('0.15ms')).toBeInTheDocument();
    });

    it('should format key codes correctly', () => {
      vi.mocked(useMacroRecorder).mockReturnValue({
        state: {
          recordingState: 'stopped',
          events: sampleEvents,
          error: null,
          isLoading: false,
        },
        startRecording: mockStartRecording,
        stopRecording: mockStopRecording,
        fetchEvents: vi.fn(),
        clearEvents: mockClearEvents,
        clearError: mockClearError,
      });

      render(<MacroRecorderPage />);

      const keyCells = screen.getAllByText(/A|S/);
      expect(keyCells.length).toBeGreaterThan(0);
    });

    it('should display Press and Release actions', () => {
      vi.mocked(useMacroRecorder).mockReturnValue({
        state: {
          recordingState: 'stopped',
          events: sampleEvents,
          error: null,
          isLoading: false,
        },
        startRecording: mockStartRecording,
        stopRecording: mockStopRecording,
        fetchEvents: vi.fn(),
        clearEvents: mockClearEvents,
        clearError: mockClearError,
      });

      render(<MacroRecorderPage />);

      const pressActions = screen.getAllByText('Press');
      const releaseActions = screen.getAllByText('Release');

      expect(pressActions).toHaveLength(2);
      expect(releaseActions).toHaveLength(2);
    });
  });

  describe('Rhai code generation', () => {
    it('should display placeholder when no events', () => {
      render(<MacroRecorderPage />);

      expect(screen.getByText(/no events recorded yet/i)).toBeInTheDocument();
      expect(screen.getByText(/click "start recording" to begin/i)).toBeInTheDocument();
    });

    it('should generate Rhai code when events exist', () => {
      vi.mocked(useMacroRecorder).mockReturnValue({
        state: {
          recordingState: 'stopped',
          events: sampleEvents,
          error: null,
          isLoading: false,
        },
        startRecording: mockStartRecording,
        stopRecording: mockStopRecording,
        fetchEvents: vi.fn(),
        clearEvents: mockClearEvents,
        clearError: mockClearError,
      });

      render(<MacroRecorderPage />);

      const codeElement = screen.getByText(/layer "default"/i);
      expect(codeElement).toBeInTheDocument();
    });

    it('should update Rhai code when trigger key changes', () => {
      vi.mocked(useMacroRecorder).mockReturnValue({
        state: {
          recordingState: 'stopped',
          events: sampleEvents,
          error: null,
          isLoading: false,
        },
        startRecording: mockStartRecording,
        stopRecording: mockStopRecording,
        fetchEvents: vi.fn(),
        clearEvents: mockClearEvents,
        clearError: mockClearError,
      });

      render(<MacroRecorderPage />);

      const triggerSelect = screen.getByLabelText(/trigger key/i);

      fireEvent.change(triggerSelect, { target: { value: 'VK_F14' } });

      // Code should contain new trigger key
      const codeElements = screen.getAllByText(/VK_F14/i);
      expect(codeElements.length).toBeGreaterThan(0);
    });

    it('should copy code to clipboard when Copy Code button clicked', async () => {
      vi.mocked(useMacroRecorder).mockReturnValue({
        state: {
          recordingState: 'stopped',
          events: sampleEvents,
          error: null,
          isLoading: false,
        },
        startRecording: mockStartRecording,
        stopRecording: mockStopRecording,
        fetchEvents: vi.fn(),
        clearEvents: mockClearEvents,
        clearError: mockClearError,
      });

      render(<MacroRecorderPage />);

      const copyButton = screen.getByRole('button', { name: /copy code/i });

      await act(async () => {
        fireEvent.click(copyButton);
      });

      expect(navigator.clipboard.writeText).toHaveBeenCalled();
      const callArg = vi.mocked(navigator.clipboard.writeText).mock.calls[0][0];
      expect(callArg).toContain('layer "default"');
    });

    it('should disable Copy Code button when no events', () => {
      render(<MacroRecorderPage />);

      const copyButton = screen.getByRole('button', { name: /copy code/i });
      expect(copyButton).toBeDisabled();
    });
  });

  describe('export functionality', () => {
    it('should export events as JSON when Export JSON button clicked', () => {
      vi.mocked(useMacroRecorder).mockReturnValue({
        state: {
          recordingState: 'stopped',
          events: sampleEvents,
          error: null,
          isLoading: false,
        },
        startRecording: mockStartRecording,
        stopRecording: mockStopRecording,
        fetchEvents: vi.fn(),
        clearEvents: mockClearEvents,
        clearError: mockClearError,
      });

      render(<MacroRecorderPage />);

      const exportButton = screen.getByRole('button', { name: /export json/i });

      fireEvent.click(exportButton);

      const createElementSpy = vi.mocked(document.createElement);
      expect(createElementSpy).toHaveBeenCalledWith('a');

      const mockLink = createElementSpy.mock.results[0].value as any;
      expect(mockLink.setAttribute).toHaveBeenCalledWith('href', expect.stringContaining('data:application/json'));
      expect(mockLink.setAttribute).toHaveBeenCalledWith('download', expect.stringMatching(/macro_\d+\.json/));
      expect(mockLink.click).toHaveBeenCalled();
    });

    it('should disable Export button when no events', () => {
      render(<MacroRecorderPage />);

      const exportButton = screen.getByRole('button', { name: /export json/i });
      expect(exportButton).toBeDisabled();
    });
  });

  describe('text snippet conversion', () => {
    it('should show text snippet panel when toggled', () => {
      render(<MacroRecorderPage />);

      const toggleButton = screen.getByRole('button', { name: '▶' });
      fireEvent.click(toggleButton);

      expect(screen.getByPlaceholderText(/enter text to convert/i)).toBeInTheDocument();
    });

    it('should hide text snippet panel when toggled again', () => {
      render(<MacroRecorderPage />);

      const toggleButton = screen.getByRole('button', { name: '▶' });
      fireEvent.click(toggleButton);

      expect(screen.getByPlaceholderText(/enter text to convert/i)).toBeInTheDocument();

      const hideButton = screen.getByRole('button', { name: '▼' });
      fireEvent.click(hideButton);

      expect(screen.queryByPlaceholderText(/enter text to convert/i)).not.toBeInTheDocument();
    });

    it('should disable Load as Macro button when text is empty', () => {
      render(<MacroRecorderPage />);

      const toggleButton = screen.getByRole('button', { name: '▶' });
      fireEvent.click(toggleButton);

      const loadButton = screen.getByRole('button', { name: /load as macro/i });
      expect(loadButton).toBeDisabled();
    });

    it('should enable Load as Macro button when text is entered', () => {
      render(<MacroRecorderPage />);

      const toggleButton = screen.getByRole('button', { name: '▶' });
      fireEvent.click(toggleButton);

      const textarea = screen.getByPlaceholderText(/enter text to convert/i);
      fireEvent.change(textarea, { target: { value: 'Hello' } });

      const loadButton = screen.getByRole('button', { name: /load as macro/i });
      expect(loadButton).not.toBeDisabled();
    });

    it('should convert text to events when Load as Macro clicked', async () => {
      render(<MacroRecorderPage />);

      const toggleButton = screen.getByRole('button', { name: '▶' });
      fireEvent.click(toggleButton);

      const textarea = screen.getByPlaceholderText(/enter text to convert/i);
      fireEvent.change(textarea, { target: { value: 'Hi' } });

      const loadButton = screen.getByRole('button', { name: /load as macro/i });

      await act(async () => {
        fireEvent.click(loadButton);
      });

      // Events should be loaded and displayed
      await waitFor(() => {
        expect(screen.queryByText(/no events recorded yet/i)).not.toBeInTheDocument();
      });
    });
  });

  describe('template library', () => {
    it('should show template library when button clicked', () => {
      render(<MacroRecorderPage />);

      const libraryButton = screen.getByRole('button', { name: /open template library/i });
      fireEvent.click(libraryButton);

      expect(screen.getByTestId('template-library')).toBeInTheDocument();
    });

    it('should hide template library when Close button clicked', () => {
      render(<MacroRecorderPage />);

      const libraryButton = screen.getByRole('button', { name: /open template library/i });
      fireEvent.click(libraryButton);

      expect(screen.getByTestId('template-library')).toBeInTheDocument();

      const closeButton = screen.getByRole('button', { name: /close library/i });
      fireEvent.click(closeButton);

      expect(screen.queryByTestId('template-library')).not.toBeInTheDocument();
    });

    it('should load template events when selected', () => {
      render(<MacroRecorderPage />);

      const libraryButton = screen.getByRole('button', { name: /open template library/i });
      fireEvent.click(libraryButton);

      const selectButton = screen.getByRole('button', { name: /select template/i });
      fireEvent.click(selectButton);

      // Library should close and events should be loaded
      expect(screen.queryByTestId('template-library')).not.toBeInTheDocument();
    });
  });

  describe('event timeline integration', () => {
    it('should render EventTimeline when events exist', () => {
      vi.mocked(useMacroRecorder).mockReturnValue({
        state: {
          recordingState: 'stopped',
          events: sampleEvents,
          error: null,
          isLoading: false,
        },
        startRecording: mockStartRecording,
        stopRecording: mockStopRecording,
        fetchEvents: vi.fn(),
        clearEvents: mockClearEvents,
        clearError: mockClearError,
      });

      render(<MacroRecorderPage />);

      expect(screen.getByTestId('event-timeline')).toBeInTheDocument();
    });

    it('should not render EventTimeline when no events', () => {
      render(<MacroRecorderPage />);

      expect(screen.queryByTestId('event-timeline')).not.toBeInTheDocument();
    });

    it('should pass editable=false when recording', () => {
      vi.mocked(useMacroRecorder).mockReturnValue({
        state: {
          recordingState: 'recording',
          events: sampleEvents,
          error: null,
          isLoading: false,
        },
        startRecording: mockStartRecording,
        stopRecording: mockStopRecording,
        fetchEvents: vi.fn(),
        clearEvents: mockClearEvents,
        clearError: mockClearError,
      });

      render(<MacroRecorderPage />);

      expect(screen.getByText('Editable: false')).toBeInTheDocument();
    });

    it('should pass editable=true when not recording', () => {
      vi.mocked(useMacroRecorder).mockReturnValue({
        state: {
          recordingState: 'stopped',
          events: sampleEvents,
          error: null,
          isLoading: false,
        },
        startRecording: mockStartRecording,
        stopRecording: mockStopRecording,
        fetchEvents: vi.fn(),
        clearEvents: mockClearEvents,
        clearError: mockClearError,
      });

      render(<MacroRecorderPage />);

      expect(screen.getByText('Editable: true')).toBeInTheDocument();
    });
  });

  describe('macro testing', () => {
    it('should disable Test Macro button when no events', () => {
      render(<MacroRecorderPage />);

      const testButton = screen.getByRole('button', { name: /test macro/i });
      expect(testButton).toBeDisabled();
    });

    it('should call simulator when Test Macro button clicked', async () => {
      vi.mocked(useMacroRecorder).mockReturnValue({
        state: {
          recordingState: 'stopped',
          events: sampleEvents,
          error: null,
          isLoading: false,
        },
        startRecording: mockStartRecording,
        stopRecording: mockStopRecording,
        fetchEvents: vi.fn(),
        clearEvents: mockClearEvents,
        clearError: mockClearError,
      });

      render(<MacroRecorderPage />);

      const testButton = screen.getByRole('button', { name: /test macro/i });

      await act(async () => {
        fireEvent.click(testButton);
      });

      expect(mockLoadConfig).toHaveBeenCalled();
      expect(mockSimulate).toHaveBeenCalled();
    });

    it('should show Testing... when simulator is loading', () => {
      vi.mocked(useMacroRecorder).mockReturnValue({
        state: {
          recordingState: 'stopped',
          events: sampleEvents,
          error: null,
          isLoading: false,
        },
        startRecording: mockStartRecording,
        stopRecording: mockStopRecording,
        fetchEvents: vi.fn(),
        clearEvents: mockClearEvents,
        clearError: mockClearError,
      });

      vi.mocked(useSimulator).mockReturnValue({
        state: {
          loadingState: 'loading',
          result: null,
          error: null,
          wasmAvailable: true,
        },
        loadConfig: mockLoadConfig,
        simulate: mockSimulate,
        clearResults: vi.fn(),
      });

      render(<MacroRecorderPage />);

      expect(screen.getByRole('button', { name: /testing/i })).toBeInTheDocument();
    });
  });
});
