# Development TODO List

## Project Maintenance
- [x] **Regular Updates**: Regularly update and organize this TODO list as tasks are completed or new requirements arise.

## Core Functionality (Backend)
- [x] **AI Inference Engine**: Replace mock implementation in `tagger.rs` with real `ort` (ONNX Runtime) integration.
    - [x] Add `ort` dependency to `src-tauri/Cargo.toml`.
    - [x] Load .onnx models (WD14 SwinV2, ConvNext).
    - [x] Implement image preprocessing (Resize to 448x448, Normalize RGB).
    - [x] Implement inference logic to get tag probabilities.
    - [x] Load tag csv files (tag index to string mapping).

- [x] **Local File Support (Windows)**:
    - [x] Implement CLI argument parsing in `lib.rs` to accept image file paths.
    - [x] Trigger inference immediately when a file path is provided.
    - [x] Add `register_context_menu` command to modify Windows Registry.
    - [ ] **Auto-Exit**: Implement logic to exit the application automatically after processing a file in headless mode (CLI/Context Menu).

- [x] **Browser Extension Support (Native Messaging)**:
    - [x] Create `native_host` binary source (`src-tauri/src/bin/native_host.rs`).
    - [x] Implement JSON message handling (Stdin/Stdout) in `native_host`.
    - [x] Forward requests from `native_host` to main app.
    - [x] Add `register_native_host` command.

## Browser Extension (Frontend)
- [ ] **Create Extension**:
    - [ ] Create `browser-extension` directory structure.
    - [ ] Create `manifest.json` (Manifest V3) with `nativeMessaging` permission.
    - [ ] Implement `background.js` (Service Worker) to register context menu ("Get Tags").
    - [ ] Implement message passing to native host (`chrome.runtime.sendNativeMessage`).
    - [ ] Add icons and other resources.

## Frontend (App UI)
- [x] **Settings**:
    - [x] Persist settings to disk (config file).
    - [x] File picker for custom ONNX models.
    - [x] Tag exclusion list management.
    - [x] Add "Add to Windows Context Menu" button.
    - [x] Add "Install Browser Extension" instructions.

- [x] **Cleanup**:
    - [x] Remove Overlay UI components (`Overlay.tsx`).
    - [x] Remove Screen Capture logic (`capture_screen`, `screenshots` crate).

## Packaging & Distribution
- [x] **Model Management**:
    - [x] Mechanism to download models on first run.
- [ ] **Bundle native_host.exe**:
    - [ ] Configure `tauri.conf.json` or build scripts to include `native_host.exe` in the output (e.g., as a sidecar or resource).
- [x] **CI/CD**:
    - [x] GitHub Actions for building Windows/macOS/Linux binaries.

## Deprecated / Removed
- [x] **Multi-Monitor Support** (Removed).
- [x] **Screen Capture Overlay** (Removed).
