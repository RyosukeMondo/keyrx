import { useState } from 'react';
import Sidebar from './components/Sidebar';
import LayoutSelector from './components/LayoutSelector';
import LayoutEditor from './components/LayoutEditor';
import './index.css';

import type { LayoutConfig } from './types';

export type ViewState = 'selector' | 'editor';

function App() {
  const [view, setView] = useState<ViewState>('selector');
  const [currentLayout, setCurrentLayout] = useState<LayoutConfig | null>(null);
  const [savedLayouts, setSavedLayouts] = useState<LayoutConfig[]>([
    { id: '1', name: 'Gaming Profile', type: 'freeform', keys: [] },
    { id: '2', name: 'Work Setup', type: 'grid', keys: [] }
  ]);

  const handleLayoutSelect = (config: LayoutConfig) => {
    setCurrentLayout(config);
    setView('editor');
  };

  const handleSaveLayout = (layout: LayoutConfig) => {
    setSavedLayouts(prev => {
      const idx = prev.findIndex(l => l.id === layout.id);
      if (idx !== -1) {
        const newLayouts = [...prev];
        newLayouts[idx] = layout;
        return newLayouts;
      } else {
        return [...prev, layout];
      }
    });
    // Also update current layout to reflect saved state
    setCurrentLayout(layout);
  };

  const handleDeleteLayout = (id: string) => {
    setSavedLayouts(prev => prev.filter(l => l.id !== id));
  };

  const handleBack = () => {
    setView('selector');
    setCurrentLayout(null);
  };

  return (
    <>
      <Sidebar />
      <div style={{ flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
        {/* App Bar */}
        <div style={{
          height: '64px',
          display: 'flex',
          alignItems: 'center',
          padding: '0 24px',
          borderBottom: '1px solid var(--border-color)',
          backgroundColor: 'var(--bg-color)'
        }}>
          {view === 'selector' ? (
            <h1 style={{ fontSize: '22px', fontWeight: 400, margin: 0 }}>Layouts</h1>
          ) : (
            <input
              type="text"
              value={currentLayout?.name || ''}
              onChange={(e) => setCurrentLayout(prev => prev ? { ...prev, name: e.target.value } : null)}
              style={{
                fontSize: '22px',
                fontWeight: 400,
                margin: 0,
                background: 'transparent',
                border: 'none',
                color: 'var(--text-color)',
                outline: 'none',
                width: '100%'
              }}
            />
          )}
          <div style={{ flex: 1 }}></div>
          <button style={{ background: 'none', border: 'none', color: 'var(--text-secondary)', cursor: 'pointer' }}>
            ⟳
          </button>
        </div>

        {view === 'selector' ? (
          <LayoutSelector
            layouts={savedLayouts}
            onSelect={handleLayoutSelect}
            onDelete={handleDeleteLayout}
          />
        ) : (
          <LayoutEditor
            layout={currentLayout!}
            onBack={handleBack}
            onSave={handleSaveLayout}
          />
        )}
      </div>
    </>
  );
}

export default App;
