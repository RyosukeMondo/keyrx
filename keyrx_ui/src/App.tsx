import { useState } from 'react'
import './App.css'
import { DeviceList } from './components/DeviceList'
import { SimulatorPanel } from './components/Simulator/SimulatorPanel'

type ActiveView = 'devices' | 'simulator'

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
        </nav>
      </header>
      <main>
        {activeView === 'devices' && <DeviceList />}
        {activeView === 'simulator' && <SimulatorPanel />}
      </main>
    </div>
  )
}

export default App
