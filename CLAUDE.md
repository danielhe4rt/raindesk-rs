# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

RainDesk is a Linux desktop application that renders animated rain drops over the entire desktop with configurable intensity, speed, angle, color, opacity, presets, and a Pomodoro timer. It's a Linux port of the macOS app raindesk.app, targeting Arch Linux + Wayland/Hyprland.

## Tech Stack

- **Frontend**: Vue.js 3 + TypeScript + Vite
- **Backend**: Tauri 2 (Rust)
- **Package Manager**: Bun
- **Rain Overlay**: Native Wayland (`wlr-layer-shell`) + OpenGL ES 3.0 (planned)
- **Config Format**: TOML with `serde`

## Build & Development Commands

```bash
# Install dependencies
bun install

# Development (starts both Vite dev server and Tauri)
npm run tauri dev

# Build for production
npm run tauri build

# Frontend-only dev server (port 1420)
npm run dev

# Type check frontend
vue-tsc --noEmit
```

## Architecture

### Three-Layer System

1. **Rain Overlay** (planned): Native Wayland layer-shell surface with OpenGL particle rendering, runs on separate Rust thread, shares config via `Arc<Mutex<RainConfig>>`

2. **Tauri Rust Backend** (`src-tauri/src/`): State management, `#[tauri::command]` handlers, config persistence (TOML), preset manager, Pomodoro timer

3. **Vue.js Frontend** (`src/`): Control panel UI with sliders, preset grid, color picker, Pomodoro timer display. Communicates with Rust via Tauri `invoke()` and `listen()`

### Why Separate Overlay?

Tauri's webkit2gtk webview cannot be a Wayland layer-shell surface and cannot set empty input region for click-through. The rain overlay must be a separate native Wayland surface.

## Key Files

- `src-tauri/src/lib.rs` - Tauri app setup and command registration
- `src-tauri/src/main.rs` - Entry point (delegates to lib.rs)
- `src-tauri/tauri.conf.json` - Tauri configuration (window size, bundle settings)
- `src/App.vue` - Main Vue component
- `src/main.ts` - Vue app bootstrap

## Core Data Model

`RainConfig` is the central shared type between Rust and Vue:
- `enabled`, `intensity` (0-1), `speed` (0.5-5), `angle` (-60 to 60)
- `drop_length`, `drop_width`, `color` (RGBA), `opacity`
- `splash_enabled`, `splash_intensity`, `preset`

## Planned Module Structure (Rust)

```
src-tauri/src/
├── state.rs      # AppState, Arc<Mutex<RainConfig>>
├── commands.rs   # Tauri command handlers
├── config.rs     # TOML persistence, hot-reload
├── presets.rs    # Preset discovery/management
├── pomodoro.rs   # Timer logic
└── rain/         # Overlay engine
    ├── overlay.rs    # Wayland layer-shell
    ├── egl.rs        # EGL context
    ├── renderer.rs   # OpenGL shaders
    └── particles.rs  # Particle system
```

## Planned Rust Dependencies

- `wayland-client` + `wlr-layer-shell-unstable-v1` for overlay
- `glow` or `wgpu` for OpenGL rendering
- `glam` for vector math
- `notify` for config file watching
- `notify-rust` for desktop notifications
