# Build Instructions

This guide covers how to set up the development environment and build antiwhisper from source across different platforms.

## Prerequisites

### All Platforms

- [Rust](https://rustup.rs/) (latest stable)
- [Bun](https://bun.sh/) package manager
- [Tauri Prerequisites](https://tauri.app/start/prerequisites/)

### Platform-Specific Requirements

#### macOS

- Xcode Command Line Tools
- Install with: `xcode-select --install`
- Homebrew `llama.cpp` package for local LLM release bundling
- Install with: `brew install llama.cpp`

#### Windows

- Microsoft C++ Build Tools
- Visual Studio 2019/2022 with C++ development tools
- Or Visual Studio Build Tools 2019/2022

#### Linux

- Build essentials
- ALSA development libraries
- Install with:

  ```bash
  # Ubuntu/Debian
  sudo apt update
  sudo apt install build-essential libasound2-dev pkg-config libssl-dev libvulkan-dev vulkan-tools glslc libgtk-3-dev libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev libgtk-layer-shell0 libgtk-layer-shell-dev patchelf cmake

  # Fedora/RHEL
  sudo dnf groupinstall "Development Tools"
  sudo dnf install alsa-lib-devel pkgconf openssl-devel vulkan-devel \
    gtk3-devel webkit2gtk4.1-devel libappindicator-gtk3-devel librsvg2-devel \
    gtk-layer-shell gtk-layer-shell-devel \
    cmake

  # Arch Linux
  sudo pacman -S base-devel alsa-lib pkgconf openssl vulkan-devel \
    gtk3 webkit2gtk-4.1 libappindicator-gtk3 librsvg gtk-layer-shell \
    cmake
  ```

## Setup Instructions

### 1. Clone the Repository

```bash
git clone git@github.com:denberek/antiwhisper.git
cd antiwhisper
```

### 2. Install Dependencies

```bash
bun install
```

### 3. Start Dev Server

```bash
bun tauri dev
```

### 4. Build a macOS Release

On macOS, `tauri build` runs `scripts/prepare-llama-runtime.sh` before bundling. That script stages the required `llama.cpp`, `ggml`, and OpenSSL runtime files into the app bundle so Gemma-based post-processing works on another machine without Homebrew installed.

```bash
PATH="$HOME/.cargo/bin:$PATH" bun run tauri build --bundles app
```

If updater signing is configured in your environment, you can use the full release build instead.
