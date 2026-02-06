use crate::config::RainConfig;
use crate::pomodoro::PomodoroState;
use std::sync::Mutex;

/// Application state shared across Tauri commands
pub struct AppState {
    /// Rain configuration
    pub config: Mutex<RainConfig>,
    /// Pomodoro timer state
    pub pomodoro: Mutex<PomodoroState>,
}

impl AppState {
    pub fn new() -> Self {
        let config = RainConfig::load().unwrap_or_default();
        Self {
            config: Mutex::new(config),
            pomodoro: Mutex::new(PomodoroState::new()),
        }
    }

    /// Get a clone of the current config
    pub fn get_config(&self) -> RainConfig {
        self.config.lock().unwrap().clone()
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
        Ok(config.clone())
    }

    /// Replace entire config
    pub fn set_config(&self, mut new_config: RainConfig) -> Result<RainConfig, String> {
        new_config.clamp();
        new_config.save().map_err(|e| e.to_string())?;
        let mut config = self.config.lock().unwrap();
        *config = new_config;
        Ok(config.clone())
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
