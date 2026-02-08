# Development TODO List

## Project Maintenance
- [x] **Regular Updates**: Regularly update and organize this TODO list as tasks are completed or new requirements arise.

## Core Functionality (Backend)
- [x] **AI Inference Engine**: Replace mock implementation in `tagger.rs` with real `ort` (ONNX Runtime) integration.
    - [x] Add `ort` dependency to `src-tauri/Cargo.toml`.
    - [x] Load .onnx models (WD14 SwinV2, ConvNext).
    - [x] Implement image preprocessing (Resize to 448x448, Normalize BGR).
    - [x] Implement inference logic to get tag probabilities.
    - [x] Load tag csv files (tag index to string mapping).

- [x] **Local File Support (Windows)**:
    - [x] Implement CLI argument parsing in `lib.rs` to accept image file paths.
    - [x] Trigger inference immediately when a file path is provided.
    - [x] Add `register_context_menu` command to modify Windows Registry.

- [x] **Browser Extension Support (Native Messaging)**:
    - [x] Create `native_host` binary for Native Messaging communication.
    - [x] Implement JSON message handling (Stdin/Stdout) in `native_host`.
    - [x] Forward requests from `native_host` to main app (via process spawning).
    - [x] Add `register_native_host` command.

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

## Browser Extension (Frontend) - **PRIORITY**
- [ ] **Create Extension**:
    - [ ] Create `browser-extension` directory structure.
    - [ ] Create `manifest.json` (Manifest V3) with `nativeMessaging` permission.
    - [ ] Implement `background.js` (Service Worker) to register context menu ("Get Tags").
    - [ ] Implement message passing to native host (`chrome.runtime.sendNativeMessage`).
    - [ ] Add icons (16, 32, 48, 128 px).
    - [ ] Add `options.html` (Optional, for extension specific settings).

## Packaging & Distribution
- [x] **Model Management**:
    - [x] Mechanism to download models on first run.
- [ ] **Bundle native_host.exe**:
    - [ ] Ensure `native_host.exe` is built and included in the installer/output directory.
    - [ ] Verify the path resolution logic in `register_native_host` works with the installed path.
- [x] **CI/CD**:
    - [x] GitHub Actions for building Windows/macOS/Linux binaries.

## Quality Assurance / Verification
- [ ] **Manual Verification (Windows)**:
    - [ ] Test "Add to Context Menu" adds registry keys correctly.
    - [ ] Test Right-click > "Get Tags" on an image file launches the app and copies tags.
    - [ ] Test "Register Host" adds the manifest file and registry key.
    - [ ] Test Browser Extension communication (once implemented).

## Refactoring & Improvements
- [ ] **Code Structure**:
    - [ ] Split `lib.rs` into smaller modules (`registry.rs`, `processor.rs`) to improve maintainability.
- [ ] **Error Handling**:
    - [ ] Improve error messages in `native_host` to be more descriptive in the JSON response.

## Documentation
- [ ] **User Guide**:
    - [ ] Update `README.md` with installation and usage instructions.
    - [ ] Document how to load the unpacked extension in Chrome/Edge.

## Deprecated / Removed
- [x] **Multi-Monitor Support** (Removed).
- [x] **Screen Capture Overlay** (Removed).
