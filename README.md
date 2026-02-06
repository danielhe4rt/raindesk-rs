# RainDesk

Animated rain overlay for Linux desktops with a built-in Pomodoro timer. A native Wayland application that renders configurable rain particles over your entire screen with click-through transparency.

Linux port of [raindesk.app](https://raindesk.app), targeting Arch Linux + Wayland (Hyprland).

## Features

### Rain Overlay
- Real-time rain particle rendering over your entire desktop
- Full click-through transparency -- interact with windows below as normal
- Configurable intensity, speed, wind angle, drop size, color, and opacity
- Splash effects when raindrops hit the bottom of the screen
- Runs at 60fps on a dedicated thread with instanced OpenGL ES 3.0 rendering

### Presets

8 built-in presets to quickly switch moods:

| Preset | Style | Vibe |
|--------|-------|------|
| **Light Drizzle** | Gentle, slow, soft | Calm afternoon |
| **Steady Rain** | Moderate, balanced | Classic rainfall |
| **Heavy Downpour** | Fast, dense, slight wind | Caught outside |
| **Windy Storm** | Strong 30-degree angle | Dramatic weather |
| **Misty** | Fine, slow, no splashes | Morning fog |
| **Cyberpunk** | Neon cyan tint | Blade Runner |
| **Warm Sunset** | Golden color, gentle | Evening glow |
| **Blood Rain** | Deep red, ominous | Horror aesthetic |

### Pomodoro Timer
- 25/5/15 minute work/short-break/long-break cycle
- Desktop notifications on phase transitions
- Customizable durations and session count
- Start, pause, reset, and skip controls

### Rain Controls
- **Intensity** -- Spawn rate from light drizzle to downpour (50-3000 drops/sec)
- **Speed** -- 0.5x to 5x velocity multiplier
- **Wind Angle** -- -60 to +60 degrees
- **Drop Length** -- 5 to 100 pixels
- **Drop Width** -- 1 to 10 pixels
- **Color** -- Full RGBA color picker
- **Opacity** -- 0-100% global overlay opacity
- **Splash Effects** -- Toggle with adjustable intensity

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Frontend | Vue.js 3 + TypeScript + Vite |
| Backend | Tauri 2 (Rust) |
| Rendering | OpenGL ES 3.0 via `glow` (instanced particle system) |
| Windowing | Wayland `wlr-layer-shell` for fullscreen overlay |
| Graphics Context | EGL (dynamic loading via `khronos-egl`) |
| Config | TOML with auto-save to `~/.config/raindesk/config.toml` |
| Package Manager | Bun |

## Architecture

RainDesk uses a three-layer architecture:

```
+-----------------------------------------+
|   Vue.js Control Panel                  |  UI: sliders, presets, pomodoro
|   (Tauri WebView, 420x700)              |  Communicates via invoke()
+-------------------+---------------------+
                    | Tauri IPC
+-------------------v---------------------+
|   Rust Backend                          |  State management, config persistence,
|   (Main Thread)                         |  preset manager, pomodoro timer
+-------------------+---------------------+
                    | mpsc::channel + Arc<Mutex<RainConfig>>
+-------------------v---------------------+
|   Rain Overlay                          |  Wayland layer-shell surface,
|   (Dedicated OS Thread)                 |  OpenGL particle rendering @ 60fps
+-----------------------------------------+
```

The overlay runs on its own OS thread with a separate Wayland connection. Config changes from the UI propagate through shared `Arc<Mutex<RainConfig>>` state and `mpsc::channel` signals. The overlay surface uses an empty input region so all clicks pass through to the desktop beneath.

### Why a separate overlay?

Tauri's `webkit2gtk` webview cannot be a Wayland layer-shell surface and cannot set an empty input region for click-through. The rain overlay must be a separate native Wayland surface running on its own thread.

## Prerequisites

- **Linux** with a Wayland compositor supporting `wlr-layer-shell` (Hyprland, Sway, etc.)
- **Rust** toolchain (1.70+)
- **Bun** (or npm/pnpm)

### Arch Linux

```bash
sudo pacman -S webkit2gtk-4.1 mesa wayland base-devel
```

### Ubuntu / Debian

```bash
sudo apt install libwebkit2gtk-4.1-dev libssl-dev libayatana-appindicator3-dev \
    librsvg2-dev libegl1-mesa-dev libwayland-dev
```

## Building

```bash
# Install frontend dependencies
bun install

# Development (hot-reload for both frontend and backend)
npm run tauri dev

# Production build
npm run tauri build
```

The production binary will be in `src-tauri/target/release/raindesk`.

### Other Commands

```bash
# Frontend-only dev server (port 1420)
npm run dev

# TypeScript type checking
vue-tsc --noEmit

# Rust type checking only
cargo check --manifest-path src-tauri/Cargo.toml
```

## Project Structure

```
src/                            # Vue frontend
  App.vue                       # Control panel UI
  types.ts                      # TypeScript types matching Rust structs
  main.ts                       # Vue entry point

src-tauri/src/                  # Rust backend
  lib.rs                        # Tauri setup and command registration
  state.rs                      # Shared state (AppState, OverlaySignal)
  config.rs                     # RainConfig + TOML persistence
  commands.rs                   # 21 Tauri command handlers
  presets.rs                    # 8 built-in rain presets
  pomodoro.rs                   # Pomodoro timer state machine
  rain/
    overlay.rs                  # Wayland layer-shell surface + frame loop
    egl.rs                      # EGL context for Wayland
    renderer.rs                 # OpenGL ES 3.0 instanced shaders
    particles.rs                # Raindrop + splash particle system
```

## Configuration

Settings are persisted to `~/.config/raindesk/config.toml` and saved automatically on every change.

```toml
enabled = true
intensity = 0.5
speed = 1.0
angle = 0.0
drop_length = 20.0
drop_width = 2.0
opacity = 0.7
splash_enabled = true
splash_intensity = 0.5

[color]
r = 174
g = 194
b = 224
a = 180
```

## Autostart

### Hyprland

Add to `~/.config/hypr/hyprland.conf`:

```
exec-once = raindesk
```

### Systemd

```ini
# ~/.config/systemd/user/raindesk.service
[Unit]
Description=RainDesk Rain Overlay
After=graphical-session.target

[Service]
ExecStart=/usr/bin/raindesk
Restart=on-failure

[Install]
WantedBy=graphical-session.target
```

```bash
systemctl --user enable --now raindesk
```

## Compositor Compatibility

| Compositor | Status |
|-----------|--------|
| Hyprland | Supported (primary target) |
| Sway | Should work (wlr-layer-shell) |
| River | Should work (wlr-layer-shell) |
| GNOME/Mutter | Not supported (no wlr-layer-shell) |
| KDE/KWin | Not supported (no wlr-layer-shell) |

## License

MIT
