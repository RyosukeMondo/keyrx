/**
 * StateIndicatorPanel Component
 *
 * Displays active modifiers, locks, and current layer as color-coded badges
 * with animations on state changes.
 */

import { useDashboardStore } from '../store/dashboardStore';
import './StateIndicatorPanel.css';

/**
 * State indicator panel with color-coded badges
 *
 * Badge colors:
 * - Modifiers: Blue (#2196f3)
 * - Locks: Orange (#ff9800)
 * - Layer: Green (#4caf50)
 *
 * Animations:
 * - Fade in when badge appears
 * - Pulse on state change
 */
export function StateIndicatorPanel() {
  const currentState = useDashboardStore((state) => state.currentState);

  return (
    <div className="state-indicator-panel">
      {/* Modifiers section */}
      <div className="state-section">
        <h3 className="state-section-title">Active Modifiers</h3>
        <div className="badge-container" role="group" aria-label="Active modifiers">
          {currentState.modifiers.length === 0 ? (
            <span className="empty-state" aria-live="polite">
              No active modifiers
            </span>
          ) : (
            currentState.modifiers.map((mod) => (
              <span
                key={mod}
                className="state-badge state-badge-modifier"
                role="status"
                aria-label={`Modifier ${mod} active`}
              >
                {mod}
              </span>
            ))
          )}
        </div>
      </div>

      {/* Locks section */}
      <div className="state-section">
        <h3 className="state-section-title">Active Locks</h3>
        <div className="badge-container" role="group" aria-label="Active locks">
          {currentState.locks.length === 0 ? (
            <span className="empty-state" aria-live="polite">
              No active locks
            </span>
          ) : (
            currentState.locks.map((lock) => (
              <span
                key={lock}
                className="state-badge state-badge-lock"
                role="status"
                aria-label={`Lock ${lock} active`}
              >
                {lock}
              </span>
            ))
          )}
        </div>
      </div>

      {/* Layer section */}
      <div className="state-section">
        <h3 className="state-section-title">Current Layer</h3>
        <div className="badge-container" role="group" aria-label="Current layer">
          <span
            className="state-badge state-badge-layer"
            role="status"
            aria-label={`Layer ${currentState.layer} active`}
          >
            {currentState.layer}
          </span>
        </div>
      </div>
    </div>
  );
}
