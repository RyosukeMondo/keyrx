/**
 * EventInjectionForm - Manual event injection interface
 *
 * This component provides a form for manually injecting keyboard events
 * into the simulator. Users can specify a key code, event type (press/release),
 * and inject the event into the simulation.
 *
 * Features:
 * - Key code input field
 * - Event type selector (press/release)
 * - Inject button to trigger event
 * - Input validation (non-empty key code)
 * - Accessible form labels and controls
 * - Disabled state support
 *
 * @example
 * ```tsx
 * <EventInjectionForm
 *   onInjectEvent={(keyCode, eventType) => {
 *     handleKeyEvent(keyCode, eventType);
 *   }}
 *   disabled={!isSimulatorReady}
 * />
 * ```
 */

import React, { useState, useCallback, FormEvent } from 'react';
import { Button } from '../Button';

/**
 * Event type for keyboard events
 */
export type EventType = 'press' | 'release';

/**
 * Data structure for injected events
 */
export interface InjectedEventData {
  /** Key code to inject (e.g., 'A', 'Enter', 'Space') */
  keyCode: string;
  /** Type of event (press or release) */
  eventType: EventType;
}

/**
 * Props for EventInjectionForm component
 */
export interface EventInjectionFormProps {
  /** Callback invoked when event should be injected */
  onInjectEvent: (data: InjectedEventData) => void;
  /** Whether the form should be disabled */
  disabled?: boolean;
  /** Optional CSS classes */
  className?: string;
}

/**
 * EventInjectionForm component
 *
 * Renders a form for manually injecting keyboard events with validation
 */
export const EventInjectionForm: React.FC<EventInjectionFormProps> = ({
  onInjectEvent,
  disabled = false,
  className = '',
}) => {
  const [keyCode, setKeyCode] = useState<string>('');
  const [eventType, setEventType] = useState<EventType>('press');
  const [validationError, setValidationError] = useState<string>('');

  /**
   * Validate key code input
   */
  const validateKeyCode = useCallback((code: string): boolean => {
    if (!code.trim()) {
      setValidationError('Key code cannot be empty');
      return false;
    }
    setValidationError('');
    return true;
  }, []);

  /**
   * Handle form submission
   */
  const handleSubmit = useCallback(
    (e: FormEvent) => {
      e.preventDefault();

      if (!validateKeyCode(keyCode)) {
        return;
      }

      onInjectEvent({
        keyCode: keyCode.trim(),
        eventType,
      });

      // Clear form after successful injection
      setKeyCode('');
      setValidationError('');
    },
    [keyCode, eventType, validateKeyCode, onInjectEvent]
  );

  /**
   * Handle key code input change
   */
  const handleKeyCodeChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const value = e.target.value;
      setKeyCode(value);
      // Clear validation error when user types
      if (validationError && value.trim()) {
        setValidationError('');
      }
    },
    [validationError]
  );

  /**
   * Handle event type change
   */
  const handleEventTypeChange = useCallback(
    (e: React.ChangeEvent<HTMLSelectElement>) => {
      setEventType(e.target.value as EventType);
    },
    []
  );

  return (
    <form
      onSubmit={handleSubmit}
      className={`flex flex-col gap-3 ${className}`}
      aria-label="Manual event injection form"
    >
      {/* Key Code Input */}
      <div className="flex flex-col gap-1">
        <label
          htmlFor="event-key-code"
          className="text-sm font-medium text-slate-300"
        >
          Key Code
        </label>
        <input
          id="event-key-code"
          type="text"
          value={keyCode}
          onChange={handleKeyCodeChange}
          disabled={disabled}
          placeholder="e.g., A, Enter, Space"
          className="px-3 py-2 bg-slate-700 border border-slate-600 rounded-md text-slate-100 placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-transparent disabled:opacity-50 disabled:cursor-not-allowed"
          aria-describedby={validationError ? 'key-code-error' : undefined}
          aria-invalid={!!validationError}
        />
        {validationError && (
          <span
            id="key-code-error"
            className="text-xs text-red-400"
            role="alert"
          >
            {validationError}
          </span>
        )}
      </div>

      {/* Event Type Selector */}
      <div className="flex flex-col gap-1">
        <label
          htmlFor="event-type"
          className="text-sm font-medium text-slate-300"
        >
          Event Type
        </label>
        <select
          id="event-type"
          value={eventType}
          onChange={handleEventTypeChange}
          disabled={disabled}
          className="px-3 py-2 bg-slate-700 border border-slate-600 rounded-md text-slate-100 focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-transparent disabled:opacity-50 disabled:cursor-not-allowed"
          aria-label="Select event type"
        >
          <option value="press">Press</option>
          <option value="release">Release</option>
        </select>
      </div>

      {/* Inject Button */}
      <Button
        type="submit"
        variant="primary"
        size="md"
        onClick={() => {}} // Handled by form onSubmit
        disabled={disabled || !keyCode.trim()}
        aria-label="Inject keyboard event"
        className="w-full min-h-[44px]"
      >
        Inject Event
      </Button>
    </form>
  );
};
