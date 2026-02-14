# OmniTagger

OmniTagger is a desktop application powered by Tauri and ONNX Runtime to automatically extract tags from images using AI models (WD14 SwinV2).

## Features

- **AI Tagging**: Uses `wd-v1-4-swinv2-tagger-v2` to generate tags for anime-style images.
- **Windows Context Menu**: Right-click on any image file and select "Get Tags" to quickly extract tags.
- **Browser Extension**: Extract tags from images on the web via a Chrome/Edge extension.
- **Privacy Focused**: All processing happens locally on your machine. No images are uploaded to the cloud.

## Installation

### Windows

1. Download the latest installer (`.msi` or `.exe`) from the Releases page.
2. Run the installer.
3. Launch OmniTagger.

### Browser Extension

1. In OmniTagger, go to **Settings**.
2. Click **Install Browser Extension**.
3. Follow the instructions to load the unpacked extension in Chrome/Edge:
   - Go to `chrome://extensions`.
   - Enable **Developer mode**.
   - Click **Load unpacked**.
   - Select the `browser-extension` folder (usually located in the installation directory or source code).

## Usage

### GUI

- Launch the application.
- Configure settings (Model, Threshold, Exclusion List).

### Context Menu (Windows)

1. In Settings, click **Add to Context Menu**.
2. Right-click on an image file in File Explorer.
3. Select **Get Tags**.
4. The application will process the image and copy the tags to your clipboard. A notification will appear when done.

### Browser Extension

1. Right-click on an image in your browser.
2. Select **OmniTagger > Get Tags**.
3. The application will process the image URL and copy tags to your clipboard.

## Development

### Prerequisites

- Rust (latest stable)
- Node.js (v18+)
- VS Code (recommended)

### Setup

1. Clone the repository.
2. Install frontend dependencies:
   ```bash
   cd omni-tagger
   npm install
   ```
3. Run in development mode:
   ```bash
   npm run tauri dev
   ```

### Building

To build the application for production:

```bash
npm run tauri build
```

This will build the frontend, the Rust backend, and the `native_host` binary.

## License

MIT
