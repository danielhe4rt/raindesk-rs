# RainDesk for Linux — Full Technical Plan

**Target:** Replicate [raindesk.app](https://raindesk.app/) on Arch Linux + Wayland/Hyprland  
**Stack:** Tauri 2 + Vue.js 3 (TypeScript) + Rust rain engine  
**Codename:** `raindesk-linux`

---

## 1. What We're Building

RainDesk is a macOS app that renders animated rain drops **over your entire desktop**, with controls for intensity, speed, angle, color, opacity, handcrafted presets, and a Pomodoro timer. We're building a Linux-native equivalent.

### Feature Parity Checklist

| Feature | RainDesk (macOS) | Our Implementation |
|---|---|---|
| Rain overlay on desktop | ✅ macOS NSWindow | `wlr-layer-shell` overlay surface |
| Click-through | ✅ | Empty Wayland input region |
| Intensity control | ✅ | Particle spawn rate |
| Speed control | ✅ | Drop velocity multiplier |
| Angle / wind | ✅ | X-axis velocity component |
| Color picker | ✅ | RGBA uniform in shader |
| Opacity | ✅ | Alpha blending |
| Drop length | ✅ | Configurable line length |
| Presets | ✅ | TOML preset files |
| Pomodoro timer | ✅ | Rust timer + Vue UI |
| Rain sounds | 🔜 (planned) | PipeWire audio playback |
| Snow effects | 🔜 (planned) | Particle shape variant |
| Multi-monitor | ✅ | One overlay per `wl_output` |
| Dock/bar impact | ✅ | Splash particles at screen edges |

---

## 2. Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                    USER'S DESKTOP                        │
│                                                         │
│  ┌───────────────────────────────────────────────────┐  │
│  │          RAIN OVERLAY (layer-shell OVERLAY)       │  │
│  │     Transparent, click-through, fullscreen        │  │
│  │     OpenGL particle system, ~60fps render loop    │  │
│  └───────────────────────┬───────────────────────────┘  │
│                          │                              │
│                 Arc<Mutex<RainConfig>>                   │
│                          │                              │
│  ┌───────────────────────┴───────────────────────────┐  │
│  │              TAURI RUST BACKEND                    │  │
│  │  • AppState (shared config)                       │  │
│  │  • #[tauri::command] handlers                     │  │
│  │  • Config persistence (TOML)                      │  │
│  │  • Preset manager                                 │  │
│  │  • Pomodoro timer (accurate Rust Instant-based)   │  │
│  │  • IPC to overlay thread/process                  │  │
│  └───────────────────────┬───────────────────────────┘  │
│                          │                              │
│                   Tauri Commands                        │
│                  (invoke / listen)                       │
│                          │                              │
│  ┌───────────────────────┴───────────────────────────┐  │
│  │              VUE.JS 3 FRONTEND                     │  │
│  │  • RainControls (sliders)                         │  │
│  │  • PresetGrid (preset cards)                      │  │
│  │  • ColorPicker                                    │  │
│  │  • PomodoroTimer (circular countdown)             │  │
│  │  • MasterToggle                                   │  │
│  │  • Pinia store (synced to Rust via invoke)        │  │
│  └───────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

### Why the Split?

Tauri's `webkit2gtk` webview **cannot** be a Wayland layer-shell surface, and it cannot set an empty input region for click-through. The rain overlay **must** be a separate native Wayland surface. Two strategies:

**Strategy A — Same Process (preferred if stable):**  
Spawn the overlay on a separate Rust thread inside the Tauri app. Share config via `Arc<Mutex<RainConfig>>`. Simpler state management.

**Strategy B — Separate Process (fallback):**  
Run `raindesk-overlay` as a child binary. Communicate via Unix domain socket or shared memory. Cleaner isolation, avoids potential conflicts between two Wayland surfaces in one process.

> **Recommendation:** Start with Strategy A. If you hit issues with `wl_display` threading, pivot to Strategy B.

---

## 3. Tech Stack

| Layer | Technology | Justification |
|---|---|---|
| Framework | Tauri 2 | Lightweight, Rust backend, native Linux support |
| Frontend | Vue.js 3 + TypeScript | Per requirement |
| State management | Pinia | Official Vue store, reactive sync to Rust |
| CSS | Tailwind CSS or UnoCSS | Rapid UI development |
| Rust backend | Tauri + custom commands | Settings, timer, config persistence |
| Overlay surface | `wayland-client` + `wlr-layer-shell-unstable-v1` | Hyprland-native transparent overlay |
| Rendering | `glow` (OpenGL ES 3.0) or `wgpu` | GPU-accelerated particle system |
| EGL context | `glutin` or raw `khronos-egl` | OpenGL on Wayland surface |
| Math | `glam` | Fast vec2/mat4 for particles |
| Config | `serde` + `toml` | Human-editable config files |
| File watching | `notify` crate | Hot-reload config changes |
| Notifications | `notify-rust` | Pomodoro desktop alerts |
| Audio (future) | `rodio` or `pipewire-rs` | Rain sound playback |

---

## 4. Project Structure

```
raindesk/
├── package.json
├── tsconfig.json
├── vite.config.ts
├── tailwind.config.js
│
├── src/                              # Vue.js frontend
│   ├── App.vue                       # Main layout + sidebar/panel
│   ├── main.ts                       # Vue app bootstrap
│   │
│   ├── types/
│   │   ├── rain.ts                   # RainConfig, Preset, PomodoroState interfaces
│   │   └── events.ts                 # Tauri event payload types
│   │
│   ├── stores/
│   │   ├── rainStore.ts              # Pinia: rain config, synced to Rust
│   │   ├── presetStore.ts            # Pinia: preset list + active preset
│   │   └── pomodoroStore.ts          # Pinia: timer state, work/break durations
│   │
│   ├── composables/
│   │   ├── useRain.ts                # invoke() wrappers for rain commands
│   │   ├── usePresets.ts             # invoke() wrappers for preset CRUD
│   │   ├── usePomodoro.ts            # invoke() + listen() for timer events
│   │   └── useTheme.ts              # Dark/light mode handling
│   │
│   ├── components/
│   │   ├── layout/
│   │   │   ├── AppShell.vue          # Window chrome, drag region, close/minimize
│   │   │   └── Sidebar.vue           # Navigation: Controls | Presets | Pomodoro
│   │   │
│   │   ├── controls/
│   │   │   ├── RainControls.vue      # All sliders grouped
│   │   │   ├── RangeSlider.vue       # Reusable slider with label + value
│   │   │   ├── ColorPicker.vue       # RGBA color selector
│   │   │   └── MasterToggle.vue      # On/off switch with animation
│   │   │
│   │   ├── presets/
│   │   │   ├── PresetGrid.vue        # Grid of preset cards
│   │   │   ├── PresetCard.vue        # Single preset (name, preview, apply)
│   │   │   └── PresetEditor.vue      # Save current config as new preset
│   │   │
│   │   └── pomodoro/
│   │       ├── PomodoroTimer.vue     # Circular progress + time display
│   │       ├── PomodoroControls.vue  # Start/pause/reset/skip buttons
│   │       └── PomodoroSettings.vue  # Work/break duration config
│   │
│   └── assets/
│       ├── styles/
│       │   └── main.css              # Tailwind imports + custom styles
│       └── icons/                    # SVG icons for UI
│
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/
│   │   └── default.json              # Tauri permissions
│   │
│   ├── src/
│   │   ├── main.rs                   # Tauri builder, spawn overlay, register commands
│   │   ├── lib.rs                    # Module declarations
│   │   │
│   │   ├── state.rs                  # AppState, Arc<Mutex<RainConfig>>, shared types
│   │   ├── commands.rs               # All #[tauri::command] handlers
│   │   ├── config.rs                 # TOML load/save, file watching, hot-reload
│   │   ├── presets.rs                # Preset discovery, loading, saving
│   │   ├── pomodoro.rs               # Timer logic (Instant-based, Tauri events)
│   │   │
│   │   └── rain/
│   │       ├── mod.rs                # Rain engine module
│   │       ├── overlay.rs            # Wayland layer-shell surface creation
│   │       ├── egl.rs                # EGL context on Wayland surface
│   │       ├── renderer.rs           # OpenGL shader program, draw calls
│   │       ├── particles.rs          # Particle system: spawn, update, recycle
│   │       └── shaders/
│   │           ├── rain.vert         # Vertex shader
│   │           └── rain.frag         # Fragment shader
│   │
│   └── presets/                      # Bundled default presets
│       ├── gentle.toml
│       ├── storm.toml
│       ├── drizzle.toml
│       ├── neon.toml
│       ├── mist.toml
│       └── downpour.toml
│
└── README.md
```

---

## 5. Data Models

### 5.1 RainConfig (shared between Rust + Vue)

**Rust (`src-tauri/src/state.rs`):**

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RainConfig {
    pub enabled: bool,
    pub intensity: f32,      // 0.0..=1.0 — maps to spawn rate (e.g. 0..2000 drops/sec)
    pub speed: f32,          // 0.5..=5.0 — velocity multiplier
    pub angle: f32,          // -60.0..=60.0 — degrees from vertical
    pub drop_length: f32,    // 5.0..=40.0 — pixels
    pub drop_width: f32,     // 1.0..=4.0 — pixels
    pub color: [f32; 4],     // RGBA, each 0.0..=1.0
    pub opacity: f32,        // 0.0..=1.0 — master overlay opacity
    pub splash_enabled: bool,
    pub splash_intensity: f32,
    pub preset: Option<String>,
}

impl Default for RainConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            intensity: 0.5,
            speed: 1.0,
            angle: 0.0,
            drop_length: 20.0,
            drop_width: 1.5,
            color: [0.7, 0.8, 1.0, 0.6],
            opacity: 1.0,
            splash_enabled: true,
            splash_intensity: 0.5,
            preset: Some("gentle".into()),
        }
    }
}
```

**TypeScript (`src/types/rain.ts`):**

```typescript
export interface RainConfig {
  enabled: boolean
  intensity: number      // 0–1
  speed: number          // 0.5–5
  angle: number          // -60 to 60
  dropLength: number     // 5–40
  dropWidth: number      // 1–4
  color: [number, number, number, number]  // RGBA 0–1
  opacity: number        // 0–1
  splashEnabled: boolean
  splashIntensity: number
  preset: string | null
}

