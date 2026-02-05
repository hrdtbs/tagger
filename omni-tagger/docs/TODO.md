# Development TODO List

## Core Functionality (Backend)
- [ ] **AI Inference Engine**: Implement `ort` (ONNX Runtime) integration in Rust.
    - [ ] Load .onnx models (WD14 SwinV2, ConvNext).
    - [ ] Implement image preprocessing (Resize to 448x448, Normalize RGB).
    - [ ] Implement inference logic to get tag probabilities.
    - [ ] Load tag csv files (tag index to string mapping).
- [ ] **Multi-Monitor Support**:
    - [ ] Update `capture_screen` to capture all screens.
    - [ ] Stitch screens together or handle multiple windows for overlay.
    - [ ] Map frontend selection coordinates back to the correct screen/pixel.

## Frontend
- [ ] **Overlay**:
    - [ ] Handle multi-monitor layouts correctly (currently assumes single viewport).
    - [ ] Improve selection UX (resize handles, move selection).
- [ ] **Settings**:
    - [ ] Persist settings to disk (config file) instead of LocalStorage.
    - [ ] File picker for custom ONNX models.
    - [ ] Tag exclusion list management.

## Packaging & Distribution
- [ ] **Model Management**:
    - [ ] Mechanism to download models on first run or bundle them (considering file size).
- [ ] **CI/CD**:
    - [ ] GitHub Actions for building Windows/macOS/Linux binaries.

## Bugs / Known Issues
- [ ] Window focus: Ensure overlay window takes focus immediately on hotkey (partially addressed in skeleton).
- [ ] Coordinate mapping: Verify `object-contain` scaling logic on different aspect ratios and high DPI screens.
