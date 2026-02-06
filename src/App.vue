<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import {
  RainConfig,
  Preset,
  PomodoroState,
  formatTime,
  colorToHex,
  hexToColor,
} from "./types";

// State
const config = ref<RainConfig | null>(null);
const presets = ref<Preset[]>([]);
const pomodoro = ref<PomodoroState | null>(null);
const colorHex = ref("#aec2e0");

// Pomodoro timer interval
let pomodoroInterval: number | null = null;

// Computed
const pomodoroTimeDisplay = computed(() => {
  if (!pomodoro.value) return "25:00";
  return formatTime(pomodoro.value.remaining_secs);
});

const pomodoroPhaseDisplay = computed(() => {
  if (!pomodoro.value) return "Work";
  const phaseMap: Record<string, string> = {
    Work: "Work",
    ShortBreak: "Short Break",
    LongBreak: "Long Break",
  };
  return phaseMap[pomodoro.value.phase] || pomodoro.value.phase;
});

const pomodoroProgress = computed(() => {
  if (!pomodoro.value) return 0;
  let total: number;
  switch (pomodoro.value.phase) {
    case "Work":
      total = pomodoro.value.work_duration_secs;
      break;
    case "ShortBreak":
      total = pomodoro.value.short_break_duration_secs;
      break;
    case "LongBreak":
      total = pomodoro.value.long_break_duration_secs;
      break;
    default:
      total = pomodoro.value.work_duration_secs;
  }
  return ((total - pomodoro.value.remaining_secs) / total) * 100;
});

// Load initial data
onMounted(async () => {
  try {
    config.value = await invoke<RainConfig>("get_config");
    presets.value = await invoke<Preset[]>("get_presets");
    pomodoro.value = await invoke<PomodoroState>("get_pomodoro");

    if (config.value) {
      colorHex.value = colorToHex(config.value.color);
    }

    // Start pomodoro tick interval
    pomodoroInterval = window.setInterval(async () => {
      if (pomodoro.value?.status === "Running") {
        pomodoro.value = await invoke<PomodoroState>("tick_pomodoro");
      }
    }, 1000);
  } catch (e) {
    console.error("Failed to load initial data:", e);
  }
});

onUnmounted(() => {
  if (pomodoroInterval) {
    clearInterval(pomodoroInterval);
  }
});

// Rain control handlers
async function toggleEnabled() {
  if (!config.value) return;
  config.value = await invoke<RainConfig>("set_enabled", {
    enabled: !config.value.enabled,
  });
}

async function updateIntensity(event: Event) {
  const value = parseFloat((event.target as HTMLInputElement).value);
  config.value = await invoke<RainConfig>("set_intensity", { intensity: value });
}

async function updateSpeed(event: Event) {
  const value = parseFloat((event.target as HTMLInputElement).value);
  config.value = await invoke<RainConfig>("set_speed", { speed: value });
}

async function updateAngle(event: Event) {
  const value = parseFloat((event.target as HTMLInputElement).value);
  config.value = await invoke<RainConfig>("set_angle", { angle: value });
}

async function updateDropLength(event: Event) {
  const value = parseFloat((event.target as HTMLInputElement).value);
  config.value = await invoke<RainConfig>("set_drop_length", { length: value });
}

async function updateDropWidth(event: Event) {
  const value = parseFloat((event.target as HTMLInputElement).value);
  config.value = await invoke<RainConfig>("set_drop_width", { width: value });
}

async function updateColor(event: Event) {
  const hex = (event.target as HTMLInputElement).value;
  colorHex.value = hex;
  const color = hexToColor(hex, config.value?.color.a ?? 180);
  config.value = await invoke<RainConfig>("set_color", {
    r: color.r,
    g: color.g,
    b: color.b,
    a: color.a,
  });
}

async function updateOpacity(event: Event) {
  const value = parseFloat((event.target as HTMLInputElement).value);
  config.value = await invoke<RainConfig>("set_opacity", { opacity: value });
}

async function toggleSplash() {
  if (!config.value) return;
  config.value = await invoke<RainConfig>("set_splash_enabled", {
    enabled: !config.value.splash_enabled,
  });
}

async function updateSplashIntensity(event: Event) {
  const value = parseFloat((event.target as HTMLInputElement).value);
  config.value = await invoke<RainConfig>("set_splash_intensity", {
    intensity: value,
  });
}

async function applyPreset(presetName: string) {
  config.value = await invoke<RainConfig>("apply_preset", {
    presetName: presetName,
  });
  if (config.value) {
    colorHex.value = colorToHex(config.value.color);
  }
}

// Pomodoro handlers
async function startPomodoro() {
  pomodoro.value = await invoke<PomodoroState>("start_pomodoro");
}

async function pausePomodoro() {
  pomodoro.value = await invoke<PomodoroState>("pause_pomodoro");
}

