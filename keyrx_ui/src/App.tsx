import './App.css'
import { DeviceList } from './components/DeviceList'

function App() {
  return (
    <div className="app">
      <header>
        <h1>KeyRX</h1>
        <p>Advanced Keyboard Remapping</p>
      </header>
      <main>
        <DeviceList />
      </main>
    </div>
  )
}

export default App
