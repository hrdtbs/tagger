import { useState, useEffect } from 'react';

export default function Settings() {
  const [model, setModel] = useState(localStorage.getItem('model') || 'wd14-vit-v2');
  const [threshold, setThreshold] = useState(parseFloat(localStorage.getItem('threshold') || '0.35'));
  const [useUnderscore, setUseUnderscore] = useState(localStorage.getItem('useUnderscore') === 'true');

  useEffect(() => {
    localStorage.setItem('model', model);
    localStorage.setItem('threshold', threshold.toString());
    localStorage.setItem('useUnderscore', useUnderscore.toString());
  }, [model, threshold, useUnderscore]);

  return (
    <div className="p-6 bg-gray-100 min-h-screen text-gray-800">
      <h1 className="text-2xl font-bold mb-4">OmniTagger Settings</h1>

      <div className="bg-white p-4 rounded shadow mb-4">
        <h2 className="text-lg font-semibold mb-2">Model Selection</h2>
        <select
          value={model}
          onChange={(e) => setModel(e.target.value)}
          className="w-full p-2 border rounded bg-gray-50"
        >
          <option value="wd14-vit-v2">WD14 ViT V2 (Mock)</option>
          <option value="wd14-convnext-v2">WD14 ConvNext V2 (Mock)</option>
          <option value="wd14-swinv2-v2">WD14 SwinV2 V2 (Mock)</option>
        </select>
        <p className="text-sm text-gray-500 mt-2">
            Note: This skeleton uses a mock inference engine.
        </p>
      </div>

      <div className="bg-white p-4 rounded shadow mb-4">
        <h2 className="text-lg font-semibold mb-2">Confidence Threshold: {threshold}</h2>
        <input
          type="range"
          min="0"
          max="1"
          step="0.01"
          value={threshold}
          onChange={(e) => setThreshold(parseFloat(e.target.value))}
          className="w-full cursor-pointer"
        />
        <div className="flex justify-between text-xs text-gray-500">
            <span>0.0</span>
            <span>1.0</span>
        </div>
      </div>

      <div className="bg-white p-4 rounded shadow mb-4">
        <h2 className="text-lg font-semibold mb-2">Formatting</h2>
        <label className="flex items-center space-x-2 cursor-pointer">
          <input
            type="checkbox"
            checked={useUnderscore}
            onChange={(e) => setUseUnderscore(e.target.checked)}
            className="w-4 h-4 text-blue-600 rounded focus:ring-blue-500"
          />
          <span>Use Underscores (e.g. <code>long_hair</code> vs <code>long hair</code>)</span>
        </label>
      </div>
    </div>
  );
}
