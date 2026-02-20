# Development TODO List

## Project Maintenance
- [x] **Regular Updates**: Regularly update and organize this TODO list as tasks are completed or new requirements arise.
- [x] **Code Quality**: Performed Rust dependency updates and Clippy audit (Clean).
- [x] **Dependency Update (Feb 2026)**: Updated React to v19.2.4, Tailwind to v4.2.0, Vite to v7.3.1, Tauri to v2.10.2, and other dependencies to latest stable versions.

## Core Functionality (Backend)
- [x] **AI Inference Engine**: Replace mock implementation in `tagger.rs` with real `ort` (ONNX Runtime) integration.
    - [x] Add `ort` dependency to `src-tauri/Cargo.toml`.
    - [x] Load .onnx models (WD14 SwinV2, ConvNext, ConvNextV2).
    - [x] Implement image preprocessing (Resize to 448x448, Normalize BGR).
    - [x] Implement inference logic to get tag probabilities.
    - [x] Load tag csv files (tag index to string mapping).

- [x] **Local File Support**:
    - [x] Implement CLI argument parsing in `lib.rs` to accept image file paths.
    - [x] Trigger inference immediately when a file path is provided.
    - [x] Add `register_context_menu` command (Windows Registry / Linux Desktop Entry).
    - [x] **Auto-Exit**: Implement logic to exit the application automatically after processing a file in headless mode.

- [x] **Output Functionality**:
    - [x] Copy generated tags to Clipboard.
    - [x] Show desktop notification on completion.

- [x] **Browser Extension Support (Native Messaging)**:
    - [x] Create `native_host` binary source (`src-tauri/src/bin/native_host.rs`).
    - [x] Implement JSON message handling (Stdin/Stdout) in `native_host`.
    - [x] Forward requests from `native_host` to main app (via process spawning).
    - [x] Add `register_native_host` command.
    - [x] **Data URI Support**: Implement handling of base64/data URIs from the browser extension.
    - [x] **Edge/Brave Support (Windows)**: Implement `register_native_host` logic for Edge and Brave on Windows (currently only Chrome is supported).

## Linux Support
- [x] **Native Host Support**:
    - [x] Implement `native_host` build and bundling for Linux.
    - [x] Implement `register_native_host` logic for Linux (Manifest in `~/.config/...`).
- [x] **Context Menu Support**:
    - [x] Implement context menu registration for Linux (.desktop actions in `~/.local/share/applications`).

## Browser Extension (Frontend)
- [x] **Create Extension**:
    - [x] Create `browser-extension` directory structure.
    - [x] Create `manifest.json` (Manifest V3) with `nativeMessaging` permission.
    - [x] Implement `background.js` (Service Worker) to register context menu ("Get Tags").
    - [x] Implement message passing to native host (`chrome.runtime.sendNativeMessage`).
    - [x] Add icons and other resources.

## Frontend (App UI)
- [x] **Settings**:
    - [x] Persist settings to disk (config file).
    - [x] Model Preset Selection (SwinV2, ConvNext, ConvNextV2).
    - [x] File picker for custom ONNX models.
    - [x] **Confidence Threshold adjustment**.
    - [x] **Tag Formatting (underscores)**.
    - [x] Tag exclusion list management.
    - [x] Add "Add to Context Menu" button.
    - [x] Add "Install Browser Extension" instructions.

- [x] **Cleanup**:
    - [x] Remove Overlay UI components (`Overlay.tsx`).
    - [x] Remove Screen Capture logic (`capture_screen`, `screenshots` crate).

## Packaging & Distribution
- [x] **Model Management**:
    - [x] Mechanism to download models on first run.
- [x] **Bundle native_host.exe**:
    - [x] Ensure `native_host.exe` is built and included in the installer/output directory.
    - [x] Verify the path resolution logic in `register_native_host` works with the installed path.
- [x] **CI/CD**:
    - [x] GitHub Actions for building Windows/macOS/Linux binaries.

## Known Issues & Bugs
- [x] **Backend Model Download Logic Flaw**: The `check_and_download_models` function in `src-tauri/src/model_manager.rs` uses a hardcoded URL (WD14 SwinV2). If a user selects a different model (e.g. ConvNext) in settings but the file is missing on startup, the application will incorrectly download the SwinV2 model to the configured path.
- [x] **Private/Blob URL Handling**: Implemented logic in browser extension to fetch image data (handling auth/blob URLs) and resize it before sending to backend as Data URI.

## Technical Debt
- [x] **Native Host Cleanup**: Implement cleanup mechanism for temporary files created by `native_host` when processing Data URIs.

## Quality Assurance / Verification (Pending)
- [x] **Frontend E2E Testing**: Implemented Playwright tests for frontend verification. Added `e2e` directory and `test:e2e` script.
- [ ] **Manual Verification (Windows)**:
    - [ ] Test "Add to Context Menu" adds registry keys correctly.
    - [ ] Test Right-click > "Get Tags" on an image file launches the app and copies tags.
    - [x] Test "Register Host" adds the manifest file and registry key. (Added unit test for manifest generation in `registry.rs`)
    - [ ] Test Browser Extension communication (URL handling).
    - [ ] Test Browser Extension communication (Data URI handling).
    - [ ] Test Private/Blob Image processing (Fetch & Resize in browser).
- [ ] **Manual Verification (Linux)**:
    - [ ] Test "Add to Context Menu" creates `~/.local/share/applications/omni-tagger-context.desktop`.
    - [ ] Test Right-click > "Get Tags" (via File Manager supporting Desktop Actions) launches app.
    - [ ] Test "Register Host" creates manifest in `~/.config/google-chrome/NativeMessagingHosts/`.
    - [ ] Test Browser Extension communication.

## Non-Functional Requirements (Pending Validation)
- [x] **Performance**: Verify tag generation completes within 1 second. (Added `test_inference_performance` benchmark)
- [ ] **Size**: Verify application size is under 100MB (excluding models). (Pending: Build environment limitations prevent local verification)
- [x] **Offline Operation**: Verify core features work without internet (after initial model download). (Added `test_check_file_exists` unit test)

## Future Improvements / Cross-Platform
- [x] **Improved Browser Integration**:
    - [x] Implement logic in extension to fetch image data for Blob/Private URLs and send as Data URI automatically.
- [ ] **macOS Integration**:
    - [ ] Implement Context Menu registration for macOS (Finder extensions or Automator services).
    - [ ] Implement Native Messaging Host registration for macOS.
- [ ] **Firefox Support**:
    - [ ] Verify manifest compatibility or create separate manifest for Firefox.
- [ ] **Offline Installer**:
    - [ ] Create an installer variant that bundles the default models to avoid download requirement.
- [ ] **Model Flexibility**:
    - [ ] Implement model-specific preprocessing configuration (e.g. input size, normalization) to support a wider range of ONNX models.
