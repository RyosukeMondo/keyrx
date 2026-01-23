/**
 * CodePanelContainer - Collapsible code editor panel with resize support
 *
 * This container component wraps the Monaco editor in a collapsible panel
 * with a draggable resize handle. It manages the panel's open/closed state
 * and height using the useCodePanel hook with localStorage persistence.
 *
 * Features:
 * - Collapsible panel with toggle button
 * - Draggable resize handle with mouse events
 * - Height persistence via useCodePanel
 * - Sync status indicators (parsing, generating, syncing)
 * - Parse error display with line/column info
 * - Fixed positioning at bottom of viewport
 *
 * @module CodePanelContainer
 */

import React from 'react';
import { MonacoEditor } from '../MonacoEditor';
import { useCodePanel } from '@/hooks/useCodePanel';
import type { RhaiSyncEngineResult } from '../RhaiSyncEngine';

interface CodePanelContainerProps {
  /** Name of the current profile */
  profileName: string;
  /** Current Rhai code content */
  rhaiCode: string;
  /** Callback when code changes */
  onChange: (code: string) => void;
  /** Sync engine instance for state and error management */
  syncEngine: RhaiSyncEngineResult;
  /** Whether the panel is open (controlled from parent) */
  isOpen: boolean;
  /** Callback to toggle panel open/closed state */
  onToggle: () => void;
}

/**
 * Collapsible code editor panel component
 *
 * @example
 * ```tsx
 * <CodePanelContainer
 *   profileName="Default"
 *   rhaiCode={syncEngine.getCode()}
 *   onChange={syncEngine.onCodeChange}
 *   syncEngine={syncEngine}
 * />
 * ```
 */
export const CodePanelContainer: React.FC<CodePanelContainerProps> = ({
  profileName,
  rhaiCode,
  onChange,
  syncEngine,
  isOpen,
  onToggle,
}) => {
  const { height, setHeight } = useCodePanel();

  /**
   * Handle mouse down on resize handle
   * Sets up mouse move and mouse up listeners for dragging
   */
  const handleResizeMouseDown = (e: React.MouseEvent<HTMLDivElement>) => {
    e.preventDefault();
    const startY = e.clientY;
    const startHeight = height;

    const handleMouseMove = (moveEvent: MouseEvent) => {
      const deltaY = startY - moveEvent.clientY;
      const newHeight = Math.max(200, Math.min(600, startHeight + deltaY));
      setHeight(newHeight);
    };

    const handleMouseUp = () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
  };

  if (!isOpen) {
    return null;
  }

  return (
    <div
      className="fixed bottom-0 left-0 right-0 bg-slate-800 border-t border-slate-600 shadow-2xl z-50 transition-all duration-300 ease-in-out"
      style={{ height: `${height}px` }}
      data-testid="code-panel-container"
    >
      {/* Header with collapse button */}
      <div className="flex items-center justify-between px-4 py-2 bg-slate-900/50 border-b border-slate-600">
        <h3 className="text-sm font-semibold text-slate-300">
          Code - {profileName}
        </h3>
        <button
          onClick={onToggle}
          className="px-3 py-1 text-xs text-slate-400 hover:text-slate-200 hover:bg-slate-700 rounded transition-colors"
          title="Hide code editor"
          aria-label="Hide code editor"
        >
          ▼ Hide
        </button>
      </div>

      {/* Resize Handle */}
      <div
        className="h-1 bg-slate-600 hover:bg-primary-500 cursor-ns-resize transition-colors"
        onMouseDown={handleResizeMouseDown}
        title="Drag to resize"
        aria-label="Resize handle"
      />

      {/* Code Panel Content */}
      <div className="h-full flex flex-col p-4 overflow-hidden">
        {/* Sync status indicators */}
        {syncEngine.state !== 'idle' && (
          <div className="flex items-center gap-2 px-4 py-2 mb-2 bg-slate-700 border border-slate-600 rounded-md">
            {syncEngine.state === 'parsing' && (
              <>
                <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-primary-400" />
                <span className="text-sm text-slate-300">
                  Parsing Rhai script...
                </span>
              </>
            )}
            {syncEngine.state === 'generating' && (
              <>
                <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-primary-400" />
                <span className="text-sm text-slate-300">
                  Generating code...
                </span>
              </>
            )}
            {syncEngine.state === 'syncing' && (
              <>
                <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-primary-400" />
                <span className="text-sm text-slate-300">Syncing...</span>
              </>
            )}
          </div>
        )}

        {/* Error display */}
        {syncEngine.error && (
          <div className="p-3 mb-2 bg-red-900/20 border border-red-500 rounded-md">
            <div className="flex items-start gap-3">
              <span className="text-red-400 text-lg">⚠️</span>
              <div className="flex-1">
                <h4 className="text-red-400 font-semibold text-sm mb-1">
                  Parse Error
                </h4>
                <p className="text-xs text-red-300 mb-1">
                  Line {syncEngine.error.line}, Column {syncEngine.error.column}
                  : {syncEngine.error.message}
                </p>
                {syncEngine.error.suggestion && (
                  <p className="text-xs text-slate-300 italic">
                    💡 {syncEngine.error.suggestion}
                  </p>
                )}
              </div>
              <button
                onClick={() => syncEngine.clearError()}
                className="text-slate-400 hover:text-slate-300 transition-colors"
                aria-label="Clear error"
              >
                ✕
              </button>
            </div>
          </div>
        )}

        {/* Code Editor with WASM validation */}
        <div className="flex-1 overflow-hidden" data-testid="code-editor">
          <MonacoEditor
            value={rhaiCode}
            onChange={onChange}
            height={`${
              height - calculateHeaderHeight(syncEngine.state, syncEngine.error)
            }px`}
          />
        </div>
      </div>
    </div>
  );
};

/**
 * Calculate header height based on sync state and error presence
 * Used to adjust Monaco editor height dynamically
 */
function calculateHeaderHeight(
  state: RhaiSyncEngineResult['state'],
  error: RhaiSyncEngineResult['error']
): number {
  let headerHeight = 60; // Base height (header + resize handle + padding)

  if (state !== 'idle') {
    headerHeight += 60; // Add sync status indicator height
  }

  if (error) {
    headerHeight += 80; // Add error display height
  }

  return headerHeight;
}
