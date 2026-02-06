# AGENTS.md

Instructions for AI coding agents working in this repository.

## Project Overview

RainDesk is a Tauri 2 desktop app (Rust backend + Vue.js 3 frontend) that renders animated rain over a Linux/Wayland desktop. It includes configurable rain settings and a Pomodoro timer.

## Build & Development Commands

```bash
# Install frontend dependencies
bun install

# Development mode (Vite dev server + Tauri together)
npm run tauri dev

# Production build
npm run tauri build

# Frontend-only dev server (port 1420)
npm run dev

# Type-check frontend
npx vue-tsc --noEmit

# Type-check/compile Rust backend
cargo check                              # in src-tauri/
cargo build                              # full build in src-tauri/

# Run all Rust tests
cargo test                               # in src-tauri/

# Run a single Rust test by name
cargo test test_name                     # in src-tauri/
cargo test test_name -- --exact          # exact match

# Run tests in a specific module
cargo test module_name::                 # in src-tauri/

# Rust linting
cargo clippy -- -D warnings             # in src-tauri/
```

There are no frontend test or lint scripts configured. No ESLint, Prettier, or Biome.
There is no CI/CD pipeline. No rustfmt.toml or clippy.toml config files exist.

## Architecture

Three-layer system:

1. **Tauri Rust Backend** (`src-tauri/src/`): State management via `Mutex`, `#[tauri::command]` handlers, TOML config persistence, presets, Pomodoro timer.
2. **Vue.js Frontend** (`src/`): Single `App.vue` component with `<script setup lang="ts">`, communicates with Rust via `invoke()`.
3. **Rain Overlay** (planned, not yet implemented): Native Wayland layer-shell surface with OpenGL particle rendering.

The frontend is a control panel UI. The backend is the source of truth for all state. Every UI mutation invokes a Tauri command and replaces local state with the response.

## Rust Code Style

### Module Organization

Flat module structure in `src-tauri/src/`. All `mod` declarations live in `lib.rs`. No nested modules.

```
lib.rs       -- App setup, mod declarations, command registration
main.rs      -- Entry point, delegates to lib.rs run()
state.rs     -- AppState with Mutex-wrapped state
commands.rs  -- All #[tauri::command] functions (thin wrappers)
config.rs    -- RainConfig, RainColor, ConfigError, TOML persistence
presets.rs   -- Preset definitions and built-in data
pomodoro.rs  -- PomodoroState, timer logic, phase transitions
```

### Imports

Crate-local imports (`use crate::...`) first, then external crates, then std. No blank lines between groups. All `use` statements are contiguous.

### Naming

- Structs/Enums: `PascalCase` (`RainConfig`, `PomodoroPhase`)
- Functions/methods: `snake_case` (`get_config`, `update_config`)
- Fields: `snake_case` (`drop_length`, `splash_enabled`)
- Enum variants: `PascalCase` (`Work`, `ShortBreak`, `LongBreak`)
- Modules: `snake_case`

### Derives

Order: `Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize`. Not all are always present:
- Serialized data structs: `Debug, Clone, Serialize, Deserialize`
- Enums with comparisons: add `Copy, PartialEq, Eq`
- Error enums: `Error` (thiserror) + `Debug`

### Error Handling

- Use `thiserror` for custom error enums with `#[error("...")]` and `#[from]` for conversions. Do NOT use `anyhow`.
- Tauri commands return `Result<T, String>`. Convert errors with `.map_err(|e| e.to_string())`.
- `Mutex::lock()` uses `.unwrap()` (poisoned mutex is unrecoverable).
- Config load failures use `.unwrap_or_default()` for graceful fallback.
- `.expect("msg")` only at the top-level app runner in `lib.rs`.

### Tauri Commands

- All defined in `commands.rs` as public free functions with `#[tauri::command]`.
- First param is `state: State<AppState>` (except stateless queries).
- Read commands return the type directly. Mutating commands return `Result<T, String>`.
- Use `state.update_config(|c| c.field = value)` for config changes.
- Use lock-mutate-clone pattern for Pomodoro: `let mut p = state.pomodoro.lock().unwrap(); p.action(); p.clone()`.
- Section comments with `// ===...===` banners to group related commands.

### Structs

- All fields are `pub`. No getter/setter patterns.
- `Default` is implemented manually, never derived.
- Constructors use `Self { ... }` not the struct name.
- `RainConfig::clamp()` is always called before `save()`.

### Comments

- `///` doc comments on all public structs, enums, fields, and methods.
- Section dividers with `// ====...====` in commands.rs.
- No `//!` module-level doc comments.

## Vue/TypeScript Code Style

### Component Structure

Single-file components with `<script setup lang="ts">` (Composition API only, no Options API). Section order: `<script setup>` then `<template>` then `<style>`.

Inside `<script setup>`, ordering is:
1. Imports
2. Reactive state (`ref` with explicit generic types)
3. Non-reactive variables
4. Computed properties
5. Lifecycle hooks (`onMounted`, `onUnmounted`)
6. Handler functions grouped by feature with section comments

### Imports

Double quotes for all strings. Order: Vue core, then Tauri/third-party, then local.

```ts
import { ref, onMounted, onUnmounted, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { RainConfig, Preset, PomodoroState } from "./types";
```

### TypeScript Conventions

- `interface` for object shapes, NOT `type` aliases (except for string unions simulating Rust enums).
- String union types for Rust enum mappings: `type PomodoroPhase = "Work" | "ShortBreak" | "LongBreak"`.
- No TypeScript `enum` keyword.
- Type assertions use `as` (not angle brackets).
- `ref<T>()` always has explicit generic type parameter.
- `invoke<T>()` always has explicit return type generic.
- Interface fields use `snake_case` to match Rust serde serialization.
- JS variables and functions use `camelCase`.

### Tauri Integration

- Only `invoke()` is used (no `listen()` events currently).
- Command names are `snake_case` strings matching Rust function names.
- Params passed as `{ camelCase: value }` objects (Tauri auto-converts to snake_case for Rust).
- All commands return the full updated state object; frontend replaces local ref entirely.

### Shared Types

TypeScript interfaces in `src/types.ts` are manually kept in sync with Rust structs. Comment header: `// Types matching Rust backend`. When modifying a Rust struct, update the matching TypeScript interface and vice versa.

### Naming

- Vue files: `PascalCase.vue`
- TypeScript files: `lowercase.ts`
- Functions: `camelCase`, verb-first (`toggleEnabled`, `updateSpeed`, `applyPreset`)
- CSS classes: `kebab-case` (`toggle-btn`, `control-group`, `preset-btn`)

### Styling

- Global unscoped CSS in `App.vue` (no `scoped` attribute, no Tailwind, no CSS modules).
- CSS custom properties on `:root` for theming (dark theme default).
- `transition: all 0.2s ease` on interactive elements.

### Error Handling

- `try/catch` wrapping all `invoke()` calls in `onMounted`. Errors go to `console.error`.
- Individual handler functions use early-return null guards: `if (!config.value) return;`.
- No user-facing error display.

### State Management

- No Pinia or Vuex. Simple `ref()` state in `App.vue`.
- Backend is the source of truth. Every change round-trips through `invoke()`.
- Pomodoro uses `setInterval` at 1000ms for ticking, cleaned up in `onUnmounted`.
