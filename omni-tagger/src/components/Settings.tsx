import { useState, useEffect } from 'react';
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from '@tauri-apps/plugin-dialog';

interface DownloadProgress {
    file: string;
    total: number;
    downloaded: number;
    percent: number;
}

interface AppConfig {
  model_path: string;
  tags_path: string;
  threshold: number;
  use_underscore: boolean;
  exclusion_list: string[];
}

const PRESETS = [
    { name: "WD14 SwinV2 (Default)", path: "models/model.onnx", url: "https://huggingface.co/SmilingWolf/wd-v1-4-swinv2-tagger-v2/resolve/main/model.onnx" },
    { name: "WD14 ConvNext", path: "models/convnext.onnx", url: "https://huggingface.co/SmilingWolf/wd-v1-4-convnext-tagger-v2/resolve/main/model.onnx" },
    { name: "WD14 ConvNextV2", path: "models/convnextv2.onnx", url: "https://huggingface.co/SmilingWolf/wd-v1-4-convnextv2-tagger-v2/resolve/main/model.onnx" }
];
const TAGS_URL = "https://huggingface.co/SmilingWolf/wd-v1-4-swinv2-tagger-v2/resolve/main/selected_tags.csv";
const TAGS_PATH = "models/tags.csv";

export default function Settings() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [loading, setLoading] = useState(true);
  const [exclusionText, setExclusionText] = useState("");
  const [downloadProgress, setDownloadProgress] = useState<DownloadProgress | null>(null);
  const [modelStatus, setModelStatus] = useState<'checking' | 'present' | 'missing'>('checking');

  useEffect(() => {
    invoke<AppConfig>('get_config')
      .then(c => {
        setConfig(c);
        setExclusionText(c.exclusion_list.join(", "));
        setLoading(false);
      })
      .catch(e => {
        console.error("Failed to load config", e);
        setLoading(false);
      });

    const unlistenProgress = listen<DownloadProgress>('model-download-progress', (event) => {
        setDownloadProgress(event.payload);
    });

    const unlistenFinished = listen('model-download-finished', () => {
        setDownloadProgress(null);
        if (config) checkModel(config.model_path);
    });

    return () => {
        unlistenProgress.then(f => f());
        unlistenFinished.then(f => f());
    };
  }, []);

  // Check model status when config.model_path changes
  useEffect(() => {
    if (config) {
        checkModel(config.model_path);
    }
  }, [config?.model_path]);

  const checkModel = async (path: string) => {
      setModelStatus('checking');
      try {
          const exists = await invoke<boolean>('check_model_exists', { pathStr: path });
          setModelStatus(exists ? 'present' : 'missing');
      } catch (e) {
          console.error("Failed to check model", e);
          setModelStatus('missing');
      }
  };

  const downloadCurrentModel = async () => {
      if (!config) return;
      const preset = PRESETS.find(p => p.path === config.model_path);
      if (!preset) return;

      try {
          await invoke('download_new_model', { url: preset.url, pathStr: preset.path });
          // Ensure tags exist too
          const tagsExists = await invoke<boolean>('check_model_exists', { pathStr: config.tags_path });
          if (!tagsExists) {
               await invoke('download_new_model', { url: TAGS_URL, pathStr: TAGS_PATH });
          }
      } catch (e) {
          console.error("Failed to download", e);
          alert("Download failed: " + e);
      }
  };

  useEffect(() => {
      if (config) {
          // This ensures that if config is updated (e.g. after save), the text area reflects the formatted list.
          // However, we must be careful not to override while typing if we were saving on change.
          // Since we save on Blur, this update happens after Blur, which is fine.
          // It reformats "tag,  tag2" to "tag, tag2".
          setExclusionText(config.exclusion_list.join(", "));
      }
  }, [config]);

  const saveConfig = async (newConfig: AppConfig) => {
      setConfig(newConfig);
      try {
          await invoke('set_config', { config: newConfig });
      } catch (e) {
          console.error("Failed to save config", e);
          alert("Failed to save config: " + e);
      }
  };

  const updateField = <K extends keyof AppConfig>(key: K, value: AppConfig[K]) => {
      if (!config) return;
      const newConfig = { ...config, [key]: value };
      saveConfig(newConfig);
  };

  if (loading) return <div className="p-6">Loading settings...</div>;
  if (!config) return <div className="p-6 text-red-500">Failed to load configuration.</div>;

  return (
    <div className="p-6 bg-gray-100 min-h-screen text-gray-800">
      <h1 className="text-2xl font-bold mb-4">OmniTagger Settings</h1>

      {downloadProgress && (
        <div className="bg-blue-100 border-l-4 border-blue-500 text-blue-700 p-4 mb-4" role="alert">
            <p className="font-bold">Downloading Model...</p>
            <p className="text-sm mb-2">{downloadProgress.file}: {downloadProgress.percent.toFixed(1)}%</p>
            <div className="w-full bg-blue-200 rounded-full h-2.5 dark:bg-blue-200">
                <div className="bg-blue-600 h-2.5 rounded-full" style={{ width: `${downloadProgress.percent}%` }}></div>
            </div>
        </div>
      )}

      {/* Model Selection */}
      <div className="bg-white p-4 rounded shadow mb-4">
        <h2 className="text-lg font-semibold mb-2">Model Selection</h2>

        <div className="mb-4">
            <label className="block text-sm font-medium text-gray-700 mb-1">Model Preset</label>
             <select
                value={PRESETS.find(p => p.path === config.model_path)?.path || "custom"}
                onChange={(e) => {
                    const val = e.target.value;
                    if (val === "custom") {
                        // Do nothing, let user browse or keep current custom
                    } else {
                        const preset = PRESETS.find(p => p.path === val);
                        if (preset) {
                             const newConfig = { ...config, model_path: preset.path, tags_path: TAGS_PATH };
                             saveConfig(newConfig);
                        }
                    }
                }}
                className="w-full p-2 border rounded bg-white mb-2 cursor-pointer"
             >
                 {PRESETS.map(p => (
                     <option key={p.path} value={p.path}>{p.name}</option>
                 ))}
                 <option value="custom">Custom</option>
             </select>

             {/* Download Button if missing */}
             {modelStatus === 'missing' && PRESETS.some(p => p.path === config.model_path) && (
                 <div className="mb-2 p-2 bg-yellow-50 text-yellow-800 border border-yellow-200 rounded flex items-center justify-between">
                     <span>Model file not found locally.</span>
                     <button
                        onClick={downloadCurrentModel}
                        disabled={!!downloadProgress}
                        className="bg-blue-600 text-white px-3 py-1 rounded text-sm hover:bg-blue-700 disabled:opacity-50 cursor-pointer"
                     >
                         {downloadProgress ? "Downloading..." : "Download Model"}
                     </button>
                 </div>
             )}
        </div>

        <div className="mb-4">
            <label className="block text-sm font-medium text-gray-700 mb-1">Model Path (.onnx)</label>
            <div className="flex gap-2">
                <input
                    type="text"
                    value={config.model_path}
                    readOnly
                    className="flex-1 p-2 border rounded bg-gray-50 text-sm"
                />
                <button
                    onClick={async () => {
                        const selected = await open({
                            filters: [{ name: 'ONNX Model', extensions: ['onnx'] }]
                        });
                        if (selected && typeof selected === 'string') {
                            updateField('model_path', selected);
                        }
                    }}
                    className="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700 cursor-pointer"
                >
                    Browse
                </button>
            </div>
        </div>

        <div className="mb-2">
            <label className="block text-sm font-medium text-gray-700 mb-1">Tags File (.csv)</label>
            <div className="flex gap-2">
                <input
                    type="text"
                    value={config.tags_path}
                    readOnly
                    className="flex-1 p-2 border rounded bg-gray-50 text-sm"
                />
                <button
                    onClick={async () => {
                        const selected = await open({
                            filters: [{ name: 'CSV File', extensions: ['csv'] }]
                        });
                        if (selected && typeof selected === 'string') {
                            updateField('tags_path', selected);
                        }
                    }}
                    className="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700 cursor-pointer"
                >
                    Browse
                </button>
            </div>
        </div>
      </div>

      {/* Threshold */}
      <div className="bg-white p-4 rounded shadow mb-4">
        <h2 className="text-lg font-semibold mb-2">Confidence Threshold: {config.threshold}</h2>
        <input
          type="range"
          min="0"
          max="1"
          step="0.01"
          value={config.threshold}
          onChange={(e) => {
              // We update config state immediately for UI responsiveness,
              // but we should ideally debounce the save.
              // For now, save immediately (might spam backend/fs).
              updateField('threshold', parseFloat(e.target.value));
          }}
          className="w-full cursor-pointer"
        />
        <div className="flex justify-between text-xs text-gray-500">
            <span>0.0</span>
            <span>1.0</span>
        </div>
      </div>

      {/* Formatting */}
      <div className="bg-white p-4 rounded shadow mb-4">
        <h2 className="text-lg font-semibold mb-2">Formatting</h2>
        <label className="flex items-center space-x-2 cursor-pointer">
          <input
            type="checkbox"
            checked={config.use_underscore}
            onChange={(e) => updateField('use_underscore', e.target.checked)}
            className="w-4 h-4 text-blue-600 rounded focus:ring-blue-500"
          />
          <span>Use Underscores (e.g. <code>long_hair</code> vs <code>long hair</code>)</span>
        </label>
      </div>

      {/* Exclusion List */}
      <div className="bg-white p-4 rounded shadow mb-4">
        <h2 className="text-lg font-semibold mb-2">Excluded Tags</h2>
        <p className="text-sm text-gray-500 mb-2">Comma separated list of tags to ignore.</p>
        <textarea
            value={exclusionText}
            onChange={(e) => setExclusionText(e.target.value)}
            onBlur={() => {
                 const list = exclusionText.split(",").map(s => s.trim()).filter(s => s.length > 0);
                 updateField('exclusion_list', list);
            }}
            className="w-full p-2 border rounded h-24 font-mono text-sm"
            placeholder="bad_hands, lowres, ..."
        />
      </div>
    </div>
  );
}
