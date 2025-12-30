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
            <h2>Event Timeline</h2>
            <div className="panel-content">
              <EventTimeline />
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

/**
 * Placeholder for EventTimeline component
 * Will be replaced with actual implementation in task 9
 */
function EventTimeline() {
  const events = useDashboardStore((state) => state.events);

  return (
    <div className="event-timeline-placeholder">
      {events.length === 0 ? (
        <div className="empty-state">No events yet</div>
      ) : (
        <div className="event-list">
          {events.slice().reverse().map((event, idx) => (
            <div key={`${event.timestamp}-${idx}`} className="event-item">
              <span className="event-time">
                {new Date(event.timestamp / 1000).toLocaleTimeString()}
              </span>
              <span className="event-type">{event.eventType}</span>
              <span className="event-key">{event.input} → {event.output}</span>
              <span className="event-latency">{(event.latency / 1000).toFixed(2)}ms</span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
