# antiwhisper

**A free, open-source, privacy-focused speech-to-text app that runs entirely on your machine.**

antiwhisper is a cross-platform desktop application built with Tauri (Rust + React/TypeScript). Press a shortcut, speak, and your words appear in any text field — without sending audio to the cloud.

## How It Works

1. **Press** a keyboard shortcut (Option+Space on macOS) to start recording
2. **Speak** your words
3. **Press** the shortcut again to stop
4. **Get** your transcribed text pasted directly into whatever app you're using

Everything runs locally — your voice never leaves your computer.

## Quick Start

### macOS Installation

1. Download the latest `.dmg` from the [Releases page](https://github.com/denberek/antiwhisper/releases)
2. Drag antiwhisper to Applications
3. Right-click the app → **Open** (required on first launch since the app is not notarized)
4. Grant **Microphone** and **Accessibility** permissions when prompted
5. Download the recommended models on first launch (~1.6 GB total)
6. Start transcribing!

> See [GETTING_STARTED.md](GETTING_STARTED.md) for a detailed first-time setup guide.

### Development Setup

For building from source, see [BUILD.md](BUILD.md).

**Prerequisites:** [Rust](https://rustup.rs/) (latest stable), [Bun](https://bun.sh/)

```bash
bun install
bun run tauri dev
```

## Features

- **Multiple STT models** — Parakeet V3 (recommended, fast, CPU-only), Whisper variants (Small/Medium/Turbo/Large), Moonshine, SenseVoice, and more
- **Local AI post-processing** — Gemma 3 1B cleans up transcriptions on-device (macOS)
- **Configurable shortcuts** — customize recording, post-processing, and cancel hotkeys
- **Push-to-talk or toggle mode** — hold to record or press to start/stop
- **Recording overlay** — visual feedback while recording and transcribing
- **Transcription history** — review and replay past transcriptions
- **17 languages** — fully localized interface
- **Cross-platform** — macOS (Intel + Apple Silicon), Windows (x64), Linux (x64)

## Architecture

antiwhisper is built as a Tauri application:

- **Frontend**: React + TypeScript with Tailwind CSS
- **Backend**: Rust for system integration, audio processing, and ML inference
- **Core Libraries**:
  - `whisper-rs` — Whisper model inference
  - `transcription-rs` — Parakeet model inference (CPU-optimized)
  - `cpal` — Cross-platform audio I/O
  - `vad-rs` — Voice Activity Detection (Silero)
  - `rdev` — Global keyboard shortcuts

### CLI Parameters

antiwhisper supports command-line flags for scripting and integration:

```bash
antiwhisper --toggle-transcription    # Toggle recording on/off
antiwhisper --toggle-post-process     # Toggle recording with post-processing
antiwhisper --cancel                  # Cancel current operation
antiwhisper --start-hidden            # Launch to tray without window
antiwhisper --debug                   # Enable verbose logging
```

> **macOS bundle:** `/Applications/antiwhisper.app/Contents/MacOS/antiwhisper --toggle-transcription`

### Debug Mode

Access advanced diagnostics: **Cmd+Shift+D** (macOS) or **Ctrl+Shift+D** (Windows/Linux)

## Known Issues

### Whisper Model Crashes

Whisper models crash on certain system configurations (Windows/Linux). If affected, use Parakeet V3 instead — it's faster and more reliable.

### Wayland Support (Linux)

Requires [`wtype`](https://github.com/atx/wtype) or [`dotool`](https://sr.ht/~geb/dotool/) for text input:

| Display Server | Tool | Install |
|----------------|------|---------|
| X11 | `xdotool` | `sudo apt install xdotool` |
| Wayland | `wtype` | `sudo apt install wtype` |
| Both | `dotool` | `sudo apt install dotool` |

The recording overlay is disabled by default on Linux because certain compositors treat it as the active window.

### Linux — Global Shortcuts on Wayland

System-level shortcuts must be configured through your desktop environment. Use the CLI flags as the command:

```bash
# Sway/i3
bindsym $mod+o exec antiwhisper --toggle-transcription

# Hyprland
bind = $mainMod, O, exec, antiwhisper --toggle-transcription
```

Unix signals also work:

| Signal | Action |
|--------|--------|
| `SIGUSR2` | Toggle transcription |
| `SIGUSR1` | Toggle with post-processing |

### Linux — Runtime Dependencies

If startup fails with `libgtk-layer-shell.so.0` error:

| Distro | Package |
|--------|---------|
| Ubuntu/Debian | `sudo apt install libgtk-layer-shell0` |
| Fedora/RHEL | `sudo dnf install gtk-layer-shell` |
| Arch | `sudo pacman -S gtk-layer-shell` |

## Troubleshooting

### Manual Model Installation

If you're behind a proxy or firewall, download models manually:

**App data directory:**
- **macOS**: `~/Library/Application Support/com.denberek.antiwhisper/`
- **Windows**: `C:\Users\{username}\AppData\Roaming\com.denberek.antiwhisper\`
- **Linux**: `~/.config/com.denberek.antiwhisper/`

Create a `models` folder inside and download from:

| Model | Size | URL |
|-------|------|-----|
| Whisper Small | 487 MB | `https://blob.handy.computer/ggml-small.bin` |
| Whisper Medium | 492 MB | `https://blob.handy.computer/whisper-medium-q4_1.bin` |
| Whisper Turbo | 1.6 GB | `https://blob.handy.computer/ggml-large-v3-turbo.bin` |
| Whisper Large | 1.1 GB | `https://blob.handy.computer/ggml-large-v3-q5_0.bin` |
| Parakeet V2 | 473 MB | `https://blob.handy.computer/parakeet-v2-int8.tar.gz` |
| Parakeet V3 | 478 MB | `https://blob.handy.computer/parakeet-v3-int8.tar.gz` |

Place `.bin` files directly in `models/`. Extract `.tar.gz` archives so the directory name matches exactly (e.g., `parakeet-tdt-0.6b-v3-int8/`). Restart antiwhisper to detect them.

### Custom Whisper Models

Place any Whisper GGML `.bin` file in the `models` directory. antiwhisper will auto-discover it on restart.

## System Requirements

**Parakeet V3 (recommended):** CPU-only, Intel Skylake (6th gen) or newer, ~5x real-time speed.

**Whisper models:** GPU recommended — Metal on macOS, Vulkan on Windows/Linux.

**Disk space:** ~1.3 GB for recommended models (Parakeet V3 + Gemma 3 1B).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

1. Check [existing issues](https://github.com/denberek/antiwhisper/issues)
2. Fork and create a feature branch
3. Test on your target platform
4. Submit a pull request

## Attribution

antiwhisper is a fork of [Handy](https://github.com/cjpais/Handy) by CJ Pais — a free, open-source speech-to-text application. We're grateful for the foundation CJ built and the community around it.

## License

MIT License — see [LICENSE](LICENSE) for details.

## Acknowledgments

- **[Handy](https://github.com/cjpais/Handy)** by CJ Pais — the original project
- **[Whisper](https://github.com/openai/whisper)** by OpenAI — speech recognition model
- **[whisper.cpp](https://github.com/ggerganov/whisper.cpp)** — cross-platform Whisper inference
- **[Silero VAD](https://github.com/snakers4/silero-vad)** — voice activity detection
- **[Tauri](https://tauri.app)** — Rust-based app framework