async function resetPomodoro() {
  pomodoro.value = await invoke<PomodoroState>("reset_pomodoro");
}

async function skipPhase() {
  pomodoro.value = await invoke<PomodoroState>("skip_pomodoro_phase");
}
</script>

<template>
  <div class="app">
    <!-- Header -->
    <header class="header">
      <h1>RainDesk</h1>
      <button
        class="toggle-btn"
        :class="{ active: config?.enabled }"
        @click="toggleEnabled"
      >
        {{ config?.enabled ? "ON" : "OFF" }}
      </button>
    </header>

    <main class="main">
      <!-- Pomodoro Timer -->
      <section class="section pomodoro-section">
        <h2>Pomodoro Timer</h2>
        <div class="pomodoro-display">
          <div class="pomodoro-phase">{{ pomodoroPhaseDisplay }}</div>
          <div class="pomodoro-time">{{ pomodoroTimeDisplay }}</div>
          <div class="pomodoro-progress">
            <div
              class="pomodoro-progress-bar"
              :style="{ width: pomodoroProgress + '%' }"
            ></div>
          </div>
          <div class="pomodoro-sessions">
            Sessions: {{ pomodoro?.completed_sessions ?? 0 }}
          </div>
        </div>
        <div class="pomodoro-controls">
          <button
            v-if="pomodoro?.status !== 'Running'"
            @click="startPomodoro"
            class="btn btn-primary"
          >
            {{ pomodoro?.status === "Paused" ? "Resume" : "Start" }}
          </button>
          <button
            v-else
            @click="pausePomodoro"
            class="btn btn-secondary"
          >
            Pause
          </button>
          <button @click="resetPomodoro" class="btn btn-secondary">
            Reset
          </button>
          <button @click="skipPhase" class="btn btn-secondary">
            Skip
          </button>
        </div>
      </section>

      <!-- Rain Controls -->
      <section class="section">
        <h2>Rain Settings</h2>

        <div class="control-group">
          <label>
            Intensity
            <span class="value">{{ (config?.intensity ?? 0.5).toFixed(2) }}</span>
          </label>
          <input
            type="range"
            min="0"
            max="1"
            step="0.01"
            :value="config?.intensity ?? 0.5"
            @input="updateIntensity"
          />
        </div>

        <div class="control-group">
          <label>
            Speed
            <span class="value">{{ (config?.speed ?? 1).toFixed(1) }}x</span>
          </label>
          <input
            type="range"
            min="0.5"
            max="5"
            step="0.1"
            :value="config?.speed ?? 1"
            @input="updateSpeed"
          />
        </div>

        <div class="control-group">
          <label>
            Angle
            <span class="value">{{ (config?.angle ?? 0).toFixed(0) }}°</span>
          </label>
          <input
            type="range"
            min="-60"
            max="60"
            step="1"
            :value="config?.angle ?? 0"
            @input="updateAngle"
          />
        </div>

        <div class="control-group">
          <label>
            Drop Length
            <span class="value">{{ (config?.drop_length ?? 20).toFixed(0) }}px</span>
          </label>
          <input
            type="range"
            min="5"
            max="100"
            step="1"
            :value="config?.drop_length ?? 20"
            @input="updateDropLength"
          />
        </div>

        <div class="control-group">
          <label>
            Drop Width
            <span class="value">{{ (config?.drop_width ?? 2).toFixed(1) }}px</span>
          </label>
          <input
            type="range"
            min="1"
            max="10"
            step="0.5"
            :value="config?.drop_width ?? 2"
            @input="updateDropWidth"
          />
        </div>

        <div class="control-group">
          <label>
            Color
            <span class="color-preview" :style="{ backgroundColor: colorHex }"></span>
          </label>
          <input
            type="color"
            :value="colorHex"
            @input="updateColor"
          />
        </div>

        <div class="control-group">
          <label>
            Opacity
            <span class="value">{{ ((config?.opacity ?? 0.7) * 100).toFixed(0) }}%</span>
          </label>
          <input
            type="range"
            min="0"
            max="1"
            step="0.01"
            :value="config?.opacity ?? 0.7"
            @input="updateOpacity"
          />
        </div>

        <div class="control-group checkbox-group">
          <label>
            <input
              type="checkbox"
              :checked="config?.splash_enabled"
              @change="toggleSplash"
            />
            Enable Splash Effects
          </label>
        </div>

        <div class="control-group" v-if="config?.splash_enabled">
          <label>
            Splash Intensity
            <span class="value">{{ ((config?.splash_intensity ?? 0.5) * 100).toFixed(0) }}%</span>
          </label>
          <input
            type="range"
            min="0"
            max="1"
            step="0.01"
            :value="config?.splash_intensity ?? 0.5"
            @input="updateSplashIntensity"
          />
        </div>
      </section>

      <!-- Presets -->
      <section class="section">
        <h2>Presets</h2>
        <div class="presets-grid">
          <button
            v-for="preset in presets"
            :key="preset.name"
            class="preset-btn"
            :class="{ active: config?.preset === preset.name }"
            @click="applyPreset(preset.name)"
            :title="preset.description"
          >
            {{ preset.name }}
          </button>
        </div>
      </section>
    </main>
  </div>
