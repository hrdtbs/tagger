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

- [x] **Browser Extension Support (Native Messaging)**:
    - [x] Create `native_host` binary for Native Messaging communication.
    - [x] Implement JSON message handling (Stdin/Stdout) in `native_host`.
    - [x] Forward requests from `native_host` to main app (or process directly if possible).
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
    - [ ] Ensure `native_host.exe` is built and included in the installer/output directory (e.g., via `tauri.conf.json` resources or sidecar config).
- [x] **CI/CD**:
    - [x] GitHub Actions for building Windows/macOS/Linux binaries.

## Deprecated / Removed
- [x] **Multi-Monitor Support** (Removed).
- [x] **Screen Capture Overlay** (Removed).
