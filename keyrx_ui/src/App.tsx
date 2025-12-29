import { useState } from 'react'
import './App.css'
import { DeviceList } from './components/DeviceList'
import { SimulatorPanel } from './components/Simulator/SimulatorPanel'
import { ConfigurationPage } from './components/ConfigurationPage'
import { MacroRecorderPage } from './components/MacroRecorderPage'

type ActiveView = 'devices' | 'simulator' | 'config' | 'macros'

function App() {
  const [activeView, setActiveView] = useState<ActiveView>('devices')

  return (
    <div className="app">
      <header>
        <h1>KeyRX</h1>
        <p>Advanced Keyboard Remapping</p>
        <nav className="app-nav">
          <button
            className={activeView === 'devices' ? 'nav-button active' : 'nav-button'}
            onClick={() => setActiveView('devices')}
          >
            Devices
          </button>
          <button
            className={activeView === 'simulator' ? 'nav-button active' : 'nav-button'}
            onClick={() => setActiveView('simulator')}
          >
            Simulator
          </button>
          <button
            className={activeView === 'config' ? 'nav-button active' : 'nav-button'}
            onClick={() => setActiveView('config')}
          >
            Config Editor
          </button>
          <button
            className={activeView === 'macros' ? 'nav-button active' : 'nav-button'}
            onClick={() => setActiveView('macros')}
          >
            Macro Recorder
          </button>
        </nav>
      </header>
      <main>
        {activeView === 'devices' && <DeviceList />}
        {activeView === 'simulator' && <SimulatorPanel />}
        {activeView === 'config' && <ConfigurationPage />}
        {activeView === 'macros' && <MacroRecorderPage />}
      </main>
    </div>
  )
}

export default App
