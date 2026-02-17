import { useState, useEffect } from 'react'
import { listen } from '@tauri-apps/api/event'
import Settings from './components/Settings'
import './App.css'

function App() {
  const [lastResult, setLastResult] = useState<string | null>(null)

  useEffect(() => {
    const unlisten = listen<string>('tag-generated', (event) => {
      setLastResult(event.payload)
    })
    return () => {
      unlisten.then((f) => f())
    }
  }, [])

  return (
    <div className="w-full h-full font-sans">
      <div className="relative">
        <Settings />
        {lastResult && (
          <div className="fixed bottom-4 right-4 bg-green-600 text-white p-4 rounded shadow-lg z-50">
            <p className="font-bold">Tags Copied!</p>
            <p className="text-sm truncate max-w-xs">{lastResult}</p>
            <button
              onClick={() => setLastResult(null)}
              className="absolute top-1 right-2 text-white font-bold"
            >
              ✕
            </button>
          </div>
        )}
      </div>
    </div>
  )
}

export default App
