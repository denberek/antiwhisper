# Getting Started with antiwhisper

A quick guide to get up and running with antiwhisper on macOS.

## Installation

1. Download **antiwhisper_0.7.7_aarch64.dmg** from the [Releases page](https://github.com/denberek/antiwhisper/releases)
2. Open the DMG and drag **antiwhisper** into your Applications folder
3. **Important:** Right-click antiwhisper in Applications → click **Open** (required on first launch since the app is not code-signed)
4. macOS will ask "Are you sure you want to open it?" → click **Open**

## Granting Permissions

On first launch, antiwhisper will ask for two system permissions:

- **Microphone** — required to hear your voice
- **Accessibility** — required to type the transcribed text into your apps

Click the **Grant** button for each. macOS will open System Settings where you toggle antiwhisper on.

## Downloading Models

After permissions are granted, antiwhisper downloads two AI models:

| Model | Purpose | Size |
|-------|---------|------|
| **Parakeet V3** | Speech-to-text transcription | ~478 MB |
| **Gemma 3 1B** | AI text cleanup (post-processing) | ~806 MB |

Wait for both downloads to complete. The app will automatically advance once ready.

## How to Use

### Basic Transcription

1. Press **Option + Space** to start recording
2. Speak your text
3. Press **Option + Space** again to stop
4. The transcribed text is automatically pasted into your active text field

> **Note:** The first transcription after launch loads the model into memory, so it may take a few extra seconds. Every transcription after that will be near-instant.

### With AI Post-Processing

Hold **Option** (left Option key alone) to record with AI cleanup. This mode fixes spelling, punctuation, capitalization, and removes filler words before pasting.

### Cancel Recording

Press **Escape** or click **Cancel** on the overlay to discard a recording.

## Settings

Access settings by:
- Clicking the antiwhisper icon in the menu bar → **Settings**
- Or pressing **Cmd + ,**

### Key Settings

| Setting | Location | Default |
|---------|----------|---------|
| Recording shortcut | General | Option + Space |
| Post-processing shortcut | General | Option |
| Push-to-talk mode | General | Off (toggle mode) |
| Auto-start on login | Advanced | On |
| Audio feedback sounds | General | On |
| Transcription history | History | 5 entries kept |

## Tips

- **Tray icon**: antiwhisper lives in your menu bar. Click it for quick access to settings or to copy your last transcript.
- **History**: Go to Settings → History to see and replay past transcriptions.
- **Custom words**: Add frequently misheard words in Settings → Advanced → Custom Words.
- **Model switching**: Go to Settings → Models to try different speech recognition engines.

## Troubleshooting

| Problem | Solution |
|---------|----------|
| "Unidentified developer" warning | Right-click → Open (first time only) |
| No text appearing after recording | Check Accessibility permission in System Settings → Privacy & Security → Accessibility |
| Shortcut not working | Check for conflicts with other apps; try a different shortcut in Settings |
| Models won't download | Check your internet connection; see [manual installation](README.md#manual-model-installation) |
| App not in menu bar | Check Settings → Advanced → Show Tray Icon is enabled |

## System Requirements

- macOS 10.15 or later (Apple Silicon or Intel)
- ~2 GB disk space for models
- A microphone (built-in or external)

## Uninstalling

1. Quit antiwhisper (menu bar icon → Quit, or Cmd+Q)
2. Delete antiwhisper from Applications
3. Optionally remove app data: `~/Library/Application Support/com.denberek.antiwhisper/`
