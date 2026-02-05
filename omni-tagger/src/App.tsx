import { useState, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import Settings from "./components/Settings";
import Overlay from "./components/Overlay";
import "./App.css";

function App() {
  const [view, setView] = useState<'settings' | 'overlay'>('settings');
  const [screenIndex, setScreenIndex] = useState<number>(0);
  const [lastResult, setLastResult] = useState<string | null>(null);

  useEffect(() => {
    const appWindow = getCurrentWindow();
    const label = appWindow.label;

    if (label.startsWith('overlay-')) {
        const index = parseInt(label.split('-')[1], 10);
        setScreenIndex(index);
        setView('overlay');
    } else {
        setView('settings');
        // Listen for tag results in main window
        const unlisten = listen<string>('tag-generated', (event) => {
            setLastResult(event.payload);
        });
        return () => {
            unlisten.then(f => f());
        };
    }
  }, []);

  return (
    <div className="w-full h-full font-sans">
      {view === 'settings' && (
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
      )}

      {view === 'overlay' && (
          // @ts-ignore - Overlay props will be updated in next step
          <Overlay
            screenIndex={screenIndex}
            onClose={() => getCurrentWindow().close()}
          />
      )}
    </div>
  );
}

export default App;