</template>

<style>
* {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
}

:root {
  --bg-primary: #1a1a2e;
  --bg-secondary: #16213e;
  --bg-tertiary: #0f3460;
  --text-primary: #eaeaea;
  --text-secondary: #a0a0a0;
  --accent: #00d9ff;
  --accent-hover: #00b8d4;
  --success: #4caf50;
  --warning: #ff9800;
  --error: #f44336;
  --border-radius: 8px;
}

body {
  font-family: "Inter", -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto,
    sans-serif;
  background-color: var(--bg-primary);
  color: var(--text-primary);
  line-height: 1.5;
}

.app {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--bg-tertiary);
}

.header h1 {
  font-size: 1.5rem;
  font-weight: 600;
  background: linear-gradient(135deg, var(--accent), #00ff88);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.toggle-btn {
  padding: 8px 20px;
  border-radius: 20px;
  border: 2px solid var(--text-secondary);
  background: transparent;
  color: var(--text-secondary);
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
}

.toggle-btn.active {
  border-color: var(--success);
  color: var(--success);
  background: rgba(76, 175, 80, 0.1);
}

.main {
  flex: 1;
  padding: 16px 20px;
  overflow-y: auto;
}

.section {
  background: var(--bg-secondary);
  border-radius: var(--border-radius);
  padding: 16px;
  margin-bottom: 16px;
}

.section h2 {
  font-size: 1rem;
  font-weight: 600;
  color: var(--text-secondary);
  margin-bottom: 16px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

/* Pomodoro */
.pomodoro-display {
  text-align: center;
  margin-bottom: 16px;
}

.pomodoro-phase {
  font-size: 0.875rem;
  color: var(--accent);
  font-weight: 500;
  margin-bottom: 4px;
}

.pomodoro-time {
  font-size: 3rem;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
  letter-spacing: 2px;
}

.pomodoro-progress {
  height: 4px;
  background: var(--bg-tertiary);
  border-radius: 2px;
  margin: 12px 0;
  overflow: hidden;
}

.pomodoro-progress-bar {
  height: 100%;
  background: linear-gradient(90deg, var(--accent), #00ff88);
  transition: width 0.3s ease;
}

.pomodoro-sessions {
  font-size: 0.875rem;
  color: var(--text-secondary);
}

.pomodoro-controls {
  display: flex;
  gap: 8px;
  justify-content: center;
}

.btn {
  padding: 10px 20px;
  border-radius: var(--border-radius);
  border: none;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-primary {
  background: var(--accent);
  color: var(--bg-primary);
}

.btn-primary:hover {
  background: var(--accent-hover);
}

.btn-secondary {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.btn-secondary:hover {
  background: #1a4a7a;
}

/* Controls */
.control-group {
  margin-bottom: 16px;
}

.control-group:last-child {
  margin-bottom: 0;
}

.control-group label {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 0.875rem;
  color: var(--text-primary);
  margin-bottom: 8px;
}

.control-group .value {
  color: var(--accent);
  font-weight: 500;
  font-variant-numeric: tabular-nums;
}

.control-group input[type="range"] {
  width: 100%;
  height: 6px;
  border-radius: 3px;
  background: var(--bg-tertiary);
  appearance: none;
  cursor: pointer;
}

.control-group input[type="range"]::-webkit-slider-thumb {
  appearance: none;
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: var(--accent);
  cursor: pointer;
  transition: transform 0.2s ease;
}

.control-group input[type="range"]::-webkit-slider-thumb:hover {
  transform: scale(1.1);
}

.control-group input[type="color"] {
  width: 100%;
  height: 40px;
  border: none;
  border-radius: var(--border-radius);
  cursor: pointer;
  background: var(--bg-tertiary);
}

.color-preview {
  width: 20px;
  height: 20px;
  border-radius: 4px;
  border: 2px solid var(--bg-tertiary);
}

.checkbox-group label {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  justify-content: flex-start;
}

.checkbox-group input[type="checkbox"] {
  width: 18px;
  height: 18px;
  accent-color: var(--accent);
  cursor: pointer;
}

/* Presets */
.presets-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 8px;
}

.preset-btn {
  padding: 12px 8px;
  border-radius: var(--border-radius);
  border: 1px solid var(--bg-tertiary);
  background: var(--bg-primary);
  color: var(--text-primary);
  font-size: 0.8rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.preset-btn:hover {
  border-color: var(--accent);
  background: var(--bg-tertiary);
}

.preset-btn.active {
  border-color: var(--accent);
  background: rgba(0, 217, 255, 0.1);
  color: var(--accent);
}
</style>
