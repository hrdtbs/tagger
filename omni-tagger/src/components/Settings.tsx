import { useState, useEffect, useRef, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
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
  {
    name: 'WD14 SwinV2 (Default)',
    path: 'models/model.onnx',
    url: 'https://huggingface.co/SmilingWolf/wd-v1-4-swinv2-tagger-v2/resolve/main/model.onnx',
  },
  {
    name: 'WD14 ConvNext',
    path: 'models/convnext.onnx',
    url: 'https://huggingface.co/SmilingWolf/wd-v1-4-convnext-tagger-v2/resolve/main/model.onnx',
  },
  {
    name: 'WD14 ConvNextV2',
    path: 'models/convnextv2.onnx',
    url: 'https://huggingface.co/SmilingWolf/wd-v1-4-convnextv2-tagger-v2/resolve/main/model.onnx',
  },
];
const TAGS_URL =
  'https://huggingface.co/SmilingWolf/wd-v1-4-swinv2-tagger-v2/resolve/main/selected_tags.csv';
const TAGS_PATH = 'models/tags.csv';

export default function Settings() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [loading, setLoading] = useState(true);
  const [exclusionText, setExclusionText] = useState('');
  const [downloadProgress, setDownloadProgress] =
    useState<DownloadProgress | null>(null);
  const [modelStatus, setModelStatus] = useState<
    'checking' | 'present' | 'missing'
  >('checking');
  const [extensionId, setExtensionId] = useState('');

  const configRef = useRef(config);
  useEffect(() => {
    configRef.current = config;
  }, [config]);

  const checkModel = useCallback(async (path: string) => {
    setModelStatus('checking');
    try {
      const exists = await invoke<boolean>('check_model_exists', {
        pathStr: path,
      });
      setModelStatus(exists ? 'present' : 'missing');
    } catch (e) {
      console.error('Failed to check model', e);
      setModelStatus('missing');
    }
  }, []);

  useEffect(() => {
    invoke<AppConfig>('get_config')
      .then((c) => {
        setConfig(c);
        setExclusionText(c.exclusion_list.join(', '));
        setLoading(false);
      })
      .catch((e) => {
        console.error('Failed to load config', e);
        setLoading(false);
      });

    const unlistenProgress = listen<DownloadProgress>(
      'model-download-progress',
      (event) => {
        setDownloadProgress(event.payload);
      }
    );

    const unlistenFinished = listen('model-download-finished', () => {
      setDownloadProgress(null);
      if (configRef.current) checkModel(configRef.current.model_path);
    });

    return () => {
      unlistenProgress.then((f) => f());
      unlistenFinished.then((f) => f());
    };
  }, [checkModel]);

  // Check model status when config.model_path changes
  useEffect(() => {
    if (config) {
      // eslint-disable-next-line
      checkModel(config.model_path);
    }
    // eslint-disable-next-line
  }, [config?.model_path, checkModel]);

  const downloadCurrentModel = async () => {
    if (!config) return;
    const preset = PRESETS.find((p) => p.path === config.model_path);
    if (!preset) return;

    try {
      await invoke('download_new_model', {
        url: preset.url,
        pathStr: preset.path,
      });
      const tagsExists = await invoke<boolean>('check_model_exists', {
        pathStr: config.tags_path,
      });
      if (!tagsExists) {
        await invoke('download_new_model', {
          url: TAGS_URL,
          pathStr: TAGS_PATH,
        });
      }
    } catch (e) {
      console.error('Failed to download', e);
      alert('Download failed: ' + e);
    }
  };

  const saveConfig = async (newConfig: AppConfig) => {
    setConfig(newConfig);
    try {
      await invoke('set_config', { config: newConfig });
    } catch (e) {
      console.error('Failed to save config', e);
      alert('Failed to save config: ' + e);
    }
  };

  const updateField = <K extends keyof AppConfig>(
    key: K,
    value: AppConfig[K]
  ) => {
    if (!config) return;
    const newConfig = { ...config, [key]: value };
    saveConfig(newConfig);
  };

  const registerContextMenu = async (enable: boolean) => {
    try {
      await invoke('register_context_menu', { enable });
      alert(
        `Successfully ${enable ? 'added to' : 'removed from'} Context Menu.`
      );
    } catch (e) {
      alert('Failed: ' + e);
    }
  };

  const registerNativeHost = async () => {
    if (!extensionId) {
      alert('Please enter the Extension ID first.');
      return;
    }
    try {
      await invoke('register_native_host', { extensionId });
      alert('Native Host registered successfully!');
    } catch (e) {
      alert('Failed: ' + e);
    }
  };

  if (loading) return <div className="p-6">Loading settings...</div>;
  if (!config)
    return (
      <div className="p-6 text-red-500">Failed to load configuration.</div>
    );

  return (
    <div className="p-6 bg-gray-100 min-h-screen text-gray-800 font-sans">
      <h1 className="text-2xl font-bold mb-6">OmniTagger Settings</h1>

      {downloadProgress && (
        <div
          className="bg-blue-100 border-l-4 border-blue-500 text-blue-700 p-4 mb-4"
          role="alert"
        >
          <p className="font-bold">Downloading Model...</p>
          <p className="text-sm mb-2">
            {downloadProgress.file}: {downloadProgress.percent.toFixed(1)}%
          </p>
          <div className="w-full bg-blue-200 rounded-full h-2.5 dark:bg-blue-200">
            <div
              className="bg-blue-600 h-2.5 rounded-full"
              style={{ width: `${downloadProgress.percent}%` }}
            ></div>
          </div>
        </div>
      )}

      {/* Integrations */}
      <div className="bg-white p-4 rounded shadow mb-6">
        <h2 className="text-lg font-semibold mb-4 border-b pb-2">
          Integrations
        </h2>

        {/* Windows Context Menu */}
        <div className="mb-6">
          <h3 className="font-medium mb-2">
            Windows Context Menu (Local Files)
          </h3>
          <div className="flex space-x-4">
            <button
              onClick={() => registerContextMenu(true)}
              className="bg-green-600 text-white px-4 py-2 rounded hover:bg-green-700 text-sm"
            >
              Add to Right Click Menu
            </button>
            <button
              onClick={() => registerContextMenu(false)}
              className="bg-red-600 text-white px-4 py-2 rounded hover:bg-red-700 text-sm"
            >
              Remove
            </button>
          </div>
        </div>

        {/* Browser Extension */}
        <div>
          <h3 className="font-medium mb-2">Browser Extension (Chrome/Edge)</h3>
          <ol className="list-decimal list-inside text-sm text-gray-600 mb-4 space-y-1">
            <li>
              Load the <code>browser-extension</code> folder as an "Unpacked
              Extension".
            </li>
            <li>
              Copy the <strong>ID</strong> of the loaded extension.
            </li>
            <li>Paste the ID below and click Register.</li>
          </ol>
          <div className="flex gap-2">
            <input
              type="text"
              placeholder="e.g. abcdefghijklmnopqrstuvwxyz"
              value={extensionId}
              onChange={(e) => setExtensionId(e.target.value)}
              className="flex-1 p-2 border rounded bg-gray-50 text-sm"
            />
            <button
              onClick={registerNativeHost}
              className="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700 text-sm"
            >
              Register Host
            </button>
          </div>
        </div>
      </div>

      {/* Model Selection */}
      <div className="bg-white p-4 rounded shadow mb-6">
        <h2 className="text-lg font-semibold mb-4 border-b pb-2">AI Model</h2>

        <div className="mb-4">
          <label className="block text-sm font-medium text-gray-700 mb-1">
            Model Preset
          </label>
          <select
            value={
              PRESETS.find((p) => p.path === config.model_path)?.path ||
              'custom'
            }
            onChange={(e) => {
              const val = e.target.value;
              if (val !== 'custom') {
                const preset = PRESETS.find((p) => p.path === val);
                if (preset) {
                  const newConfig = {
                    ...config,
                    model_path: preset.path,
                    tags_path: TAGS_PATH,
                  };
                  saveConfig(newConfig);
                }
              }
            }}
            className="w-full p-2 border rounded bg-white mb-2 cursor-pointer"
          >
            {PRESETS.map((p) => (
              <option key={p.path} value={p.path}>
                {p.name}
              </option>
            ))}
            <option value="custom">Custom</option>
          </select>

          {modelStatus === 'missing' &&
            PRESETS.some((p) => p.path === config.model_path) && (
              <div className="mt-2 p-2 bg-yellow-50 text-yellow-800 border border-yellow-200 rounded flex items-center justify-between">
                <span className="text-sm">Model file not found locally.</span>
                <button
                  onClick={downloadCurrentModel}
                  disabled={!!downloadProgress}
                  className="bg-blue-600 text-white px-3 py-1 rounded text-sm hover:bg-blue-700 disabled:opacity-50"
                >
                  {downloadProgress ? 'Downloading...' : 'Download Model'}
                </button>
              </div>
            )}
        </div>

        <div className="mb-4">
          <label className="block text-sm font-medium text-gray-700 mb-1">
            Model Path (.onnx)
          </label>
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
                  filters: [{ name: 'ONNX Model', extensions: ['onnx'] }],
                });
                if (selected && typeof selected === 'string') {
                  updateField('model_path', selected);
                }
              }}
              className="bg-gray-200 text-gray-800 px-3 py-2 rounded hover:bg-gray-300 text-sm"
            >
              Browse
            </button>
          </div>
        </div>

        <div className="mb-2">
          <label className="block text-sm font-medium text-gray-700 mb-1">
            Tags File (.csv)
          </label>
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
                  filters: [{ name: 'CSV File', extensions: ['csv'] }],
                });
                if (selected && typeof selected === 'string') {
                  updateField('tags_path', selected);
                }
              }}
              className="bg-gray-200 text-gray-800 px-3 py-2 rounded hover:bg-gray-300 text-sm"
            >
              Browse
            </button>
          </div>
        </div>
      </div>

      {/* Threshold & Formatting */}
      <div className="bg-white p-4 rounded shadow mb-6">
        <h2 className="text-lg font-semibold mb-4 border-b pb-2">Processing</h2>

        <div className="mb-6">
          <h3 className="text-sm font-medium text-gray-700 mb-2">
            Confidence Threshold: {config.threshold}
          </h3>
          <input
            type="range"
            min="0"
            max="1"
            step="0.01"
            value={config.threshold}
            onChange={(e) =>
              updateField('threshold', parseFloat(e.target.value))
            }
            className="w-full cursor-pointer"
          />
          <div className="flex justify-between text-xs text-gray-500 mt-1">
            <span>0.0</span>
            <span>1.0</span>
          </div>
        </div>

        <div className="mb-6">
          <label className="flex items-center space-x-2 cursor-pointer">
            <input
              type="checkbox"
              checked={config.use_underscore}
              onChange={(e) => updateField('use_underscore', e.target.checked)}
              className="w-4 h-4 text-blue-600 rounded focus:ring-blue-500"
            />
            <span className="text-sm font-medium">
              Use Underscores (e.g. <code>long_hair</code>)
            </span>
          </label>
        </div>

        <div>
          <h3 className="text-sm font-medium text-gray-700 mb-2">
            Excluded Tags
          </h3>
          <textarea
            value={exclusionText}
            onChange={(e) => setExclusionText(e.target.value)}
            onBlur={() => {
              const list = exclusionText
                .split(',')
                .map((s) => s.trim())
                .filter((s) => s.length > 0);
              updateField('exclusion_list', list);
            }}
            className="w-full p-2 border rounded h-24 font-mono text-sm"
            placeholder="bad_hands, lowres, ..."
          />
        </div>
      </div>
    </div>
  );
}
