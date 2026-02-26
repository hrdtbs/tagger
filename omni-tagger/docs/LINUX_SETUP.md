# Linux Development Setup

This guide details the system dependencies required to develop and test OmniTagger on Linux. OmniTagger uses Tauri v2, which relies on WebKit2GTK and other system libraries.

## Dependencies

### Debian / Ubuntu (22.04+)

```bash
sudo apt update
sudo apt install -y libwebkit2gtk-4.1-dev \
    build-essential \
    curl \
    wget \
    file \
    libxdo-dev \
    libssl-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    libgtk-3-dev \
    pkg-config \
    libxcb-render0-dev \
    libxcb-shape0-dev \
    libxcb-xfixes0-dev
```

### Arch Linux

```bash
sudo pacman -Syu
sudo pacman -S --needed \
    webkit2gtk-4.1 \
    base-devel \
    curl \
    wget \
    file \
    openssl \
    appmenu-gtk-module \
    libappindicator-gtk3 \
    librsvg \
    xdotool
```

### Fedora

```bash
sudo dnf check-update
sudo dnf install \
    webkit2gtk4.1-devel \
    openssl-devel \
    curl \
    wget \
    file \
    libappindicator-gtk3-devel \
    librsvg2-devel \
    libxdo-devel

sudo dnf group install "c-development"
```

## Running Tests

Once dependencies are installed, you can run the backend tests:

```bash
cd omni-tagger/src-tauri
cargo test
```
