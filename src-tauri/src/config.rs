use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to get config directory")]
    NoConfigDir,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML serialization error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),
    #[error("TOML deserialization error: {0}")]
    TomlDeserialize(#[from] toml::de::Error),
}

/// RGBA color representation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RainColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Default for RainColor {
    fn default() -> Self {
        Self {
            r: 174,
            g: 194,
            b: 224,
            a: 180,
        }
    }
}

/// Central configuration shared between Rust backend and Vue frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RainConfig {
    /// Whether rain effect is enabled
    pub enabled: bool,

    /// Rain intensity (0.0 - 1.0)
    pub intensity: f32,

    /// Rain speed multiplier (0.5 - 5.0)
    pub speed: f32,

    /// Rain angle in degrees (-60 to 60)
    pub angle: f32,

    /// Rain drop length in pixels
    pub drop_length: f32,

    /// Rain drop width in pixels
    pub drop_width: f32,

    /// Rain color (RGBA)
    pub color: RainColor,

    /// Overall opacity (0.0 - 1.0)
    pub opacity: f32,

    /// Whether splash effect is enabled
    pub splash_enabled: bool,

    /// Splash intensity (0.0 - 1.0)
    pub splash_intensity: f32,

    /// Current preset name (if any)
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
            drop_width: 2.0,
            color: RainColor::default(),
            opacity: 0.7,
            splash_enabled: true,
            splash_intensity: 0.5,
            preset: None,
        }
    }
}

impl RainConfig {
    /// Get the config file path
    pub fn config_path() -> Result<PathBuf, ConfigError> {
        let proj_dirs = directories::ProjectDirs::from("com", "danielhe4rt", "raindesk")
            .ok_or(ConfigError::NoConfigDir)?;
        let config_dir = proj_dirs.config_dir();
        Ok(config_dir.join("config.toml"))
    }

    /// Load config from disk, or return default if not found
    pub fn load() -> Result<Self, ConfigError> {
        let path = Self::config_path()?;
        if path.exists() {
            let contents = fs::read_to_string(&path)?;
            let config: RainConfig = toml::from_str(&contents)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    /// Save config to disk
    pub fn save(&self) -> Result<(), ConfigError> {
        let path = Self::config_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let contents = toml::to_string_pretty(self)?;
        fs::write(&path, contents)?;
        Ok(())
    }

    /// Clamp all values to valid ranges
    pub fn clamp(&mut self) {
        self.intensity = self.intensity.clamp(0.0, 1.0);
        self.speed = self.speed.clamp(0.5, 5.0);
        self.angle = self.angle.clamp(-60.0, 60.0);
        self.drop_length = self.drop_length.clamp(5.0, 100.0);
        self.drop_width = self.drop_width.clamp(1.0, 10.0);
        self.opacity = self.opacity.clamp(0.0, 1.0);
        self.splash_intensity = self.splash_intensity.clamp(0.0, 1.0);
    }
}
