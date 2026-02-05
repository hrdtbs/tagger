# Development TODO List

## Project Maintenance
- [ ] **Regular Updates**: Regularly update and organize this TODO list as tasks are completed or new requirements arise.

## Core Functionality (Backend)
- [x] **AI Inference Engine**: Replace mock implementation in `tagger.rs` with real `ort` (ONNX Runtime) integration.
    - [x] Add `ort` dependency to `src-tauri/Cargo.toml`.
    - [x] Load .onnx models (WD14 SwinV2, ConvNext).
    - [x] Implement image preprocessing (Resize to 448x448, Normalize RGB).
    - [x] Implement inference logic to get tag probabilities.
    - [x] Load tag csv files (tag index to string mapping).
- [x] **Multi-Monitor Support**:
    - [x] Update `capture_screen` in `lib.rs` to capture all screens (currently only captures primary).
    - [x] Stitch screens together or handle multiple windows for overlay.
    - [x] Map frontend selection coordinates back to the correct screen/pixel.

## Frontend
- [ ] **Overlay**:
    - [x] Prevent overlay self-capture (hide window during capture).
    - [x] Enable fullscreen mode for overlay.
    - [x] Handle multi-monitor layouts correctly (currently assumes single viewport).
    - [ ] Improve selection UX (resize handles, move selection).
- [x] **Settings**:
    - [x] Persist settings to disk (config file) instead of LocalStorage.
    - [x] File picker for custom ONNX models.
    - [x] Tag exclusion list management.

## Packaging & Distribution
- [ ] **Model Management**:
    - [ ] Mechanism to download models on first run or bundle them (considering file size).
- [ ] **CI/CD**:
    - [ ] GitHub Actions for building Windows/macOS/Linux binaries.

## Bugs / Known Issues
- [ ] Window focus: Ensure overlay window takes focus immediately on hotkey (partially addressed in skeleton).
- [ ] Coordinate mapping: Verify `object-contain` scaling logic on different aspect ratios and high DPI screens.
