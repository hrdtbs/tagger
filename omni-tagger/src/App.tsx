import { useState, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import Settings from "./components/Settings";
import Overlay from "./components/Overlay";
import "./App.css";

function App() {
  const [view, setView] = useState<'settings' | 'overlay'>('settings');
  const [processing, setProcessing] = useState(false);
  const [lastResult, setLastResult] = useState<string | null>(null);

  useEffect(() => {
    const appWindow = getCurrentWindow();
    if (view === 'overlay') {
      appWindow.setFullscreen(true);
    } else {
      appWindow.setFullscreen(false);
    }
  }, [view]);

  useEffect(() => {
    const unlisten = listen('show-overlay', () => {
      console.log("Show overlay event received");
      setView('overlay');
    });

    return () => {
      unlisten.then(f => f());
    };
  }, []);

  const handleProcess = async (selection: {x: number, y: number, w: number, h: number}) => {
      setProcessing(true);
      try {
          const tags = await invoke<string>('process_selection', {
              x: Math.round(selection.x),
              y: Math.round(selection.y),
              w: Math.round(selection.w),
              h: Math.round(selection.h)
          });

          setLastResult(tags);

          // Return to settings
          setView('settings');

      } catch (e) {
          console.error(e);
          alert("Error processing: " + e);
      } finally {
          setProcessing(false);
      }
  };

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
          <Overlay
            onClose={() => setView('settings')}
            onProcess={handleProcess}
          />
      )}

      {processing && (
          <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-[100] text-white font-bold text-xl">
              Processing...
          </div>
      )}
    </div>
  );
}

export default App;
