mod commands;
mod config;
mod pomodoro;
mod presets;
mod state;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            // Rain config commands
            commands::get_config,
            commands::set_config,
            commands::set_enabled,
            commands::set_intensity,
            commands::set_speed,
            commands::set_angle,
            commands::set_drop_length,
            commands::set_drop_width,
            commands::set_color,
            commands::set_opacity,
            commands::set_splash_enabled,
            commands::set_splash_intensity,
            // Preset commands
            commands::get_presets,
            commands::apply_preset,
            // Pomodoro commands
            commands::get_pomodoro,
            commands::start_pomodoro,
            commands::pause_pomodoro,
            commands::reset_pomodoro,
            commands::skip_pomodoro_phase,
            commands::tick_pomodoro,
            commands::set_pomodoro_durations,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
