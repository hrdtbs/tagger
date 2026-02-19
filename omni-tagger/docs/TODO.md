# Development TODO List

## Project Status
Core functionality for Windows and Linux is implemented, including AI inference, context menu integration, and browser extension support. The project is currently in the verification and refinement phase.

## Pending Tasks

### QA / Pending Verification (Manual)
- [ ] **Manual Verification (Windows)**:
    - [ ] Test "Add to Context Menu" adds registry keys correctly.
    - [ ] Test Right-click > "Get Tags" on an image file launches the app and copies tags.
    - [ ] Test "Register Host" adds the manifest file and registry key.
    - [ ] Test Browser Extension communication (URL handling).
    - [ ] Test Browser Extension communication (Data URI handling).
- [ ] **Manual Verification (Linux)**:
    - [ ] Test "Add to Context Menu" creates `~/.local/share/applications/omni-tagger-context.desktop`.
    - [ ] Test Right-click > "Get Tags" (via File Manager supporting Desktop Actions) launches app.
    - [ ] Test "Register Host" creates manifest in `~/.config/google-chrome/NativeMessagingHosts/`.
    - [ ] Test Browser Extension communication.

### Non-Functional Requirements (Pending Validation)
- [ ] **Size**: Verify application size is under 100MB (excluding models).

### Future Improvements / Cross-Platform
- [ ] **macOS Integration**:
    - [ ] Implement Context Menu registration for macOS (Finder extensions or Automator services).
    - [ ] Implement Native Messaging Host registration for macOS.
- [ ] **Firefox Support**:
    - [ ] Verify manifest compatibility or create separate manifest for Firefox.
- [ ] **Offline Installer**:
    - [ ] Create an installer variant that bundles the default models to avoid download requirement.
- [ ] **Model Flexibility**:
    - [ ] Implement model-specific preprocessing configuration to support a wider range of ONNX models.

---

## Completed Features

### Core Functionality (Backend)
- [x] **AI Inference Engine**: Integrated `ort` (ONNX Runtime) with `image` crate preprocessing (Resize 448x448, Normalize BGR).
- [x] **Local File Support**: Implemented CLI argument parsing and auto-exit logic for headless operation.
- [x] **Output Functionality**: Clipboard copy and desktop notifications.
- [x] **Context Menu Integration**: Windows Registry and Linux `.desktop` entry generation.

### Browser Extension Support
- [x] **Native Messaging Host**: Implemented `native_host` binary for communication between extension and main app.
- [x] **Data URI Handling**: Implemented logic to decode base64 images to temporary files, process them, and clean them up (`--delete-after`).
- [x] **Extension Frontend**: Manifest V3 extension with Context Menu ("Get Tags") and message passing.

### Frontend (App UI)
- [x] **Settings Interface**: Configuration for Model selection, Thresholds, Tag formatting (underscores), and Exclusion lists.
- [x] **Integrations UI**: Buttons to register Context Menu and Native Host.
- [x] **Model Management**: Automatic download of default models (WD14 SwinV2) and tags CSV if missing.

### Packaging & Distribution
- [x] **CI/CD**: GitHub Actions workflows for Windows, Linux, and macOS builds.
- [x] **Versioning**: Automated version generation based on date and commit hash.
- [x] **Native Host Bundling**: `native_host` executable is built and included in resources.

### Resolved Issues & Technical Debt
- [x] **Backend Model Download Logic**: Fixed hardcoded URL issue in `model_manager.rs` to support correct model downloads based on filename.
- [x] **Native Host Cleanup**: Implemented automatic deletion of temporary files created from Data URIs (`processor.rs`).

### Implemented Tests
- [x] **Frontend E2E**: Playwright tests for UI components and settings.
- [x] **Performance Benchmark**: `test_inference_performance` in `tagger.rs` (verifies < 1s inference).
- [x] **Offline Capability**: `test_check_file_exists` in `model_manager.rs`.
- [x] **File Cleanup**: `test_process_inputs_with_actions_delete_after` in `processor.rs`.
