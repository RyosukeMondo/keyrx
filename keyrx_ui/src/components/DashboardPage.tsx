/**
 * DashboardPage Component
 *
 * Real-time daemon monitoring dashboard with state indicators, metrics chart, and event timeline.
 * Connects to daemon via WebSocket and displays live updates.
 */

import { useDaemonWebSocket } from '../hooks/useDaemonWebSocket';
import { useDashboardStore } from '../store/dashboardStore';
import { StateIndicatorPanel } from './StateIndicatorPanel';
import { MetricsChart } from './MetricsChart';
import { DashboardEventTimeline } from './DashboardEventTimeline';
import './DashboardPage.css';

/**
 * Main dashboard page component
 *
 * Layout:
 * - Connection status banner
 * - State indicator panel (modifiers, locks, layer)
 * - Metrics chart (latency over time)
 * - Event timeline (last 100 events)
 */
export function DashboardPage() {
  // WebSocket connection
  const { isConnected, isConnecting, isDisconnected } = useDaemonWebSocket();

  // Dashboard state
  const connectionStatus = useDashboardStore((state) => state.connectionStatus);

  return (
    <div className="dashboard-page">
      {/* Connection status banner */}
      <div className={`connection-banner connection-${connectionStatus}`}>
        <div className="connection-status">
          {isConnecting && (
            <>
              <div className="status-indicator connecting" />
              <span>Connecting to daemon...</span>
            </>
          )}
          {isConnected && (
            <>
              <div className="status-indicator connected" />
              <span>Connected to daemon</span>
            </>
          )}
          {isDisconnected && (
            <>
              <div className="status-indicator disconnected" />
              <span>Disconnected - attempting to reconnect...</span>
            </>
          )}
        </div>
      </div>

      {/* Main dashboard content */}
      <div className="dashboard-content">
        <div className="dashboard-grid">
          {/* State indicator panel - top */}
          <div className="panel state-panel">
            <h2>Daemon State</h2>
            <div className="panel-content">
              <StateIndicatorPanel />
            </div>
          </div>

          {/* Metrics chart - middle left */}
          <div className="panel metrics-panel">
            <h2>Latency Metrics</h2>
            <div className="panel-content">
              <MetricsChart />
            </div>
          </div>

          {/* Event timeline - middle right */}
          <div className="panel events-panel">
            <DashboardEventTimeline height={500} />
          </div>
        </div>
      </div>
    </div>
  );
}
