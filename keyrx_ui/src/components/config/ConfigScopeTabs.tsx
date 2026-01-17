import React from 'react';

interface ConfigScopeTabsProps {
  activePane: 'global' | 'device';
  onPaneChange: (pane: 'global' | 'device') => void;
}

/**
 * Accessible tab navigation for switching between global and device configuration views
 */
export const ConfigScopeTabs: React.FC<ConfigScopeTabsProps> = ({
  activePane,
  onPaneChange,
}) => {
  return (
    <div
      role="tablist"
      aria-label="Keyboard configuration scope"
      className="flex gap-2 border-b border-slate-700"
      data-testid="pane-switcher"
    >
      <button
        role="tab"
        aria-selected={activePane === 'global'}
        aria-controls="panel-global"
        id="tab-global"
        onClick={() => onPaneChange('global')}
        data-testid="pane-global"
        className={`flex-1 px-4 py-2 font-medium transition-colors focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-2 focus:ring-offset-slate-900 ${
          activePane === 'global'
            ? 'text-primary-400 border-b-2 border-primary-400'
            : 'text-slate-400 hover:text-slate-300'
        }`}
      >
        Global Keys
      </button>
      <button
        role="tab"
        aria-selected={activePane === 'device'}
        aria-controls="panel-device"
        id="tab-device"
        onClick={() => onPaneChange('device')}
        data-testid="pane-device"
        className={`flex-1 px-4 py-2 font-medium transition-colors focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-2 focus:ring-offset-slate-900 ${
          activePane === 'device'
            ? 'text-primary-400 border-b-2 border-primary-400'
            : 'text-slate-400 hover:text-slate-300'
        }`}
      >
        Device Keys
      </button>
    </div>
  );
};
