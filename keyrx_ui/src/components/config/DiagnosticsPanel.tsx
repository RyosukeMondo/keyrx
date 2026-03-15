/**
 * DiagnosticsPanel - Collapsible debug panel for the config page
 *
 * Shows internal state at a glance: connection, profile, mappings, layout, sync.
 * Provides copy-to-clipboard and log level controls for quick debugging.
 *
 * Default: collapsed (fixed bottom-right button).
 * Expanded: shows formatted JSON of all diagnostic data.
 */

import React, { useState, useCallback, useMemo } from 'react';
import { ReadyState } from 'react-use-websocket';
import type { RhaiSyncEngineResult } from '@/components/RhaiSyncEngine';
import type { SyncStatus } from '@/hooks/useConfigSync';
import type { KeyMapping } from '@/types';
import { env } from '@/config/env';

type LogLevel = 'error' | 'warn' | 'info' | 'debug';

const READY_STATE_LABELS: Record<number, string> = {
  [ReadyState.CONNECTING]: 'CONNECTING',
  [ReadyState.OPEN]: 'OPEN',
  [ReadyState.CLOSING]: 'CLOSING',
  [ReadyState.CLOSED]: 'CLOSED',
};

export interface DiagnosticsPanelProps {
  isConnected: boolean;
  readyState: ReadyState;
  lastError: Error | null;
  selectedProfile: string;
  profileConfig: { source: string } | undefined;
  syncEngine: RhaiSyncEngineResult;
  syncStatus: SyncStatus;
  lastSaveTime: Date | null;
  configStore: {
    activeLayer: string;
    globalSelected: boolean;
    selectedDevices: string[];
    getLayerMappings: (layer: string) => Map<string, KeyMapping>;
    getAllLayers: () => string[];
  };
  keyboardLayout: string;
  layoutKeyCount: number;
  /** Detected keyboard layout from daemon */
  detectedLayout?: string;
}

function buildDiagnosticData(props: DiagnosticsPanelProps) {
  return {
    connection: {
      isConnected: props.isConnected,
      readyState: READY_STATE_LABELS[props.readyState] || 'UNKNOWN',
      lastError: props.lastError?.message || null,
    },
    profile: {
      name: props.selectedProfile,
      configLoaded: !!props.profileConfig?.source,
      sourceLength: props.profileConfig?.source?.length || 0,
      parseState: props.syncEngine.state,
      parseError: props.syncEngine.error,
    },
    mappings: {
      baseMappingCount: props.configStore.getLayerMappings('base').size,
      availableLayers: props.configStore.getAllLayers(),
      activeLayer: props.configStore.activeLayer,
      globalSelected: props.configStore.globalSelected,
      selectedDevices: props.configStore.selectedDevices,
    },
    layout: {
      type: props.keyboardLayout,
      keyCount: props.layoutKeyCount,
      detectedLayout: props.detectedLayout || 'Not detected',
    },
    sync: {
      status: props.syncStatus,
      lastSaveTime: props.lastSaveTime?.toISOString() || null,
    },
  };
}

export const DiagnosticsPanel: React.FC<DiagnosticsPanelProps> = (props) => {
  const [expanded, setExpanded] = useState(false);
  const [logLevel, setLogLevel] = useState<LogLevel>('info');
  const [copyFeedback, setCopyFeedback] = useState(false);

  const diagnosticData = useMemo(() => buildDiagnosticData(props), [
    props.isConnected,
    props.readyState,
    props.lastError,
    props.selectedProfile,
    props.profileConfig,
    props.syncEngine.state,
    props.syncEngine.error,
    props.syncStatus,
    props.lastSaveTime,
    props.configStore.activeLayer,
    props.configStore.globalSelected,
    props.configStore.selectedDevices,
    props.keyboardLayout,
    props.layoutKeyCount,
    props.configStore,
  ]);

  const handleCopy = useCallback(() => {
    const json = JSON.stringify(diagnosticData, null, 2);
    navigator.clipboard.writeText(json).then(() => {
      setCopyFeedback(true);
      setTimeout(() => setCopyFeedback(false), 1500);
    }).catch((err) => {
      console.error('Failed to copy diagnostics:', err);
    });
  }, [diagnosticData]);

  const handleLogLevel = useCallback(async (level: LogLevel) => {
    try {
      await fetch(`${env.apiUrl}/api/debug/log-level`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ level }),
      });
      setLogLevel(level);
    } catch (e) {
      console.error('Failed to set log level:', e);
    }
  }, []);

  const statusColor = props.isConnected ? '#4ade80' : '#ef4444';

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
          border: `1px solid ${statusColor}`,
          borderRadius: 6,
          padding: '4px 12px',
          cursor: 'pointer',
          fontSize: 12,
          fontFamily: 'monospace',
          display: 'flex',
          alignItems: 'center',
          gap: 6,
        }}
        aria-label="Open debug panel"
      >
        <span
          style={{
            display: 'inline-block',
            width: 8,
            height: 8,
            borderRadius: '50%',
            background: statusColor,
          }}
        />
        Debug
      </button>
    );
  }

  const logLevels: LogLevel[] = ['error', 'warn', 'info', 'debug'];

  return (
    <div
      style={{
        position: 'fixed',
        bottom: 0,
        right: 0,
        width: 480,
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
      aria-label="Debug panel"
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
        }}
      >
        <span style={{ color: '#e2e8f0', fontWeight: 'bold', display: 'flex', alignItems: 'center', gap: 6 }}>
          <span
            style={{
              display: 'inline-block',
              width: 8,
              height: 8,
              borderRadius: '50%',
              background: statusColor,
            }}
          />
          Debug Panel
        </span>
        <div style={{ display: 'flex', gap: 4 }}>
          {logLevels.map((level) => (
            <button
              key={level}
              onClick={() => handleLogLevel(level)}
              style={{
                padding: '2px 6px',
                fontSize: 10,
                cursor: 'pointer',
                borderRadius: 3,
                border: '1px solid #475569',
                background: logLevel === level ? '#3b82f6' : '#1e293b',
                color: logLevel === level ? '#fff' : '#94a3b8',
                fontFamily: 'monospace',
              }}
              aria-label={`Set log level to ${level}`}
              aria-pressed={logLevel === level}
            >
              {level}
            </button>
          ))}
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
              transition: 'background 0.2s',
            }}
            aria-label="Copy diagnostics to clipboard"
          >
            {copyFeedback ? 'Copied' : 'Copy'}
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
      {/* Content */}
      <pre
        style={{
          margin: 0,
          padding: 12,
          overflow: 'auto',
          flex: 1,
          color: '#e2e8f0',
          fontFamily: 'monospace',
          fontSize: 11,
          lineHeight: 1.5,
        }}
      >
        {JSON.stringify(diagnosticData, null, 2)}
      </pre>
    </div>
  );
};
