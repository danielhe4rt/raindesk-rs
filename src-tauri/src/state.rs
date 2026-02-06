use crate::config::RainConfig;
use crate::pomodoro::PomodoroState;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

/// Signals sent from Tauri commands to the overlay thread
#[derive(Debug)]
#[allow(dead_code)]
pub enum OverlaySignal {
    /// Config has been updated, overlay should re-read from Arc
    ConfigChanged,
    /// Shutdown the overlay thread
    Shutdown,
}

/// Application state shared across Tauri commands
pub struct AppState {
    /// Rain configuration (shared with overlay thread via Arc)
    pub config: Arc<Mutex<RainConfig>>,
    /// Pomodoro timer state
    pub pomodoro: Mutex<PomodoroState>,
    /// Channel to signal the overlay thread
    pub overlay_tx: Mutex<Option<mpsc::Sender<OverlaySignal>>>,
}

impl AppState {
    pub fn new() -> Self {
        let config = RainConfig::load().unwrap_or_default();
        Self {
            config: Arc::new(Mutex::new(config)),
            pomodoro: Mutex::new(PomodoroState::new()),
            overlay_tx: Mutex::new(None),
        }
    }

    /// Get a clone of the current config
    pub fn get_config(&self) -> RainConfig {
        self.config.lock().unwrap().clone()
    }

    /// Signal the overlay thread that config changed
    fn notify_overlay(&self) {
        if let Some(tx) = self.overlay_tx.lock().unwrap().as_ref() {
            let _ = tx.send(OverlaySignal::ConfigChanged);
        }
    }

    /// Update config and save to disk
    pub fn update_config<F>(&self, f: F) -> Result<RainConfig, String>
    where
        F: FnOnce(&mut RainConfig),
    {
        let mut config = self.config.lock().unwrap();
        f(&mut config);
        config.clamp();
        config.save().map_err(|e| e.to_string())?;
        let result = config.clone();
        drop(config);
        self.notify_overlay();
        Ok(result)
    }

    /// Replace entire config
    pub fn set_config(&self, mut new_config: RainConfig) -> Result<RainConfig, String> {
        new_config.clamp();
        new_config.save().map_err(|e| e.to_string())?;
        let mut config = self.config.lock().unwrap();
        *config = new_config;
        let result = config.clone();
        drop(config);
        self.notify_overlay();
        Ok(result)
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
