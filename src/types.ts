// Types matching Rust backend

export interface RainColor {
  r: number;
  g: number;
  b: number;
  a: number;
}

export interface RainConfig {
  enabled: boolean;
  intensity: number;
  speed: number;
  angle: number;
  drop_length: number;
  drop_width: number;
  color: RainColor;
  opacity: number;
  splash_enabled: boolean;
  splash_intensity: number;
  preset: string | null;
}

export interface PresetConfig {
  intensity: number;
  speed: number;
  angle: number;
  drop_length: number;
  drop_width: number;
  color: RainColor;
  opacity: number;
  splash_enabled: boolean;
  splash_intensity: number;
}

export interface Preset {
  name: string;
  description: string;
  config: PresetConfig;
}

export type PomodoroPhase = "Work" | "ShortBreak" | "LongBreak";
export type PomodoroStatus = "Idle" | "Running" | "Paused";

export interface PomodoroState {
  phase: PomodoroPhase;
  status: PomodoroStatus;
  remaining_secs: number;
  completed_sessions: number;
  work_duration_secs: number;
  short_break_duration_secs: number;
  long_break_duration_secs: number;
  sessions_until_long_break: number;
}

// Helper functions
export function formatTime(totalSecs: number): string {
  const minutes = Math.floor(totalSecs / 60);
  const seconds = totalSecs % 60;
  return `${minutes.toString().padStart(2, "0")}:${seconds.toString().padStart(2, "0")}`;
}

export function colorToHex(color: RainColor): string {
  const r = color.r.toString(16).padStart(2, "0");
  const g = color.g.toString(16).padStart(2, "0");
  const b = color.b.toString(16).padStart(2, "0");
  return `#${r}${g}${b}`;
}

export function hexToColor(hex: string, alpha: number = 255): RainColor {
  const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
  if (result) {
    return {
      r: parseInt(result[1], 16),
      g: parseInt(result[2], 16),
      b: parseInt(result[3], 16),
      a: alpha,
    };
  }
  return { r: 174, g: 194, b: 224, a: alpha };
}

export function colorToRgba(color: RainColor): string {
  return `rgba(${color.r}, ${color.g}, ${color.b}, ${color.a / 255})`;
}
