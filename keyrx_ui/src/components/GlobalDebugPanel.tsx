/**
 * GlobalDebugPanel - App-wide console + API log viewer
 *
 * Captures:
 * - console.log/info/warn/error
 * - All fetch() requests and responses (method, url, status)
 * - Uncaught errors and unhandled promise rejections
 *
 * Available on every page via Layout.
 * Default: collapsed (small button). Expanded: scrollable log feed.
 */

import React, { useState, useEffect, useRef, useCallback } from 'react';
import { env } from '@/config/env';

type LogLevel = 'log' | 'info' | 'warn' | 'error';

interface LogEntry {
  id: number;
  level: LogLevel;
  timestamp: string;
  message: string;
}

const MAX_ENTRIES = 500;

const LEVEL_COLORS: Record<LogLevel, string> = {
  log: '#94a3b8',
  info: '#38bdf8',
  warn: '#fbbf24',
  error: '#f87171',
};

const LEVEL_BADGES: Record<LogLevel, string> = {
  log: 'LOG',
  info: 'INF',
  warn: 'WRN',
  error: 'ERR',
};

let nextId = 0;

/** Shared log store so entries survive re-mounts */
const logStore: LogEntry[] = [];
const listeners = new Set<() => void>();

function pushEntry(level: LogLevel, args: unknown[]) {
  const message = args
    .map((a) => {
      if (typeof a === 'string') return a;
      try {
        return JSON.stringify(a, null, 2);
      } catch {
        return String(a);
      }
    })
    .join(' ');

  const entry: LogEntry = {
    id: nextId++,
    level,
    timestamp: new Date().toISOString().slice(11, 23),
    message,
  };

  logStore.push(entry);
  if (logStore.length > MAX_ENTRIES) {
    logStore.splice(0, logStore.length - MAX_ENTRIES);
  }
  listeners.forEach((fn) => fn());
}

// === Install interceptors immediately at module scope ===

const origLog = console.log.bind(console);
const origInfo = console.info.bind(console);
const origWarn = console.warn.bind(console);
const origError = console.error.bind(console);

console.log = (...args: unknown[]) => {
  origLog(...args);
  pushEntry('log', args);
};
console.info = (...args: unknown[]) => {
  origInfo(...args);
  pushEntry('info', args);
};
console.warn = (...args: unknown[]) => {
  origWarn(...args);
  pushEntry('warn', args);
};
console.error = (...args: unknown[]) => {
  origError(...args);
  pushEntry('error', args);
};

// Intercept fetch to log all API requests/responses
const origFetch = window.fetch.bind(window);
window.fetch = async (input: RequestInfo | URL, init?: RequestInit) => {
  const method = init?.method || 'GET';
  const url = typeof input === 'string' ? input : input instanceof URL ? input.href : input.url;

  // Only log API calls (skip static assets)
  const isApi = url.includes('/api/') || url.includes('/ws');
  if (!isApi) {
    return origFetch(input, init);
  }

  const shortUrl = url.replace(/^https?:\/\/[^/]+/, '');
  pushEntry('info', [`→ ${method} ${shortUrl}`]);

  try {
    const response = await origFetch(input, init);

    if (response.ok) {
      pushEntry('info', [`← ${response.status} ${shortUrl}`]);
    } else {
      // Clone so the original consumer can still read the body
      const clone = response.clone();
      let errorBody = '';
      try {
        const json = await clone.json();
        errorBody = json.message || json.error || JSON.stringify(json);
      } catch {
        errorBody = await clone.text().catch(() => '');
      }
      pushEntry('error', [`← ${response.status} ${shortUrl}: ${errorBody}`]);
    }

    return response;
  } catch (err) {
    pushEntry('error', [`← FETCH FAILED ${shortUrl}: ${err}`]);
    throw err;
  }
};

// Capture uncaught errors
window.addEventListener('error', (event) => {
  pushEntry('error', [`Uncaught: ${event.message} at ${event.filename}:${event.lineno}`]);
});

window.addEventListener('unhandledrejection', (event) => {
  const reason = event.reason instanceof Error ? event.reason.message : String(event.reason);
  pushEntry('error', [`Unhandled rejection: ${reason}`]);
});

pushEntry('info', ['Debug panel initialized']);

// === React component ===

