import React, { useRef, useEffect, useState, useCallback } from 'react';
import { VariableSizeList as List } from 'react-window';
import { ChevronRight, ChevronDown } from 'lucide-react';

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
const COLLAPSED_HEIGHT = 40;
const EXPANDED_HEIGHT = 120;

export const EventLogList: React.FC<EventLogListProps> = ({
  events,
  maxEvents,
  height = 300,
  autoScroll = true,
}) => {
  const listRef = useRef<List>(null);
  const [expandedIndex, setExpandedIndex] = useState<number | null>(null);

  // Limit events if maxEvents is specified
  const displayEvents = maxEvents ? events.slice(-maxEvents) : events;

  // Reset expanded row when events change significantly
  useEffect(() => {
    setExpandedIndex(null);
  }, [displayEvents.length]);

  // Auto-scroll to top when new events arrive (newest first)
  useEffect(() => {
    if (autoScroll && listRef.current && displayEvents.length > 0) {
      listRef.current.scrollToItem(0, 'start');
    }
  }, [displayEvents.length, autoScroll]);

  const getItemSize = useCallback(
    (index: number) => (index === expandedIndex ? EXPANDED_HEIGHT : COLLAPSED_HEIGHT),
    [expandedIndex],
  );

  // Reset list cache when expanded row changes
  useEffect(() => {
    if (listRef.current) {
      listRef.current.resetAfterIndex(0);
    }
  }, [expandedIndex]);

  const handleRowClick = useCallback((index: number) => {
    setExpandedIndex((prev) => (prev === index ? null : index));
  }, []);

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

  const formatTimeFull = (timestamp: number): string => {
    return new Date(timestamp).toISOString();
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

    const isExpanded = index === expandedIndex;

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

    const ChevronIcon = isExpanded ? ChevronDown : ChevronRight;

    return (
      <div
        style={style}
        className="border-b border-slate-700"
        role="row"
      >
        <div
          className="flex items-center gap-3 px-4 h-10 text-sm font-mono cursor-pointer hover:bg-slate-700/50 select-none"
          title={`Device: ${event.deviceName || event.deviceId || 'Unknown'}`}
          onClick={() => handleRowClick(index)}
          onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') handleRowClick(index); }}
          tabIndex={0}
          role="button"
          aria-expanded={isExpanded}
          aria-label={`Event: ${event.type} ${formatKey(event.input || event.keyCode)}`}
        >
          <ChevronIcon className="w-3.5 h-3.5 text-slate-500 shrink-0" aria-hidden="true" />
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
          <span
            className={`w-6 text-center ${
              hasMappingTriggered ? 'text-green-400' : 'text-slate-600'
            }`}
            role="cell"
            aria-label={hasMappingTriggered ? 'Mapping triggered' : 'No mapping'}
          >
            {hasMappingTriggered ? '→' : '–'}
          </span>
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
          <span
            className="w-16 text-slate-500 text-xs truncate"
            title={event.mappingType || 'passthrough'}
            role="cell"
            aria-label={`Mapping type: ${event.mappingType || (wasRemapped ? 'remap' : 'passthrough')}`}
          >
            {event.mappingType || (wasRemapped ? 'remap' : '–')}
          </span>
          <span
            className="flex-1 text-slate-500 text-xs truncate"
            title={event.deviceName || event.deviceId}
            role="cell"
            aria-label={`Device: ${shortDeviceName}`}
          >
            {shortDeviceName}
          </span>
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

        {/* Expanded detail panel */}
        {isExpanded && (
          <div className="px-4 py-2 bg-slate-800/60 text-xs text-slate-300 grid grid-cols-2 md:grid-cols-4 gap-x-6 gap-y-1 font-mono">
            <div>
              <span className="text-slate-500">Timestamp: </span>
              {formatTimeFull(event.timestamp)}
            </div>
            <div>
              <span className="text-slate-500">Raw keyCode: </span>
              {event.keyCode}
            </div>
            <div>
              <span className="text-slate-500">Device ID: </span>
              {event.deviceId || '–'}
            </div>
            <div>
              <span className="text-slate-500">Device: </span>
              {event.deviceName || '–'}
            </div>
            <div>
              <span className="text-slate-500">Input: </span>
              {event.input || '–'}
            </div>
            <div>
              <span className="text-slate-500">Output: </span>
              {event.output || '–'}
            </div>
            <div>
              <span className="text-slate-500">Mapping: </span>
              {event.mappingType || 'none'}
              {event.mappingTriggered ? ' (triggered)' : ''}
            </div>
            <div>
              <span className="text-slate-500">Latency: </span>
              {formatLatency(event.latency)}
            </div>
          </div>
        )}
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
        <span className="w-3.5" aria-hidden="true" />
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
        itemSize={getItemSize}
        width="100%"
        className="bg-slate-900"
      >
        {EventRow}
      </List>
    </div>
  );
};

export default EventLogList;
