mod commands;
mod config;
mod pomodoro;
mod presets;
mod rain;
mod state;

use state::AppState;
use std::panic;
use std::sync::mpsc;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = AppState::new();

    // Set up overlay communication channel
    let (overlay_tx, overlay_rx) = mpsc::channel();
    *app_state.overlay_tx.lock().unwrap() = Some(overlay_tx);

    // Clone the Arc<Mutex<RainConfig>> for the overlay thread
    let overlay_config = app_state.config.clone();

    // Spawn overlay on a dedicated OS thread (it runs a blocking Wayland event loop)
    std::thread::Builder::new()
        .name("raindesk-overlay".to_string())
        .spawn(move || {
            match panic::catch_unwind(panic::AssertUnwindSafe(|| {
                rain::overlay::run_overlay(overlay_config, overlay_rx);
            })) {
                Ok(()) => {}
                Err(e) => {
                    let msg = if let Some(s) = e.downcast_ref::<&str>() {
                        s.to_string()
                    } else if let Some(s) = e.downcast_ref::<String>() {
                        s.clone()
                    } else {
                        "unknown panic".to_string()
                    };
                    eprintln!("[raindesk overlay] PANIC: {}", msg);
                }
            }
        })
        .expect("failed to spawn overlay thread");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_state)
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