export interface Preset {
  name: string
  displayName: string
  description: string
  config: Partial<RainConfig>
  builtin: boolean
}

export interface PomodoroState {
  phase: 'idle' | 'work' | 'break' | 'longBreak'
  remaining: number       // seconds
  total: number           // seconds
  sessionsCompleted: number
  workDuration: number    // minutes
  breakDuration: number   // minutes
  longBreakDuration: number
  longBreakInterval: number
}
```

### 5.2 Preset File Format

`~/.config/raindesk/presets/storm.toml`:

```toml
[meta]
name = "storm"
display_name = "Storm"
description = "Heavy downpour with strong wind"

[config]
intensity = 0.95
speed = 3.5
angle = 25.0
drop_length = 35.0
drop_width = 2.0
color = [0.6, 0.7, 0.9, 0.8]
opacity = 1.0
splash_enabled = true
splash_intensity = 0.9
```

### 5.3 App Config

`~/.config/raindesk/config.toml`:

```toml
[rain]
enabled = true
intensity = 0.5
speed = 1.0
angle = 0.0
drop_length = 20.0
drop_width = 1.5
color = [0.7, 0.8, 1.0, 0.6]
opacity = 1.0
splash_enabled = true
splash_intensity = 0.5
preset = "gentle"

[pomodoro]
work_duration = 25
break_duration = 5
long_break_duration = 15
long_break_interval = 4
auto_start_breaks = true
auto_start_work = false
notification_sound = true

