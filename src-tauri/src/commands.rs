use crate::config::RainConfig;
use crate::pomodoro::{PomodoroPhase, PomodoroState};
use crate::presets::Preset;
use crate::state::AppState;
use tauri::State;

// ============================================================================
// Rain Config Commands
// ============================================================================

#[tauri::command]
pub fn get_config(state: State<AppState>) -> RainConfig {
    state.get_config()
}

#[tauri::command]
pub fn set_config(state: State<AppState>, config: RainConfig) -> Result<RainConfig, String> {
    state.set_config(config)
}

#[tauri::command]
pub fn set_enabled(state: State<AppState>, enabled: bool) -> Result<RainConfig, String> {
    state.update_config(|c| c.enabled = enabled)
}

#[tauri::command]
pub fn set_intensity(state: State<AppState>, intensity: f32) -> Result<RainConfig, String> {
    state.update_config(|c| c.intensity = intensity)
}

#[tauri::command]
pub fn set_speed(state: State<AppState>, speed: f32) -> Result<RainConfig, String> {
    state.update_config(|c| c.speed = speed)
}

#[tauri::command]
pub fn set_angle(state: State<AppState>, angle: f32) -> Result<RainConfig, String> {
    state.update_config(|c| c.angle = angle)
}

#[tauri::command]
pub fn set_drop_length(state: State<AppState>, length: f32) -> Result<RainConfig, String> {
    state.update_config(|c| c.drop_length = length)
}

#[tauri::command]
pub fn set_drop_width(state: State<AppState>, width: f32) -> Result<RainConfig, String> {
    state.update_config(|c| c.drop_width = width)
}

#[tauri::command]
pub fn set_color(
    state: State<AppState>,
    r: u8,
    g: u8,
    b: u8,
    a: u8,
) -> Result<RainConfig, String> {
    state.update_config(|c| {
        c.color.r = r;
        c.color.g = g;
        c.color.b = b;
        c.color.a = a;
    })
}

#[tauri::command]
pub fn set_opacity(state: State<AppState>, opacity: f32) -> Result<RainConfig, String> {
    state.update_config(|c| c.opacity = opacity)
}

#[tauri::command]
pub fn set_splash_enabled(
    state: State<AppState>,
    enabled: bool,
) -> Result<RainConfig, String> {
    state.update_config(|c| c.splash_enabled = enabled)
}

#[tauri::command]
pub fn set_splash_intensity(
    state: State<AppState>,
    intensity: f32,
) -> Result<RainConfig, String> {
    state.update_config(|c| c.splash_intensity = intensity)
}

// ============================================================================
// Preset Commands
// ============================================================================

#[tauri::command]
pub fn get_presets() -> Vec<Preset> {
    crate::presets::get_builtin_presets()
}

#[tauri::command]
pub fn apply_preset(state: State<AppState>, preset_name: String) -> Result<RainConfig, String> {
    let presets = crate::presets::get_builtin_presets();
    let preset = presets
        .iter()
        .find(|p| p.name == preset_name)
        .ok_or_else(|| format!("Preset '{}' not found", preset_name))?;

    state.update_config(|c| {
        c.intensity = preset.config.intensity;
        c.speed = preset.config.speed;
        c.angle = preset.config.angle;
        c.drop_length = preset.config.drop_length;
        c.drop_width = preset.config.drop_width;
        c.color = preset.config.color.clone();
        c.opacity = preset.config.opacity;
        c.splash_enabled = preset.config.splash_enabled;
        c.splash_intensity = preset.config.splash_intensity;
        c.preset = Some(preset_name.clone());
    })
}

// ============================================================================
// Pomodoro Commands
// ============================================================================

#[tauri::command]
pub fn get_pomodoro(state: State<AppState>) -> PomodoroState {
    state.pomodoro.lock().unwrap().clone()
}

#[tauri::command]
pub fn start_pomodoro(state: State<AppState>) -> PomodoroState {
    let mut pomodoro = state.pomodoro.lock().unwrap();
    pomodoro.start();
    pomodoro.clone()
}

#[tauri::command]
pub fn pause_pomodoro(state: State<AppState>) -> PomodoroState {
    let mut pomodoro = state.pomodoro.lock().unwrap();
    pomodoro.pause();
    pomodoro.clone()
}

#[tauri::command]
pub fn reset_pomodoro(state: State<AppState>) -> PomodoroState {
    let mut pomodoro = state.pomodoro.lock().unwrap();
    pomodoro.reset();
    pomodoro.clone()
}

#[tauri::command]
pub fn skip_pomodoro_phase(state: State<AppState>) -> PomodoroState {
    let mut pomodoro = state.pomodoro.lock().unwrap();
    pomodoro.skip_phase();
    pomodoro.clone()
}

#[tauri::command]
pub fn tick_pomodoro(state: State<AppState>) -> PomodoroState {
    let mut pomodoro = state.pomodoro.lock().unwrap();
    if let Some(new_phase) = pomodoro.tick() {
        let (summary, body) = match new_phase {
            PomodoroPhase::Work => ("Back to Work!", "Focus time has started."),
            PomodoroPhase::ShortBreak => ("Short Break", "Take a quick breather."),
            PomodoroPhase::LongBreak => ("Long Break", "Great job! Take a longer rest."),
        };
        std::thread::spawn(move || {
            let _ = notify_rust::Notification::new()
                .appname("RainDesk")
                .summary(summary)
                .body(body)
                .timeout(5000)
                .show();
        });
    }
    pomodoro.clone()
}

#[tauri::command]
pub fn set_pomodoro_durations(
    state: State<AppState>,
    work_mins: u32,
    short_break_mins: u32,
    long_break_mins: u32,
    sessions_until_long_break: u32,
) -> PomodoroState {
    let mut pomodoro = state.pomodoro.lock().unwrap();
    pomodoro.work_duration_secs = work_mins * 60;
    pomodoro.short_break_duration_secs = short_break_mins * 60;
    pomodoro.long_break_duration_secs = long_break_mins * 60;
    pomodoro.sessions_until_long_break = sessions_until_long_break;
    // Reset timer if durations changed
    if pomodoro.phase == PomodoroPhase::Work {
        pomodoro.remaining_secs = pomodoro.work_duration_secs;
    }
    pomodoro.clone()
}