function useLogEntries(): LogEntry[] {
  const [, setTick] = useState(0);

  useEffect(() => {
    const cb = () => setTick((t) => t + 1);
    listeners.add(cb);
    return () => { listeners.delete(cb); };
  }, []);

  return logStore;
}

type DaemonLogLevel = 'error' | 'warn' | 'info' | 'debug';

export const GlobalDebugPanel: React.FC = () => {
  const entries = useLogEntries();
  const [expanded, setExpanded] = useState(false);
  const [filter, setFilter] = useState<LogLevel | 'all'>('all');
  const [daemonLevel, setDaemonLevel] = useState<DaemonLogLevel>('info');
  const [copyFeedback, setCopyFeedback] = useState(false);
  const scrollRef = useRef<HTMLDivElement>(null);

  // Auto-scroll to bottom on new entries
  useEffect(() => {
    if (expanded && scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [entries.length, expanded]);

  const filtered = filter === 'all'
    ? entries
    : entries.filter((e) => e.level === filter);

  const errorCount = entries.filter((e) => e.level === 'error').length;
  const warnCount = entries.filter((e) => e.level === 'warn').length;

  const handleClear = useCallback(() => {
    logStore.length = 0;
    listeners.forEach((fn) => fn());
  }, []);

  const handleCopy = useCallback(() => {
    const text = filtered
      .map((e) => `[${e.timestamp}] ${LEVEL_BADGES[e.level]} ${e.message}`)
      .join('\n');
    navigator.clipboard.writeText(text).then(() => {
      setCopyFeedback(true);
      setTimeout(() => setCopyFeedback(false), 1500);
    }).catch(() => {});
  }, [filtered]);

  const handleDaemonLevel = useCallback(async (level: DaemonLogLevel) => {
    try {
      await fetch(`${env.apiUrl}/api/debug/log-level`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ level }),
      });
      setDaemonLevel(level);
    } catch {
      // error will appear in the log panel via fetch interceptor
    }
  }, []);

  const badgeColor = errorCount > 0
    ? '#ef4444'
    : warnCount > 0
      ? '#fbbf24'
      : '#4ade80';

  if (!expanded) {
    return (
      <button
        onClick={() => setExpanded(true)}
        style={{
          position: 'fixed',
          bottom: 8,
          right: 8,
          zIndex: 9999,
          background: '#1e293b',
          color: '#94a3b8',
          border: `1px solid ${badgeColor}`,
          borderRadius: 6,
          padding: '4px 12px',
          cursor: 'pointer',
          fontSize: 12,
          fontFamily: 'monospace',
          display: 'flex',
          alignItems: 'center',
          gap: 6,
        }}
        aria-label={`Open debug panel (${errorCount} errors, ${warnCount} warnings)`}
      >
        <span
          style={{
            display: 'inline-block',
            width: 8,
            height: 8,
            borderRadius: '50%',
            background: badgeColor,
          }}
        />
        Debug
        {errorCount > 0 && (
          <span style={{ color: '#f87171', fontWeight: 'bold' }}>
            {errorCount}
          </span>
        )}
      </button>
    );
  }

  const filterOptions: Array<LogLevel | 'all'> = ['all', 'error', 'warn', 'info', 'log'];
  const daemonLevels: DaemonLogLevel[] = ['error', 'warn', 'info', 'debug'];

  return (
    <div
      style={{
        position: 'fixed',
        bottom: 0,
        right: 0,
        width: 560,
        maxHeight: '50vh',
        background: '#0f172a',
        border: '1px solid #334155',
        borderRadius: '8px 0 0 0',
        zIndex: 9999,
        display: 'flex',
        flexDirection: 'column',
        fontSize: 12,
        fontFamily: 'monospace',
      }}
      role="region"
      aria-label="Console log panel"
    >
      {/* Header */}
      <div
        style={{
          display: 'flex',
          justifyContent: 'space-between',
          padding: '6px 12px',
          borderBottom: '1px solid #334155',
          alignItems: 'center',
          flexShrink: 0,
          flexWrap: 'wrap',
          gap: 4,
        }}
      >
        <span style={{
          color: '#e2e8f0',
          fontWeight: 'bold',
          display: 'flex',
          alignItems: 'center',
          gap: 6,
        }}>
          <span style={{
            display: 'inline-block',
            width: 8,
            height: 8,
            borderRadius: '50%',
            background: badgeColor,
          }} />
          Console ({filtered.length})
        </span>
        <div style={{ display: 'flex', gap: 3, flexWrap: 'wrap' }}>
          {/* Filter buttons */}
          {filterOptions.map((f) => (
            <button
              key={f}
              onClick={() => setFilter(f)}
              style={{
                padding: '2px 6px',
                fontSize: 10,
                cursor: 'pointer',
                borderRadius: 3,
                border: '1px solid #475569',
                background: filter === f ? '#3b82f6' : '#1e293b',
                color: filter === f ? '#fff' : '#94a3b8',
                fontFamily: 'monospace',
              }}
              aria-label={`Filter ${f}`}
              aria-pressed={filter === f}
            >
              {f}
            </button>
          ))}
          <span style={{ width: 1, background: '#475569', margin: '0 2px' }} />
          {/* Daemon log level */}
          {daemonLevels.map((level) => (
            <button
              key={`d-${level}`}
              onClick={() => handleDaemonLevel(level)}
              style={{
                padding: '2px 6px',
                fontSize: 10,
                cursor: 'pointer',
                borderRadius: 3,
                border: `1px solid ${daemonLevel === level ? '#8b5cf6' : '#475569'}`,
                background: daemonLevel === level ? '#7c3aed' : '#1e293b',
                color: daemonLevel === level ? '#fff' : '#94a3b8',
                fontFamily: 'monospace',
              }}
              aria-label={`Set daemon log level to ${level}`}
              aria-pressed={daemonLevel === level}
              title={`Daemon: ${level}`}
            >
              D:{level}
            </button>
          ))}
          <span style={{ width: 1, background: '#475569', margin: '0 2px' }} />
          <button
            onClick={handleCopy}
            style={{
              padding: '2px 8px',
              fontSize: 10,
              cursor: 'pointer',
              background: copyFeedback ? '#22c55e' : '#1e293b',
              color: copyFeedback ? '#fff' : '#94a3b8',
              border: '1px solid #475569',
              borderRadius: 3,
              fontFamily: 'monospace',
            }}
            aria-label="Copy logs to clipboard"
          >
            {copyFeedback ? 'Copied' : 'Copy'}
          </button>
          <button
            onClick={handleClear}
            style={{
              padding: '2px 8px',
              fontSize: 10,
              cursor: 'pointer',
              background: '#1e293b',
              color: '#94a3b8',
              border: '1px solid #475569',
              borderRadius: 3,
              fontFamily: 'monospace',
            }}
            aria-label="Clear logs"
          >
            Clear
          </button>
          <button
            onClick={() => setExpanded(false)}
            style={{
              padding: '2px 8px',
              fontSize: 10,
              cursor: 'pointer',
              background: '#1e293b',
              color: '#94a3b8',
              border: '1px solid #475569',
              borderRadius: 3,
              fontFamily: 'monospace',
            }}
            aria-label="Close debug panel"
          >
            Close
          </button>
        </div>
      </div>
      {/* Log entries */}
      <div
        ref={scrollRef}
        style={{
          margin: 0,
          padding: 8,
          overflow: 'auto',
          flex: 1,
          fontFamily: 'monospace',
          fontSize: 11,
          lineHeight: 1.4,
        }}
      >
        {filtered.length === 0 && (
          <div style={{ color: '#475569', textAlign: 'center', padding: 16 }}>
            No logs captured yet
          </div>
        )}
        {filtered.map((entry) => (
          <div
            key={entry.id}
            style={{
              display: 'flex',
              gap: 8,
              padding: '2px 0',
              borderBottom: '1px solid #1e293b',
              alignItems: 'flex-start',
            }}
          >
            <span style={{ color: '#475569', flexShrink: 0 }}>
              {entry.timestamp}
            </span>
            <span
              style={{
                color: LEVEL_COLORS[entry.level],
                fontWeight: 'bold',
                flexShrink: 0,
                width: 28,
              }}
            >
              {LEVEL_BADGES[entry.level]}
            </span>
            <span
              style={{
                color: entry.level === 'error' ? '#fca5a5' : '#e2e8f0',
                whiteSpace: 'pre-wrap',
                wordBreak: 'break-all',
              }}
            >
              {entry.message}
            </span>
          </div>
        ))}
      </div>
    </div>
  );
};