[display]
target_fps = 60
max_drops = 3000
multi_monitor = true
```

---

## 6. Tauri Commands API

All commands registered in `src-tauri/src/commands.rs`:

```rust
// ── Rain Control ──
#[tauri::command]
fn get_rain_config(state: State<'_, AppState>) -> RainConfig;

#[tauri::command]
fn set_rain_config(state: State<'_, AppState>, config: RainConfig);

#[tauri::command]
fn toggle_rain(state: State<'_, AppState>) -> bool; // returns new enabled state

#[tauri::command]
fn set_rain_param(state: State<'_, AppState>, param: String, value: f64);
// e.g. set_rain_param("intensity", 0.8)

// ── Presets ──
#[tauri::command]
fn list_presets(state: State<'_, AppState>) -> Vec<PresetMeta>;

#[tauri::command]
fn apply_preset(state: State<'_, AppState>, name: String);

#[tauri::command]
fn save_preset(state: State<'_, AppState>, name: String, display_name: String, description: String);

#[tauri::command]
fn delete_preset(state: State<'_, AppState>, name: String) -> Result<(), String>;

// ── Pomodoro ──
#[tauri::command]
fn get_pomodoro_state(state: State<'_, AppState>) -> PomodoroState;

#[tauri::command]
fn pomodoro_start(state: State<'_, AppState>);

#[tauri::command]
fn pomodoro_pause(state: State<'_, AppState>);

#[tauri::command]
fn pomodoro_reset(state: State<'_, AppState>);

#[tauri::command]
fn pomodoro_skip(state: State<'_, AppState>);

#[tauri::command]
fn set_pomodoro_settings(state: State<'_, AppState>, settings: PomodoroSettings);
```

### Tauri Events (Rust → Vue)

```rust
// Emitted every second during active pomodoro
app.emit("pomodoro-tick", PomodoroState { ... });

// Emitted on phase transitions
app.emit("pomodoro-phase-change", PomodoroPhaseEvent {
    from: "work",
    to: "break",
    sessions_completed: 2,
});

// Emitted when config changes from external source (file edit, CLI)
app.emit("rain-config-changed", RainConfig { ... });
```

---

## 7. Rain Overlay Engine — Detailed Design

### 7.1 Wayland Surface Setup (`overlay.rs`)

```
1. wl_display::connect_to_env()
2. Get globals: wl_compositor, wl_shm, zwlr_layer_shell_v1
3. For each wl_output (monitor):
   a. compositor.create_surface() → wl_surface
   b. layer_shell.get_layer_surface(surface, output, OVERLAY, "raindesk")
   c. Set anchor: TOP | BOTTOM | LEFT | RIGHT (fullscreen)
   d. Set exclusive_zone: -1 (don't push windows)
   e. Set keyboard_interactivity: NONE
   f. surface.set_input_region(empty_region)  ← CRITICAL for click-through
   g. surface.commit()
4. Wait for configure event → get actual width/height
5. Initialize EGL on the wl_surface
6. Enter render loop
```

### 7.2 Particle System (`particles.rs`)

```rust
struct Raindrop {
    pos: Vec2,          // current position
    velocity: Vec2,     // per-frame movement
    length: f32,        // visual length in pixels
    width: f32,
    opacity: f32,       // individual drop opacity
    lifetime: f32,      // 0.0..1.0, for fade-in at spawn
}

struct SplashParticle {
    pos: Vec2,
    velocity: Vec2,
    life: f32,          // countdown to death
    max_life: f32,
    size: f32,
}

struct ParticleSystem {
    drops: Vec<Raindrop>,        // pre-allocated pool (max_drops)
    splashes: Vec<SplashParticle>,
    active_drops: usize,
    screen_width: f32,
    screen_height: f32,
}
```

**Per-frame update loop:**

```
1. Read config from Arc<Mutex<RainConfig>>
2. Calculate spawn_count = intensity * MAX_SPAWN_RATE * dt
3. Spawn new drops at top edge (randomized x, slight y offset)
   - velocity.y = base_speed * config.speed * random(0.8..1.2)
   - velocity.x = velocity.y * tan(config.angle.to_radians())
   - length = config.drop_length * random(0.7..1.3)
4. Update all active drops:
   - pos += velocity * dt
   - lifetime += dt * 3.0 (fade-in over ~0.3s)
   - If pos.y > screen_height → spawn splash + recycle drop
   - If pos.x outside screen → recycle drop
5. Update splash particles:
   - pos += velocity * dt
   - velocity.y += gravity * dt
   - life -= dt
   - Remove if life <= 0
6. Upload positions to GPU → draw
```

### 7.3 OpenGL Rendering (`renderer.rs`)

**Vertex shader (`rain.vert`):**

```glsl
#version 300 es
precision highp float;

layout(location = 0) in vec2 a_start;    // drop start position
layout(location = 1) in vec2 a_end;      // drop end position
layout(location = 2) in float a_opacity;
layout(location = 3) in float a_width;

uniform mat4 u_projection;               // orthographic

out float v_opacity;

void main() {
    v_opacity = a_opacity;
    // Each drop is a line from a_start to a_end
    // Using gl_VertexID to alternate between start/end
    vec2 pos = (gl_VertexID % 2 == 0) ? a_start : a_end;
    gl_Position = u_projection * vec4(pos, 0.0, 1.0);
}
```

**Fragment shader (`rain.frag`):**

```glsl
#version 300 es
precision highp float;

uniform vec4 u_color;
uniform float u_master_opacity;

in float v_opacity;
out vec4 fragColor;

void main() {
    fragColor = vec4(u_color.rgb, u_color.a * v_opacity * u_master_opacity);
}
```

**Draw call strategy:**
- Use a single VBO with instanced rendering or a large vertex buffer
- Upload all drop start/end positions each frame (dynamic buffer)
- `glDrawArrays(GL_LINES, 0, active_drops * 2)`
- Splash particles: separate draw call with `GL_POINTS` or small quads
- Blend mode: `glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA)`

### 7.4 Multi-Monitor

On Hyprland, each `wl_output` event represents a monitor. The overlay should:

1. Listen for `wl_registry` global events for `wl_output`
2. Create one layer-shell surface per output
3. Each surface gets its own EGL context (shared GL objects if possible)
4. All surfaces read from the same `Arc<Mutex<RainConfig>>`
5. Handle output add/remove events (monitor plug/unplug)

---

## 8. Vue.js Frontend — Detailed Design

### 8.1 App Layout

```
┌──────────────────────────────────┐
│  ☁ RainDesk            ─ □ ✕    │  ← Drag region + window controls
├────────┬─────────────────────────┤
│        │                         │
│  🌧️   │  ┌─ Intensity ────────┐ │
│Controls│  │ ═══════●══════     │ │
│        │  └────────────────────┘ │
│  🎨   │  ┌─ Speed ─────────── ┐ │
│Presets │  │ ════●══════════    │ │
│        │  └────────────────────┘ │
│  ⏱️   │  ┌─ Angle ─────────── ┐ │
│Pomodoro│  │ ═══════════●═════  │ │
│        │  └────────────────────┘ │
│        │                         │
│        │  [Color] [Opacity]      │
│        │                         │
│        │  ┌──────┐  ┌──────┐    │
│        │  │Master│  │Splash│    │
│        │  │ ON ⚡│  │ ON   │    │
│        │  └──────┘  └──────┘    │
├────────┴─────────────────────────┤
│  v0.1.0          Made with 🌧️   │
└──────────────────────────────────┘
```

Window size: ~420×580px, non-resizable (or constrained min/max).

### 8.2 Composables Detail

**`useRain.ts`:**

```typescript
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { RainConfig } from '@/types/rain'

export function useRain() {
  const getConfig = () => invoke<RainConfig>('get_rain_config')
  const setConfig = (config: RainConfig) => invoke('set_rain_config', { config })
  const toggle = () => invoke<boolean>('toggle_rain')
  const setParam = (param: string, value: number) =>
    invoke('set_rain_param', { param, value })

  const onConfigChanged = (cb: (config: RainConfig) => void) =>
    listen<RainConfig>('rain-config-changed', (e) => cb(e.payload))

  return { getConfig, setConfig, toggle, setParam, onConfigChanged }
}
```

**`usePomodoro.ts`:**

```typescript
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { PomodoroState } from '@/types/rain'

export function usePomodoro() {
  const getState = () => invoke<PomodoroState>('get_pomodoro_state')
  const start = () => invoke('pomodoro_start')
  const pause = () => invoke('pomodoro_pause')
  const reset = () => invoke('pomodoro_reset')
  const skip = () => invoke('pomodoro_skip')

  const onTick = (cb: (state: PomodoroState) => void) =>
    listen<PomodoroState>('pomodoro-tick', (e) => cb(e.payload))

  const onPhaseChange = (cb: (event: any) => void) =>
    listen('pomodoro-phase-change', (e) => cb(e.payload))

  return { getState, start, pause, reset, skip, onTick, onPhaseChange }
}
```

### 8.3 Pinia Store — Rain

```typescript
// stores/rainStore.ts
import { defineStore } from 'pinia'
import { ref, watch } from 'vue'
import { useDebounceFn } from '@vueuse/core'
import { useRain } from '@/composables/useRain'
import type { RainConfig } from '@/types/rain'

export const useRainStore = defineStore('rain', () => {
  const { getConfig, setConfig, onConfigChanged } = useRain()

  const config = ref<RainConfig>({
    enabled: true,
    intensity: 0.5,
    speed: 1.0,
    angle: 0.0,
    dropLength: 20.0,
    dropWidth: 1.5,
    color: [0.7, 0.8, 1.0, 0.6],
    opacity: 1.0,
    splashEnabled: true,
    splashIntensity: 0.5,
    preset: 'gentle',
  })

  const loading = ref(true)

  // Load initial config from Rust
  async function init() {
    config.value = await getConfig()
    loading.value = false
  }

  // Debounced sync: Vue → Rust (50ms, so sliders feel smooth)
  const syncToRust = useDebounceFn((val: RainConfig) => {
    setConfig(val)
  }, 50)

  watch(config, (val) => syncToRust(val), { deep: true })

  // Listen for external config changes (file edit, CLI)
  onConfigChanged((newConfig) => {
    config.value = newConfig
  })

  return { config, loading, init }
})
```

---

## 9. Pomodoro Timer — Detailed Design

### Rust Side (`pomodoro.rs`)

```rust
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};

#[derive(Clone, Serialize, Deserialize)]
pub enum Phase { Idle, Work, Break, LongBreak }

#[derive(Clone)]
pub struct PomodoroEngine {
    pub phase: Phase,
    pub started_at: Option<Instant>,
    pub paused_remaining: Option<Duration>,
    pub sessions_completed: u32,
    pub settings: PomodoroSettings,
}

impl PomodoroEngine {
    /// Call this from a dedicated thread every 200ms
    pub fn tick(&mut self, app: &AppHandle) {
        if let (Phase::Idle, _) | (_, None) = (&self.phase, &self.started_at) {
            return;
        }

        let elapsed = self.started_at.unwrap().elapsed();
        let total = self.phase_duration();
        let remaining = total.saturating_sub(elapsed);

        // Emit tick event to frontend
        let _ = app.emit("pomodoro-tick", PomodoroState {
            phase: self.phase.clone(),
            remaining: remaining.as_secs(),
            total: total.as_secs(),
            sessions_completed: self.sessions_completed,
            /* ... */
        });

        // Phase complete
        if remaining.is_zero() {
            self.transition(app);
        }
    }

    fn transition(&mut self, app: &AppHandle) {
        let from = self.phase.clone();
        match self.phase {
            Phase::Work => {
                self.sessions_completed += 1;
                if self.sessions_completed % self.settings.long_break_interval == 0 {
                    self.phase = Phase::LongBreak;
                } else {
                    self.phase = Phase::Break;
                }
            }
            Phase::Break | Phase::LongBreak => {
                self.phase = Phase::Work;
            }
            _ => {}
        }
        self.started_at = Some(Instant::now());

        // Desktop notification
        notify_rust::Notification::new()
            .summary("RainDesk Pomodoro")
            .body(&format!("{:?} complete! Starting {:?}.", from, self.phase))
            .show()
            .ok();

        let _ = app.emit("pomodoro-phase-change", /* ... */);
    }
}
```

### Vue Side (`PomodoroTimer.vue`)

Circular SVG progress ring showing remaining time, with start/pause/reset buttons below. The store listens to `pomodoro-tick` events and updates reactively.

---

## 10. Configuration & Persistence

### File Locations

```
~/.config/raindesk/
├── config.toml             # Current rain + pomodoro settings
└── presets/
    ├── gentle.toml         # Bundled (copied on first run)
    ├── storm.toml
    ├── drizzle.toml
    ├── neon.toml
    ├── mist.toml
    ├── downpour.toml
    └── my-custom.toml      # User-created
```

### Hot Reload

Use the `notify` crate to watch `config.toml`. When the file changes externally (user edits in vim, or a CLI tool writes it):

1. Re-parse the TOML
2. Update `Arc<Mutex<RainConfig>>`
3. Emit `rain-config-changed` event to Vue frontend

### First-Run Bootstrap

On first launch, if `~/.config/raindesk/` doesn't exist:

1. Create directory structure
2. Copy bundled presets from app resources to `~/.config/raindesk/presets/`
3. Write default `config.toml`

---

## 11. Build & Distribution

### System Dependencies (Arch)

```bash
# Core
pacman -S wayland wayland-protocols hyprland

# Build tools
pacman -S rust cargo nodejs npm

# Tauri deps
pacman -S webkit2gtk-4.1 gtk3 libappindicator-gtk3 \
          librsvg libsoup3 patchelf

# OpenGL / EGL
pacman -S mesa libglvnd egl-wayland

# Notifications
pacman -S libnotify

# Audio (future)
pacman -S pipewire pipewire-audio
```

### Build Commands

```bash
# Development
npm install
npm run tauri dev

# Production
npm run tauri build
# Output: src-tauri/target/release/bundle/
```

### AUR Package

Create a `PKGBUILD` for distribution:

```bash
pkgname=raindesk-linux
pkgver=0.1.0
pkgrel=1
pkgdesc="Rain overlay for Wayland/Hyprland with Pomodoro timer"
arch=('x86_64')
depends=('webkit2gtk-4.1' 'gtk3' 'wayland' 'mesa' 'libnotify')
makedepends=('rust' 'cargo' 'nodejs' 'npm')
```

### Autostart

Systemd user service (`~/.config/systemd/user/raindesk.service`):

```ini
[Unit]
Description=RainDesk Rain Overlay
After=graphical-session.target

[Service]
ExecStart=/usr/bin/raindesk --background
Restart=on-failure
Environment=WAYLAND_DISPLAY=wayland-1

[Install]
WantedBy=graphical-session.target
```

Or Hyprland autostart in `~/.config/hypr/hyprland.conf`:

```
exec-once = raindesk --background
```

---

## 12. Implementation Phases & Timeline

### Phase 1 — Skeleton + Proof of Concept (Days 1–3)

**Goal:** Transparent overlay with falling white lines on Hyprland.

- [ ] Scaffold Tauri 2 + Vue 3 + TypeScript project
- [ ] Add Rust dependencies (`wayland-client`, `glow`, `glam`, etc.)
- [ ] Implement `overlay.rs`: connect to Wayland, create layer-shell surface
- [ ] Implement `egl.rs`: initialize EGL/OpenGL context on the surface
- [ ] Implement basic `renderer.rs`: clear to transparent, draw 100 falling lines
- [ ] Confirm click-through works on Hyprland
- [ ] Spawn overlay thread from `main.rs` alongside Tauri

**Deliverable:** Rain lines falling on desktop, Tauri window opens separately.

### Phase 2 — Rain Engine (Days 4–7)

**Goal:** Full particle system with all configurable parameters.

- [ ] Implement `particles.rs`: particle pool, spawn/update/recycle
- [ ] Add wind angle (x-velocity component from angle)
- [ ] Variable drop sizes (randomized around config value)
- [ ] Per-drop opacity fade-in
- [ ] Splash particles at bottom edge
- [ ] Instanced rendering or batched GL_LINES for performance
- [ ] Support `Arc<Mutex<RainConfig>>` reads at 60fps
- [ ] Performance target: 3000 drops at 60fps on integrated GPU

**Deliverable:** Beautiful, configurable rain at interactive framerates.

### Phase 3 — Vue Frontend + Tauri Bridge (Days 8–12)

**Goal:** Full settings UI controlling the rain in real-time.

- [ ] Define TypeScript interfaces (`types/rain.ts`)
- [ ] Implement all Tauri commands (`commands.rs`)
- [ ] Implement Pinia stores with debounced sync
- [ ] Build `RainControls.vue` with sliders for all parameters
- [ ] Build `ColorPicker.vue` (RGBA)
- [ ] Build `MasterToggle.vue`
- [ ] Build `AppShell.vue` with sidebar navigation
- [ ] Style everything (dark theme to match rainy aesthetic)
- [ ] Test real-time slider → rain updates

**Deliverable:** Working app where sliders control rain overlay live.

### Phase 4 — Presets (Days 13–14)

**Goal:** Preset system with bundled + custom presets.

- [ ] Implement `presets.rs`: scan directory, load TOML, apply
- [ ] Create 6 bundled presets (gentle, storm, drizzle, neon, mist, downpour)
- [ ] Build `PresetGrid.vue` + `PresetCard.vue`
- [ ] Build `PresetEditor.vue` (save current as new preset)
- [ ] Smooth transitions between presets (lerp config values over ~500ms)
- [ ] First-run bootstrap: copy bundled presets to config dir

**Deliverable:** Click a preset card, rain smoothly transitions.

### Phase 5 — Pomodoro Timer (Days 15–17)

**Goal:** Fully functional Pomodoro with desktop notifications.

- [ ] Implement `pomodoro.rs`: timer engine on dedicated thread
- [ ] Emit tick events + phase change events via Tauri
- [ ] Build `PomodoroTimer.vue` with SVG circular progress
- [ ] Build `PomodoroControls.vue` (start/pause/reset/skip)
- [ ] Build `PomodoroSettings.vue` (durations config)
- [ ] Desktop notifications via `notify-rust` on phase transitions
- [ ] Optional: flash rain color briefly on phase change

**Deliverable:** Working Pomodoro timer synced with rain overlay.

### Phase 6 — Config Persistence + Hot Reload (Days 18–19)

**Goal:** Settings survive restarts, external edits reflect live.

- [ ] Implement `config.rs`: save on every change, load on startup
- [ ] File watcher with `notify` crate for hot-reload
- [ ] Emit `rain-config-changed` to frontend on external edit
- [ ] Handle corrupt/missing config gracefully (fallback to defaults)

**Deliverable:** Edit config.toml in vim → rain changes immediately.

### Phase 7 — Multi-Monitor (Days 20–21)

**Goal:** Rain renders on all connected monitors.

- [ ] Listen for `wl_output` events in overlay
- [ ] Create one layer-shell surface per monitor
- [ ] Handle monitor hotplug (add/remove)
- [ ] Shared GL context or per-monitor contexts

**Deliverable:** Rain on all monitors, plug/unplug handled.

### Phase 8 — Polish & Distribution (Days 22–25)

**Goal:** Production-ready release.

- [ ] Performance profiling + optimization (GPU memory, CPU usage)
- [ ] Error handling: graceful degradation if layer-shell unavailable
- [ ] CLI flags: `--background`, `--preset <name>`, `--toggle`
- [ ] Systemd user service file
- [ ] Hyprland exec-once integration
- [ ] PKGBUILD for AUR
- [ ] README with screenshots + install instructions
- [ ] App icon + about dialog

**Deliverable:** Installable, polished app.

---

## 13. Stretch Goals (Post-v1)

| Feature | Approach |
|---|---|
| **Rain sounds** | `rodio` or `pipewire-rs`, looped ambient audio, volume tied to intensity |
| **Snow mode** | Alternate particle shape (dots/flakes), slower, no splash, drift side-to-side |
| **Fog/mist** | Perlin noise overlay with low-opacity white layer |
| **CLI control** | `raindesk-cli set intensity 0.8` via Unix socket IPC |
| **Tray icon** | `tauri-plugin-system-tray` or `ksni` for system tray integration |
| **Keyboard shortcuts** | Global keybinds via Hyprland `bind` or `libinput` |
| **Window-aware rain** | Query `hyprctl clients` to render rain only on desktop, not over focused window |
| **Screen recording friendly** | Option to render rain into a virtual `v4l2` source |

---

## 14. Risk Register

| Risk | Impact | Mitigation |
|---|---|---|
| Two Wayland surfaces in one process conflicts | High | Fallback to Strategy B (separate overlay process) |
| `webkit2gtk` + raw Wayland in same process crashes | High | Isolate overlay in child process, communicate via Unix socket |
| OpenGL context creation fails on some GPUs | Medium | Fallback to software renderer (`LIBGL_ALWAYS_SOFTWARE=1`) |
| Layer-shell not available (non-wlroots compositors) | Medium | Detect at runtime, show error dialog, document compositor requirements |
| Performance on integrated GPUs | Medium | Cap max_drops, reduce FPS to 30, simplify shaders |
| `notify` file watcher misses events | Low | Periodic poll fallback every 5s |
| Tauri 2 Wayland support regressions | Low | Pin Tauri version, test on updates |

---

## 15. Testing Checklist

- [ ] Overlay renders on Hyprland
- [ ] Click-through: can interact with windows behind overlay
- [ ] All sliders update rain in real-time (< 100ms latency)
- [ ] Preset transitions smooth
- [ ] Pomodoro timer accurate (< 1s drift over 25 minutes)
- [ ] Desktop notifications fire on phase change
- [ ] Config persists across app restart
- [ ] Hot-reload: edit TOML → rain changes
- [ ] Multi-monitor: rain on all screens
- [ ] Memory stable after 1 hour runtime (no leaks)
- [ ] CPU usage < 5% idle, < 15% during storm preset
- [ ] GPU usage reasonable (check with `nvidia-smi` or `intel_gpu_top`)
- [ ] Graceful startup if Hyprland not running (show error, don't crash)
- [ ] Graceful handling of monitor plug/unplug
