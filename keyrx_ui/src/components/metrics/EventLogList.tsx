import React, { useRef, useEffect } from 'react';
import { FixedSizeList as List } from 'react-window';

/**
 * Event log entry interface
 */
export interface EventLogEntry {
  id: string;
  timestamp: number;
  type: 'press' | 'release' | 'tap' | 'hold' | 'macro' | 'layer_switch';
  keyCode: string;
  action?: string;
  latency: number;
  input?: string;
  output?: string;
  deviceId?: string;
  deviceName?: string;
  mappingType?: string;
  mappingTriggered?: boolean;
}

/**
 * Props for EventLogList component
 */
export interface EventLogListProps {
  /**
   * Array of event log entries to display
   */
  events: EventLogEntry[];

  /**
   * Maximum number of events to display
   * @default undefined (show all events)
   */
  maxEvents?: number;

  /**
   * Height of the list container in pixels
   * @default 300
   */
  height?: number;

  /**
   * Height of each row in pixels
   * @default 40
   */
  rowHeight?: number;

  /**
   * Whether to auto-scroll to the latest event
   * @default true
   */
  autoScroll?: boolean;
}

/**
 * EventLogList component displays a virtualized list of keyboard events
 * with performance optimizations for large lists.
 *
 * Features:
 * - Virtualized rendering for 1000+ events
 * - Auto-scroll to latest events
 * - Color-coded event types
 * - Mapping detection and highlighting
 * - Device information display
 * - Latency monitoring
 *
 * @example
 * ```tsx
 * <EventLogList
 *   events={eventLog}
 *   height={400}
 *   autoScroll={true}
 * />
 * ```
 */
export const EventLogList: React.FC<EventLogListProps> = ({
  events,
  maxEvents,
  height = 300,
  rowHeight = 40,
  autoScroll = true,
}) => {
  const listRef = useRef<List>(null);

  // Limit events if maxEvents is specified
  const displayEvents = maxEvents ? events.slice(-maxEvents) : events;

  // Auto-scroll to bottom when new events arrive
  useEffect(() => {
    if (autoScroll && listRef.current && displayEvents.length > 0) {
      listRef.current.scrollToItem(displayEvents.length - 1, 'end');
    }
  }, [displayEvents.length, autoScroll]);

  // Format timestamp for display
  const formatTime = (timestamp: number): string => {
    const date = new Date(timestamp);
    return date.toLocaleTimeString('en-US', {
      hour12: false,
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
    });
  };

  // Format latency for display
  const formatLatency = (latency: number): string => {
    return `${latency.toFixed(2)}ms`;
  };

  // Format key code for display
  const formatKey = (key: string | undefined): string => {
    if (!key) return '–';
    return key.replace(/^KEY_/, '').replace(/^VK_/, '');
  };

  // Event type color mapping
  const typeColors: Record<EventLogEntry['type'], string> = {
    press: 'text-green-400',
    release: 'text-red-400',
    tap: 'text-blue-400',
    hold: 'text-yellow-400',
    macro: 'text-purple-400',
    layer_switch: 'text-cyan-400',
  };

  // Event type symbol mapping
  const typeSymbols: Record<EventLogEntry['type'], string> = {
    press: '↓',
    release: '↑',
    tap: '⇥',
    hold: '⏎',
    macro: '⌘',
    layer_switch: '⇧',
  };

  // Event row renderer for react-window
  const EventRow = ({
    index,
    style,
  }: {
    index: number;
    style: React.CSSProperties;
  }) => {
    const event = displayEvents[index];
    if (!event) return null;

    // Check if input differs from output (remapping occurred)
    const wasRemapped =
      event.input && event.output && event.input !== event.output;
    const hasMappingTriggered =
      event.mappingTriggered ||
      wasRemapped ||
      ['tap', 'hold', 'macro', 'layer_switch'].includes(event.type);

    // Get short device name
    const shortDeviceName = event.deviceName
      ? event.deviceName.length > 15
        ? event.deviceName.slice(0, 12) + '…'
        : event.deviceName
      : event.deviceId?.slice(0, 8) || '–';

    return (
      <div
        style={style}
        className="flex items-center gap-3 px-4 text-sm font-mono border-b border-slate-700 hover:bg-slate-700/50"
        title={`Device: ${event.deviceName || event.deviceId || 'Unknown'}`}
        role="row"
      >
        <span
          className="w-20 text-slate-400 text-xs"
          role="cell"
          aria-label={`Time: ${formatTime(event.timestamp)}`}
        >
          {formatTime(event.timestamp)}
        </span>
        <span
          className={`w-14 ${typeColors[event.type]}`}
          title={event.type}
          role="cell"
          aria-label={`Event type: ${event.type}`}
        >
          {typeSymbols[event.type]} {event.type.slice(0, 3)}
        </span>
        <span
          className="w-20 text-slate-200 truncate"
          title={event.input || event.keyCode}
          role="cell"
          aria-label={`Input: ${formatKey(event.input || event.keyCode)}`}
        >
          {formatKey(event.input || event.keyCode)}
        </span>
        {/* Mapping indicator */}
        <span
          className={`w-6 text-center ${
            hasMappingTriggered ? 'text-green-400' : 'text-slate-600'
          }`}
          role="cell"
          aria-label={hasMappingTriggered ? 'Mapping triggered' : 'No mapping'}
        >
          {hasMappingTriggered ? '→' : '–'}
        </span>
        {/* Output */}
        <span
          className={`w-20 truncate ${
            wasRemapped ? 'text-blue-400' : 'text-slate-400'
          }`}
          title={event.output}
          role="cell"
          aria-label={`Output: ${formatKey(event.output)}`}
        >
          {formatKey(event.output)}
        </span>
        {/* Mapping type */}
        <span
          className="w-16 text-slate-500 text-xs truncate"
          title={event.mappingType || 'passthrough'}
          role="cell"
          aria-label={`Mapping type: ${event.mappingType || 'passthrough'}`}
        >
          {event.mappingType || (wasRemapped ? 'remap' : '–')}
        </span>
        {/* Device */}
        <span
          className="flex-1 text-slate-500 text-xs truncate"
          title={event.deviceName || event.deviceId}
          role="cell"
          aria-label={`Device: ${shortDeviceName}`}
        >
          {shortDeviceName}
        </span>
        {/* Latency */}
        <span
          className={`w-16 text-right ${
            event.latency > 1 ? 'text-yellow-400' : 'text-slate-400'
          }`}
          role="cell"
          aria-label={`Latency: ${formatLatency(event.latency)}`}
        >
          {formatLatency(event.latency)}
        </span>
      </div>
    );
  };

  return (
    <div role="table" aria-label="Event log">
      {/* Table Header - hide some columns on mobile */}
      <div
        className="hidden md:flex items-center gap-3 px-4 py-2 bg-slate-800 border-b border-slate-700 text-sm font-semibold text-slate-300"
        role="row"
      >
        <span className="w-20" role="columnheader">
          Time
        </span>
        <span className="w-14" role="columnheader">
          Type
        </span>
        <span className="w-20" role="columnheader">
          Input
        </span>
        <span
          className="w-6 text-center"
          title="Mapping Triggered"
          role="columnheader"
        >
          →
        </span>
        <span className="w-20" role="columnheader">
          Output
        </span>
        <span className="w-16" role="columnheader">
          Map Type
        </span>
        <span className="flex-1 truncate" role="columnheader">
          Device
        </span>
        <span className="w-16 text-right" role="columnheader">
          Latency
        </span>
      </div>

      {/* Virtual Scrolling List */}
      <List
        ref={listRef}
        height={height}
        itemCount={displayEvents.length}
        itemSize={rowHeight}
        width="100%"
        className="bg-slate-900"
        role="rowgroup"
      >
        {EventRow}
      </List>
    </div>
  );
};

export default EventLogList;
